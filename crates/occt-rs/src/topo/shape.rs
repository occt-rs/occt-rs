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

use crate::{
    error::{CommonError, FuseError},
    topo::{face::OcFace, ShapeType},
    OcEdge, OcctError,
};

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
    pub fn shape_type(&self) -> ShapeType {
        ShapeType::from(occt_sys::ffi::topods_shape_type(self.as_ffi()))
    }
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
    /// Fuse (union) this shape with `other`, returning a new `OcShape`.
    ///
    /// Wraps `BRepAlgoAPI_Fuse` via the preferred SetArguments/SetTools/Build
    /// pattern. The builder and its history are not preserved; if Modified/
    /// Generated/IsDeleted are needed in future, promote to an explicit FuseBuilder.
    pub fn oc_fuse(&self, other: &OcShape) -> Result<OcShape, FuseError> {
        let result = occt_sys::ffi::fuse_shapes(
            self.inner
                .as_ref()
                .expect("OcShape invariant: inner is non-null"),
            other
                .inner
                .as_ref()
                .expect("OcShape invariant: inner is non-null"),
        )
        .map_err(|e| FuseError::Occt(e.into()))?;
        let result = OcShape::from_ffi(result);
        if result.shape_type() == ShapeType::Compound
            && occt_sys::ffi::topods_compound_child_count(result.as_ffi()) > 1
        {
            return Err(FuseError::DisjointInputs(result));
        }
        Ok(result)
    }
    /// Subtract `tool` from `self`, returning a new `OcShape`.
    ///
    /// Wraps `BRepAlgoAPI_Cut` via the preferred SetArguments/SetTools/Build
    /// pattern. `self` is the "object" (left operand); `tool` is subtracted from it.
    ///
    /// For disjoint inputs, OCCT returns `self` unchanged as a solid — this is
    /// a valid `Ok` result. No compound detection is needed.
    pub fn oc_cut(&self, tool: &OcShape) -> Result<OcShape, OcctError> {
        let result = occt_sys::ffi::cut_shapes(
            self.inner
                .as_ref()
                .expect("OcShape invariant: inner is non-null"),
            tool.inner
                .as_ref()
                .expect("OcShape invariant: inner is non-null"),
        )
        .map_err(|e| OcctError::from(e))?;
        Ok(OcShape::from_ffi(result))
    }

    /// Intersect `self` with `other`, returning a new `OcShape`.
    ///
    /// Wraps `BRepAlgoAPI_Common` via the preferred SetArguments/SetTools/Build
    /// pattern.
    ///
    /// Returns `Err(CommonError::NoIntersection)` when the inputs are disjoint —
    /// OCCT returns an empty `TopoDS_Compound` in this case (`IsDone()==true`).
    /// The empty compound is not forwarded to the caller.
    pub fn oc_common(&self, other: &OcShape) -> Result<OcShape, CommonError> {
        let result = occt_sys::ffi::common_shapes(
            self.inner
                .as_ref()
                .expect("OcShape invariant: inner is non-null"),
            other
                .inner
                .as_ref()
                .expect("OcShape invariant: inner is non-null"),
        )
        .map_err(|e| CommonError::Occt(e.into()))?;
        let result = OcShape::from_ffi(result);
        if occt_sys::ffi::topods_compound_child_count(result.as_ffi()) == 0 {
            return Err(CommonError::NoIntersection);
        }
        Ok(result)
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
    use crate::OcVec;

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
    // A 1×1 square face in the XY plane, offset by `x_offset` on X,
    // extruded 1 unit along Z to produce a unit box.
    fn box_solid(x_offset: f64) -> crate::topo::OcSolid {
        let x0 = x_offset;
        let x1 = x_offset + 1.0;
        let edges = vec![
            OcEdge::from_pnts(OcPnt::new(x0, 0.0, 0.0), OcPnt::new(x1, 0.0, 0.0)).unwrap(),
            OcEdge::from_pnts(OcPnt::new(x1, 0.0, 0.0), OcPnt::new(x1, 1.0, 0.0)).unwrap(),
            OcEdge::from_pnts(OcPnt::new(x1, 1.0, 0.0), OcPnt::new(x0, 1.0, 0.0)).unwrap(),
            OcEdge::from_pnts(OcPnt::new(x0, 1.0, 0.0), OcPnt::new(x0, 0.0, 0.0)).unwrap(),
        ];
        let wire = OcWire::from_edges(&edges).unwrap();
        let face = OcFace::from_wire(&wire, true).unwrap();
        face.extrude(OcVec::new(0.0, 0.0, 1.0)).unwrap()
    }

    #[test]
    fn fuse_overlapping_solids_succeeds() {
        // Box A: x 0..1, Box B: x 0.5..1.5 — they overlap in x 0.5..1.
        let a = box_solid(0.0).as_shape();
        let b = box_solid(0.5).as_shape();
        let result = a.oc_fuse(&b);
        assert!(
            result.is_ok(),
            "fuse of overlapping solids should succeed: {:?}",
            result.err()
        );
    }

    #[test]
    fn fused_shape_tessellates_with_faces() {
        let a = box_solid(0.0).as_shape();
        let b = box_solid(0.5).as_shape();
        let fused = a.oc_fuse(&b).unwrap();
        let tess = crate::tessellate::compute(&fused, 0.1, 0.5)
            .expect("tessellation of fused shape should not fail");
        assert!(
            !tess.faces.is_empty(),
            "fused shape should produce at least one tessellated face"
        );
    }

    #[test]
    fn fuse_is_not_identity_of_either_input() {
        // The fused bounding box spans both inputs.
        // Tessellate vertex x-coords should exceed x=1.0, proving B was included.
        let a = box_solid(0.0).as_shape();
        let b = box_solid(0.5).as_shape();
        let fused = a.oc_fuse(&b).unwrap();
        let tess = crate::tessellate::compute(&fused, 0.1, 0.5).unwrap();
        let max_x = tess
            .vertices
            .iter()
            .map(|v| v.point[0])
            .fold(f32::NEG_INFINITY, f32::max);
        assert!(
            max_x > 1.0,
            "fused shape should extend past x=1.0; max_x was {max_x}"
        );
    }
    #[test]
    fn shape_type_of_solid_is_solid() {
        let s = box_solid(0.0).as_shape();
        assert_eq!(s.shape_type(), ShapeType::Solid);
    }
    #[test]
    fn cut_overlapping_solids_succeeds() {
        // Box A: x 0..1, Box B: x 0.5..1.5 — A minus B should leave x 0..0.5 region.
        let a = box_solid(0.0).as_shape();
        let b = box_solid(0.5).as_shape();
        let result = a.oc_cut(&b);
        assert!(
            result.is_ok(),
            "cut of overlapping solids should succeed: {:?}",
            result.err()
        );
    }

    #[test]
    fn cut_disjoint_solids_returns_argument_unchanged() {
        let a = box_solid(0.0).as_shape();
        let b = box_solid(10.0).as_shape();
        let result = a.oc_cut(&b);
        assert!(
            result.is_ok(),
            "cut of disjoint solids should succeed: {:?}",
            result.err()
        );
        // OCCT wraps the result in a TopoDS_Compound (as with all boolean ops).
        // The compound contains the argument shape unchanged.
        let tess = crate::tessellate::compute(&result.unwrap(), 0.1, 0.5).unwrap();
        assert!(
            !tess.faces.is_empty(),
            "disjoint cut result should tessellate"
        );
    }

    #[test]
    fn cut_is_noncommutative() {
        // A.cut(B) and B.cut(A) should produce geometrically distinct results.
        let a = box_solid(0.0).as_shape();
        let b = box_solid(0.5).as_shape();
        let a_minus_b = a.oc_cut(&b).unwrap();
        let b_minus_a = b.oc_cut(&a).unwrap();
        let tess_ab = crate::tessellate::compute(&a_minus_b, 0.1, 0.5).unwrap();
        let tess_ba = crate::tessellate::compute(&b_minus_a, 0.1, 0.5).unwrap();
        // A−B should not extend past x=0.5 (the tool removed that part).
        let max_x_ab = tess_ab
            .vertices
            .iter()
            .map(|v| v.point[0])
            .fold(f32::NEG_INFINITY, f32::max);
        // B−A should not extend below x=0.5.
        let min_x_ba = tess_ba
            .vertices
            .iter()
            .map(|v| v.point[0])
            .fold(f32::INFINITY, f32::min);
        assert!(
            max_x_ab <= 0.5 + 1e-4,
            "A-B should not extend past x=0.5, got {max_x_ab}"
        );
        assert!(
            min_x_ba >= 0.5 - 1e-4,
            "B-A should not extend below x=0.5, got {min_x_ba}"
        );
    }

    #[test]
    fn common_overlapping_solids_succeeds() {
        let a = box_solid(0.0).as_shape();
        let b = box_solid(0.5).as_shape();
        let result = a.oc_common(&b);
        assert!(
            result.is_ok(),
            "common of overlapping solids should succeed: {:?}",
            result.err()
        );
    }

    #[test]
    fn common_overlap_region_is_correct() {
        // Intersection of x 0..1 and x 0.5..1.5 should be x 0.5..1.
        let a = box_solid(0.0).as_shape();
        let b = box_solid(0.5).as_shape();
        let common = a.oc_common(&b).unwrap();
        let tess = crate::tessellate::compute(&common, 0.1, 0.5).unwrap();
        let min_x = tess
            .vertices
            .iter()
            .map(|v| v.point[0])
            .fold(f32::INFINITY, f32::min);
        let max_x = tess
            .vertices
            .iter()
            .map(|v| v.point[0])
            .fold(f32::NEG_INFINITY, f32::max);
        assert!(
            min_x >= 0.5 - 1e-4,
            "common min_x should be ~0.5, got {min_x}"
        );
        assert!(
            max_x <= 1.0 + 1e-4,
            "common max_x should be ~1.0, got {max_x}"
        );
    }

    #[test]
    fn common_disjoint_solids_returns_no_intersection() {
        let a = box_solid(0.0).as_shape();
        let b = box_solid(10.0).as_shape();
        let result = a.oc_common(&b);
        assert!(
            matches!(result, Err(CommonError::NoIntersection)),
            "common of disjoint solids should return NoIntersection, got: {:?}",
            result.ok().map(|_| "Ok")
        );
    }
}
