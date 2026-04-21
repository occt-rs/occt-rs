//! Topological edge type.
//!
//! `OcEdge` is a bounded curve in 3-D space connecting two vertices.
//! This module binds only the straight-line case constructed from two
//! `OcVertex` values via `BRepBuilderAPI_MakeEdge(V1, V2)`.
//!
//! Reference: <https://dev.opencascade.org/doc/refman/html/class_b_rep_builder_a_p_i___make_edge.html>

use crate::error::{OcctError, OcctErrorKind};
use crate::gp::OcPnt;
use crate::topo::{OcShape, OcVertex};
use occt_sys::ffi;
use std::marker::PhantomData;

/// A BRep topological edge — a straight line segment between two vertices.
///
/// Wraps `TopoDS_Edge`.  Internally reference-counted by OCCT; `Clone` is
/// cheap and shares the underlying `TShape` handle.
///
/// # Thread safety
///
/// OCCT's `Handle` reference-counting is not atomic.  `OcEdge` must not
/// be sent across thread boundaries.
pub struct OcEdge {
    inner: cxx::UniquePtr<ffi::TopodsEdge>,
    _not_send: PhantomData<*mut ()>,
}

impl std::fmt::Debug for OcEdge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OcEdge").finish_non_exhaustive()
    }
}

impl OcEdge {
    /// Constructs a straight-line edge between two existing vertices.
    ///
    /// Returns `Err(ConstructionError)` if the vertices are coincident.
    pub fn from_vertices(v1: &OcVertex, v2: &OcVertex) -> Result<Self, OcctError> {
        let mut builder = ffi::new_make_edge_builder(v1.as_ffi(), v2.as_ffi());
        if builder.is_done() {
            Ok(Self {
                inner: builder.pin_mut().edge(),
                _not_send: PhantomData,
            })
        } else {
            Err(OcctError {
                kind: OcctErrorKind::ConstructionError,
                message: builder.error().to_string(),
            })
        }
    }

    /// Constructs a straight-line edge between two points.
    ///
    /// Returns `Err(ConstructionError)` if the points are coincident.
    pub fn from_pnts(p1: OcPnt, p2: OcPnt) -> Result<Self, OcctError> {
        let v1 = OcVertex::from_pnt(&p1);
        let v2 = OcVertex::from_pnt(&p2);
        Self::from_vertices(&v1, &v2)
    }

    /// Widens this edge to a general [`OcShape`] for use with shape-level
    /// APIs such as tessellation.
    ///
    /// The conversion is a cheap TShape handle reference-count increment;
    /// no geometry is copied.
    pub fn as_shape(&self) -> OcShape {
        OcShape::from_ffi(ffi::clone_shape(ffi::edge_as_shape(&self.inner)))
    }

    pub fn start_vertex(&self) -> OcVertex {
        OcVertex::from_ffi(ffi::edge_start_vertex(&self.inner))
    }

    pub fn end_vertex(&self) -> OcVertex {
        OcVertex::from_ffi(ffi::edge_end_vertex(&self.inner))
    }

    pub(crate) fn as_ffi(&self) -> &ffi::TopodsEdge {
        &self.inner
    }

    pub(crate) fn from_ffi(inner: cxx::UniquePtr<ffi::TopodsEdge>) -> Self {
        Self {
            inner,
            _not_send: PhantomData,
        }
    }
}

impl Clone for OcEdge {
    fn clone(&self) -> Self {
        Self {
            inner: ffi::clone_edge(&self.inner),
            _not_send: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_pnts_distinct() {
        let e = OcEdge::from_pnts(OcPnt::new(0.0, 0.0, 0.0), OcPnt::new(1.0, 0.0, 0.0));
        assert!(e.is_ok());
    }

    #[test]
    fn from_pnts_coincident_fails() {
        let p = OcPnt::new(1.0, 2.0, 3.0);
        let e = OcEdge::from_pnts(p, p);
        assert!(e.is_err());
    }

    #[test]
    fn clone_is_valid() {
        let e = OcEdge::from_pnts(OcPnt::new(0.0, 0.0, 0.0), OcPnt::new(0.0, 1.0, 0.0)).unwrap();
        let _ = e.clone();
    }

    #[test]
    fn coincident_fails_with_construction_error() {
        let p = OcPnt::new(1.0, 2.0, 3.0);
        let err = OcEdge::from_pnts(p, p).unwrap_err();
        assert_eq!(err.kind, crate::error::OcctErrorKind::ConstructionError);
    }

    #[test]
    fn as_shape_widens() {
        let e = OcEdge::from_pnts(OcPnt::new(0.0, 0.0, 0.0), OcPnt::new(1.0, 0.0, 0.0)).unwrap();
        let _shape = e.as_shape();
    }
}
