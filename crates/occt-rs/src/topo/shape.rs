//! General topological shape wrapper.
//!
//! `OcShape` is the Rust equivalent of `TopoDS_Shape` — the polymorphic base
//! for all OCCT topological entities.  It is the input type for operations
//! that span multiple shape kinds, such as tessellation.
//!
//! Typed wrappers (`OcFace`, `OcSolid`, etc.) widen to `OcShape` via their
//! `as_shape()` method.  The conversion is a cheap TShape reference-count
//! increment; no geometry is copied.
//!
//! Reference: <https://dev.opencascade.org/doc/refman/html/class_topo_d_s___shape.html>

use occt_sys::ffi;
use std::marker::PhantomData;

/// A polymorphic BRep topological shape.
///
/// Wraps `TopoDS_Shape`.  Internally reference-counted via the `TShape`
/// handle; `Clone` is cheap (handle increment, no geometry copy).
///
/// Construct via the `as_shape()` method on typed wrappers (`OcFace`,
/// `OcSolid`, `OcEdge`, `OcVertex`, `OcWire`).
///
/// # Thread safety
///
/// OCCT's `Handle` reference-counting is not atomic.  `OcShape` must not
/// be sent across thread boundaries.
pub struct OcShape {
    pub(crate) inner: cxx::UniquePtr<ffi::TopodsShape>,
    _not_send: PhantomData<*mut ()>,
}

impl std::fmt::Debug for OcShape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OcShape").finish_non_exhaustive()
    }
}

impl OcShape {
    pub(crate) fn from_ffi(inner: cxx::UniquePtr<ffi::TopodsShape>) -> Self {
        Self {
            inner,
            _not_send: PhantomData,
        }
    }

    pub(crate) fn as_ffi(&self) -> &ffi::TopodsShape {
        &self.inner
    }
}

impl Clone for OcShape {
    /// Cheap clone: increments the `TShape` handle reference count.
    fn clone(&self) -> Self {
        Self {
            inner: ffi::clone_shape(&self.inner),
            _not_send: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gp::OcPnt;
    use crate::topo::{OcEdge, OcFace, OcWire};

    fn triangle_face() -> OcFace {
        let edges = vec![
            OcEdge::from_pnts(OcPnt::new(0.0, 0.0, 0.0), OcPnt::new(1.0, 0.0, 0.0)).unwrap(),
            OcEdge::from_pnts(OcPnt::new(1.0, 0.0, 0.0), OcPnt::new(0.5, 1.0, 0.0)).unwrap(),
            OcEdge::from_pnts(OcPnt::new(0.5, 1.0, 0.0), OcPnt::new(0.0, 0.0, 0.0)).unwrap(),
        ];
        let wire = OcWire::from_edges(&edges).unwrap();
        OcFace::from_wire(&wire, true).unwrap()
    }

    #[test]
    fn as_shape_and_clone() {
        let face = triangle_face();
        let shape = face.as_shape();
        let cloned = shape.clone();
        // Both remain valid; no assertion needed beyond "no panic".
        let _ = cloned;
    }
}
