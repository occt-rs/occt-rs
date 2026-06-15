//! TDF label wrapper.
//!
//! `OcLabel` is a non-owning reference into a `TDF_Data` tree.  The lifetime
//! parameter `'doc` enforces at compile time that no label outlives the
//! [`OcDocument`] that owns the tree.
//!
//! Labels are cheap to clone (the underlying OCCT type holds a Handle to the
//! label node, which is ref-counted).
//!
//! [`OcDocument`]: crate::topo::OcDocument

use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

use occt_sys::ffi;

use crate::topo::document::Command;

/// A non-owning reference to a node in a document's label tree.
///
/// Wraps `TDF_Label`, a non-owning reference into the `TDF_Data` tree owned
/// by an [`OcDocument`].
///
/// **The document tie is not tracked by the type system.** `OcLabel` carries
/// no `'doc` parameter — deliberately: parameterising it conflicts with
/// `Command<'doc>`, which holds a mutable borrow of the document and would
/// make it impossible to obtain a label (e.g. via `doc.main()`) while a
/// command is live. That a label must not be used after its document is
/// dropped is therefore a *caller obligation*, not a compile-time guarantee.
///
/// A null label (returned by [`find_child`] when the child is absent) has
/// [`is_null()`] returning `true`.  Calling any
/// structural method on a null label is undefined at the OCCT level; always
/// check [`is_null()`] when `create = false`.
///
/// [`find_child`]: OcLabel::find_child
/// [`is_null()`]: OcLabel::is_null
pub struct OcLabel {
    // Fixme: audit how to construct a TnamingBuilder. Currently: `TnamingBuilder::new(new_tnaming_builder(&label.inner))`
    pub(crate) inner: cxx::UniquePtr<ffi::TdfLabel>,
    /// `!Send` / `!Sync` marker only. Does **not** tie the label to any
    /// document lifetime (`*mut ()` carries no lifetime); see the type-level
    /// note on why `OcLabel` is intentionally un-parameterised.
    _not_send: PhantomData<*mut ()>,
}

impl OcLabel {
    pub(crate) fn from_ffi(inner: cxx::UniquePtr<ffi::TdfLabel>) -> Self {
        Self {
            inner,
            _not_send: PhantomData,
        }
    }

    /// Returns `true` when this label has no associated node.
    ///
    /// A null label results from [`find_child`](Self::find_child) when the
    /// requested child does not exist.
    pub fn is_null(&self) -> bool {
        ffi::tdf_label_is_null(&self.inner)
    }

    /// Returns `true` when this label is the root of the framework.
    pub fn is_root(&self) -> bool {
        ffi::tdf_label_is_root(&self.inner)
    }

    /// The integer tag identifying this label among its siblings.
    pub fn tag(&self) -> i32 {
        ffi::tdf_label_tag(&self.inner)
    }

    /// The parent label.  Returns a null label when called on the root.
    pub fn father(&self) -> OcLabel {
        OcLabel::from_ffi(ffi::tdf_label_father(&self.inner))
    }

    /// Finds a direct child label with the given `tag`.
    ///
    /// Returns `None` if no child with this tag exists. Never creates.
    pub fn find_child(&self, tag: i32) -> Option<OcLabel> {
        let inner = ffi::tdf_label_find_child(&self.inner, tag, false);
        let label = OcLabel::from_ffi(inner);
        if label.is_null() {
            None
        } else {
            Some(label)
        }
    }

    /// Finds or creates a direct child label with the given `tag`.
    ///
    /// Always succeeds. Label creation via `FindChild(tag, true)` is
    /// captured by OCAF's Backup/Delta mechanism, so this requires an open
    /// [`Command`].
    pub fn get_or_create_child(&self, _cmd: &Command<'_>, tag: i32) -> OcLabel {
        OcLabel::from_ffi(ffi::tdf_label_find_child(&self.inner, tag, true))
    }

    /// Returns `true` when at least one attribute is attached to this label.
    pub fn has_attribute(&self) -> bool {
        ffi::tdf_label_has_attribute(&self.inner)
    }

    /// Count of attributes attached to this label.
    pub fn nb_attributes(&self) -> i32 {
        ffi::tdf_label_nb_attributes(&self.inner)
    }

    /// The label's path as a colon-delimited entry string, e.g. `"0:1:2:3"`.
    ///
    /// Useful for debugging and within-session identification.  Not persistent
    /// across process restarts.
    pub fn entry(&self) -> String {
        ffi::tdf_label_entry(&self.inner)
    }

    /// Returns an iterator over the direct children of this label.
    ///
    /// Pass `all_levels = true` to iterate all descendants recursively.
    pub fn children(&self, all_levels: bool) -> OcChildIterator<'_> {
        OcChildIterator {
            inner: ffi::new_tdf_child_iterator(&self.inner, all_levels),
            _phantom: PhantomData,
            _not_send: PhantomData,
        }
    }
    /// The root label of the data framework (depth 0).
    ///
    /// Same shim pattern as [`father`](Self::father); no `Handle(TDF_Data)`
    /// involved.
    pub fn root(&self) -> OcLabel {
        OcLabel::from_ffi(ffi::tdf_label_root(&self.inner))
    }

    /// Forgets all attributes on this label, and on every descendant if
    /// `clear_children` is `true`.
    ///
    /// Captured by OCAF's Backup/Delta mechanism — requires an open
    /// [`Command`].
    pub fn forget_all_attributes(&self, _cmd: &Command<'_>, clear_children: bool) {
        ffi::tdf_label_forget_all_attributes(&self.inner, clear_children);
    }

    /// This label's path from the document root, as a sequence of child tags.
    ///
    /// Pure Rust: walks [`father`](Self::father)/[`is_root`](Self::is_root)
    /// up to (but not including) the framework root. [`LabelPath`]'s
    /// `Display` produces the same colon-joined form as
    /// [`entry`](Self::entry).
    pub fn path(&self) -> LabelPath {
        let mut tags = Vec::new();
        let mut current = self.clone();
        while !current.is_root() {
            tags.push(current.tag());
            current = current.father();
        }
        tags.reverse();
        LabelPath(tags)
    }
    /// Resolves `path` relative to `self`, creating any missing descendant
    /// labels along the way. Always succeeds.
    ///
    /// Requires an open [`Command`]. Obtain `self` (e.g. `doc.main().root()`)
    /// before opening the command — see the borrow note on
    /// [`get_or_create_child`](Self::get_or_create_child).
    pub fn get_or_create_descendant(&self, cmd: &Command<'_>, path: &LabelPath) -> OcLabel {
        let mut current = self.clone();
        for &tag in &path.0 {
            current = current.get_or_create_child(cmd, tag);
        }
        current
    }
}

impl Clone for OcLabel {
    /// Cheap clone: increments the Handle(TDF_LabelNode) ref-count.
    fn clone(&self) -> Self {
        Self {
            inner: ffi::clone_tdf_label(&self.inner),
            _not_send: PhantomData,
        }
    }
}

impl std::fmt::Debug for OcLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_null() {
            f.write_str("OcLabel(null)")
        } else {
            f.debug_struct("OcLabel")
                .field("entry", &self.entry())
                .field("tag", &self.tag())
                .finish()
        }
    }
}

// ── OcChildIterator ───────────────────────────────────────────────────────────

/// A Rust [`Iterator`] over the children (or descendants) of an [`OcLabel`].
///
/// Constructed via [`OcLabel::children`].  Each `Item` is an [`OcLabel`]
/// with the same document lifetime as the label it was created from.
pub struct OcChildIterator<'doc> {
    inner: cxx::UniquePtr<ffi::TdfChildIteratorShim>,
    _phantom: PhantomData<&'doc ()>,
    _not_send: PhantomData<*mut ()>,
}

impl<'doc> Iterator for OcChildIterator<'doc> {
    type Item = OcLabel;

    fn next(&mut self) -> Option<OcLabel> {
        if !self.inner.more() {
            return None;
        }
        // value() is const — reads current without advancing.
        let inner = self.inner.value();
        // next() is non-const — advances the iterator.
        self.inner.pin_mut().next();
        Some(OcLabel::from_ffi(inner))
    }
}
// ── LabelPath ───────────────────────────────────────────────────────────────

/// A label's location as a sequence of child tags from the document root.
///
/// `Display` produces the colon-joined form `"1:2:3"`, matching
/// [`OcLabel::entry`]. `FromStr` parses it back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabelPath(pub Vec<i32>);

impl fmt::Display for LabelPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut tags = self.0.iter();
        if let Some(first) = tags.next() {
            write!(f, "{first}")?;
            for tag in tags {
                write!(f, ":{tag}")?;
            }
        }
        Ok(())
    }
}

/// Error returned by [`LabelPath`]'s `FromStr` when a segment is not a valid `i32`.
#[derive(Debug)]
pub struct LabelPathParseError {
    segment: String,
}

impl fmt::Display for LabelPathParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "label path: invalid tag segment {:?}", self.segment)
    }
}

impl std::error::Error for LabelPathParseError {}

impl FromStr for LabelPath {
    type Err = LabelPathParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Ok(LabelPath(Vec::new()));
        }
        s.split(':')
            .map(|seg| {
                seg.parse::<i32>().map_err(|_| LabelPathParseError {
                    segment: seg.to_string(),
                })
            })
            .collect::<Result<Vec<_>, _>>()
            .map(LabelPath)
    }
}
