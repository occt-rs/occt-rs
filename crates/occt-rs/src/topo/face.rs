//! Topological face type.
//!
//! `OcFace` is a bounded surface in 3-D space.  This module binds the
//! planar-wire case constructed via `BRepBuilderAPI_MakeFace(wire, only_plane)`.
//!
//! Reference: <https://dev.opencascade.org/doc/refman/html/class_b_rep_builder_a_p_i___make_face.html>

use crate::error::{OcctError, OcctErrorKind};
use crate::topo::OcWire;
use occt_sys::ffi;
use std::marker::PhantomData;

/// A BRep topological face — a bounded surface.
///
/// Wraps `TopoDS_Face`.  Internally reference-counted by OCCT; `Clone` is
/// cheap and shares the underlying `TShape` handle.
///
/// # Thread safety
///
/// OCCT's `Handle` reference-counting is not atomic.  `OcFace` must not
/// be sent across thread boundaries.
pub struct OcFace {
    inner: cxx::UniquePtr<ffi::TopdsFace>,
    _not_send: PhantomData<*mut ()>,
}

impl std::fmt::Debug for OcFace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OcFace").finish_non_exhaustive()
    }
}

impl OcFace {
    /// Constructs a face from a closed wire.
    ///
    /// When `only_plane` is `true`, OCCT rejects the wire if it is not planar
    /// and returns `Err(ConstructionError)`.  When `false` (OCCT's default),
    /// OCCT attempts to find a fitting surface; this succeeds for planar wires
    /// and may succeed for other geometries where OCCT can infer a surface.
    ///
    /// For most use cases — building flat faces from planar profiles — pass
    /// `only_plane: true` to get an explicit error rather than a silently
    /// mis-shaped result.
    ///
    /// Returns `Err(ConstructionError)` when `IsDone()` is false.
    pub fn from_wire(wire: &OcWire, only_plane: bool) -> Result<Self, OcctError> {
        let mut builder = ffi::new_make_face_from_wire(wire.as_ffi(), only_plane);
        if builder.is_done() {
            Ok(Self {
                inner: builder.pin_mut().face(),
                _not_send: PhantomData,
            })
        } else {
            Err(OcctError {
                kind: OcctErrorKind::ConstructionError,
                message: format!(
                    "BRepBuilderAPI_MakeFace failed (error code {})",
                    builder.error()
                ),
            })
        }
    }

    /// Returns the outer boundary wire of this face.
    ///
    /// Calls `BRepTools::OuterWire`.
    /// Reference: <https://dev.opencascade.org/doc/refman/html/class_b_rep_tools.html>
    pub fn outer_wire(&self) -> OcWire {
        OcWire::from_ffi(ffi::face_outer_wire(&self.inner))
    }

    /// Returns a reference to the underlying `TopoDS_Face` for use by
    /// other OCCT API bindings within this crate.
    pub(crate) fn as_ffi(&self) -> &ffi::TopdsFace {
        &self.inner
    }

    pub(crate) fn from_ffi(inner: cxx::UniquePtr<ffi::TopdsFace>) -> Self {
        Self {
            inner,
            _not_send: PhantomData,
        }
    }
}

impl Clone for OcFace {
    fn clone(&self) -> Self {
        Self {
            inner: ffi::clone_face(&self.inner),
            _not_send: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gp::OcPnt;
    use crate::topo::OcEdge;

    fn triangle_wire() -> OcWire {
        let edges = vec![
            OcEdge::from_pnts(OcPnt::new(0.0, 0.0, 0.0), OcPnt::new(1.0, 0.0, 0.0)).unwrap(),
            OcEdge::from_pnts(OcPnt::new(1.0, 0.0, 0.0), OcPnt::new(0.5, 1.0, 0.0)).unwrap(),
            OcEdge::from_pnts(OcPnt::new(0.5, 1.0, 0.0), OcPnt::new(0.0, 0.0, 0.0)).unwrap(),
        ];
        OcWire::from_edges(&edges).unwrap()
    }

    #[test]
    fn from_planar_wire_succeeds() {
        let wire = triangle_wire();
        assert!(OcFace::from_wire(&wire, true).is_ok());
    }

    #[test]
    fn from_planar_wire_only_plane_false_succeeds() {
        let wire = triangle_wire();
        assert!(OcFace::from_wire(&wire, false).is_ok());
    }

    #[test]
    fn outer_wire_has_expected_edge_count() {
        let wire = triangle_wire();
        let face = OcFace::from_wire(&wire, true).unwrap();
        let outer = face.outer_wire();
        assert_eq!(outer.edges().len(), 3);
    }

    #[test]
    fn clone_is_valid() {
        let wire = triangle_wire();
        let face = OcFace::from_wire(&wire, true).unwrap();
        let cloned = face.clone();
        // Both originals and clones must survive; outer_wire is readable on both.
        assert_eq!(face.outer_wire().edges().len(), 3);
        assert_eq!(cloned.outer_wire().edges().len(), 3);
    }

    #[test]
    fn construction_error_kind() {
        // A wire with a single edge is not a valid closed face boundary.
        let single_edge =
            OcEdge::from_pnts(OcPnt::new(0.0, 0.0, 0.0), OcPnt::new(1.0, 0.0, 0.0)).unwrap();
        let open_wire = OcWire::from_edges(&[single_edge]).unwrap();
        let err = OcFace::from_wire(&open_wire, true).unwrap_err();
        assert_eq!(err.kind, crate::error::OcctErrorKind::ConstructionError);
    }
}
