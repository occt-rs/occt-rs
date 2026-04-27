//! Standard scalar TDF attributes: Name, Integer, Real.
//!
//! Each type wraps a `Handle(TDataStd_*)` shim.  The three operations per type are:
//!
//! - **`set`** — attaches or updates the attribute on a label (must be inside a command).
//! - **`get`** — reads the current value from an already-retrieved attribute handle.
//! - **`find`** — probes whether the attribute is present on a label.
//!
//! GUIDs are kept on the C++ side; the Rust API never names them.

use std::marker::PhantomData;

use occt_sys::ffi;

use crate::error::OcctError;
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
    ///
    /// [`Command`]: crate::topo::OcCommand
    pub fn set(label: &OcLabel, value: &str) -> Result<Self, OcctError> {
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
    ///
    /// [`Command`]: crate::topo::OcCommand
    pub fn set(label: &OcLabel, value: i32) -> Result<Self, OcctError> {
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
    ///
    /// [`Command`]: crate::topo::OcCommand
    pub fn set(label: &OcLabel, value: f64) -> Result<Self, OcctError> {
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
}

impl std::fmt::Debug for OcReal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OcReal")
            .field("value", &self.get())
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
        let label = doc.main().find_child(1, true).unwrap();
        {
            let cmd = doc.begin_command().unwrap();
            OcName::set(&label, "hello").unwrap();
            cmd.commit().unwrap();
        }
        let attr = OcName::find(&label).expect("name attribute should be present");
        assert_eq!(attr.get(), "hello");
    }

    #[test]
    fn name_find_absent_returns_none() {
        let (_app, doc) = new_doc();
        let label = doc.main().find_child(1, true).unwrap();
        assert!(OcName::find(&label).is_none());
    }

    #[test]
    fn name_update_overwrites() {
        let (_app, mut doc) = new_doc();
        let label = doc.main().find_child(1, true).unwrap();
        {
            let cmd = doc.begin_command().unwrap();
            OcName::set(&label, "first").unwrap();
            cmd.commit().unwrap();
        }
        {
            let cmd = doc.begin_command().unwrap();
            OcName::set(&label, "second").unwrap();
            cmd.commit().unwrap();
        }
        assert_eq!(OcName::find(&label).unwrap().get(), "second");
    }

    #[test]
    fn name_undo_restores() {
        let (_app, mut doc) = new_doc();
        let label = doc.main().find_child(1, true).unwrap();
        {
            let cmd = doc.begin_command().unwrap();
            OcName::set(&label, "before").unwrap();
            cmd.commit().unwrap();
        }
        {
            let cmd = doc.begin_command().unwrap();
            OcName::set(&label, "after").unwrap();
            cmd.commit().unwrap();
        }
        doc.undo().unwrap();
        assert_eq!(OcName::find(&label).unwrap().get(), "before");
    }

    // ── OcInteger ────────────────────────────────────────────────────────────

    #[test]
    fn integer_round_trip() {
        let (_app, mut doc) = new_doc();
        let label = doc.main().find_child(1, true).unwrap();
        {
            let cmd = doc.begin_command().unwrap();
            OcInteger::set(&label, 42).unwrap();
            cmd.commit().unwrap();
        }
        assert_eq!(OcInteger::find(&label).unwrap().get(), 42);
    }

    #[test]
    fn integer_find_absent_returns_none() {
        let (_app, doc) = new_doc();
        let label = doc.main().find_child(1, true).unwrap();
        assert!(OcInteger::find(&label).is_none());
    }

    #[test]
    fn integer_undo_restores() {
        let (_app, mut doc) = new_doc();
        let label = doc.main().find_child(1, true).unwrap();
        {
            let cmd = doc.begin_command().unwrap();
            OcInteger::set(&label, 1).unwrap();
            cmd.commit().unwrap();
        }
        {
            let cmd = doc.begin_command().unwrap();
            OcInteger::set(&label, 2).unwrap();
            cmd.commit().unwrap();
        }
        doc.undo().unwrap();
        assert_eq!(OcInteger::find(&label).unwrap().get(), 1);
    }

    // ── OcReal ───────────────────────────────────────────────────────────────

    #[test]
    fn real_round_trip() {
        let (_app, mut doc) = new_doc();
        let label = doc.main().find_child(1, true).unwrap();
        {
            let cmd = doc.begin_command().unwrap();
            OcReal::set(&label, 3.14).unwrap();
            cmd.commit().unwrap();
        }
        let v = OcReal::find(&label).unwrap().get();
        assert!((v - 3.14).abs() < 1e-12);
    }

    #[test]
    fn real_find_absent_returns_none() {
        let (_app, doc) = new_doc();
        let label = doc.main().find_child(1, true).unwrap();
        assert!(OcReal::find(&label).is_none());
    }

    // ── Mixed ────────────────────────────────────────────────────────────────

    #[test]
    fn multiple_attributes_on_same_label() {
        let (_app, mut doc) = new_doc();
        let label = doc.main().find_child(1, true).unwrap();
        {
            let cmd = doc.begin_command().unwrap();
            OcName::set(&label, "part_a").unwrap();
            OcInteger::set(&label, 7).unwrap();
            OcReal::set(&label, 1.5).unwrap();
            cmd.commit().unwrap();
        }
        assert_eq!(OcName::find(&label).unwrap().get(), "part_a");
        assert_eq!(OcInteger::find(&label).unwrap().get(), 7);
        assert!((OcReal::find(&label).unwrap().get() - 1.5).abs() < 1e-12);
    }
}
