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

use std::marker::PhantomData;

use occt_sys::ffi;

/// A non-owning reference to a node in a document's label tree.
///
/// Wraps `TDF_Label`.  `'doc` is the lifetime of the [`OcDocument`] that
/// owns the underlying `TDF_Data` tree; labels cannot outlive it.
///
/// A null label (returned by [`find_child`] with `create = false` when the
/// child is absent) has [`is_null()`] returning `true`.  Calling any
/// structural method on a null label is undefined at the OCCT level; always
/// check [`is_null()`] when `create = false`.
///
/// [`find_child`]: OcLabel::find_child
/// [`is_null()`]: OcLabel::is_null
pub struct OcLabel {
    pub(crate) inner: cxx::UniquePtr<ffi::TdfLabel>,
    /// Ties this label to the document lifetime — labels cannot outlive
    /// the TDF_Data tree owned by the document.
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
    /// A null label results from [`find_child`] with `create = false` when
    /// the requested child does not exist.
    ///
    /// [`find_child`]: OcLabel::find_child
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

    /// Finds or creates a direct child label with the given `tag`.
    ///
    /// `create = true` — creates the child if absent; result is never null.
    /// `create = false` — returns `None` if no child with this tag exists.
    pub fn find_child(&self, tag: i32, create: bool) -> Option<OcLabel> {
        let inner = ffi::tdf_label_find_child(&self.inner, tag, create);
        let label = OcLabel::from_ffi(inner);
        if label.is_null() {
            None
        } else {
            Some(label)
        }
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
