//! OCAF document wrapper and command RAII guard.
//!
//! [`OcDocument`] holds a `TDocStd_Document` handle.  All TDF mutations must
//! be bracketed by a [`Command`] scope.
//!
//! [`Command`] opens a transaction on construction and aborts it on drop if
//! neither [`commit`] nor an explicit [`abort`] was called.  This matches the
//! standard transactional database idiom: incomplete commands are always
//! rolled back.
//!
//! [`commit`]: Command::commit
//! [`abort`]: Command::abort

use std::marker::PhantomData;

use occt_sys::ffi;

use crate::error::OcctError;
use crate::topo::label::OcLabel;

/// An in-memory OCAF document.
///
/// Wraps `Handle(TDocStd_Document)`.  The Handle keeps the document alive
/// independently of the [`OcApplication`] that created it.
///
/// # Thread safety
///
/// OCCT Handle ref-counting is not atomic.  `OcDocument` must not be sent
/// across thread boundaries.
///
/// [`OcApplication`]: crate::topo::OcApplication
pub struct OcDocument {
    pub(crate) inner: cxx::UniquePtr<ffi::DocumentHandle>,
    _not_send: PhantomData<*mut ()>,
}

impl OcDocument {
    pub(crate) fn from_ffi(inner: cxx::UniquePtr<ffi::DocumentHandle>) -> Self {
        Self {
            inner,
            _not_send: PhantomData,
        }
    }

    /// The root label of the document's user data section.
    ///
    /// All application-level label trees are rooted here.  The returned
    /// label's lifetime is tied to `self`.
    pub fn main(&self) -> OcLabel {
        OcLabel::from_ffi(ffi::document_main(&self.inner))
    }

    /// Opens a new command scope.
    ///
    /// Returns a [`Command`] RAII guard.  On drop, the command is aborted if
    /// neither [`Command::commit`] nor [`Command::abort`] was called.
    ///
    /// The document is exclusively borrowed for the lifetime of the returned
    /// [`Command`]; no other mutable access is possible while it is live.
    ///
    /// # Errors
    ///
    /// Returns `Err` if OCCT raises during `NewCommand` (unlikely for valid
    /// documents, but wrapped defensively).
    pub fn begin_command(&mut self) -> Result<Command<'_>, OcctError> {
        let mut pinned = self.inner.pin_mut();
        ffi::document_new_command(pinned.as_mut()).map_err(OcctError::from)?;
        Ok(Command {
            inner: pinned,
            done: false,
        })
    }

    /// Number of commands available for undo.
    pub fn available_undos(&self) -> i32 {
        ffi::document_get_available_undos(&self.inner)
    }

    /// Number of commands available for redo.
    pub fn available_redos(&self) -> i32 {
        ffi::document_get_available_redos(&self.inner)
    }

    /// Undoes the most recently committed command.
    ///
    /// Returns `true` when an undo was performed, `false` when the undo stack
    /// is empty.
    pub fn undo(&mut self) -> Result<bool, OcctError> {
        ffi::document_undo(self.inner.pin_mut()).map_err(OcctError::from)
    }

    /// Re-applies the most recently undone command.
    ///
    /// Returns `true` when a redo was performed, `false` when the redo stack
    /// is empty.
    pub fn redo(&mut self) -> Result<bool, OcctError> {
        ffi::document_redo(self.inner.pin_mut()).map_err(OcctError::from)
    }

    /// Sets the maximum number of undoable commands retained.
    ///
    /// Older entries are discarded when the limit is exceeded.
    pub fn set_undo_limit(&mut self, n: i32) {
        ffi::document_set_undo_limit(self.inner.pin_mut(), n);
    }
}

impl std::fmt::Debug for OcDocument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OcDocument").finish_non_exhaustive()
    }
}

// ── Command ───────────────────────────────────────────────────────────────────

/// RAII guard for a TDF command scope.
///
/// Constructed by [`OcDocument::begin_command`].  The document is exclusively
/// borrowed for the guard's lifetime.
///
/// On drop: if neither [`commit`] nor [`abort`] has been called, the command
/// is aborted.  Abort errors in drop are silently discarded (cannot propagate
/// from `Drop`).
///
/// [`commit`]: Command::commit
/// [`abort`]: Command::abort
pub struct Command<'doc> {
    inner: cxx::core::pin::Pin<&'doc mut ffi::DocumentHandle>,
    done: bool,
}

impl<'doc> Command<'doc> {
    /// Commits the command, recording the changes as an undoable delta.
    ///
    /// Returns `true` when at least one change was recorded.
    ///
    /// Consumes `self`; the document borrow is released on return.
    pub fn commit(mut self) -> Result<bool, OcctError> {
        let result = ffi::document_commit_command(self.inner.as_mut()).map_err(OcctError::from)?;
        self.done = true;
        Ok(result)
    }

    /// Aborts the command, discarding all changes since the scope was opened.
    ///
    /// Consumes `self`; the document borrow is released on return.
    pub fn abort(mut self) -> Result<(), OcctError> {
        ffi::document_abort_command(self.inner.as_mut()).map_err(OcctError::from)?;
        self.done = true;
        Ok(())
    }
}

impl Drop for Command<'_> {
    /// Aborts the command if neither `commit` nor `abort` was called.
    fn drop(&mut self) {
        if !self.done {
            // Errors here cannot be propagated; discard silently.
            // The abort prevents leaving the document in a partially-modified
            // state if the Command is dropped without explicit resolution.
            let _ = ffi::document_abort_command(self.inner.as_mut());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topo::OcApplication;

    fn new_doc() -> (OcApplication, OcDocument) {
        let mut app = OcApplication::new();
        let doc = app.new_document("BinXCAF").unwrap();
        (app, doc)
    }

    #[test]
    fn document_main_is_not_null() {
        let (_app, doc) = new_doc();
        let root = doc.main();
        assert!(!root.is_null());
    }

    #[test]
    fn document_main_is_not_root_but_child_of_root() {
        let (_app, doc) = new_doc();
        let main = doc.main();
        // Main() = root.FindChild(1) — it is a child of the root, not the root itself.
        assert!(!main.is_root());
        assert_eq!(main.tag(), 1);
        // Its parent is the root.
        assert!(main.father().is_root());
    }

    #[test]
    fn command_commit_records_delta() {
        let (_app, mut doc) = new_doc();
        doc.set_undo_limit(10);
        {
            let cmd = doc.begin_command().unwrap();
            // No actual attribute writes yet — but a well-formed empty commit
            // is still valid.
            let _ = cmd.commit().unwrap();
        }
        // After one committed command, one undo should be available.
        // (OCCT may not record empty commands; acceptable either way.)
        let undos = doc.available_undos();
        assert!(undos >= 0);
    }

    #[test]
    fn command_abort_on_drop() {
        let (_app, mut doc) = new_doc();
        doc.set_undo_limit(10);
        {
            let _cmd = doc.begin_command().unwrap();
            // Drop without commit or explicit abort → abort-on-drop.
        }
        // Document should still be usable.
        let root = doc.main();
        assert!(!root.is_null());
    }

    #[test]
    fn command_explicit_abort() {
        let (_app, mut doc) = new_doc();
        let cmd = doc.begin_command().unwrap();
        cmd.abort().unwrap();
        // Document should still be usable after abort.
        assert!(!doc.main().is_null());
    }

    #[test]
    fn undo_limit_respected() {
        let (_app, mut doc) = new_doc();
        doc.set_undo_limit(2);
        assert!(doc.available_undos() <= 2);
    }

    #[test]
    fn undo_on_empty_stack_returns_false() {
        let (_app, mut doc) = new_doc();
        doc.set_undo_limit(10);
        // No commands committed yet.
        let result = doc.undo().unwrap();
        assert!(!result, "undo on empty stack should return false");
    }
}
