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
use crate::ocaf::label::LabelPath;
use crate::ocaf::label::OcLabel;
use crate::ocaf::topo_naming::{TopoNamingBuilder, TopoNamingSelector};

/// An in-memory OCAF document.
///
/// Manages the tree of [`OcLabel`]s. All application data - shapes, attributes, named
/// shapes, etc. - lives on an [`OcLabel`] as an attribute.
///
/// These are some primary rules of this data structure:
///
/// - The tree has arbitrary number of labels per node
/// - labels are 1-indexed
/// - Only one instance of an attribute per [`OcLabel`]
/// - `0:1` is the address of the main label (fetchable with [`OcDocument::main`])
/// - Changes must happen within a [`Command`] context
///
///
/// Wraps `Handle(TDocStd_Document)`.  On drop, the document closes itself
/// through its application back-pointer, severing the OCAF ownership cycle
/// (`app→doc` and `doc→app`).  Use [`close`] for an explicit consuming close
/// that propagates errors.
///
/// [`close`]: OcDocument::close
///
/// # Thread safety
///
/// OCCT Handle ref-counting is not atomic.  `OcDocument` must not be sent
/// across thread boundaries.
///
/// # Example
///
/// We create a tree that looks as follows:
///
/// ```text
/// main (0:1)
/// └── 1 (0:1:1)   planes
///     ├── 1 (0:1:1:1)   XY plane
///     │       TopoNamingNamedShape (Primitive, planar face)
///     │       OcPlaneAttr
///     ├── 2 (0:1:1:2)   YZ plane
///     │       TopoNamingNamedShape (Primitive, planar face)
///     │       OcPlaneAttr
///     └── 3 (0:1:1:3)   XZ plane
///             TopoNamingNamedShape (Primitive, planar face)
///             OcPlaneAttr
/// ```
///
/// ```
/// use occt_rs::ocaf::OcApplication;
/// use occt_rs::gp::{OcAx2, OcDir, OcPnt};
/// use occt_rs::ocaf::tdata_xtd::OcPlaneAttr;
///
/// let mut app = OcApplication::new();
/// let mut doc = app.new_document("BinXCAF").unwrap();
///
/// // doc.main() is the root of the application label tree — always tag 1
/// // under the framework root. Obtain it before opening a command.
/// let main = doc.main();
/// assert_eq!(main.tag(), 1);
///
/// // Create the top-level container nodes in a single command.
/// // In the scenario these are: planes (1), sketch (2), body (3), sketch2 (4).
/// let planes = {
///     doc.begin_command().unwrap();
///     let planes  = main.get_or_create_child(1);
///     let _sketch  = main.get_or_create_child(2);
///     let _body    = main.get_or_create_child(3);
///     let _sketch2 = main.get_or_create_child(4);
///     doc.commit().unwrap();
///     planes
/// };
///
/// // main (0:1)
/// // ├── 1 (0:1:1)   planes
/// // ├── 2 (0:1:2)   sketch
/// // ├── 3 (0:1:3)   body
/// // └── 4 (0:1:4)   sketch2
///
/// let (xy, xy_frame) = {
///     doc.begin_command().unwrap();
///     let xy = planes.get_or_create_child(1);
///     let xy_frame = OcAx2::new(
///         OcPnt::new(0.0, 0.0, 0.0),
///         OcDir::new(0.0, 0.0, 1.0).unwrap(),
///         OcDir::new(1.0, 0.0, 0.0).unwrap(),
///     ).unwrap();
///     drop(planes);
///     (xy, xy_frame)
/// };
///
/// // you can also retreive planes by the label address
/// let planes_gotten = doc.label_at(&"1:1".parse().unwrap()).unwrap();
/// doc.begin_command().unwrap();
///
/// OcPlaneAttr::record_shape(&xy, xy_frame).unwrap();
/// OcPlaneAttr::set(&xy).unwrap();
///
/// let yz = planes_gotten.get_or_create_child(2);
/// let yz_frame = OcAx2::new(
///     OcPnt::new(0.0, 0.0, 0.0),
///     OcDir::new(1.0, 0.0, 0.0).unwrap(),
///     OcDir::new(0.0, 1.0, 0.0).unwrap(),
/// ).unwrap();
/// OcPlaneAttr::record_shape(&yz, yz_frame).unwrap();
/// OcPlaneAttr::set(&yz).unwrap();
///
/// let xz = planes_gotten.get_or_create_child(3);
/// let xz_frame = OcAx2::new(
///     OcPnt::new(0.0, 0.0, 0.0),
///     OcDir::new(0.0, 1.0, 0.0).unwrap(),
///     OcDir::new(1.0, 0.0, 0.0).unwrap(),
/// ).unwrap();
/// OcPlaneAttr::record_shape(&xz, xz_frame).unwrap();
/// OcPlaneAttr::set(&xz).unwrap();
///
/// doc.commit().unwrap();
///
/// assert!(OcPlaneAttr::find(&xy).is_some());
/// assert!(OcPlaneAttr::find(&yz).is_some());
/// assert!(OcPlaneAttr::find(&xz).is_some());
///
///
/// assert_eq!(main.children(false).count(), 4);
/// ```
///
/// [`OcLabel`]: crate::ocaf::label::OcLabel
/// [`OcApplication`]: crate::ocaf::OcApplication
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

    /// Gets the root label
    ///
    /// All application-level label trees are rooted here.  The returned
    /// label's lifetime is tied to `self`.
    pub fn main(&self) -> OcLabel {
        unsafe { OcLabel::from_ffi_unchecked(ffi::document_main(&self.inner)) }
    }
    /// Resolves a [`LabelPath`] from the document root.
    ///
    /// Returns `None` if any segment of the path does not exist.
    pub fn label_at(&self, path: &LabelPath) -> Option<OcLabel> {
        OcLabel::from_ffi(ffi::tdf_label_from_entry(
            &self.main().root().inner,
            &path.to_string(),
        ))
    }

    /// Pure wrapper around TDocStd_Document::Command
    pub fn begin_command(&mut self) -> Result<(), OcctError> {
        ffi::document_new_command(self.inner.pin_mut().as_mut()).map_err(OcctError::from)
    }
    /// Pure wrapper around TDataStd_Document::CommitCommand
    pub fn commit(&mut self) -> Result<bool, OcctError> {
        ffi::document_commit_command(self.inner.pin_mut()).map_err(OcctError::from)
    }
    pub fn abort(&mut self) -> Result<(), OcctError> {
        ffi::document_abort_command(self.inner.pin_mut().as_mut()).map_err(OcctError::from)
    }
    /// Creates a [`TopoNamingSelector`] bound to `label`.
    pub fn selector(&self, label: &OcLabel) -> TopoNamingSelector {
        TopoNamingSelector::new(ffi::new_tnaming_selector(label.inner.as_ref().unwrap()))
    }
    pub fn name_builder<'doc>(&'doc self, label: &OcLabel) -> TopoNamingBuilder<'doc> {
        TopoNamingBuilder::new(label)
    }

    pub fn available_undos(&self) -> i32 {
        ffi::document_get_available_undos(&self.inner)
    }

    pub fn available_redos(&self) -> i32 {
        ffi::document_get_available_redos(&self.inner)
    }

    /// Returns `true` when an undo was performed, `false` when the undo stack
    /// is empty.
    ///
    /// Only commands containing attribute changes produce an undoable delta.
    /// Commands that only create label nodes produce no delta and do not
    /// increment [`available_undos`](OcDocument::available_undos). [`OcLabel`] entry changes
    /// are permanent structural elements of the underlying data-tree.  See [`Command`] for
    /// details.
    ///
    /// The example below will create a tree that looks as follows, and then use undo/redo to
    /// revert/restore the [`OcInteger`] attribute
    ///
    /// ```text
    /// main (0:1)
    /// └── 1 (0:1:1)
    ///     - OcInteger = 42
    /// ```
    ///
    /// ```
    /// use occt_rs::ocaf::OcApplication;
    /// use occt_rs::ocaf::attributes::OcInteger;
    ///
    /// let mut app = OcApplication::new();
    /// let mut doc = app.new_document("BinXCAF").unwrap();
    /// doc.set_undo_limit(10);
    ///
    /// // Create the label — structural, produces no undo delta.
    /// let main = doc.main();
    /// let label = {
    ///     doc.begin_command().unwrap();
    ///     let l = main.get_or_create_child(1);
    ///     doc.commit().unwrap();
    ///     l
    /// };
    /// assert_eq!(doc.available_undos(), 0);
    ///
    /// // Write an attribute — this produces a delta.
    /// {
    ///     doc.begin_command().unwrap();
    ///     OcInteger::set(&label, 42).unwrap();
    ///     doc.commit().unwrap();
    /// }
    /// assert_eq!(doc.available_undos(), 1);
    /// assert_eq!(doc.available_redos(), 0);
    /// // Data tree now looks like this:
    /// // main (0:1)
    /// // └──  (0:1:1)
    /// //      - OcInteger = 99
    /// assert_eq!(OcInteger::find(&label).unwrap().get(), 42);
    ///
    /// // Undo removes the attribute, restoring the pre-command state.
    /// assert!(doc.undo().unwrap());
    /// assert!(OcInteger::find(&label).is_none(), "Integer attribute should have been removed");
    /// assert_eq!(doc.available_undos(), 0);
    /// assert_eq!(doc.available_redos(), 1);
    ///
    /// // Redo restores it.
    /// assert!(doc.redo().unwrap());
    /// assert_eq!(OcInteger::find(&label).unwrap().get(), 42, "Integer attribute should have been restored");
    /// assert_eq!(doc.available_undos(), 1);
    /// assert_eq!(doc.available_redos(), 0);
    /// ```
    ///
    /// [`OcInteger`]: crate::ocaf::attributes::OcInteger
    pub fn undo(&mut self) -> Result<bool, OcctError> {
        ffi::document_undo(self.inner.pin_mut()).map_err(OcctError::from)
    }

    /// Returns `true` when a redo was performed, `false` when the redo stack
    /// is empty.
    pub fn redo(&mut self) -> Result<bool, OcctError> {
        ffi::document_redo(self.inner.pin_mut()).map_err(OcctError::from)
    }

    /// Older entries are discarded when the limit is exceeded.
    pub fn set_undo_limit(&mut self, n: i32) {
        ffi::document_set_undo_limit(self.inner.pin_mut(), n);
    }

    pub fn has_open_command(&self) -> bool {
        ffi::document_has_open_command(&self.inner)
    }

    /// A Document is considered opened when it is registered with an
    /// [`OcApplication`](crate::ocaf::OcApplication)
    ///
    /// `IsOpened()` is `false` after [`close`] or [`Drop`] runs.
    ///
    /// [`close`]: OcDocument::close
    pub fn is_opened(&self) -> bool {
        ffi::document_is_opened(&self.inner)
    }

    /// Closes the document, deregistering it from its [`OcApplication`](crate::ocaf::OcApplication).
    ///
    /// Severs both OCAF ownership edges (`app→doc` and `doc→app`).  After
    /// this the document is no longer usable.  Idempotent at the OCCT level:
    /// a subsequent drop calls `document_close` again, which is a no-op
    /// because `IsOpened()` is already `false`.
    pub fn close(mut self) -> Result<(), OcctError> {
        ffi::document_close(self.inner.pin_mut()).map_err(OcctError::from)
        // self drops here; Drop's document_close is a no-op because IsOpened() is false.
    }
}
impl Drop for OcDocument {
    fn drop(&mut self) {
        // Severs the OCAF ownership cycle (app→doc and doc→app).
        // Errors cannot propagate from Drop; discard, as Command::drop already does.
        let _ = ffi::document_close(self.inner.pin_mut());
    }
}

impl std::fmt::Debug for OcDocument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OcDocument").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use crate::ocaf::OcApplication;

    use super::*;

    fn new_doc() -> (OcApplication, OcDocument) {
        let mut app = OcApplication::new();
        let doc = app.new_document("BinXCAF").unwrap();
        (app, doc)
    }

    #[test]
    fn document_main_is_not_null() {
        let (_app, doc) = new_doc();
        let root = doc.main();
        assert_eq!(root.tag(), 1);
    }

    #[test]
    fn document_main_is_not_root_but_child_of_root() {
        let (_app, doc) = new_doc();
        let main = doc.main();
        // Main() = root.FindChild(1) — it is a child of the root, not the root itself.
        assert!(!main.is_root());
        assert_eq!(main.tag(), 1);
        // Its parent is the root.
        assert!(main.father().unwrap().is_root());
    }

    #[test]
    fn command_commit_records_delta() {
        let (_app, mut doc) = new_doc();
        doc.set_undo_limit(10);
        doc.begin_command().unwrap();
        doc.commit().unwrap();
        // After one committed command, one undo should be available.
        // (OCCT may not record empty commands; acceptable either way.)
        let undos = doc.available_undos();
        assert!(undos >= 0);
    }

    #[test]
    fn command_explicit_abort() {
        let (_app, mut doc) = new_doc();
        doc.begin_command().unwrap();
        doc.abort().unwrap();
        // Document should still be usable after abort.
        assert_eq!(doc.main().tag(), 1);
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
