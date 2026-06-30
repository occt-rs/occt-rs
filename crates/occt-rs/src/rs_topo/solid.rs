//! Topological solid type.
//!
//! Reference: <https://dev.opencascade.org/doc/refman/html/class_topo_d_s___solid.html>

use crate::rs_topo::OcShape;
use occt_sys::ffi;
use std::marker::PhantomData;

/// A BRep topological solid — a closed, bounded volume.
///
/// Wraps `TopoDS_Solid`.  Internally reference-counted by OCCT; `Clone` is
/// cheap and shares the underlying `TShape` handle.
///
/// # Thread safety
///
/// OCCT's `Handle` reference-counting is not atomic.  `OcSolid` must not
/// be sent across thread boundaries.
pub struct OcSolid {
    inner: cxx::UniquePtr<ffi::TopdsSolid>,
    _not_send: PhantomData<*mut ()>,
}

impl std::fmt::Debug for OcSolid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OcSolid").finish_non_exhaustive()
    }
}

impl OcSolid {
    /// Widens this solid to a general [`OcShape`] for use with shape-level
    /// APIs such as tessellation.
    ///
    /// The conversion is a cheap TShape handle reference-count increment;
    /// no geometry is copied.
    pub fn as_shape(&self) -> OcShape {
        // Safety: solid_as_shape is a zero-cost upcast; clone_shape is make_unique — non-null.
        unsafe { OcShape::from_ffi_unchecked(ffi::clone_shape(ffi::solid_as_shape(&self.inner))) }
    }

    /// Returns `None` if `inner` is a null `UniquePtr` or the `TopoDS_Solid`
    /// has a null TShape handle (`IsNull()`).
    #[allow(dead_code)]
    pub(crate) fn from_ffi(inner: cxx::UniquePtr<ffi::TopdsSolid>) -> Option<Self> {
        if inner.is_null() {
            return None;
        }
        if ffi::topods_solid_is_null(inner.as_ref().unwrap()) {
            return None;
        }
        Some(Self {
            inner,
            _not_send: PhantomData,
        })
    }

    /// Constructs an `OcSolid` without null checks.
    ///
    /// # Safety
    ///
    /// Caller guarantees that `inner` is a non-null `UniquePtr` wrapping a
    /// `TopoDS_Solid` whose TShape handle is non-null (`!IsNull()`).
    pub(crate) unsafe fn from_ffi_unchecked(inner: cxx::UniquePtr<ffi::TopdsSolid>) -> Self {
        Self {
            inner,
            _not_send: PhantomData,
        }
    }

    pub(crate) fn _as_ffi(&self) -> &ffi::TopdsSolid {
        &self.inner
    }
}

impl Clone for OcSolid {
    fn clone(&self) -> Self {
        // Safety: clone_solid is make_unique<TopoDS_Solid>(s) — non-null.
        unsafe { Self::from_ffi_unchecked(ffi::clone_solid(&self.inner)) }
    }
}
