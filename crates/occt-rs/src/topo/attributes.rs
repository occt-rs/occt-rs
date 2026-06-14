//! Standard TDF attributes: scalars (Name, Integer, Real, Comment,
//! AsciiString) and list/array attributes (ReferenceList, ReferenceArray).
//!
//! Each type wraps a `Handle(TDataStd_*)` shim.  The operations per type are:
//!
//! - **`set`** — attaches or updates the attribute on a label (inside a command).
//! - **`get`** — reads the current value from an already-retrieved attribute handle.
//! - **`find`** — probes whether the attribute is present on a label.
//! - **`forget`** — removes the attribute from a label (inside a command).
//!
//! GUIDs are kept on the C++ side; the Rust API never names them.

use std::marker::PhantomData;

use occt_sys::ffi;

use crate::error::OcctError;
use crate::topo::document::Command;
use crate::topo::label::OcLabel;

// ── OcName ────────────────────────────────────────────────────────────────────

/// A `TDataStd_Name` attribute handle — a UTF-8 string attached to a label.
///
/// Construct via [`OcName::set`] inside an open command scope.
/// Retrieve from an existing label via [`OcName::find`].
pub struct OcName {
    inner: cxx::UniquePtr<ffi::TDataStdNameHandle>,
    _not_send: PhantomData<*mut ()>,
}

impl OcName {
    /// Attaches or updates a `TDataStd_Name` attribute on `label`.
    ///
    /// Must be called inside an open [`Command`] scope.
    pub fn set(_cmd: &Command<'_>, label: &OcLabel, value: &str) -> Result<Self, OcctError> {
        let inner = ffi::tdatastd_name_set(&label.inner, value).map_err(OcctError::from)?;
        Ok(Self {
            inner,
            _not_send: PhantomData,
        })
    }

    /// Reads the string value of this attribute.
    pub fn get(&self) -> String {
        ffi::tdatastd_name_get(&self.inner)
    }

    /// Probes for a `TDataStd_Name` attribute on `label`.
    ///
    /// Returns `None` when the attribute is not present.
    /// No command scope required for read-only access.
    pub fn find(label: &OcLabel) -> Option<Self> {
        let inner = ffi::tdatastd_name_find(&label.inner);
        if inner.is_null() {
            None
        } else {
            Some(Self {
                inner,
                _not_send: PhantomData,
            })
        }
    }

    /// Removes the `TDataStd_Name` attribute from `label`, if present.
    ///
    /// Returns `false` if the attribute was not present. Must be called
    /// inside an open [`Command`] scope.
    pub fn forget(_cmd: &Command<'_>, label: &OcLabel) -> bool {
        ffi::tdatastd_name_forget(&label.inner)
    }
}

impl std::fmt::Debug for OcName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OcName")
            .field("value", &self.get())
            .finish()
    }
}

// ── OcInteger ─────────────────────────────────────────────────────────────────

/// A `TDataStd_Integer` attribute handle — an `i32` attached to a label.
pub struct OcInteger {
    inner: cxx::UniquePtr<ffi::TDataStdIntegerHandle>,
    _not_send: PhantomData<*mut ()>,
}

impl OcInteger {
    /// Attaches or updates a `TDataStd_Integer` attribute on `label`.
    ///
    /// Must be called inside an open [`Command`] scope.
    pub fn set(_cmd: &Command<'_>, label: &OcLabel, value: i32) -> Result<Self, OcctError> {
        let inner = ffi::tdatastd_integer_set(&label.inner, value).map_err(OcctError::from)?;
        Ok(Self {
            inner,
            _not_send: PhantomData,
        })
    }

    /// Reads the integer value of this attribute.
    pub fn get(&self) -> i32 {
        ffi::tdatastd_integer_get(&self.inner)
    }

    /// Probes for a `TDataStd_Integer` attribute on `label`.
    ///
    /// Returns `None` when the attribute is not present.
    pub fn find(label: &OcLabel) -> Option<Self> {
        let inner = ffi::tdatastd_integer_find(&label.inner);
        if inner.is_null() {
            None
        } else {
            Some(Self {
                inner,
                _not_send: PhantomData,
            })
        }
    }

    /// Removes the `TDataStd_Integer` attribute from `label`, if present.
    ///
    /// Returns `false` if the attribute was not present. Must be called
    /// inside an open [`Command`] scope.
    pub fn forget(_cmd: &Command<'_>, label: &OcLabel) -> bool {
        ffi::tdatastd_integer_forget(&label.inner)
    }
}

impl std::fmt::Debug for OcInteger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OcInteger")
            .field("value", &self.get())
            .finish()
    }
}

// ── OcReal ────────────────────────────────────────────────────────────────────

/// A `TDataStd_Real` attribute handle — an `f64` attached to a label.
pub struct OcReal {
    inner: cxx::UniquePtr<ffi::TDataStdRealHandle>,
    _not_send: PhantomData<*mut ()>,
}

impl OcReal {
    /// Attaches or updates a `TDataStd_Real` attribute on `label`.
    ///
    /// Must be called inside an open [`Command`] scope.
    pub fn set(_cmd: &Command<'_>, label: &OcLabel, value: f64) -> Result<Self, OcctError> {
        let inner = ffi::tdatastd_real_set(&label.inner, value).map_err(OcctError::from)?;
        Ok(Self {
            inner,
            _not_send: PhantomData,
        })
    }

    /// Reads the real value of this attribute.
    pub fn get(&self) -> f64 {
        ffi::tdatastd_real_get(&self.inner)
    }

    /// Probes for a `TDataStd_Real` attribute on `label`.
    ///
    /// Returns `None` when the attribute is not present.
    pub fn find(label: &OcLabel) -> Option<Self> {
        let inner = ffi::tdatastd_real_find(&label.inner);
        if inner.is_null() {
            None
        } else {
            Some(Self {
                inner,
                _not_send: PhantomData,
            })
        }
    }

    /// Removes the `TDataStd_Real` attribute from `label`, if present.
    ///
    /// Returns `false` if the attribute was not present. Must be called
    /// inside an open [`Command`] scope.
    pub fn forget(_cmd: &Command<'_>, label: &OcLabel) -> bool {
        ffi::tdatastd_real_forget(&label.inner)
    }
}

impl std::fmt::Debug for OcReal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OcReal")
            .field("value", &self.get())
            .finish()
    }
}
// ── OcComment ─────────────────────────────────────────────────────────────────

/// A `TDataStd_Comment` attribute handle — a UTF-8 string attached to a label.
///
/// Construct via [`OcComment::set`] inside an open command scope.
/// Retrieve from an existing label via [`OcComment::find`].
pub struct OcComment {
    inner: cxx::UniquePtr<ffi::TDataStdCommentHandle>,
    _not_send: PhantomData<*mut ()>,
}

impl OcComment {
    /// Attaches or updates a `TDataStd_Comment` attribute on `label`.
    ///
    /// Must be called inside an open [`Command`] scope.
    pub fn set(_cmd: &Command<'_>, label: &OcLabel, value: &str) -> Result<Self, OcctError> {
        let inner = ffi::tdatastd_comment_set(&label.inner, value).map_err(OcctError::from)?;
        Ok(Self {
            inner,
            _not_send: PhantomData,
        })
    }

    /// Reads the string value of this attribute.
    pub fn get(&self) -> String {
        ffi::tdatastd_comment_get(&self.inner)
    }

    /// Probes for a `TDataStd_Comment` attribute on `label`.
    ///
    /// Returns `None` when the attribute is not present.
    /// No command scope required for read-only access.
    pub fn find(label: &OcLabel) -> Option<Self> {
        let inner = ffi::tdatastd_comment_find(&label.inner);
        if inner.is_null() {
            None
        } else {
            Some(Self {
                inner,
                _not_send: PhantomData,
            })
        }
    }

    /// Removes the `TDataStd_Comment` attribute from `label`, if present.
    ///
    /// Returns `false` if the attribute was not present. Must be called
    /// inside an open [`Command`] scope.
    pub fn forget(_cmd: &Command<'_>, label: &OcLabel) -> bool {
        ffi::tdatastd_comment_forget(&label.inner)
    }
}

impl std::fmt::Debug for OcComment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OcComment")
            .field("value", &self.get())
            .finish()
    }
}

// ── OcAsciiString ─────────────────────────────────────────────────────────────

/// A `TDataStd_AsciiString` attribute handle — an ASCII string attached to a label.
///
/// Despite the name, `TCollection_AsciiString` is an unvalidated 8-bit byte
/// buffer — any valid-UTF-8 `&str` round-trips unchanged through
/// [`set`](OcAsciiString::set)/[`get`](OcAsciiString::get) regardless of content.
///
/// Construct via [`OcAsciiString::set`] inside an open command scope.
/// Retrieve from an existing label via [`OcAsciiString::find`].
pub struct OcAsciiString {
    inner: cxx::UniquePtr<ffi::TDataStdAsciiStringHandle>,
    _not_send: PhantomData<*mut ()>,
}

impl OcAsciiString {
    /// Attaches or updates a `TDataStd_AsciiString` attribute on `label`.
    ///
    /// Must be called inside an open [`Command`] scope.
    pub fn set(_cmd: &Command<'_>, label: &OcLabel, value: &str) -> Result<Self, OcctError> {
        let inner = ffi::tdatastd_asciistring_set(&label.inner, value).map_err(OcctError::from)?;
        Ok(Self {
            inner,
            _not_send: PhantomData,
        })
    }

    /// Reads the ASCII string value of this attribute.
    pub fn get(&self) -> String {
        ffi::tdatastd_asciistring_get(&self.inner)
    }

    /// Probes for a `TDataStd_AsciiString` attribute on `label`.
    ///
    /// Returns `None` when the attribute is not present.
    /// No command scope required for read-only access.
    pub fn find(label: &OcLabel) -> Option<Self> {
        let inner = ffi::tdatastd_asciistring_find(&label.inner);
        if inner.is_null() {
            None
        } else {
            Some(Self {
                inner,
                _not_send: PhantomData,
            })
        }
    }

    /// Removes the `TDataStd_AsciiString` attribute from `label`, if present.
    ///
    /// Returns `false` if the attribute was not present. Must be called
    /// inside an open [`Command`] scope.
    pub fn forget(_cmd: &Command<'_>, label: &OcLabel) -> bool {
        ffi::tdatastd_asciistring_forget(&label.inner)
    }
}

impl std::fmt::Debug for OcAsciiString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OcAsciiString")
            .field("value", &self.get())
            .finish()
    }
}

// ── OcReferenceList ───────────────────────────────────────────────────────────

/// A `TDataStd_ReferenceList` attribute handle — an ordered list of label
/// references attached to a label.
///
/// Unlike the scalar attributes, [`set`](OcReferenceList::set) finds-or-creates
/// an *empty* list — there's no single value to set. Use
/// [`append`](OcReferenceList::append) to populate it.
///
/// Indices are 0-based and resolved by walking the underlying OCCT list on
/// each [`at`](OcReferenceList::at)/[`to_vec`](OcReferenceList::to_vec) call —
/// fine for the small argument/result lists this attribute is typically used
/// for, but O(n) per access.
pub struct OcReferenceList {
    inner: cxx::UniquePtr<ffi::TDataStdReferenceListHandle>,
    _not_send: PhantomData<*mut ()>,
}

impl OcReferenceList {
    /// Finds, or creates, an empty `TDataStd_ReferenceList` attribute on `label`.
    ///
    /// Must be called inside an open [`Command`] scope.
    pub fn set(_cmd: &Command<'_>, label: &OcLabel) -> Result<Self, OcctError> {
        let inner = ffi::tdatastd_referencelist_set(&label.inner).map_err(OcctError::from)?;
        Ok(Self {
            inner,
            _not_send: PhantomData,
        })
    }

    /// Probes for a `TDataStd_ReferenceList` attribute on `label`.
    ///
    /// Returns `None` when the attribute is not present.
    /// No command scope required for read-only access.
    pub fn find(label: &OcLabel) -> Option<Self> {
        let inner = ffi::tdatastd_referencelist_find(&label.inner);
        if inner.is_null() {
            None
        } else {
            Some(Self {
                inner,
                _not_send: PhantomData,
            })
        }
    }

    /// Removes the `TDataStd_ReferenceList` attribute from `label`, if present.
    ///
    /// Returns `false` if the attribute was not present. Must be called
    /// inside an open [`Command`] scope.
    pub fn forget(_cmd: &Command<'_>, label: &OcLabel) -> bool {
        ffi::tdatastd_referencelist_forget(&label.inner)
    }

    /// Number of label references in this list.
    pub fn extent(&self) -> i32 {
        ffi::tdatastd_referencelist_extent(&self.inner)
    }

    /// Returns `true` if this list contains no references.
    pub fn is_empty(&self) -> bool {
        ffi::tdatastd_referencelist_is_empty(&self.inner)
    }

    /// Returns the label at `index` (0-based).
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds (`>= extent()`).
    pub fn at(&self, index: i32) -> OcLabel {
        assert!(index >= 0 && index < self.extent(), "index out of bounds");
        OcLabel::from_ffi(ffi::tdatastd_referencelist_at(&self.inner, index))
    }

    /// Appends `value` to the end of this list.
    ///
    /// Must be called inside an open [`Command`] scope.
    pub fn append(&self, _cmd: &Command<'_>, value: &OcLabel) {
        ffi::tdatastd_referencelist_append(&self.inner, &value.inner);
    }

    /// Collects all label references into a `Vec`, in order.
    pub fn to_vec(&self) -> Vec<OcLabel> {
        let n = self.extent();
        (0..n)
            .map(|i| OcLabel::from_ffi(ffi::tdatastd_referencelist_at(&self.inner, i)))
            .collect()
    }
}

impl std::fmt::Debug for OcReferenceList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OcReferenceList")
            .field("extent", &self.extent())
            .finish()
    }
}

// ── OcReferenceArray ──────────────────────────────────────────────────────────

/// A `TDataStd_ReferenceArray` attribute handle — a fixed-length array of
/// label references attached to a label.
///
/// Indices are 0-based; internally this always calls
/// `TDataStd_ReferenceArray::Set` with bounds `[0, len-1]`, normalizing
/// OCCT's caller-chosen bounds to Rust's slice/Vec convention. Unlike
/// [`OcReferenceList`], [`value`](OcReferenceArray::value)/
/// [`set_value`](OcReferenceArray::set_value) are O(1) direct array access.
///
/// Elements are null labels until explicitly set via
/// [`set_value`](OcReferenceArray::set_value).
///
/// `len` must be >= 1. OCCT's underlying TColStd_Array1 storage requires
/// Lower <= Upper, so `Set(label, 0, -1)` (the `len == 0` case) raises
/// Standard_RangeError from `Init`; [`set`](Self::set) propagates this as
/// `Err`. For possibly-empty collections, use [`OcReferenceList`] instead.
pub struct OcReferenceArray {
    inner: cxx::UniquePtr<ffi::TDataStdReferenceArrayHandle>,
    _not_send: PhantomData<*mut ()>,
}

impl OcReferenceArray {
    /// Finds, or creates, a `TDataStd_ReferenceArray` attribute on `label`
    /// with `len` elements (0-based indices `0..len`).
    ///
    /// Must be called inside an open [`Command`] scope.
    pub fn set(_cmd: &Command<'_>, label: &OcLabel, len: i32) -> Result<Self, OcctError> {
        let inner = ffi::tdatastd_referencearray_set(&label.inner, len).map_err(OcctError::from)?;
        Ok(Self {
            inner,
            _not_send: PhantomData,
        })
    }

    /// Probes for a `TDataStd_ReferenceArray` attribute on `label`.
    ///
    /// Returns `None` when the attribute is not present.
    /// No command scope required for read-only access.
    pub fn find(label: &OcLabel) -> Option<Self> {
        let inner = ffi::tdatastd_referencearray_find(&label.inner);
        if inner.is_null() {
            None
        } else {
            Some(Self {
                inner,
                _not_send: PhantomData,
            })
        }
    }

    /// Removes the `TDataStd_ReferenceArray` attribute from `label`, if present.
    ///
    /// Returns `false` if the attribute was not present. Must be called
    /// inside an open [`Command`] scope.
    pub fn forget(_cmd: &Command<'_>, label: &OcLabel) -> bool {
        ffi::tdatastd_referencearray_forget(&label.inner)
    }

    /// Number of elements in this array (the `len` passed to [`set`](Self::set)).
    pub fn len(&self) -> i32 {
        ffi::tdatastd_referencearray_length(&self.inner)
    }

    /// Returns `true` if this array has zero elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the label at `index` (0-based).
    ///
    /// # Errors
    ///
    /// Returns `Err` if `index` is outside `[0, len()-1]`.
    pub fn value(&self, index: i32) -> Result<OcLabel, OcctError> {
        let inner =
            ffi::tdatastd_referencearray_value(&self.inner, index).map_err(OcctError::from)?;
        Ok(OcLabel::from_ffi(inner))
    }

    /// Sets the label at `index` (0-based).
    ///
    /// Must be called inside an open [`Command`] scope.
    ///
    /// # Errors
    ///
    /// Returns `Err` if `index` is outside `[0, len()-1]`.
    pub fn set_value(
        &self,
        _cmd: &Command<'_>,
        index: i32,
        value: &OcLabel,
    ) -> Result<(), OcctError> {
        ffi::tdatastd_referencearray_set_value(&self.inner, index, &value.inner)
            .map_err(OcctError::from)
    }

    /// Collects all elements into a `Vec`, in index order.
    pub fn to_vec(&self) -> Vec<OcLabel> {
        (0..self.len())
            .map(|i| {
                self.value(i)
                    .expect("index in [0, len()) is in bounds by construction")
            })
            .collect()
    }
}

impl std::fmt::Debug for OcReferenceArray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OcReferenceArray")
            .field("len", &self.len())
            .finish()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topo::{OcApplication, OcDocument};

    fn new_doc() -> (OcApplication, OcDocument) {
        let mut app = OcApplication::new();
        let mut doc = app.new_document("BinXCAF").unwrap();
        doc.set_undo_limit(10);
        (app, doc)
    }

    // ── OcName ──────────────────────────────────────────────────────────────

    #[test]
    fn name_round_trip() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            OcName::set(&cmd, &label, "hello").unwrap();
            cmd.commit().unwrap();
        }
        let attr = OcName::find(&label).expect("name attribute should be present");
        assert_eq!(attr.get(), "hello");
    }

    #[test]
    fn name_find_absent_returns_none() {
        let (_app, mut doc) = new_doc();
        let main = doc.main();
        let cmd = doc.begin_command().unwrap();
        let label = main.get_or_create_child(&cmd, 1);
        cmd.commit().unwrap();
        assert!(OcName::find(&label).is_none());
    }

    #[test]
    fn name_update_overwrites() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            OcName::set(&cmd, &label, "first").unwrap();
            cmd.commit().unwrap();
        }
        {
            let cmd = doc.begin_command().unwrap();
            OcName::set(&cmd, &label, "second").unwrap();
            cmd.commit().unwrap();
        }
        assert_eq!(OcName::find(&label).unwrap().get(), "second");
    }

    #[test]
    fn name_undo_restores() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            OcName::set(&cmd, &label, "before").unwrap();
            cmd.commit().unwrap();
        }
        {
            let cmd = doc.begin_command().unwrap();
            OcName::set(&cmd, &label, "after").unwrap();
            cmd.commit().unwrap();
        }
        doc.undo().unwrap();
        assert_eq!(OcName::find(&label).unwrap().get(), "before");
    }
    #[test]
    fn name_unicode_round_trip() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            OcName::set(&cmd, &label, "café").unwrap();
            cmd.commit().unwrap();
        }
        assert_eq!(OcName::find(&label).unwrap().get(), "café");
    }
    #[test]
    fn name_unicode_round_trip_non_bmp() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            // U+1F600 (4-byte UTF-8) — outside the BMP, exercises any
            // surrogate-pair handling in ExtendedString's UTF-8 decode.
            OcName::set(&cmd, &label, "😀").unwrap();
            cmd.commit().unwrap();
        }
        assert_eq!(OcName::find(&label).unwrap().get(), "😀");
    }

    // ── OcInteger ────────────────────────────────────────────────────────────

    #[test]
    fn integer_round_trip() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            OcInteger::set(&cmd, &label, 42).unwrap();
            cmd.commit().unwrap();
        }
        assert_eq!(OcInteger::find(&label).unwrap().get(), 42);
    }

    #[test]
    fn integer_find_absent_returns_none() {
        let (_app, mut doc) = new_doc();
        let main = doc.main();
        let cmd = doc.begin_command().unwrap();
        let label = main.get_or_create_child(&cmd, 1);
        cmd.commit().unwrap();
        assert!(OcInteger::find(&label).is_none());
    }

    #[test]
    fn integer_undo_restores() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            OcInteger::set(&cmd, &label, 1).unwrap();
            cmd.commit().unwrap();
        }
        {
            let cmd = doc.begin_command().unwrap();
            OcInteger::set(&cmd, &label, 2).unwrap();
            cmd.commit().unwrap();
        }
        doc.undo().unwrap();
        assert_eq!(OcInteger::find(&label).unwrap().get(), 1);
    }

    #[test]
    fn integer_forget_removes_attribute() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            OcInteger::set(&cmd, &label, 42).unwrap();
            cmd.commit().unwrap();
        }
        {
            let cmd = doc.begin_command().unwrap();
            assert!(OcInteger::forget(&cmd, &label));
            cmd.commit().unwrap();
        }
        assert!(OcInteger::find(&label).is_none());
    }

    #[test]
    fn integer_forget_absent_returns_false() {
        let (_app, mut doc) = new_doc();
        let main = doc.main();
        let cmd = doc.begin_command().unwrap();
        let label = main.get_or_create_child(&cmd, 1);
        assert!(!OcInteger::forget(&cmd, &label));
        cmd.commit().unwrap();
    }

    // ── OcReal ───────────────────────────────────────────────────────────────

    #[test]
    fn real_round_trip() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            OcReal::set(&cmd, &label, 3.14).unwrap();
            cmd.commit().unwrap();
        }
        let v = OcReal::find(&label).unwrap().get();
        assert!((v - 3.14).abs() < 1e-12);
    }

    #[test]
    fn real_find_absent_returns_none() {
        let (_app, mut doc) = new_doc();
        let main = doc.main();
        let cmd = doc.begin_command().unwrap();
        let label = main.get_or_create_child(&cmd, 1);
        cmd.commit().unwrap();
        assert!(OcReal::find(&label).is_none());
    }
    // ── OcComment ────────────────────────────────────────────────────────────

    #[test]
    fn comment_round_trip() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            OcComment::set(&cmd, &label, "a comment").unwrap();
            cmd.commit().unwrap();
        }
        assert_eq!(OcComment::find(&label).unwrap().get(), "a comment");
    }

    #[test]
    fn comment_find_absent_returns_none() {
        let (_app, mut doc) = new_doc();
        let main = doc.main();
        let cmd = doc.begin_command().unwrap();
        let label = main.get_or_create_child(&cmd, 1);
        cmd.commit().unwrap();
        assert!(OcComment::find(&label).is_none());
    }

    #[test]
    fn comment_update_overwrites() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            OcComment::set(&cmd, &label, "first").unwrap();
            cmd.commit().unwrap();
        }
        {
            let cmd = doc.begin_command().unwrap();
            OcComment::set(&cmd, &label, "second").unwrap();
            cmd.commit().unwrap();
        }
        assert_eq!(OcComment::find(&label).unwrap().get(), "second");
    }

    #[test]
    fn comment_undo_restores() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            OcComment::set(&cmd, &label, "before").unwrap();
            cmd.commit().unwrap();
        }
        {
            let cmd = doc.begin_command().unwrap();
            OcComment::set(&cmd, &label, "after").unwrap();
            cmd.commit().unwrap();
        }
        doc.undo().unwrap();
        assert_eq!(OcComment::find(&label).unwrap().get(), "before");
    }

    // ── OcAsciiString ────────────────────────────────────────────────────────

    #[test]
    fn asciistring_round_trip() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            OcAsciiString::set(&cmd, &label, "PART-001").unwrap();
            cmd.commit().unwrap();
        }
        assert_eq!(OcAsciiString::find(&label).unwrap().get(), "PART-001");
    }

    #[test]
    fn asciistring_find_absent_returns_none() {
        let (_app, mut doc) = new_doc();
        let main = doc.main();
        let cmd = doc.begin_command().unwrap();
        let label = main.get_or_create_child(&cmd, 1);
        cmd.commit().unwrap();
        assert!(OcAsciiString::find(&label).is_none());
    }

    #[test]
    fn asciistring_update_overwrites() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            OcAsciiString::set(&cmd, &label, "first").unwrap();
            cmd.commit().unwrap();
        }
        {
            let cmd = doc.begin_command().unwrap();
            OcAsciiString::set(&cmd, &label, "second").unwrap();
            cmd.commit().unwrap();
        }
        assert_eq!(OcAsciiString::find(&label).unwrap().get(), "second");
    }

    #[test]
    fn asciistring_undo_restores() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            OcAsciiString::set(&cmd, &label, "before").unwrap();
            cmd.commit().unwrap();
        }
        {
            let cmd = doc.begin_command().unwrap();
            OcAsciiString::set(&cmd, &label, "after").unwrap();
            cmd.commit().unwrap();
        }
        doc.undo().unwrap();
        assert_eq!(OcAsciiString::find(&label).unwrap().get(), "before");
    }

    #[test]
    fn asciistring_round_trip_preserves_non_ascii_bytes() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            OcAsciiString::set(&cmd, &label, "café").unwrap();
            cmd.commit().unwrap();
        }
        // TCollection_AsciiString is an unvalidated byte buffer — bytes
        // round-trip unchanged regardless of content.
        assert_eq!(OcAsciiString::find(&label).unwrap().get(), "café");
    }

    // ── Mixed ────────────────────────────────────────────────────────────────

    #[test]
    fn multiple_attributes_on_same_label() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            OcName::set(&cmd, &label, "part_a").unwrap();
            OcInteger::set(&cmd, &label, 7).unwrap();
            OcReal::set(&cmd, &label, 1.5).unwrap();
            cmd.commit().unwrap();
        }
        assert_eq!(OcName::find(&label).unwrap().get(), "part_a");
        assert_eq!(OcInteger::find(&label).unwrap().get(), 7);
        assert!((OcReal::find(&label).unwrap().get() - 1.5).abs() < 1e-12);
    }
    // ── OcReferenceList ──────────────────────────────────────────────────────

    #[test]
    fn referencelist_set_creates_empty_and_is_findable() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            let list = OcReferenceList::set(&cmd, &label).unwrap();
            assert!(list.is_empty());
            assert_eq!(list.extent(), 0);
            cmd.commit().unwrap();
        }
        let found = OcReferenceList::find(&label).expect("reference list should be present");
        assert!(found.is_empty());
    }

    #[test]
    fn referencelist_find_absent_returns_none() {
        let (_app, mut doc) = new_doc();
        let main = doc.main();
        let cmd = doc.begin_command().unwrap();
        let label = main.get_or_create_child(&cmd, 1);
        cmd.commit().unwrap();
        assert!(OcReferenceList::find(&label).is_none());
    }

    #[test]
    fn referencelist_forget_removes_attribute() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            OcReferenceList::set(&cmd, &label).unwrap();
            cmd.commit().unwrap();
        }
        {
            let cmd = doc.begin_command().unwrap();
            assert!(OcReferenceList::forget(&cmd, &label));
            cmd.commit().unwrap();
        }
        assert!(OcReferenceList::find(&label).is_none());
    }

    #[test]
    fn referencelist_append_preserves_order() {
        let (_app, mut doc) = new_doc();
        let list;
        let tags;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let list_label = main.get_or_create_child(&cmd, 1);
            let a = main.get_or_create_child(&cmd, 2);
            let b = main.get_or_create_child(&cmd, 3);
            let c = main.get_or_create_child(&cmd, 4);
            tags = vec![a.tag(), b.tag(), c.tag()];
            list = OcReferenceList::set(&cmd, &list_label).unwrap();
            list.append(&cmd, &a);
            list.append(&cmd, &b);
            list.append(&cmd, &c);
            cmd.commit().unwrap();
        }
        assert_eq!(list.extent(), 3);
        let got: Vec<i32> = list.to_vec().iter().map(|l| l.tag()).collect();
        assert_eq!(got, tags);
    }

    #[test]
    fn referencelist_undo_restores() {
        let (_app, mut doc) = new_doc();
        let list;
        let a_tag;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let list_label = main.get_or_create_child(&cmd, 1);
            let a = main.get_or_create_child(&cmd, 2);
            a_tag = a.tag();
            list = OcReferenceList::set(&cmd, &list_label).unwrap();
            list.append(&cmd, &a);
            cmd.commit().unwrap();
        }
        {
            let b = doc
                .main()
                .get_or_create_child(&doc.begin_command().unwrap(), 3);
            let cmd = doc.begin_command().unwrap();
            list.append(&cmd, &b);
            cmd.commit().unwrap();
        }
        assert_eq!(list.extent(), 2);
        doc.undo().unwrap();
        assert_eq!(list.extent(), 1);
        assert_eq!(list.at(0).tag(), a_tag);
    }
    // ── OcReferenceArray ─────────────────────────────────────────────────────

    #[test]
    fn referencearray_set_creates_with_length() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            let arr = OcReferenceArray::set(&cmd, &label, 3).unwrap();
            assert_eq!(arr.len(), 3);
            cmd.commit().unwrap();
        }
        let found = OcReferenceArray::find(&label).expect("reference array should be present");
        assert_eq!(found.len(), 3);
    }

    #[test]
    fn referencearray_find_absent_returns_none() {
        let (_app, mut doc) = new_doc();
        let main = doc.main();
        let cmd = doc.begin_command().unwrap();
        let label = main.get_or_create_child(&cmd, 1);
        cmd.commit().unwrap();
        assert!(OcReferenceArray::find(&label).is_none());
    }

    #[test]
    fn referencearray_forget_removes_attribute() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            OcReferenceArray::set(&cmd, &label, 2).unwrap();
            cmd.commit().unwrap();
        }
        {
            let cmd = doc.begin_command().unwrap();
            assert!(OcReferenceArray::forget(&cmd, &label));
            cmd.commit().unwrap();
        }
        assert!(OcReferenceArray::find(&label).is_none());
    }

    #[test]
    fn referencearray_set_value_and_value_round_trip() {
        let (_app, mut doc) = new_doc();
        let arr;
        let tags;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let arr_label = main.get_or_create_child(&cmd, 1);
            let a = main.get_or_create_child(&cmd, 2);
            let b = main.get_or_create_child(&cmd, 3);
            tags = vec![a.tag(), b.tag()];
            arr = OcReferenceArray::set(&cmd, &arr_label, 2).unwrap();
            arr.set_value(&cmd, 0, &a).unwrap();
            arr.set_value(&cmd, 1, &b).unwrap();
            cmd.commit().unwrap();
        }
        let got: Vec<i32> = arr.to_vec().iter().map(|l| l.tag()).collect();
        assert_eq!(got, tags);
    }

    #[test]
    fn referencearray_value_out_of_range_errors() {
        let (_app, mut doc) = new_doc();
        let arr;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let label = main.get_or_create_child(&cmd, 1);
            arr = OcReferenceArray::set(&cmd, &label, 2).unwrap();
            cmd.commit().unwrap();
        }
        assert!(arr.value(2).is_err());
        assert!(arr.value(-1).is_err());
    }

    #[test]
    fn referencearray_undo_restores() {
        let (_app, mut doc) = new_doc();
        let arr;
        let a_tag;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let arr_label = main.get_or_create_child(&cmd, 1);
            let a = main.get_or_create_child(&cmd, 2);
            a_tag = a.tag();
            arr = OcReferenceArray::set(&cmd, &arr_label, 1).unwrap();
            arr.set_value(&cmd, 0, &a).unwrap();
            cmd.commit().unwrap();
        }
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let b = main.get_or_create_child(&cmd, 3);
            arr.set_value(&cmd, 0, &b).unwrap();
            cmd.commit().unwrap();
        }
        doc.undo().unwrap();
        assert_eq!(arr.value(0).unwrap().tag(), a_tag);
    }

    #[test]
    fn referencearray_zero_length_is_err() {
        let (_app, mut doc) = new_doc();
        let main = doc.main();
        let cmd = doc.begin_command().unwrap();
        let label = main.get_or_create_child(&cmd, 1);
        // TColStd_Array1 requires Lower <= Upper; len == 0 -> Set(label, 0, -1)
        // raises Standard_RangeError from Init. Use OcReferenceList for
        // possibly-empty collections.
        assert!(OcReferenceArray::set(&cmd, &label, 0).is_err());
        cmd.abort().unwrap();
    }
}
