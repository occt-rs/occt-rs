//! Topological solid type.
//!
//! `OcSolid` is a closed, bounded volume in 3-D space.  This module
//! introduces the type only; construction via extrusion (`BRepPrimAPI_MakePrism`,
//! TKPrim) is bound in the next step.
//!
//! Reference: <https://dev.opencascade.org/doc/refman/html/class_topo_d_s___solid.html>

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
    pub(crate) fn from_ffi(inner: cxx::UniquePtr<ffi::TopdsSolid>) -> Self {
        Self {
            inner,
            _not_send: PhantomData,
        }
    }

    pub(crate) fn as_ffi(&self) -> &ffi::TopdsSolid {
        &self.inner
    }
}

impl Clone for OcSolid {
    fn clone(&self) -> Self {
        Self {
            inner: ffi::clone_solid(&self.inner),
            _not_send: PhantomData,
        }
    }
}
