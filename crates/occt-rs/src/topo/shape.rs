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
use std::marker::PhantomData;

use occt_sys::ffi;

use crate::{OcEdge, topo::face::OcFace};

/// TopAbs_ShapeEnum::TopAbs_FACE.
/// Reference: https://dev.opencascade.org/doc/refman/html/namespace_top_abs.html
const TOP_ABS_FACE: i32 = 4;
const TOP_ABS_EDGE: i32 = 6;

/// Within-session identity for a placed topological sub-shape instance.
///
/// Encodes TShape (geometry), Location (placement), and Orientation — the
/// three components that together distinguish a placed instance in OCCT.
/// Two faces that share underlying geometry but sit at different positions
/// (e.g. the top and bottom caps of a `BRepPrimAPI_MakePrism` solid, which
/// share a `TShape` but differ by `Location`) receive distinct keys.
///
/// The key is a hash of those three components; collisions are
/// astronomically unlikely for any realistic number of shapes in a session.
///
/// **Not persistent.** Keys are meaningless across serialise/deserialise
/// cycles and process restarts.  When the TDF attribute layer is added,
/// `ShapeKey` values will compose with `TDF_Label` identifiers for
/// persistent identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShapeKey(pub usize);

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

    /// Returns all `TopoDS_Face` sub-shapes of this shape as typed wrappers.
    ///
    /// Traverses using `TopExp_Explorer` with `TopAbs_FACE`.  Results are in
    /// exploration order; `TopExp_Explorer` does not deduplicate — a face
    /// shared by multiple shells may appear more than once.  Filter on
    /// [`ShapeKey`] if unique faces are required.
    ///
    /// Reference: <https://dev.opencascade.org/doc/refman/html/class_top_exp___explorer.html>
    pub fn faces(&self) -> Vec<OcFace> {
        let mut result = Vec::new();
        let mut exp = ffi::new_shape_explorer(self.as_ffi(), TOP_ABS_FACE);
        while exp.more() {
            result.push(OcFace::from_ffi(ffi::shape_as_face(exp.current())));
            exp.pin_mut().next();
        }
        result
    }
    
    /// Returns all `TopoDS_Edge` sub-shapes of this shape as typed wrappers.
    ///
    /// Traverses using `TopExp_Explorer` with `TopAbs_EDGE`.  Results are in
    /// exploration order; `TopExp_Explorer` does not deduplicate — an edge
    /// shared by two faces appears twice.  Filter on [`ShapeKey`] if unique
    /// edges are required.
    ///
    /// Reference: <https://dev.opencascade.org/doc/refman/html/class_top_exp___explorer.html>
    pub fn edges(&self) -> Vec<OcEdge> {
        let mut result = Vec::new();
        let mut exp = ffi::new_shape_explorer(self.as_ffi(), TOP_ABS_EDGE);
        while exp.more() {
            result.push(OcEdge::from_ffi(ffi::shape_as_edge(exp.current())));
            exp.pin_mut().next();
        }
        result
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

    #[test]
    fn faces_of_prism() {
        use crate::gp::{OcPnt, OcVec};
        use crate::topo::{OcEdge, OcWire};
        let edges = vec![
            OcEdge::from_pnts(OcPnt::new(0.0, 0.0, 0.0), OcPnt::new(1.0, 0.0, 0.0)).unwrap(),
            OcEdge::from_pnts(OcPnt::new(1.0, 0.0, 0.0), OcPnt::new(0.5, 1.0, 0.0)).unwrap(),
            OcEdge::from_pnts(OcPnt::new(0.5, 1.0, 0.0), OcPnt::new(0.0, 0.0, 0.0)).unwrap(),
        ];
        let wire = OcWire::from_edges(&edges).unwrap();
        let face = OcFace::from_wire(&wire, true).unwrap();
        let solid = face.extrude(OcVec::new(0.0, 0.0, 1.0)).unwrap();
        let shape = solid.as_shape();
        assert_eq!(shape.faces().len(), 5);
    }

    #[test]
    fn faces_of_single_face_shape() {
        let edges = vec![
            OcEdge::from_pnts(OcPnt::new(0.0, 0.0, 0.0), OcPnt::new(1.0, 0.0, 0.0)).unwrap(),
            OcEdge::from_pnts(OcPnt::new(1.0, 0.0, 0.0), OcPnt::new(0.5, 1.0, 0.0)).unwrap(),
            OcEdge::from_pnts(OcPnt::new(0.5, 1.0, 0.0), OcPnt::new(0.0, 0.0, 0.0)).unwrap(),
        ];
        let wire = OcWire::from_edges(&edges).unwrap();
        let face = OcFace::from_wire(&wire, true).unwrap();
        assert_eq!(face.as_shape().faces().len(), 1);
    }
    #[test]
    fn edges_of_prism() {
        use crate::gp::{OcPnt, OcVec};
        use crate::topo::{OcEdge, OcWire};
        let edges = vec![
            OcEdge::from_pnts(OcPnt::new(0.0, 0.0, 0.0), OcPnt::new(1.0, 0.0, 0.0)).unwrap(),
            OcEdge::from_pnts(OcPnt::new(1.0, 0.0, 0.0), OcPnt::new(0.5, 1.0, 0.0)).unwrap(),
            OcEdge::from_pnts(OcPnt::new(0.5, 1.0, 0.0), OcPnt::new(0.0, 0.0, 0.0)).unwrap(),
        ];
        let wire = OcWire::from_edges(&edges).unwrap();
        let face = OcFace::from_wire(&wire, true).unwrap();
        let solid = face.extrude(OcVec::new(0.0, 0.0, 1.0)).unwrap();
        // TopExp_Explorer visits each edge once per adjacent face, so a prism's
        // 9 edges appear 18 times (each edge bounds exactly 2 faces).
        assert_eq!(solid.as_shape().edges().len(), 18);
    }
}
