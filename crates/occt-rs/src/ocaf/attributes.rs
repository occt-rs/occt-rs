//! Standard TDF attributes: scalars (Name, Integer, Real, Comment,
//! AsciiString) and list/array attributes (ReferenceList, ReferenceArray,
//! RealArray, IntegerArray, BooleanArray, ByteArray, ExtStringArray,
//! IntegerList, RealList, ExtStringList, BooleanList), plus the
//! presence-only UAttribute marker (TDataStd_UAttribute) and the OcGuid
//! value type its caller-supplied identification uses, plus the scalar
//! groups (Integer/Real/String/Byte) of NamedData (TDataStd_NamedData).
//!
//! Each type wraps a `Handle(TDataStd_*)` shim.  The operations per type are:
//!
//! - **`set`** — attaches or updates the attribute on a label (inside a command).
//! - **`get`** — reads the current value from an already-retrieved attribute handle.
//! - **`find`** — probes whether the attribute is present on a label.
//! - **`forget`** — removes the attribute from a label (inside a command).
//!
//! GUIDs are kept on the C++ side; the Rust API never names them.
//!
//! ## Indexing: lists panic, arrays return `Result`
//!
//! The fixed arrays (`Oc*Array`) expose `value`/`set_value` returning `Result`,
//! mirroring OCCT's `Value(index)`/`SetValue(index, _)`, which raise
//! `OutOfRange`. The lists (`Oc*List`) expose `at`, which panics on an
//! out-of-bounds index like `Vec` indexing. OCCT's list types have no indexed
//! accessor, so `at` is a Rust convenience over `List()` and takes Rust's
//! indexing semantics rather than inventing a fallible one. The split is
//! deliberate and follows the underlying API shape.
//!
//! ## Writes take `&self`
//!
//! `append`/`set_value` mutate document state through `&self`, not `&mut self`.
//! An `Oc*List`/`Oc*Array` handle is a view onto an attribute owned by the
//! document's `TDF_Data`, not the storage itself — the write lands in the
//! document, and the `_cmd: &Command<'_>` argument is what gates it to an open
//! transaction. `&mut self` would falsely imply the handle has exclusive
//! ownership of the attribute (it doesn't; several handles to the same one can
//! coexist).

use std::marker::PhantomData;

use occt_sys::ffi;

use crate::error::{OcctError, OcctErrorKind};
use crate::ocaf::document::Command;
use crate::ocaf::label::OcLabel;

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

    pub(crate) fn inner(&self) -> &cxx::UniquePtr<ffi::TDataStdRealHandle> {
        &self.inner
    }
    pub(crate) fn from_ffi(inner: cxx::UniquePtr<ffi::TDataStdRealHandle>) -> Self {
        Self {
            inner,
            _not_send: std::marker::PhantomData,
        }
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

// ── OcIntegerList ────────────────────────────────────────────────────────────

/// A `TDataStd_IntegerList` attribute handle — an ordered list of `i32`
/// values attached to a label.
///
/// Same shape as [`OcReferenceList`]: [`set`](Self::set) finds-or-creates an
/// *empty* list; populate via [`append`](Self::append). Indices are 0-based
/// and resolved by walking the underlying OCCT list on each
/// [`at`](Self::at)/[`to_vec`](Self::to_vec) call — O(n) per access, fine
/// for small lists.
pub struct OcIntegerList {
    inner: cxx::UniquePtr<ffi::TDataStdIntegerListHandle>,
    _not_send: PhantomData<*mut ()>,
}

impl OcIntegerList {
    /// Finds, or creates, an empty `TDataStd_IntegerList` attribute on `label`.
    ///
    /// Must be called inside an open [`Command`] scope.
    pub fn set(_cmd: &Command<'_>, label: &OcLabel) -> Result<Self, OcctError> {
        let inner = ffi::tdatastd_integerlist_set(&label.inner).map_err(OcctError::from)?;
        Ok(Self {
            inner,
            _not_send: PhantomData,
        })
    }

    /// Probes for a `TDataStd_IntegerList` attribute on `label`.
    ///
    /// Returns `None` when the attribute is not present.
    /// No command scope required for read-only access.
    pub fn find(label: &OcLabel) -> Option<Self> {
        let inner = ffi::tdatastd_integerlist_find(&label.inner);
        if inner.is_null() {
            None
        } else {
            Some(Self {
                inner,
                _not_send: PhantomData,
            })
        }
    }

    /// Removes the `TDataStd_IntegerList` attribute from `label`, if present.
    ///
    /// Returns `false` if the attribute was not present. Must be called
    /// inside an open [`Command`] scope.
    pub fn forget(_cmd: &Command<'_>, label: &OcLabel) -> bool {
        ffi::tdatastd_integerlist_forget(&label.inner)
    }

    /// Number of elements in this list.
    pub fn extent(&self) -> i32 {
        ffi::tdatastd_integerlist_extent(&self.inner)
    }

    /// Returns `true` if this list contains no elements.
    pub fn is_empty(&self) -> bool {
        ffi::tdatastd_integerlist_is_empty(&self.inner)
    }

    /// Returns the element at `index` (0-based).
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds (`>= extent()`).
    pub fn at(&self, index: i32) -> i32 {
        assert!(index >= 0 && index < self.extent(), "index out of bounds");
        ffi::tdatastd_integerlist_at(&self.inner, index)
    }

    /// Appends `value` to the end of this list.
    ///
    /// Must be called inside an open [`Command`] scope.
    pub fn append(&self, _cmd: &Command<'_>, value: i32) {
        ffi::tdatastd_integerlist_append(&self.inner, value);
    }

    /// Collects all elements into a `Vec`, in order.
    pub fn to_vec(&self) -> Vec<i32> {
        (0..self.extent()).map(|i| self.at(i)).collect()
    }
}

impl std::fmt::Debug for OcIntegerList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OcIntegerList")
            .field("extent", &self.extent())
            .finish()
    }
}

// ── OcRealList ───────────────────────────────────────────────────────────────

/// A `TDataStd_RealList` attribute handle — an ordered list of `f64` values
/// attached to a label.
///
/// Same shape as [`OcIntegerList`]/[`OcReferenceList`]: [`set`](Self::set)
/// finds-or-creates an *empty* list; populate via [`append`](Self::append).
/// Indices are 0-based, O(n) per [`at`](Self::at)/[`to_vec`](Self::to_vec) call.
pub struct OcRealList {
    inner: cxx::UniquePtr<ffi::TDataStdRealListHandle>,
    _not_send: PhantomData<*mut ()>,
}

impl OcRealList {
    /// Finds, or creates, an empty `TDataStd_RealList` attribute on `label`.
    ///
    /// Must be called inside an open [`Command`] scope.
    pub fn set(_cmd: &Command<'_>, label: &OcLabel) -> Result<Self, OcctError> {
        let inner = ffi::tdatastd_reallist_set(&label.inner).map_err(OcctError::from)?;
        Ok(Self {
            inner,
            _not_send: PhantomData,
        })
    }

    /// Probes for a `TDataStd_RealList` attribute on `label`.
    ///
    /// Returns `None` when the attribute is not present.
    /// No command scope required for read-only access.
    pub fn find(label: &OcLabel) -> Option<Self> {
        let inner = ffi::tdatastd_reallist_find(&label.inner);
        if inner.is_null() {
            None
        } else {
            Some(Self {
                inner,
                _not_send: PhantomData,
            })
        }
    }

    /// Removes the `TDataStd_RealList` attribute from `label`, if present.
    ///
    /// Returns `false` if the attribute was not present. Must be called
    /// inside an open [`Command`] scope.
    pub fn forget(_cmd: &Command<'_>, label: &OcLabel) -> bool {
        ffi::tdatastd_reallist_forget(&label.inner)
    }

    /// Number of elements in this list.
    pub fn extent(&self) -> i32 {
        ffi::tdatastd_reallist_extent(&self.inner)
    }

    /// Returns `true` if this list contains no elements.
    pub fn is_empty(&self) -> bool {
        ffi::tdatastd_reallist_is_empty(&self.inner)
    }

    /// Returns the element at `index` (0-based).
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds (`>= extent()`).
    pub fn at(&self, index: i32) -> f64 {
        assert!(index >= 0 && index < self.extent(), "index out of bounds");
        ffi::tdatastd_reallist_at(&self.inner, index)
    }

    /// Appends `value` to the end of this list.
    ///
    /// Must be called inside an open [`Command`] scope.
    pub fn append(&self, _cmd: &Command<'_>, value: f64) {
        ffi::tdatastd_reallist_append(&self.inner, value);
    }

    /// Collects all elements into a `Vec`, in order.
    pub fn to_vec(&self) -> Vec<f64> {
        (0..self.extent()).map(|i| self.at(i)).collect()
    }
}

impl std::fmt::Debug for OcRealList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OcRealList")
            .field("extent", &self.extent())
            .finish()
    }
}

// ── OcExtStringList ──────────────────────────────────────────────────────────

/// A `TDataStd_ExtStringList` attribute handle — an ordered list of UTF-8
/// strings attached to a label.
///
/// Same shape as [`OcIntegerList`]/[`OcReferenceList`]: [`set`](Self::set)
/// finds-or-creates an *empty* list; populate via [`append`](Self::append).
/// Indices are 0-based, O(n) per [`at`](Self::at)/[`to_vec`](Self::to_vec) call.
///
/// Each element undergoes the same UTF-8 <-> `TCollection_ExtendedString`
/// conversion as [`OcName`]/[`OcComment`]/[`OcExtStringArray`]
/// (`isMultiByte = Standard_True`), applied per element.
pub struct OcExtStringList {
    inner: cxx::UniquePtr<ffi::TDataStdExtStringListHandle>,
    _not_send: PhantomData<*mut ()>,
}

impl OcExtStringList {
    /// Finds, or creates, an empty `TDataStd_ExtStringList` attribute on `label`.
    ///
    /// Must be called inside an open [`Command`] scope.
    pub fn set(_cmd: &Command<'_>, label: &OcLabel) -> Result<Self, OcctError> {
        let inner = ffi::tdatastd_extstringlist_set(&label.inner).map_err(OcctError::from)?;
        Ok(Self {
            inner,
            _not_send: PhantomData,
        })
    }

    /// Probes for a `TDataStd_ExtStringList` attribute on `label`.
    ///
    /// Returns `None` when the attribute is not present.
    /// No command scope required for read-only access.
    pub fn find(label: &OcLabel) -> Option<Self> {
        let inner = ffi::tdatastd_extstringlist_find(&label.inner);
        if inner.is_null() {
            None
        } else {
            Some(Self {
                inner,
                _not_send: PhantomData,
            })
        }
    }

    /// Removes the `TDataStd_ExtStringList` attribute from `label`, if present.
    ///
    /// Returns `false` if the attribute was not present. Must be called
    /// inside an open [`Command`] scope.
    pub fn forget(_cmd: &Command<'_>, label: &OcLabel) -> bool {
        ffi::tdatastd_extstringlist_forget(&label.inner)
    }

    /// Number of elements in this list.
    pub fn extent(&self) -> i32 {
        ffi::tdatastd_extstringlist_extent(&self.inner)
    }

    /// Returns `true` if this list contains no elements.
    pub fn is_empty(&self) -> bool {
        ffi::tdatastd_extstringlist_is_empty(&self.inner)
    }

    /// Returns the element at `index` (0-based).
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds (`>= extent()`).
    pub fn at(&self, index: i32) -> String {
        assert!(index >= 0 && index < self.extent(), "index out of bounds");
        ffi::tdatastd_extstringlist_at(&self.inner, index)
    }

    /// Appends `value` to the end of this list.
    ///
    /// Must be called inside an open [`Command`] scope.
    pub fn append(&self, _cmd: &Command<'_>, value: &str) {
        ffi::tdatastd_extstringlist_append(&self.inner, value);
    }

    /// Collects all elements into a `Vec`, in order.
    pub fn to_vec(&self) -> Vec<String> {
        (0..self.extent()).map(|i| self.at(i)).collect()
    }
}

impl std::fmt::Debug for OcExtStringList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OcExtStringList")
            .field("extent", &self.extent())
            .finish()
    }
}

// ── OcBooleanList ────────────────────────────────────────────────────────────

/// A `TDataStd_BooleanList` attribute handle — an ordered list of `bool`
/// values attached to a label.
///
/// Same shape as [`OcIntegerList`]/[`OcReferenceList`]: [`set`](Self::set)
/// finds-or-creates an *empty* list; populate via [`append`](Self::append).
/// Indices are 0-based, O(n) per [`at`](Self::at)/[`to_vec`](Self::to_vec) call.
///
/// Underlying storage is `TDataStd_ListOfByte` (1=true/0=false per OCCT's
/// documented convention) — converted to/from `bool` transparently. Unlike
/// [`OcBooleanArray`], this is a representation detail only, not a
/// bounds-safety concern: `NCollection_List<Standard_Byte>` has no packing.
pub struct OcBooleanList {
    inner: cxx::UniquePtr<ffi::TDataStdBooleanListHandle>,
    _not_send: PhantomData<*mut ()>,
}

impl OcBooleanList {
    /// Finds, or creates, an empty `TDataStd_BooleanList` attribute on `label`.
    ///
    /// Must be called inside an open [`Command`] scope.
    pub fn set(_cmd: &Command<'_>, label: &OcLabel) -> Result<Self, OcctError> {
        let inner = ffi::tdatastd_booleanlist_set(&label.inner).map_err(OcctError::from)?;
        Ok(Self {
            inner,
            _not_send: PhantomData,
        })
    }

    /// Probes for a `TDataStd_BooleanList` attribute on `label`.
    ///
    /// Returns `None` when the attribute is not present.
    /// No command scope required for read-only access.
    pub fn find(label: &OcLabel) -> Option<Self> {
        let inner = ffi::tdatastd_booleanlist_find(&label.inner);
        if inner.is_null() {
            None
        } else {
            Some(Self {
                inner,
                _not_send: PhantomData,
            })
        }
    }

    /// Removes the `TDataStd_BooleanList` attribute from `label`, if present.
    ///
    /// Returns `false` if the attribute was not present. Must be called
    /// inside an open [`Command`] scope.
    pub fn forget(_cmd: &Command<'_>, label: &OcLabel) -> bool {
        ffi::tdatastd_booleanlist_forget(&label.inner)
    }

    /// Number of elements in this list.
    pub fn extent(&self) -> i32 {
        ffi::tdatastd_booleanlist_extent(&self.inner)
    }

    /// Returns `true` if this list contains no elements.
    pub fn is_empty(&self) -> bool {
        ffi::tdatastd_booleanlist_is_empty(&self.inner)
    }

    /// Returns the element at `index` (0-based).
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds (`>= extent()`).
    pub fn at(&self, index: i32) -> bool {
        assert!(index >= 0 && index < self.extent(), "index out of bounds");
        ffi::tdatastd_booleanlist_at(&self.inner, index)
    }

    /// Appends `value` to the end of this list.
    ///
    /// Must be called inside an open [`Command`] scope.
    pub fn append(&self, _cmd: &Command<'_>, value: bool) {
        ffi::tdatastd_booleanlist_append(&self.inner, value);
    }

    /// Collects all elements into a `Vec`, in order.
    pub fn to_vec(&self) -> Vec<bool> {
        (0..self.extent()).map(|i| self.at(i)).collect()
    }
}

impl std::fmt::Debug for OcBooleanList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OcBooleanList")
            .field("extent", &self.extent())
            .finish()
    }
}

// ── OcGuid ───────────────────────────────────────────────────────────────────

/// A 128-bit GUID, as used for OCAF attribute identification (e.g.
/// [`OcUAttribute`]'s local ID).
///
/// Pure-Rust value type — construction, [`Display`](std::fmt::Display), and
/// [`FromStr`](std::str::FromStr) (canonical `8-4-4-4-12` hex form) are all
/// arithmetic on the fields below, with no FFI involved. `Standard_GUID` is
/// materialized via its 10-scalar constructor (matching `a32b`/`a16b1..3`/
/// `a8[0..6]` below) only at the point of an actual OCCT call
/// (`Set`/`FindAttribute`/`ForgetAttribute`) — the same zero-cost-abstraction
/// shape as the `gp_*` value types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OcGuid {
    a32b: u32,
    a16b1: u16,
    a16b2: u16,
    a16b3: u16,
    a8: [u8; 6],
}

impl OcGuid {
    /// Constructs a GUID from its raw fields, matching the grouping of
    /// `Standard_GUID`'s 10-scalar constructor
    /// (`a32b`-`a16b1`-`a16b2`-`a16b3`-`a8[0..6]`).
    pub const fn from_fields(a32b: u32, a16b1: u16, a16b2: u16, a16b3: u16, a8: [u8; 6]) -> Self {
        Self {
            a32b,
            a16b1,
            a16b2,
            a16b3,
            a8,
        }
    }
}

impl std::fmt::Display for OcGuid {
    /// Canonical `8-4-4-4-12` hex form, e.g.
    /// `12345678-1234-1234-1234-123456789abc`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:08x}-{:04x}-{:04x}-{:04x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            self.a32b,
            self.a16b1,
            self.a16b2,
            self.a16b3,
            self.a8[0],
            self.a8[1],
            self.a8[2],
            self.a8[3],
            self.a8[4],
            self.a8[5],
        )
    }
}

impl std::str::FromStr for OcGuid {
    type Err = std::num::ParseIntError;

    /// Parses the canonical `8-4-4-4-12` hex form.
    ///
    /// On malformed input (wrong group count or non-hex characters), returns
    /// a [`ParseIntError`](std::num::ParseIntError) from whichever group
    /// failed first — this does not distinguish "wrong group count" from
    /// "invalid hex digit" in the error itself. Acceptable for the primary
    /// use case (round-tripping a hardcoded feature-type GUID literal); a
    /// dedicated error type would be warranted if this ever parses untrusted
    /// input where the distinction matters to the caller.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('-').collect();
        let a32b = u32::from_str_radix(parts.first().copied().unwrap_or(""), 16)?;
        let a16b1 = u16::from_str_radix(parts.get(1).copied().unwrap_or(""), 16)?;
        let a16b2 = u16::from_str_radix(parts.get(2).copied().unwrap_or(""), 16)?;
        let a16b3 = u16::from_str_radix(parts.get(3).copied().unwrap_or(""), 16)?;
        let tail = parts.get(4).copied().unwrap_or("");
        let mut a8 = [0u8; 6];
        for (i, byte) in a8.iter_mut().enumerate() {
            let chunk = tail.get(i * 2..i * 2 + 2).unwrap_or("");
            *byte = u8::from_str_radix(chunk, 16)?;
        }
        Ok(Self {
            a32b,
            a16b1,
            a16b2,
            a16b3,
            a8,
        })
    }
}

// ── OcUAttribute ─────────────────────────────────────────────────────────────

/// `TDataStd_UAttribute` — a presence-only marker attribute identified by a
/// caller-supplied [`OcGuid`].
///
/// Unlike every other type in this module, the GUID is not a fixed per-type
/// `GetID()` baked into the shim — it's a parameter to every operation.
/// There is no value to retrieve: an `OcUAttribute` either is or isn't
/// present on a label for a given GUID. This is a zero-sized marker type;
/// all operations are associated functions, not methods on an instance.
pub struct OcUAttribute;

impl OcUAttribute {
    /// Finds, or creates, a `TDataStd_UAttribute` marker identified by `guid`
    /// on `label`.
    ///
    /// Must be called inside an open [`Command`] scope.
    pub fn set(_cmd: &Command<'_>, label: &OcLabel, guid: OcGuid) -> Result<(), OcctError> {
        ffi::tdatastd_uattribute_set(
            &label.inner,
            guid.a32b,
            guid.a16b1,
            guid.a16b2,
            guid.a16b3,
            guid.a8[0],
            guid.a8[1],
            guid.a8[2],
            guid.a8[3],
            guid.a8[4],
            guid.a8[5],
        )
        .map_err(OcctError::from)
    }

    /// Returns `true` if a `TDataStd_UAttribute` marker identified by `guid`
    /// is present on `label`.
    ///
    /// No command scope required for read-only access.
    pub fn is_present(label: &OcLabel, guid: OcGuid) -> bool {
        ffi::tdatastd_uattribute_is_present(
            &label.inner,
            guid.a32b,
            guid.a16b1,
            guid.a16b2,
            guid.a16b3,
            guid.a8[0],
            guid.a8[1],
            guid.a8[2],
            guid.a8[3],
            guid.a8[4],
            guid.a8[5],
        )
    }

    /// Removes the `TDataStd_UAttribute` marker identified by `guid` from
    /// `label`, if present.
    ///
    /// Returns `false` if it was not present. Must be called inside an open
    /// [`Command`] scope.
    pub fn forget(_cmd: &Command<'_>, label: &OcLabel, guid: OcGuid) -> bool {
        ffi::tdatastd_uattribute_forget(
            &label.inner,
            guid.a32b,
            guid.a16b1,
            guid.a16b2,
            guid.a16b3,
            guid.a8[0],
            guid.a8[1],
            guid.a8[2],
            guid.a8[3],
            guid.a8[4],
            guid.a8[5],
        )
    }
}

// ── OcNamedData ──────────────────────────────────────────────────────────────

/// A `TDataStd_NamedData` attribute handle — a keyed property bag holding
/// named integers, reals, strings, and bytes.
///
/// Unlike the scalar attributes, [`set`](OcNamedData::set) finds-or-creates
/// an attribute with no entries — populate via the per-type setters
/// ([`set_integer`](Self::set_integer), [`set_real`](Self::set_real),
/// [`set_string`](Self::set_string), [`set_byte`](Self::set_byte)).
///
/// Keys are UTF-8 strings, converted to `TCollection_ExtendedString` via the
/// same `isMultiByte = Standard_True` conversion as [`OcName`]/[`OcComment`],
/// applied on every call — every `has_*`/`get_*`/`set_*` does a key
/// conversion. String *values* (not just keys) undergo the same conversion.
///
/// `get_*` methods return a default (`0`, `0.0`, empty string) if `name` is
/// not present, per OCCT's own documented convention — call the
/// corresponding `has_*` first if the distinction matters.
///
/// This binds the four scalar-valued groups (Integer/Real/String/Byte).
/// `GetXContainer`/`ChangeX` (bulk access via `TColStd_DataMapOfStringX` /
/// `TDataStd_DataMapOfStringX` — C++-only collection types, same situation as
/// `TDF_LabelList`) and the two array-valued groups
/// (`ArrayOfIntegers`/`ArrayOfReals`, via `Handle(TColStd_HArray1OfX)`) are
/// deferred.
pub struct OcNamedData {
    inner: cxx::UniquePtr<ffi::TDataStdNamedDataHandle>,
    _not_send: PhantomData<*mut ()>,
}

impl OcNamedData {
    /// Finds, or creates, a `TDataStd_NamedData` attribute on `label` with no
    /// entries.
    ///
    /// Must be called inside an open [`Command`] scope.
    pub fn set(_cmd: &Command<'_>, label: &OcLabel) -> Result<Self, OcctError> {
        let inner = ffi::tdatastd_nameddata_set(&label.inner).map_err(OcctError::from)?;
        Ok(Self {
            inner,
            _not_send: PhantomData,
        })
    }

    /// Probes for a `TDataStd_NamedData` attribute on `label`.
    ///
    /// Returns `None` when the attribute is not present.
    /// No command scope required for read-only access.
    pub fn find(label: &OcLabel) -> Option<Self> {
        let inner = ffi::tdatastd_nameddata_find(&label.inner);
        if inner.is_null() {
            None
        } else {
            Some(Self {
                inner,
                _not_send: PhantomData,
            })
        }
    }

    /// Removes the `TDataStd_NamedData` attribute from `label`, if present.
    ///
    /// Returns `false` if the attribute was not present. Must be called
    /// inside an open [`Command`] scope.
    pub fn forget(_cmd: &Command<'_>, label: &OcLabel) -> bool {
        ffi::tdatastd_nameddata_forget(&label.inner)
    }

    // ── Integers ─────────────────────────────────────────────────────────

    /// Returns `true` if at least one named integer is present.
    pub fn has_integers(&self) -> bool {
        ffi::tdatastd_nameddata_has_integers(&self.inner)
    }

    /// Returns `true` if `name` has an associated integer value.
    pub fn has_integer(&self, name: &str) -> bool {
        ffi::tdatastd_nameddata_has_integer(&self.inner, name)
    }

    /// Returns the integer value for `name`, or `0` if not present.
    pub fn get_integer(&self, name: &str) -> i32 {
        ffi::tdatastd_nameddata_get_integer(&self.inner, name)
    }

    /// Sets the integer value for `name`, creating or overwriting it.
    ///
    /// Must be called inside an open [`Command`] scope.
    pub fn set_integer(&self, _cmd: &Command<'_>, name: &str, value: i32) {
        ffi::tdatastd_nameddata_set_integer(&self.inner, name, value);
    }

    // ── Reals ────────────────────────────────────────────────────────────

    /// Returns `true` if at least one named real is present.
    pub fn has_reals(&self) -> bool {
        ffi::tdatastd_nameddata_has_reals(&self.inner)
    }

    /// Returns `true` if `name` has an associated real value.
    pub fn has_real(&self, name: &str) -> bool {
        ffi::tdatastd_nameddata_has_real(&self.inner, name)
    }

    /// Returns the real value for `name`, or `0.0` if not present.
    pub fn get_real(&self, name: &str) -> f64 {
        ffi::tdatastd_nameddata_get_real(&self.inner, name)
    }

    /// Sets the real value for `name`, creating or overwriting it.
    ///
    /// Must be called inside an open [`Command`] scope.
    pub fn set_real(&self, _cmd: &Command<'_>, name: &str, value: f64) {
        ffi::tdatastd_nameddata_set_real(&self.inner, name, value);
    }

    // ── Strings ──────────────────────────────────────────────────────────

    /// Returns `true` if at least one named string is present.
    pub fn has_strings(&self) -> bool {
        ffi::tdatastd_nameddata_has_strings(&self.inner)
    }

    /// Returns `true` if `name` has an associated string value.
    pub fn has_string(&self, name: &str) -> bool {
        ffi::tdatastd_nameddata_has_string(&self.inner, name)
    }

    /// Returns the string value for `name`, or an empty string if not present.
    pub fn get_string(&self, name: &str) -> String {
        ffi::tdatastd_nameddata_get_string(&self.inner, name)
    }

    /// Sets the string value for `name`, creating or overwriting it.
    ///
    /// Must be called inside an open [`Command`] scope.
    pub fn set_string(&self, _cmd: &Command<'_>, name: &str, value: &str) {
        ffi::tdatastd_nameddata_set_string(&self.inner, name, value);
    }

    // ── Bytes ────────────────────────────────────────────────────────────

    /// Returns `true` if at least one named byte is present.
    pub fn has_bytes(&self) -> bool {
        ffi::tdatastd_nameddata_has_bytes(&self.inner)
    }

    /// Returns `true` if `name` has an associated byte value.
    pub fn has_byte(&self, name: &str) -> bool {
        ffi::tdatastd_nameddata_has_byte(&self.inner, name)
    }

    /// Returns the byte value for `name`, or `0` if not present.
    pub fn get_byte(&self, name: &str) -> u8 {
        ffi::tdatastd_nameddata_get_byte(&self.inner, name)
    }

    /// Sets the byte value for `name`, creating or overwriting it.
    ///
    /// Must be called inside an open [`Command`] scope.
    pub fn set_byte(&self, _cmd: &Command<'_>, name: &str, value: u8) {
        ffi::tdatastd_nameddata_set_byte(&self.inner, name, value);
    }
}

impl std::fmt::Debug for OcNamedData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OcNamedData")
            .field("has_integers", &self.has_integers())
            .field("has_reals", &self.has_reals())
            .field("has_strings", &self.has_strings())
            .field("has_bytes", &self.has_bytes())
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
    ///
    /// Always `>= 1`: `len < 1` is rejected at construction, so there is no
    /// `is_empty` — it would be a constant `false` and read as a live check
    /// that can never fire.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> i32 {
        ffi::tdatastd_referencearray_length(&self.inner)
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

// ── OcRealArray ──────────────────────────────────────────────────────────────

/// A `TDataStd_RealArray` attribute handle — a fixed-length array of `f64`
/// values attached to a label.
///
/// Same convention as [`OcReferenceArray`]: indices are 0-based, `len` must
/// be >= 1 (`TColStd_Array1` requires `Lower <= Upper`; `len == 0` raises
/// `Standard_RangeError` from `Init`, propagated as `Err`). Elements are
/// zero-initialized until explicitly set via
/// [`set_value`](OcRealArray::set_value).
///
/// OCCT's `Set` takes an `isDelta` parameter controlling undo-delta
/// computation for element modifications; occt-rs omits it, taking OCCT's
/// compiled-in default (`Standard_False`).
pub struct OcRealArray {
    inner: cxx::UniquePtr<ffi::TDataStdRealArrayHandle>,
    _not_send: PhantomData<*mut ()>,
}

impl OcRealArray {
    /// Finds, or creates, a `TDataStd_RealArray` attribute on `label` with
    /// `len` elements (0-based indices `0..len`).
    ///
    /// Must be called inside an open [`Command`] scope.
    ///
    /// # Errors
    ///
    /// Returns `Err` if `len < 1`.
    pub fn set(_cmd: &Command<'_>, label: &OcLabel, len: i32) -> Result<Self, OcctError> {
        let inner = ffi::tdatastd_realarray_set(&label.inner, len).map_err(OcctError::from)?;
        Ok(Self {
            inner,
            _not_send: PhantomData,
        })
    }

    /// Probes for a `TDataStd_RealArray` attribute on `label`.
    ///
    /// Returns `None` when the attribute is not present.
    /// No command scope required for read-only access.
    pub fn find(label: &OcLabel) -> Option<Self> {
        let inner = ffi::tdatastd_realarray_find(&label.inner);
        if inner.is_null() {
            None
        } else {
            Some(Self {
                inner,
                _not_send: PhantomData,
            })
        }
    }

    /// Removes the `TDataStd_RealArray` attribute from `label`, if present.
    ///
    /// Returns `false` if the attribute was not present. Must be called
    /// inside an open [`Command`] scope.
    pub fn forget(_cmd: &Command<'_>, label: &OcLabel) -> bool {
        ffi::tdatastd_realarray_forget(&label.inner)
    }

    /// Number of elements in this array (the `len` passed to [`set`](Self::set)).
    ///
    /// Always `>= 1`: `len < 1` is rejected at construction, so there is no
    /// `is_empty` — it would be a constant `false` and read as a live check
    /// that can never fire.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> i32 {
        ffi::tdatastd_realarray_length(&self.inner)
    }

    /// Returns the value at `index` (0-based).
    ///
    /// # Errors
    ///
    /// Returns `Err` if `index` is outside `[0, len()-1]`.
    pub fn value(&self, index: i32) -> Result<f64, OcctError> {
        ffi::tdatastd_realarray_value(&self.inner, index).map_err(OcctError::from)
    }

    /// Sets the value at `index` (0-based).
    ///
    /// Must be called inside an open [`Command`] scope.
    ///
    /// # Errors
    ///
    /// Returns `Err` if `index` is outside `[0, len()-1]`.
    pub fn set_value(&self, _cmd: &Command<'_>, index: i32, value: f64) -> Result<(), OcctError> {
        ffi::tdatastd_realarray_set_value(&self.inner, index, value).map_err(OcctError::from)
    }

    /// Collects all elements into a `Vec`, in index order.
    pub fn to_vec(&self) -> Vec<f64> {
        (0..self.len())
            .map(|i| {
                self.value(i)
                    .expect("index in [0, len()) is in bounds by construction")
            })
            .collect()
    }
}

impl std::fmt::Debug for OcRealArray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OcRealArray")
            .field("len", &self.len())
            .finish()
    }
}

// ── OcIntegerArray ───────────────────────────────────────────────────────────

/// A `TDataStd_IntegerArray` attribute handle — a fixed-length array of `i32`
/// values attached to a label.
///
/// Same convention as [`OcReferenceArray`]/[`OcRealArray`]: indices are
/// 0-based, `len` must be >= 1. Elements are zero-initialized until
/// explicitly set via [`set_value`](OcIntegerArray::set_value).
///
/// OCCT's `Set` takes an `isDelta` parameter controlling undo-delta
/// computation for element modifications; occt-rs omits it, taking OCCT's
/// compiled-in default (`Standard_False`).
pub struct OcIntegerArray {
    inner: cxx::UniquePtr<ffi::TDataStdIntegerArrayHandle>,
    _not_send: PhantomData<*mut ()>,
}

impl OcIntegerArray {
    /// Finds, or creates, a `TDataStd_IntegerArray` attribute on `label` with
    /// `len` elements (0-based indices `0..len`).
    ///
    /// Must be called inside an open [`Command`] scope.
    ///
    /// # Errors
    ///
    /// Returns `Err` if `len < 1`.
    pub fn set(_cmd: &Command<'_>, label: &OcLabel, len: i32) -> Result<Self, OcctError> {
        let inner = ffi::tdatastd_integerarray_set(&label.inner, len).map_err(OcctError::from)?;
        Ok(Self {
            inner,
            _not_send: PhantomData,
        })
    }

    /// Probes for a `TDataStd_IntegerArray` attribute on `label`.
    ///
    /// Returns `None` when the attribute is not present.
    /// No command scope required for read-only access.
    pub fn find(label: &OcLabel) -> Option<Self> {
        let inner = ffi::tdatastd_integerarray_find(&label.inner);
        if inner.is_null() {
            None
        } else {
            Some(Self {
                inner,
                _not_send: PhantomData,
            })
        }
    }

    /// Removes the `TDataStd_IntegerArray` attribute from `label`, if present.
    ///
    /// Returns `false` if the attribute was not present. Must be called
    /// inside an open [`Command`] scope.
    pub fn forget(_cmd: &Command<'_>, label: &OcLabel) -> bool {
        ffi::tdatastd_integerarray_forget(&label.inner)
    }

    /// Number of elements in this array (the `len` passed to [`set`](Self::set)).
    ///
    /// Always `>= 1`: `len < 1` is rejected at construction, so there is no
    /// `is_empty` — it would be a constant `false` and read as a live check
    /// that can never fire.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> i32 {
        ffi::tdatastd_integerarray_length(&self.inner)
    }

    /// Returns the value at `index` (0-based).
    ///
    /// # Errors
    ///
    /// Returns `Err` if `index` is outside `[0, len()-1]`.
    pub fn value(&self, index: i32) -> Result<i32, OcctError> {
        ffi::tdatastd_integerarray_value(&self.inner, index).map_err(OcctError::from)
    }

    /// Sets the value at `index` (0-based).
    ///
    /// Must be called inside an open [`Command`] scope.
    ///
    /// # Errors
    ///
    /// Returns `Err` if `index` is outside `[0, len()-1]`.
    pub fn set_value(&self, _cmd: &Command<'_>, index: i32, value: i32) -> Result<(), OcctError> {
        ffi::tdatastd_integerarray_set_value(&self.inner, index, value).map_err(OcctError::from)
    }

    /// Collects all elements into a `Vec`, in index order.
    pub fn to_vec(&self) -> Vec<i32> {
        (0..self.len())
            .map(|i| {
                self.value(i)
                    .expect("index in [0, len()) is in bounds by construction")
            })
            .collect()
    }
}

impl std::fmt::Debug for OcIntegerArray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OcIntegerArray")
            .field("len", &self.len())
            .finish()
    }
}

// ── OcBooleanArray ───────────────────────────────────────────────────────────

/// A `TDataStd_BooleanArray` attribute handle — a fixed-length array of
/// `bool` values attached to a label.
///
/// Same convention as [`OcReferenceArray`]/[`OcRealArray`]/[`OcIntegerArray`]:
/// indices are 0-based, `len` must be >= 1. Elements are `false` until
/// explicitly set via [`set_value`](OcBooleanArray::set_value).
///
/// Unlike the other array types, `TDataStd_BooleanArray::Set` takes no
/// `isDelta` parameter — nothing to omit here.
pub struct OcBooleanArray {
    inner: cxx::UniquePtr<ffi::TDataStdBooleanArrayHandle>,
    _not_send: PhantomData<*mut ()>,
}

impl OcBooleanArray {
    /// Finds, or creates, a `TDataStd_BooleanArray` attribute on `label` with
    /// `len` elements (0-based indices `0..len`).
    ///
    /// Must be called inside an open [`Command`] scope.
    ///
    /// # Errors
    ///
    /// Returns `Err` if `len < 1`.
    pub fn set(_cmd: &Command<'_>, label: &OcLabel, len: i32) -> Result<Self, OcctError> {
        let inner = ffi::tdatastd_booleanarray_set(&label.inner, len).map_err(OcctError::from)?;
        Ok(Self {
            inner,
            _not_send: PhantomData,
        })
    }

    /// Probes for a `TDataStd_BooleanArray` attribute on `label`.
    ///
    /// Returns `None` when the attribute is not present.
    /// No command scope required for read-only access.
    pub fn find(label: &OcLabel) -> Option<Self> {
        let inner = ffi::tdatastd_booleanarray_find(&label.inner);
        if inner.is_null() {
            None
        } else {
            Some(Self {
                inner,
                _not_send: PhantomData,
            })
        }
    }

    /// Removes the `TDataStd_BooleanArray` attribute from `label`, if present.
    ///
    /// Returns `false` if the attribute was not present. Must be called
    /// inside an open [`Command`] scope.
    pub fn forget(_cmd: &Command<'_>, label: &OcLabel) -> bool {
        ffi::tdatastd_booleanarray_forget(&label.inner)
    }

    /// Number of elements in this array (the `len` passed to [`set`](Self::set)).
    ///
    /// Always `>= 1`: `len < 1` is rejected at construction, so there is no
    /// `is_empty` — it would be a constant `false` and read as a live check
    /// that can never fire.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> i32 {
        ffi::tdatastd_booleanarray_length(&self.inner)
    }

    /// Returns the value at `index` (0-based).
    ///
    /// # Errors
    ///
    /// Returns `Err` if `index` is outside `[0, len()-1]`.
    ///
    /// This bound is checked on the Rust side before calling into OCCT:
    /// `TDataStd_BooleanArray` is internally bit-packed into a byte array
    /// (`TColStd_HArray1OfByte`), so OCCT's own `OutOfRange` check operates
    /// on byte bounds, not on `Length()` — an index within the same byte as
    /// the last valid element (e.g. index 2..7 for a 2-element array) would
    /// otherwise be silently accepted.
    pub fn value(&self, index: i32) -> Result<bool, OcctError> {
        if index < 0 || index >= self.len() {
            return Err(OcctError {
                kind: OcctErrorKind::OutOfRange,
                message: format!(
                    "TDataStd_BooleanArray::Value: index {index} out of range [0, {})",
                    self.len()
                ),
            });
        }
        ffi::tdatastd_booleanarray_value(&self.inner, index).map_err(OcctError::from)
    }

    /// Sets the value at `index` (0-based).
    ///
    /// Must be called inside an open [`Command`] scope.
    ///
    /// # Errors
    ///
    /// Returns `Err` if `index` is outside `[0, len()-1]`. See [`value`](Self::value)
    /// for why this is checked on the Rust side rather than relying on OCCT.
    pub fn set_value(&self, _cmd: &Command<'_>, index: i32, value: bool) -> Result<(), OcctError> {
        if index < 0 || index >= self.len() {
            return Err(OcctError {
                kind: OcctErrorKind::OutOfRange,
                message: format!(
                    "TDataStd_BooleanArray::SetValue: index {index} out of range [0, {})",
                    self.len()
                ),
            });
        }
        ffi::tdatastd_booleanarray_set_value(&self.inner, index, value).map_err(OcctError::from)
    }

    /// Collects all elements into a `Vec`, in index order.
    pub fn to_vec(&self) -> Vec<bool> {
        (0..self.len())
            .map(|i| {
                self.value(i)
                    .expect("index in [0, len()) is in bounds by construction")
            })
            .collect()
    }
}

impl std::fmt::Debug for OcBooleanArray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OcBooleanArray")
            .field("len", &self.len())
            .finish()
    }
}

// ── OcByteArray ──────────────────────────────────────────────────────────────

/// A `TDataStd_ByteArray` attribute handle — a fixed-length array of `u8`
/// values attached to a label.
///
/// Same convention as [`OcReferenceArray`]/[`OcRealArray`]/[`OcIntegerArray`]:
/// indices are 0-based, `len` must be >= 1. Elements are zero-initialized
/// until explicitly set via [`set_value`](OcByteArray::set_value).
///
/// OCCT's `Set` takes an `isDelta` parameter controlling undo-delta
/// computation for element modifications; occt-rs omits it, taking OCCT's
/// compiled-in default (`Standard_False`).
pub struct OcByteArray {
    inner: cxx::UniquePtr<ffi::TDataStdByteArrayHandle>,
    _not_send: PhantomData<*mut ()>,
}

impl OcByteArray {
    /// Finds, or creates, a `TDataStd_ByteArray` attribute on `label` with
    /// `len` elements (0-based indices `0..len`).
    ///
    /// Must be called inside an open [`Command`] scope.
    ///
    /// # Errors
    ///
    /// Returns `Err` if `len < 1`.
    pub fn set(_cmd: &Command<'_>, label: &OcLabel, len: i32) -> Result<Self, OcctError> {
        let inner = ffi::tdatastd_bytearray_set(&label.inner, len).map_err(OcctError::from)?;
        Ok(Self {
            inner,
            _not_send: PhantomData,
        })
    }

    /// Probes for a `TDataStd_ByteArray` attribute on `label`.
    ///
    /// Returns `None` when the attribute is not present.
    /// No command scope required for read-only access.
    pub fn find(label: &OcLabel) -> Option<Self> {
        let inner = ffi::tdatastd_bytearray_find(&label.inner);
        if inner.is_null() {
            None
        } else {
            Some(Self {
                inner,
                _not_send: PhantomData,
            })
        }
    }

    /// Removes the `TDataStd_ByteArray` attribute from `label`, if present.
    ///
    /// Returns `false` if the attribute was not present. Must be called
    /// inside an open [`Command`] scope.
    pub fn forget(_cmd: &Command<'_>, label: &OcLabel) -> bool {
        ffi::tdatastd_bytearray_forget(&label.inner)
    }

    /// Number of elements in this array (the `len` passed to [`set`](Self::set)).
    ///
    /// Always `>= 1`: `len < 1` is rejected at construction, so there is no
    /// `is_empty` — it would be a constant `false` and read as a live check
    /// that can never fire.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> i32 {
        ffi::tdatastd_bytearray_length(&self.inner)
    }

    /// Returns the value at `index` (0-based).
    ///
    /// # Errors
    ///
    /// Returns `Err` if `index` is outside `[0, len()-1]`.
    pub fn value(&self, index: i32) -> Result<u8, OcctError> {
        ffi::tdatastd_bytearray_value(&self.inner, index).map_err(OcctError::from)
    }

    /// Sets the value at `index` (0-based).
    ///
    /// Must be called inside an open [`Command`] scope.
    ///
    /// # Errors
    ///
    /// Returns `Err` if `index` is outside `[0, len()-1]`.
    pub fn set_value(&self, _cmd: &Command<'_>, index: i32, value: u8) -> Result<(), OcctError> {
        ffi::tdatastd_bytearray_set_value(&self.inner, index, value).map_err(OcctError::from)
    }

    /// Collects all elements into a `Vec`, in index order.
    pub fn to_vec(&self) -> Vec<u8> {
        (0..self.len())
            .map(|i| {
                self.value(i)
                    .expect("index in [0, len()) is in bounds by construction")
            })
            .collect()
    }
}

impl std::fmt::Debug for OcByteArray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OcByteArray")
            .field("len", &self.len())
            .finish()
    }
}

// ── OcExtStringArray ─────────────────────────────────────────────────────────

/// A `TDataStd_ExtStringArray` attribute handle — a fixed-length array of
/// UTF-8 strings attached to a label.
///
/// Same convention as [`OcReferenceArray`]/[`OcRealArray`]/[`OcIntegerArray`]:
/// indices are 0-based, `len` must be >= 1. Elements are empty strings until
/// explicitly set via [`set_value`](OcExtStringArray::set_value).
///
/// Each element undergoes the same UTF-8 <-> `TCollection_ExtendedString`
/// conversion as [`OcName`]/[`OcComment`] (`isMultiByte = Standard_True`),
/// applied per index — confirmed correct for both BMP and non-BMP input via
/// those types' round-trip tests.
///
/// OCCT's `Set` takes an `isDelta` parameter controlling undo-delta
/// computation for element modifications; occt-rs omits it, taking OCCT's
/// compiled-in default (`Standard_False`).
pub struct OcExtStringArray {
    inner: cxx::UniquePtr<ffi::TDataStdExtStringArrayHandle>,
    _not_send: PhantomData<*mut ()>,
}

impl OcExtStringArray {
    /// Finds, or creates, a `TDataStd_ExtStringArray` attribute on `label`
    /// with `len` elements (0-based indices `0..len`).
    ///
    /// Must be called inside an open [`Command`] scope.
    ///
    /// # Errors
    ///
    /// Returns `Err` if `len < 1`.
    pub fn set(_cmd: &Command<'_>, label: &OcLabel, len: i32) -> Result<Self, OcctError> {
        let inner = ffi::tdatastd_extstringarray_set(&label.inner, len).map_err(OcctError::from)?;
        Ok(Self {
            inner,
            _not_send: PhantomData,
        })
    }

    /// Probes for a `TDataStd_ExtStringArray` attribute on `label`.
    ///
    /// Returns `None` when the attribute is not present.
    /// No command scope required for read-only access.
    pub fn find(label: &OcLabel) -> Option<Self> {
        let inner = ffi::tdatastd_extstringarray_find(&label.inner);
        if inner.is_null() {
            None
        } else {
            Some(Self {
                inner,
                _not_send: PhantomData,
            })
        }
    }

    /// Removes the `TDataStd_ExtStringArray` attribute from `label`, if present.
    ///
    /// Returns `false` if the attribute was not present. Must be called
    /// inside an open [`Command`] scope.
    pub fn forget(_cmd: &Command<'_>, label: &OcLabel) -> bool {
        ffi::tdatastd_extstringarray_forget(&label.inner)
    }

    /// Number of elements in this array (the `len` passed to [`set`](Self::set)).
    ///
    /// Always `>= 1`: `len < 1` is rejected at construction, so there is no
    /// `is_empty` — it would be a constant `false` and read as a live check
    /// that can never fire.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> i32 {
        ffi::tdatastd_extstringarray_length(&self.inner)
    }

    /// Returns the string at `index` (0-based).
    ///
    /// # Errors
    ///
    /// Returns `Err` if `index` is outside `[0, len()-1]`.
    pub fn value(&self, index: i32) -> Result<String, OcctError> {
        ffi::tdatastd_extstringarray_value(&self.inner, index).map_err(OcctError::from)
    }

    /// Sets the string at `index` (0-based).
    ///
    /// Must be called inside an open [`Command`] scope.
    ///
    /// # Errors
    ///
    /// Returns `Err` if `index` is outside `[0, len()-1]`.
    pub fn set_value(&self, _cmd: &Command<'_>, index: i32, value: &str) -> Result<(), OcctError> {
        ffi::tdatastd_extstringarray_set_value(&self.inner, index, value).map_err(OcctError::from)
    }

    /// Collects all elements into a `Vec`, in index order.
    pub fn to_vec(&self) -> Vec<String> {
        (0..self.len())
            .map(|i| {
                self.value(i)
                    .expect("index in [0, len()) is in bounds by construction")
            })
            .collect()
    }
}

impl std::fmt::Debug for OcExtStringArray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OcExtStringArray")
            .field("len", &self.len())
            .finish()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ocaf::{OcApplication, OcDocument};

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
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let b = main.get_or_create_child(&cmd, 3);
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
    // ── OcRealArray ──────────────────────────────────────────────────────────

    #[test]
    fn realarray_set_creates_with_length() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            let arr = OcRealArray::set(&cmd, &label, 3).unwrap();
            assert_eq!(arr.len(), 3);
            cmd.commit().unwrap();
        }
        let found = OcRealArray::find(&label).expect("real array should be present");
        assert_eq!(found.len(), 3);
    }

    #[test]
    fn realarray_find_absent_returns_none() {
        let (_app, mut doc) = new_doc();
        let main = doc.main();
        let cmd = doc.begin_command().unwrap();
        let label = main.get_or_create_child(&cmd, 1);
        cmd.commit().unwrap();
        assert!(OcRealArray::find(&label).is_none());
    }

    #[test]
    fn realarray_forget_removes_attribute() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            OcRealArray::set(&cmd, &label, 2).unwrap();
            cmd.commit().unwrap();
        }
        {
            let cmd = doc.begin_command().unwrap();
            assert!(OcRealArray::forget(&cmd, &label));
            cmd.commit().unwrap();
        }
        assert!(OcRealArray::find(&label).is_none());
    }

    #[test]
    fn realarray_set_value_and_value_round_trip() {
        let (_app, mut doc) = new_doc();
        let arr;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let label = main.get_or_create_child(&cmd, 1);
            arr = OcRealArray::set(&cmd, &label, 2).unwrap();
            arr.set_value(&cmd, 0, 1.5).unwrap();
            arr.set_value(&cmd, 1, -2.25).unwrap();
            cmd.commit().unwrap();
        }
        let got = arr.to_vec();
        assert!((got[0] - 1.5).abs() < 1e-12);
        assert!((got[1] - (-2.25)).abs() < 1e-12);
    }

    #[test]
    fn realarray_value_out_of_range_errors() {
        let (_app, mut doc) = new_doc();
        let arr;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let label = main.get_or_create_child(&cmd, 1);
            arr = OcRealArray::set(&cmd, &label, 2).unwrap();
            cmd.commit().unwrap();
        }
        assert!(arr.value(2).is_err());
        assert!(arr.value(-1).is_err());
    }

    #[test]
    fn realarray_undo_restores() {
        let (_app, mut doc) = new_doc();
        let arr;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let label = main.get_or_create_child(&cmd, 1);
            arr = OcRealArray::set(&cmd, &label, 1).unwrap();
            arr.set_value(&cmd, 0, 1.0).unwrap();
            cmd.commit().unwrap();
        }
        {
            let cmd = doc.begin_command().unwrap();
            arr.set_value(&cmd, 0, 2.0).unwrap();
            cmd.commit().unwrap();
        }
        doc.undo().unwrap();
        assert!((arr.value(0).unwrap() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn realarray_zero_length_is_err() {
        let (_app, mut doc) = new_doc();
        let main = doc.main();
        let cmd = doc.begin_command().unwrap();
        let label = main.get_or_create_child(&cmd, 1);
        assert!(OcRealArray::set(&cmd, &label, 0).is_err());
        cmd.abort().unwrap();
    }

    // ── OcIntegerArray ───────────────────────────────────────────────────────

    #[test]
    fn integerarray_set_creates_with_length() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            let arr = OcIntegerArray::set(&cmd, &label, 3).unwrap();
            assert_eq!(arr.len(), 3);
            cmd.commit().unwrap();
        }
        let found = OcIntegerArray::find(&label).expect("integer array should be present");
        assert_eq!(found.len(), 3);
    }

    #[test]
    fn integerarray_find_absent_returns_none() {
        let (_app, mut doc) = new_doc();
        let main = doc.main();
        let cmd = doc.begin_command().unwrap();
        let label = main.get_or_create_child(&cmd, 1);
        cmd.commit().unwrap();
        assert!(OcIntegerArray::find(&label).is_none());
    }

    #[test]
    fn integerarray_forget_removes_attribute() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            OcIntegerArray::set(&cmd, &label, 2).unwrap();
            cmd.commit().unwrap();
        }
        {
            let cmd = doc.begin_command().unwrap();
            assert!(OcIntegerArray::forget(&cmd, &label));
            cmd.commit().unwrap();
        }
        assert!(OcIntegerArray::find(&label).is_none());
    }

    #[test]
    fn integerarray_set_value_and_value_round_trip() {
        let (_app, mut doc) = new_doc();
        let arr;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let label = main.get_or_create_child(&cmd, 1);
            arr = OcIntegerArray::set(&cmd, &label, 2).unwrap();
            arr.set_value(&cmd, 0, 7).unwrap();
            arr.set_value(&cmd, 1, -3).unwrap();
            cmd.commit().unwrap();
        }
        assert_eq!(arr.to_vec(), vec![7, -3]);
    }

    #[test]
    fn integerarray_value_out_of_range_errors() {
        let (_app, mut doc) = new_doc();
        let arr;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let label = main.get_or_create_child(&cmd, 1);
            arr = OcIntegerArray::set(&cmd, &label, 2).unwrap();
            cmd.commit().unwrap();
        }
        assert!(arr.value(2).is_err());
        assert!(arr.value(-1).is_err());
    }

    #[test]
    fn integerarray_undo_restores() {
        let (_app, mut doc) = new_doc();
        let arr;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let label = main.get_or_create_child(&cmd, 1);
            arr = OcIntegerArray::set(&cmd, &label, 1).unwrap();
            arr.set_value(&cmd, 0, 1).unwrap();
            cmd.commit().unwrap();
        }
        {
            let cmd = doc.begin_command().unwrap();
            arr.set_value(&cmd, 0, 2).unwrap();
            cmd.commit().unwrap();
        }
        doc.undo().unwrap();
        assert_eq!(arr.value(0).unwrap(), 1);
    }

    #[test]
    fn integerarray_zero_length_is_err() {
        let (_app, mut doc) = new_doc();
        let main = doc.main();
        let cmd = doc.begin_command().unwrap();
        let label = main.get_or_create_child(&cmd, 1);
        assert!(OcIntegerArray::set(&cmd, &label, 0).is_err());
        cmd.abort().unwrap();
    }
    // ── OcBooleanArray ───────────────────────────────────────────────────────

    #[test]
    fn booleanarray_set_creates_with_length() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            let arr = OcBooleanArray::set(&cmd, &label, 3).unwrap();
            assert_eq!(arr.len(), 3);
            cmd.commit().unwrap();
        }
        let found = OcBooleanArray::find(&label).expect("boolean array should be present");
        assert_eq!(found.len(), 3);
    }

    #[test]
    fn booleanarray_find_absent_returns_none() {
        let (_app, mut doc) = new_doc();
        let main = doc.main();
        let cmd = doc.begin_command().unwrap();
        let label = main.get_or_create_child(&cmd, 1);
        cmd.commit().unwrap();
        assert!(OcBooleanArray::find(&label).is_none());
    }

    #[test]
    fn booleanarray_forget_removes_attribute() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            OcBooleanArray::set(&cmd, &label, 2).unwrap();
            cmd.commit().unwrap();
        }
        {
            let cmd = doc.begin_command().unwrap();
            assert!(OcBooleanArray::forget(&cmd, &label));
            cmd.commit().unwrap();
        }
        assert!(OcBooleanArray::find(&label).is_none());
    }

    #[test]
    fn booleanarray_set_value_and_value_round_trip() {
        let (_app, mut doc) = new_doc();
        let arr;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let label = main.get_or_create_child(&cmd, 1);
            arr = OcBooleanArray::set(&cmd, &label, 2).unwrap();
            arr.set_value(&cmd, 0, true).unwrap();
            arr.set_value(&cmd, 1, false).unwrap();
            cmd.commit().unwrap();
        }
        assert_eq!(arr.to_vec(), vec![true, false]);
    }

    #[test]
    fn booleanarray_value_out_of_range_errors() {
        let (_app, mut doc) = new_doc();
        let arr;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let label = main.get_or_create_child(&cmd, 1);
            arr = OcBooleanArray::set(&cmd, &label, 2).unwrap();
            cmd.commit().unwrap();
        }
        assert!(arr.value(2).is_err());
        assert!(arr.value(-1).is_err());
    }
    #[test]
    fn booleanarray_value_oob_crosses_byte_boundary() {
        // len 9 spans two packed bytes (indices 0..=8); indices 9..=15 occupy
        // real, allocated slack in byte 1, so OCCT's byte-level OutOfRange
        // would accept them. Only the Rust-side guard rejects these.
        let (_app, mut doc) = new_doc();
        let arr;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let label = main.get_or_create_child(&cmd, 1);
            arr = OcBooleanArray::set(&cmd, &label, 9).unwrap();
            cmd.commit().unwrap();
        }
        // Last valid index is fine.
        assert!(arr.value(8).is_ok());
        // Upper-byte slack — would be silently accepted without the guard.
        assert!(arr.value(9).is_err());
        assert!(arr.value(15).is_err());
    }

    #[test]
    fn booleanarray_undo_restores() {
        let (_app, mut doc) = new_doc();
        let arr;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let label = main.get_or_create_child(&cmd, 1);
            arr = OcBooleanArray::set(&cmd, &label, 1).unwrap();
            arr.set_value(&cmd, 0, true).unwrap();
            cmd.commit().unwrap();
        }
        {
            let cmd = doc.begin_command().unwrap();
            arr.set_value(&cmd, 0, false).unwrap();
            cmd.commit().unwrap();
        }
        doc.undo().unwrap();
        assert_eq!(arr.value(0).unwrap(), true);
    }

    #[test]
    fn booleanarray_zero_length_is_err() {
        let (_app, mut doc) = new_doc();
        let main = doc.main();
        let cmd = doc.begin_command().unwrap();
        let label = main.get_or_create_child(&cmd, 1);
        assert!(OcBooleanArray::set(&cmd, &label, 0).is_err());
        cmd.abort().unwrap();
    }

    // ── OcByteArray ──────────────────────────────────────────────────────────

    #[test]
    fn bytearray_set_creates_with_length() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            let arr = OcByteArray::set(&cmd, &label, 3).unwrap();
            assert_eq!(arr.len(), 3);
            cmd.commit().unwrap();
        }
        let found = OcByteArray::find(&label).expect("byte array should be present");
        assert_eq!(found.len(), 3);
    }

    #[test]
    fn bytearray_find_absent_returns_none() {
        let (_app, mut doc) = new_doc();
        let main = doc.main();
        let cmd = doc.begin_command().unwrap();
        let label = main.get_or_create_child(&cmd, 1);
        cmd.commit().unwrap();
        assert!(OcByteArray::find(&label).is_none());
    }

    #[test]
    fn bytearray_forget_removes_attribute() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            OcByteArray::set(&cmd, &label, 2).unwrap();
            cmd.commit().unwrap();
        }
        {
            let cmd = doc.begin_command().unwrap();
            assert!(OcByteArray::forget(&cmd, &label));
            cmd.commit().unwrap();
        }
        assert!(OcByteArray::find(&label).is_none());
    }

    #[test]
    fn bytearray_set_value_and_value_round_trip() {
        let (_app, mut doc) = new_doc();
        let arr;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let label = main.get_or_create_child(&cmd, 1);
            arr = OcByteArray::set(&cmd, &label, 2).unwrap();
            arr.set_value(&cmd, 0, 0).unwrap();
            arr.set_value(&cmd, 1, 255).unwrap();
            cmd.commit().unwrap();
        }
        assert_eq!(arr.to_vec(), vec![0u8, 255u8]);
    }

    #[test]
    fn bytearray_value_out_of_range_errors() {
        let (_app, mut doc) = new_doc();
        let arr;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let label = main.get_or_create_child(&cmd, 1);
            arr = OcByteArray::set(&cmd, &label, 2).unwrap();
            cmd.commit().unwrap();
        }
        assert!(arr.value(2).is_err());
        assert!(arr.value(-1).is_err());
    }

    #[test]
    fn bytearray_undo_restores() {
        let (_app, mut doc) = new_doc();
        let arr;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let label = main.get_or_create_child(&cmd, 1);
            arr = OcByteArray::set(&cmd, &label, 1).unwrap();
            arr.set_value(&cmd, 0, 1).unwrap();
            cmd.commit().unwrap();
        }
        {
            let cmd = doc.begin_command().unwrap();
            arr.set_value(&cmd, 0, 2).unwrap();
            cmd.commit().unwrap();
        }
        doc.undo().unwrap();
        assert_eq!(arr.value(0).unwrap(), 1);
    }

    #[test]
    fn bytearray_zero_length_is_err() {
        let (_app, mut doc) = new_doc();
        let main = doc.main();
        let cmd = doc.begin_command().unwrap();
        let label = main.get_or_create_child(&cmd, 1);
        assert!(OcByteArray::set(&cmd, &label, 0).is_err());
        cmd.abort().unwrap();
    }

    // ── OcExtStringArray ─────────────────────────────────────────────────────

    #[test]
    fn extstringarray_set_creates_with_length() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            let arr = OcExtStringArray::set(&cmd, &label, 3).unwrap();
            assert_eq!(arr.len(), 3);
            cmd.commit().unwrap();
        }
        let found = OcExtStringArray::find(&label).expect("ext string array should be present");
        assert_eq!(found.len(), 3);
    }

    #[test]
    fn extstringarray_find_absent_returns_none() {
        let (_app, mut doc) = new_doc();
        let main = doc.main();
        let cmd = doc.begin_command().unwrap();
        let label = main.get_or_create_child(&cmd, 1);
        cmd.commit().unwrap();
        assert!(OcExtStringArray::find(&label).is_none());
    }

    #[test]
    fn extstringarray_forget_removes_attribute() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            OcExtStringArray::set(&cmd, &label, 2).unwrap();
            cmd.commit().unwrap();
        }
        {
            let cmd = doc.begin_command().unwrap();
            assert!(OcExtStringArray::forget(&cmd, &label));
            cmd.commit().unwrap();
        }
        assert!(OcExtStringArray::find(&label).is_none());
    }

    #[test]
    fn extstringarray_set_value_and_value_round_trip() {
        let (_app, mut doc) = new_doc();
        let arr;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let label = main.get_or_create_child(&cmd, 1);
            arr = OcExtStringArray::set(&cmd, &label, 2).unwrap();
            arr.set_value(&cmd, 0, "first").unwrap();
            arr.set_value(&cmd, 1, "second").unwrap();
            cmd.commit().unwrap();
        }
        assert_eq!(
            arr.to_vec(),
            vec!["first".to_string(), "second".to_string()]
        );
    }

    #[test]
    fn extstringarray_unicode_round_trip() {
        let (_app, mut doc) = new_doc();
        let arr;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let label = main.get_or_create_child(&cmd, 1);
            arr = OcExtStringArray::set(&cmd, &label, 1).unwrap();
            arr.set_value(&cmd, 0, "café 😀").unwrap();
            cmd.commit().unwrap();
        }
        assert_eq!(arr.value(0).unwrap(), "café 😀");
    }

    #[test]
    fn extstringarray_value_out_of_range_errors() {
        let (_app, mut doc) = new_doc();
        let arr;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let label = main.get_or_create_child(&cmd, 1);
            arr = OcExtStringArray::set(&cmd, &label, 2).unwrap();
            cmd.commit().unwrap();
        }
        assert!(arr.value(2).is_err());
        assert!(arr.value(-1).is_err());
    }

    #[test]
    fn extstringarray_undo_restores() {
        let (_app, mut doc) = new_doc();
        let arr;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let label = main.get_or_create_child(&cmd, 1);
            arr = OcExtStringArray::set(&cmd, &label, 1).unwrap();
            arr.set_value(&cmd, 0, "before").unwrap();
            cmd.commit().unwrap();
        }
        {
            let cmd = doc.begin_command().unwrap();
            arr.set_value(&cmd, 0, "after").unwrap();
            cmd.commit().unwrap();
        }
        doc.undo().unwrap();
        assert_eq!(arr.value(0).unwrap(), "before");
    }

    #[test]
    fn extstringarray_zero_length_is_err() {
        let (_app, mut doc) = new_doc();
        let main = doc.main();
        let cmd = doc.begin_command().unwrap();
        let label = main.get_or_create_child(&cmd, 1);
        assert!(OcExtStringArray::set(&cmd, &label, 0).is_err());
        cmd.abort().unwrap();
    }
    // ── OcIntegerList ────────────────────────────────────────────────────────

    #[test]
    fn integerlist_set_creates_empty_and_is_findable() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            let list = OcIntegerList::set(&cmd, &label).unwrap();
            assert!(list.is_empty());
            assert_eq!(list.extent(), 0);
            cmd.commit().unwrap();
        }
        let found = OcIntegerList::find(&label).expect("integer list should be present");
        assert!(found.is_empty());
    }

    #[test]
    fn integerlist_find_absent_returns_none() {
        let (_app, mut doc) = new_doc();
        let main = doc.main();
        let cmd = doc.begin_command().unwrap();
        let label = main.get_or_create_child(&cmd, 1);
        cmd.commit().unwrap();
        assert!(OcIntegerList::find(&label).is_none());
    }

    #[test]
    fn integerlist_forget_removes_attribute() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            OcIntegerList::set(&cmd, &label).unwrap();
            cmd.commit().unwrap();
        }
        {
            let cmd = doc.begin_command().unwrap();
            assert!(OcIntegerList::forget(&cmd, &label));
            cmd.commit().unwrap();
        }
        assert!(OcIntegerList::find(&label).is_none());
    }

    #[test]
    fn integerlist_append_preserves_order() {
        let (_app, mut doc) = new_doc();
        let list;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let label = main.get_or_create_child(&cmd, 1);
            list = OcIntegerList::set(&cmd, &label).unwrap();
            list.append(&cmd, 1);
            list.append(&cmd, 2);
            list.append(&cmd, 3);
            cmd.commit().unwrap();
        }
        assert_eq!(list.to_vec(), vec![1, 2, 3]);
    }

    #[test]
    fn integerlist_undo_restores() {
        let (_app, mut doc) = new_doc();
        let list;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let label = main.get_or_create_child(&cmd, 1);
            list = OcIntegerList::set(&cmd, &label).unwrap();
            list.append(&cmd, 1);
            cmd.commit().unwrap();
        }
        {
            let cmd = doc.begin_command().unwrap();
            list.append(&cmd, 2);
            cmd.commit().unwrap();
        }
        assert_eq!(list.extent(), 2);
        doc.undo().unwrap();
        assert_eq!(list.to_vec(), vec![1]);
    }

    // ── OcRealList ───────────────────────────────────────────────────────────

    #[test]
    fn reallist_set_creates_empty_and_is_findable() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            let list = OcRealList::set(&cmd, &label).unwrap();
            assert!(list.is_empty());
            assert_eq!(list.extent(), 0);
            cmd.commit().unwrap();
        }
        let found = OcRealList::find(&label).expect("real list should be present");
        assert!(found.is_empty());
    }

    #[test]
    fn reallist_find_absent_returns_none() {
        let (_app, mut doc) = new_doc();
        let main = doc.main();
        let cmd = doc.begin_command().unwrap();
        let label = main.get_or_create_child(&cmd, 1);
        cmd.commit().unwrap();
        assert!(OcRealList::find(&label).is_none());
    }

    #[test]
    fn reallist_forget_removes_attribute() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            OcRealList::set(&cmd, &label).unwrap();
            cmd.commit().unwrap();
        }
        {
            let cmd = doc.begin_command().unwrap();
            assert!(OcRealList::forget(&cmd, &label));
            cmd.commit().unwrap();
        }
        assert!(OcRealList::find(&label).is_none());
    }

    #[test]
    fn reallist_append_preserves_order() {
        let (_app, mut doc) = new_doc();
        let list;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let label = main.get_or_create_child(&cmd, 1);
            list = OcRealList::set(&cmd, &label).unwrap();
            list.append(&cmd, 1.5);
            list.append(&cmd, -2.25);
            cmd.commit().unwrap();
        }
        let got = list.to_vec();
        assert!((got[0] - 1.5).abs() < 1e-12);
        assert!((got[1] - (-2.25)).abs() < 1e-12);
    }

    #[test]
    fn reallist_undo_restores() {
        let (_app, mut doc) = new_doc();
        let list;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let label = main.get_or_create_child(&cmd, 1);
            list = OcRealList::set(&cmd, &label).unwrap();
            list.append(&cmd, 1.0);
            cmd.commit().unwrap();
        }
        {
            let cmd = doc.begin_command().unwrap();
            list.append(&cmd, 2.0);
            cmd.commit().unwrap();
        }
        assert_eq!(list.extent(), 2);
        doc.undo().unwrap();
        assert_eq!(list.extent(), 1);
    }

    // ── OcExtStringList ──────────────────────────────────────────────────────

    #[test]
    fn extstringlist_set_creates_empty_and_is_findable() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            let list = OcExtStringList::set(&cmd, &label).unwrap();
            assert!(list.is_empty());
            assert_eq!(list.extent(), 0);
            cmd.commit().unwrap();
        }
        let found = OcExtStringList::find(&label).expect("ext string list should be present");
        assert!(found.is_empty());
    }

    #[test]
    fn extstringlist_find_absent_returns_none() {
        let (_app, mut doc) = new_doc();
        let main = doc.main();
        let cmd = doc.begin_command().unwrap();
        let label = main.get_or_create_child(&cmd, 1);
        cmd.commit().unwrap();
        assert!(OcExtStringList::find(&label).is_none());
    }

    #[test]
    fn extstringlist_forget_removes_attribute() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            OcExtStringList::set(&cmd, &label).unwrap();
            cmd.commit().unwrap();
        }
        {
            let cmd = doc.begin_command().unwrap();
            assert!(OcExtStringList::forget(&cmd, &label));
            cmd.commit().unwrap();
        }
        assert!(OcExtStringList::find(&label).is_none());
    }

    #[test]
    fn extstringlist_append_preserves_order() {
        let (_app, mut doc) = new_doc();
        let list;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let label = main.get_or_create_child(&cmd, 1);
            list = OcExtStringList::set(&cmd, &label).unwrap();
            list.append(&cmd, "first");
            list.append(&cmd, "second");
            cmd.commit().unwrap();
        }
        assert_eq!(
            list.to_vec(),
            vec!["first".to_string(), "second".to_string()]
        );
    }

    #[test]
    fn extstringlist_unicode_round_trip() {
        let (_app, mut doc) = new_doc();
        let list;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let label = main.get_or_create_child(&cmd, 1);
            list = OcExtStringList::set(&cmd, &label).unwrap();
            list.append(&cmd, "café 😀");
            cmd.commit().unwrap();
        }
        assert_eq!(list.at(0), "café 😀");
    }

    #[test]
    fn extstringlist_undo_restores() {
        let (_app, mut doc) = new_doc();
        let list;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let label = main.get_or_create_child(&cmd, 1);
            list = OcExtStringList::set(&cmd, &label).unwrap();
            list.append(&cmd, "before");
            cmd.commit().unwrap();
        }
        {
            let cmd = doc.begin_command().unwrap();
            list.append(&cmd, "after");
            cmd.commit().unwrap();
        }
        assert_eq!(list.extent(), 2);
        doc.undo().unwrap();
        assert_eq!(list.to_vec(), vec!["before".to_string()]);
    }

    // ── OcBooleanList ────────────────────────────────────────────────────────

    #[test]
    fn booleanlist_set_creates_empty_and_is_findable() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            let list = OcBooleanList::set(&cmd, &label).unwrap();
            assert!(list.is_empty());
            assert_eq!(list.extent(), 0);
            cmd.commit().unwrap();
        }
        let found = OcBooleanList::find(&label).expect("boolean list should be present");
        assert!(found.is_empty());
    }

    #[test]
    fn booleanlist_find_absent_returns_none() {
        let (_app, mut doc) = new_doc();
        let main = doc.main();
        let cmd = doc.begin_command().unwrap();
        let label = main.get_or_create_child(&cmd, 1);
        cmd.commit().unwrap();
        assert!(OcBooleanList::find(&label).is_none());
    }

    #[test]
    fn booleanlist_forget_removes_attribute() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            OcBooleanList::set(&cmd, &label).unwrap();
            cmd.commit().unwrap();
        }
        {
            let cmd = doc.begin_command().unwrap();
            assert!(OcBooleanList::forget(&cmd, &label));
            cmd.commit().unwrap();
        }
        assert!(OcBooleanList::find(&label).is_none());
    }

    #[test]
    fn booleanlist_append_preserves_order() {
        let (_app, mut doc) = new_doc();
        let list;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let label = main.get_or_create_child(&cmd, 1);
            list = OcBooleanList::set(&cmd, &label).unwrap();
            list.append(&cmd, true);
            list.append(&cmd, false);
            list.append(&cmd, true);
            cmd.commit().unwrap();
        }
        assert_eq!(list.to_vec(), vec![true, false, true]);
    }

    #[test]
    fn booleanlist_undo_restores() {
        let (_app, mut doc) = new_doc();
        let list;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let label = main.get_or_create_child(&cmd, 1);
            list = OcBooleanList::set(&cmd, &label).unwrap();
            list.append(&cmd, true);
            cmd.commit().unwrap();
        }
        {
            let cmd = doc.begin_command().unwrap();
            list.append(&cmd, false);
            cmd.commit().unwrap();
        }
        assert_eq!(list.extent(), 2);
        doc.undo().unwrap();
        assert_eq!(list.to_vec(), vec![true]);
    }
    // ── OcGuid ───────────────────────────────────────────────────────────────

    #[test]
    fn guid_display_and_parse_round_trip() {
        let g = OcGuid::from_fields(
            0x12345678,
            0x1234,
            0x5678,
            0x9abc,
            [0xde, 0xf0, 0x12, 0x34, 0x56, 0x78],
        );
        let s = g.to_string();
        assert_eq!(s, "12345678-1234-5678-9abc-def012345678");
        let parsed: OcGuid = s.parse().unwrap();
        assert_eq!(parsed, g);
    }

    #[test]
    fn guid_parse_rejects_malformed() {
        assert!("not-a-guid".parse::<OcGuid>().is_err());
        assert!("12345678-1234-5678-9abc".parse::<OcGuid>().is_err());
    }

    // ── OcUAttribute ─────────────────────────────────────────────────────────

    const TEST_GUID: OcGuid = OcGuid::from_fields(
        0x12345678,
        0x1234,
        0x5678,
        0x9abc,
        [0xde, 0xf0, 0x12, 0x34, 0x56, 0x78],
    );
    const OTHER_GUID: OcGuid =
        OcGuid::from_fields(0x00000000, 0x0000, 0x0000, 0x0000, [0, 0, 0, 0, 0, 1]);

    #[test]
    fn uattribute_set_and_is_present() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            OcUAttribute::set(&cmd, &label, TEST_GUID).unwrap();
            cmd.commit().unwrap();
        }
        assert!(OcUAttribute::is_present(&label, TEST_GUID));
    }

    #[test]
    fn uattribute_is_present_absent_returns_false() {
        let (_app, mut doc) = new_doc();
        let main = doc.main();
        let cmd = doc.begin_command().unwrap();
        let label = main.get_or_create_child(&cmd, 1);
        cmd.commit().unwrap();
        assert!(!OcUAttribute::is_present(&label, TEST_GUID));
    }

    #[test]
    fn uattribute_distinguishes_different_guids() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            OcUAttribute::set(&cmd, &label, TEST_GUID).unwrap();
            cmd.commit().unwrap();
        }
        assert!(OcUAttribute::is_present(&label, TEST_GUID));
        assert!(!OcUAttribute::is_present(&label, OTHER_GUID));
    }

    #[test]
    fn uattribute_forget_removes() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            OcUAttribute::set(&cmd, &label, TEST_GUID).unwrap();
            cmd.commit().unwrap();
        }
        {
            let cmd = doc.begin_command().unwrap();
            assert!(OcUAttribute::forget(&cmd, &label, TEST_GUID));
            cmd.commit().unwrap();
        }
        assert!(!OcUAttribute::is_present(&label, TEST_GUID));
    }

    #[test]
    fn uattribute_forget_absent_returns_false() {
        let (_app, mut doc) = new_doc();
        let main = doc.main();
        let cmd = doc.begin_command().unwrap();
        let label = main.get_or_create_child(&cmd, 1);
        assert!(!OcUAttribute::forget(&cmd, &label, TEST_GUID));
        cmd.commit().unwrap();
    }

    #[test]
    fn uattribute_undo_restores() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            cmd.commit().unwrap();
        }
        {
            let cmd = doc.begin_command().unwrap();
            OcUAttribute::set(&cmd, &label, TEST_GUID).unwrap();
            cmd.commit().unwrap();
        }
        assert!(OcUAttribute::is_present(&label, TEST_GUID));
        doc.undo().unwrap();
        assert!(!OcUAttribute::is_present(&label, TEST_GUID));
    }
    // ── OcNamedData ──────────────────────────────────────────────────────────

    #[test]
    fn nameddata_set_creates_and_is_findable() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            OcNamedData::set(&cmd, &label).unwrap();
            cmd.commit().unwrap();
        }
        let found = OcNamedData::find(&label).expect("named data should be present");
        assert!(!found.has_integers());
        assert!(!found.has_reals());
        assert!(!found.has_strings());
        assert!(!found.has_bytes());
    }

    #[test]
    fn nameddata_find_absent_returns_none() {
        let (_app, mut doc) = new_doc();
        let main = doc.main();
        let cmd = doc.begin_command().unwrap();
        let label = main.get_or_create_child(&cmd, 1);
        cmd.commit().unwrap();
        assert!(OcNamedData::find(&label).is_none());
    }

    #[test]
    fn nameddata_forget_removes_attribute() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            OcNamedData::set(&cmd, &label).unwrap();
            cmd.commit().unwrap();
        }
        {
            let cmd = doc.begin_command().unwrap();
            assert!(OcNamedData::forget(&cmd, &label));
            cmd.commit().unwrap();
        }
        assert!(OcNamedData::find(&label).is_none());
    }

    #[test]
    fn nameddata_integer_round_trip() {
        let (_app, mut doc) = new_doc();
        let nd;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let label = main.get_or_create_child(&cmd, 1);
            nd = OcNamedData::set(&cmd, &label).unwrap();
            assert!(!nd.has_integer("count"));
            assert_eq!(nd.get_integer("count"), 0);
            nd.set_integer(&cmd, "count", 42);
            cmd.commit().unwrap();
        }
        assert!(nd.has_integers());
        assert!(nd.has_integer("count"));
        assert_eq!(nd.get_integer("count"), 42);
    }

    #[test]
    fn nameddata_real_round_trip() {
        let (_app, mut doc) = new_doc();
        let nd;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let label = main.get_or_create_child(&cmd, 1);
            nd = OcNamedData::set(&cmd, &label).unwrap();
            assert!(!nd.has_real("radius"));
            assert_eq!(nd.get_real("radius"), 0.0);
            nd.set_real(&cmd, "radius", 3.5);
            cmd.commit().unwrap();
        }
        assert!(nd.has_reals());
        assert!(nd.has_real("radius"));
        assert!((nd.get_real("radius") - 3.5).abs() < 1e-12);
    }

    #[test]
    fn nameddata_string_round_trip() {
        let (_app, mut doc) = new_doc();
        let nd;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let label = main.get_or_create_child(&cmd, 1);
            nd = OcNamedData::set(&cmd, &label).unwrap();
            assert!(!nd.has_string("material"));
            assert_eq!(nd.get_string("material"), "");
            nd.set_string(&cmd, "material", "aluminum");
            cmd.commit().unwrap();
        }
        assert!(nd.has_strings());
        assert!(nd.has_string("material"));
        assert_eq!(nd.get_string("material"), "aluminum");
    }

    #[test]
    fn nameddata_string_unicode_round_trip() {
        let (_app, mut doc) = new_doc();
        let nd;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let label = main.get_or_create_child(&cmd, 1);
            nd = OcNamedData::set(&cmd, &label).unwrap();
            // Both key and value carry non-ASCII / non-BMP content.
            nd.set_string(&cmd, "matériau 😀", "café 😀");
            cmd.commit().unwrap();
        }
        assert!(nd.has_string("matériau 😀"));
        assert_eq!(nd.get_string("matériau 😀"), "café 😀");
    }

    #[test]
    fn nameddata_byte_round_trip() {
        let (_app, mut doc) = new_doc();
        let nd;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let label = main.get_or_create_child(&cmd, 1);
            nd = OcNamedData::set(&cmd, &label).unwrap();
            assert!(!nd.has_byte("flags"));
            assert_eq!(nd.get_byte("flags"), 0);
            nd.set_byte(&cmd, "flags", 0xFF);
            cmd.commit().unwrap();
        }
        assert!(nd.has_bytes());
        assert!(nd.has_byte("flags"));
        assert_eq!(nd.get_byte("flags"), 0xFF);
    }

    #[test]
    fn nameddata_set_overwrites_existing_key() {
        let (_app, mut doc) = new_doc();
        let nd;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let label = main.get_or_create_child(&cmd, 1);
            nd = OcNamedData::set(&cmd, &label).unwrap();
            nd.set_integer(&cmd, "count", 1);
            cmd.commit().unwrap();
        }
        {
            let cmd = doc.begin_command().unwrap();
            nd.set_integer(&cmd, "count", 2);
            cmd.commit().unwrap();
        }
        assert_eq!(nd.get_integer("count"), 2);
    }

    #[test]
    fn nameddata_undo_restores() {
        let (_app, mut doc) = new_doc();
        let nd;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let label = main.get_or_create_child(&cmd, 1);
            nd = OcNamedData::set(&cmd, &label).unwrap();
            nd.set_integer(&cmd, "count", 1);
            cmd.commit().unwrap();
        }
        {
            let cmd = doc.begin_command().unwrap();
            nd.set_integer(&cmd, "count", 2);
            cmd.commit().unwrap();
        }
        assert_eq!(nd.get_integer("count"), 2);
        doc.undo().unwrap();
        assert_eq!(nd.get_integer("count"), 1);
    }
}
