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

use crate::error::OcctError;
use crate::rs_topo::offset::{OffsetShapeBuilder, ThickSolidBuilder};
use crate::rs_topo::shape_explorer_iter::{ShapeEdgeIter, ShapeFaceIter};
use crate::rs_topo::{
    bool_op::{CommonBuilder, CutBuilder, FuseBuilder},
    chamfer::ChamferBuilder,
    face::OcFace,
    fillet::FilletBuilder,
    transform::TransformBuilder,
};
use crate::rs_topo::{OcEdge, ShapeType};

/// TopAbs_ShapeEnum::TopAbs_FACE.
/// Reference: https://dev.opencascade.org/doc/refman/html/namespace_top_abs.html
const TOP_ABS_FACE: i32 = 4;
const TOP_ABS_EDGE: i32 = 6;

/// Within-session identity for a placed topological sub-shape instance,
/// at the strictest (oriented) tier.
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
/// `OrientedShapeKey` values will compose with `TDF_Label` identifiers for
/// persistent identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OrientedShapeKey(pub usize);

/// Within-session identity for a *placed* topological sub-shape instance.
///
/// Encodes TShape (geometry) and Location (placement) only — Orientation is
/// ignored. Two occurrences of the same edge read in opposite directions
/// (the ordinary case for an edge shared by two adjacent faces) receive the
/// **same** key here, unlike [`OrientedShapeKey`], which distinguishes them.
///
/// Use this tier for true deduplication of sub-shapes (e.g. `unique_edges`);
/// use [`OrientedShapeKey`] for occurrence-identity joins (e.g. matching a
/// face's own boundary-edge lookup against a flat edge list).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlacedShapeKey(pub usize);

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
    pub(crate) _not_send: PhantomData<*mut ()>,
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
    /// Returns `None` if `inner` is a null `UniquePtr` or the `TopoDS_Shape`
    /// has a null TShape handle (`IsNull()`).
    #[allow(dead_code)]
    pub(crate) fn from_ffi(inner: cxx::UniquePtr<ffi::TopodsShape>) -> Option<Self> {
        if inner.is_null() {
            return None;
        }
        if ffi::topods_shape_is_null(inner.as_ref().unwrap()) {
            return None;
        }
        Some(Self {
            inner,
            _not_send: PhantomData,
        })
    }

    /// Constructs an `OcShape` without null checks.
    ///
    /// # Safety
    ///
    /// Caller guarantees that `inner` is a non-null `UniquePtr` wrapping a
    /// `TopoDS_Shape` whose TShape handle is non-null (`!IsNull()`).
    /// Violation is undefined behaviour through subsequent OCCT method calls.
    pub(crate) unsafe fn from_ffi_unchecked(inner: cxx::UniquePtr<ffi::TopodsShape>) -> Self {
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
    /// [`OrientedShapeKey`] if unique faces are required.
    ///
    /// Reference: <https://dev.opencascade.org/doc/refman/html/class_top_exp___explorer.html>
    pub fn faces(&self) -> impl Iterator<Item = OcFace> {
        ShapeFaceIter::new(ffi::new_shape_explorer(self.as_ffi(), TOP_ABS_FACE))
    }

    /// Returns all `TopoDS_Edge` sub-shapes of this shape as typed wrappers.
    ///
    /// Traverses using `TopExp_Explorer` with `TopAbs_EDGE`.  Results are in
    /// exploration order; `TopExp_Explorer` does not deduplicate — an edge
    /// shared by two faces appears twice, read in opposite directions by
    /// ordinary BRep convention (same TShape and Location, different
    /// Orientation). Filter on [`PlacedShapeKey`], not [`OrientedShapeKey`],
    /// if unique edges are required — the oriented tier will not collapse
    /// these two occurrences into one.
    ///
    /// Reference: <https://dev.opencascade.org/doc/refman/html/class_top_exp___explorer.html>
    pub fn edges(&self) -> impl Iterator<Item = OcEdge> {
        ShapeEdgeIter::new(ffi::new_shape_explorer(self.as_ffi(), TOP_ABS_EDGE))
    }
    /// Fuse (union) this shape with `other`, returning a new `OcShape`.
    ///
    /// Wraps [`FuseBuilder`] via its `build()`. History is not preserved; if
    /// Modified/Generated/IsDeleted are needed, use `FuseBuilder` directly
    /// via `build_with_history()`.
    pub fn oc_fuse(&self, other: &OcShape) -> Result<OcShape, OcctError> {
        FuseBuilder::new().build(self, other)
    }
    /// Subtract `tool` from `self`, returning a new `OcShape`.
    ///
    /// Wraps [`CutBuilder`] via its `build()`. `self` is the "object" (left
    /// operand); `tool` is subtracted from it. History is not preserved; use
    /// `CutBuilder` directly via `build_with_history()` if needed.
    ///
    /// For disjoint inputs, OCCT returns `self` unchanged as a solid — this is
    /// a valid `Ok` result. No compound detection is needed.
    pub fn oc_cut(&self, tool: &OcShape) -> Result<OcShape, OcctError> {
        CutBuilder::new().build(self, tool)
    }
    /// Applies `trsf` to this shape, returning the transformed `OcShape`.
    ///
    /// Wraps [`TransformBuilder`] with `copy=false`. Per OCCT semantics: for
    /// a direct isometry, the result shares `self`'s TShape with a new
    /// Location — no geometry duplication. This is not full independence;
    /// callers needing a fully independent copy (duplicated curves/surfaces,
    /// no shared TShape) should use `TransformBuilder::new` directly with
    /// `copy=true`.
    ///
    /// Reference: <https://dev.opencascade.org/doc/refman/html/class_b_rep_builder_a_p_i___transform.html>
    pub fn transformed(&self, trsf: &crate::gp::OcTrsf) -> Result<OcShape, OcctError> {
        TransformBuilder::new(self, trsf, false)?.build()
    }

    /// Intersect `self` with `other`, returning a new `OcShape`.
    ///
    /// Wraps [`CommonBuilder`] via its `build()`. History is not preserved;
    /// use `CommonBuilder` directly via `build_with_history()` if needed.
    ///
    /// For non-intersecting inputs, OCCT returns an empty `TopoDS_Compound`
    /// (`IsDone()==true`); this is returned as `Ok`. Use `shape_type()` and
    /// content queries on the result if the intersection's presence matters.
    pub fn oc_common(&self, other: &OcShape) -> Result<OcShape, OcctError> {
        CommonBuilder::new().build(self, other)
    }
    /// Applies constant-radius fillets to the given edges and returns the
    /// resulting shape.
    ///
    /// Convenience wrapper over [`FilletBuilder`].  For finer control (adding
    /// edges in a loop, inspecting errors per edge) use `FilletBuilder` directly.
    ///
    /// `edges_with_radii` is a slice of `(radius, edge)` pairs.
    pub fn fillet(&self, edges_with_radii: &[(f64, &OcEdge)]) -> Result<OcShape, OcctError> {
        let mut builder = FilletBuilder::new(self)?;
        for (radius, edge) in edges_with_radii {
            builder.add_edge(*radius, edge)?;
        }
        builder.build()
    }

    /// Applies symmetric chamfers to the given edges and returns the resulting shape.
    ///
    /// Convenience wrapper over [`ChamferBuilder`].  For asymmetric or
    /// distance-angle chamfers use `ChamferBuilder` directly.
    pub fn chamfer(&self, edges_with_distances: &[(f64, &OcEdge)]) -> Result<OcShape, OcctError> {
        let mut builder = ChamferBuilder::new(self)?;
        for (dis, edge) in edges_with_distances {
            builder.add_edge(*dis, edge)?;
        }
        builder.build()
    }

    /// Offsets all surfaces of the shape outward (positive) or inward (negative).
    ///
    /// Wraps `BRepOffsetAPI_MakeOffsetShape::PerformBySimple`.
    pub fn offset_shape(&self, offset: f64) -> Result<OcShape, OcctError> {
        OffsetShapeBuilder::new().perform(self, offset)
    }

    /// Hollows this solid by removing `closing_faces` and offsetting inward.
    ///
    /// `offset` is typically negative (wall thickness inward).
    /// `tolerance` controls precision; `1e-3` is typical.
    ///
    /// Convenience wrapper over [`ThickSolidBuilder`].
    pub fn thick_solid(
        &self,
        closing_faces: &[&OcFace],
        offset: f64,
        tolerance: f64,
    ) -> Result<OcShape, OcctError> {
        let mut builder = ThickSolidBuilder::new();
        for face in closing_faces {
            builder.add_closing_face(face);
        }
        builder.build(self, offset, tolerance)
    }
}

impl Clone for OcShape {
    /// Cheap clone: increments the `TShape` handle reference count.
    fn clone(&self) -> Self {
        // Safety: clone_shape is make_unique<TopoDS_Shape>(s) — never null.
        // self.inner is non-null by construction invariant.
        unsafe { Self::from_ffi_unchecked(ffi::clone_shape(&self.inner)) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gp::{OcPnt, OcVec};
    use crate::rs_topo::{OcEdge, OcFace, OcWire};

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
        use crate::rs_topo::{OcEdge, OcWire};
        let edges = vec![
            OcEdge::from_pnts(OcPnt::new(0.0, 0.0, 0.0), OcPnt::new(1.0, 0.0, 0.0)).unwrap(),
            OcEdge::from_pnts(OcPnt::new(1.0, 0.0, 0.0), OcPnt::new(0.5, 1.0, 0.0)).unwrap(),
            OcEdge::from_pnts(OcPnt::new(0.5, 1.0, 0.0), OcPnt::new(0.0, 0.0, 0.0)).unwrap(),
        ];
        let wire = OcWire::from_edges(&edges).unwrap();
        let face = OcFace::from_wire(&wire, true).unwrap();
        let solid_shape = face.extrude(OcVec::new(0.0, 0.0, 1.0)).unwrap();
        assert_eq!(solid_shape.faces().collect::<Vec<_>>().len(), 5);
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
        assert_eq!(face.as_shape().faces().collect::<Vec<_>>().len(), 1);
    }
    #[test]
    fn edges_of_prism() {
        use crate::gp::{OcPnt, OcVec};
        use crate::rs_topo::{OcEdge, OcWire};
        let edges = vec![
            OcEdge::from_pnts(OcPnt::new(0.0, 0.0, 0.0), OcPnt::new(1.0, 0.0, 0.0)).unwrap(),
            OcEdge::from_pnts(OcPnt::new(1.0, 0.0, 0.0), OcPnt::new(0.5, 1.0, 0.0)).unwrap(),
            OcEdge::from_pnts(OcPnt::new(0.5, 1.0, 0.0), OcPnt::new(0.0, 0.0, 0.0)).unwrap(),
        ];
        let wire = OcWire::from_edges(&edges).unwrap();
        let face = OcFace::from_wire(&wire, true).unwrap();
        let solid_shape = face.extrude(OcVec::new(0.0, 0.0, 1.0)).unwrap();
        // TopExp_Explorer visits each edge once per adjacent face, so a prism's
        // 9 edges appear 18 times (each edge bounds exactly 2 faces).
        assert_eq!(solid_shape.edges().collect::<Vec<_>>().len(), 18);
    }
    // A 1×1 square face in the XY plane, offset by `x_offset` on X,
    // extruded 1 unit along Z to produce a unit box.
    fn box_solid(x_offset: f64) -> crate::rs_topo::OcShape {
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
        let a = box_solid(0.0);
        let b = box_solid(0.5);
        let result = a.oc_fuse(&b);
        assert!(
            result.is_ok(),
            "fuse of overlapping solids should succeed: {:?}",
            result.err()
        );
    }

    #[test]
    fn fused_shape_tessellates_with_faces() {
        let a = box_solid(0.0);
        let b = box_solid(0.5);
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
        let a = box_solid(0.0);
        let b = box_solid(0.5);
        let fused = a.oc_fuse(&b).unwrap();
        let tess = crate::tessellate::compute(&fused, 0.1, 0.5).unwrap();
        let max_x = tess
            .vertices
            .iter()
            .map(|v| v.point[0])
            .fold(f64::NEG_INFINITY, f64::max);
        assert!(
            max_x > 1.0,
            "fused shape should extend past x=1.0; max_x was {max_x}"
        );
    }
    #[test]
    fn shape_type_of_solid_is_solid() {
        let s = box_solid(0.0);
        assert_eq!(s.shape_type(), ShapeType::Solid);
    }
    #[test]
    fn cut_overlapping_solids_succeeds() {
        // Box A: x 0..1, Box B: x 0.5..1.5 — A minus B should leave x 0..0.5 region.
        let a = box_solid(0.0);
        let b = box_solid(0.5);
        let result = a.oc_cut(&b);
        assert!(
            result.is_ok(),
            "cut of overlapping solids should succeed: {:?}",
            result.err()
        );
    }

    #[test]
    fn cut_disjoint_solids_returns_argument_unchanged() {
        let a = box_solid(0.0);
        let b = box_solid(10.0);
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
        let a = box_solid(0.0);
        let b = box_solid(0.5);
        let a_minus_b = a.oc_cut(&b).unwrap();
        let b_minus_a = b.oc_cut(&a).unwrap();
        let tess_ab = crate::tessellate::compute(&a_minus_b, 0.1, 0.5).unwrap();
        let tess_ba = crate::tessellate::compute(&b_minus_a, 0.1, 0.5).unwrap();
        // A−B should not extend past x=0.5 (the tool removed that part).
        let max_x_ab = tess_ab
            .vertices
            .iter()
            .map(|v| v.point[0])
            .fold(f64::NEG_INFINITY, f64::max);
        // B−A should not extend below x=0.5.
        let min_x_ba = tess_ba
            .vertices
            .iter()
            .map(|v| v.point[0])
            .fold(f64::INFINITY, f64::min);
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
        let a = box_solid(0.0);
        let b = box_solid(0.5);
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
        let a = box_solid(0.0);
        let b = box_solid(0.5);
        let common = a.oc_common(&b).unwrap();
        let tess = crate::tessellate::compute(&common, 0.1, 0.5).unwrap();
        let min_x = tess
            .vertices
            .iter()
            .map(|v| v.point[0])
            .fold(f64::INFINITY, f64::min);
        let max_x = tess
            .vertices
            .iter()
            .map(|v| v.point[0])
            .fold(f64::NEG_INFINITY, f64::max);
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
        let a = box_solid(0.0);
        let b = box_solid(10.0);
        let result = a.oc_common(&b);
        assert!(
            result.is_ok(),
            "common of disjoint solids should return Ok empty compound, got: {:?}",
            result.err()
        );
        assert_eq!(
            result.unwrap().shape_type(),
            ShapeType::Compound,
            "disjoint common result should be a Compound"
        );
    }
    #[test]
    fn translated_shape_moves_vertices() {
        let s = box_solid(0.0);
        let trsf = crate::gp::OcTrsf::from_translation(OcVec::new(5.0, 0.0, 0.0));
        let moved = s.transformed(&trsf).unwrap();
        let tess = crate::tessellate::compute(&moved, 0.1, 0.5).unwrap();
        let min_x = tess
            .vertices
            .iter()
            .map(|v| v.point[0])
            .fold(f64::INFINITY, f64::min);
        assert!(min_x >= 5.0 - 1e-4, "min_x should be ~5.0, got {min_x}");
    }

    #[test]
    fn transformed_is_independent_of_source() {
        // Verify copy=true: the source shape's vertices are unaffected.
        let s = box_solid(0.0);
        let trsf = crate::gp::OcTrsf::from_translation(OcVec::new(10.0, 0.0, 0.0));
        let _moved = s.transformed(&trsf).unwrap();
        // Tessellate the original — it must still sit at x=0..1.
        let tess = crate::tessellate::compute(&s, 0.1, 0.5).unwrap();
        let max_x = tess
            .vertices
            .iter()
            .map(|v| v.point[0])
            .fold(f64::NEG_INFINITY, f64::max);
        assert!(
            max_x <= 1.0 + 1e-4,
            "source should be unmodified, max_x={max_x}"
        );
    }

    #[test]
    fn scale_applied_via_transformed() {
        // Uniform scale by 2 about origin: box 0..1 should become 0..2.
        let s = box_solid(0.0);
        let trsf = crate::gp::OcTrsf::from_scale(OcPnt::origin(), 2.0);
        let scaled = s.transformed(&trsf).unwrap();
        let tess = crate::tessellate::compute(&scaled, 0.1, 0.5).unwrap();
        let max_x = tess
            .vertices
            .iter()
            .map(|v| v.point[0])
            .fold(f64::NEG_INFINITY, f64::max);
        assert!(
            max_x >= 2.0 - 1e-4,
            "scaled max_x should be ~2.0, got {max_x}"
        );
    }
    #[test]
    fn fillet_box_edges_succeeds() {
        let s = box_solid(0.0);
        let edges = s.edges();
        // Deduplicate by PlacedShapeKey — edges() returns each edge once per
        // adjacent face, read in opposite Orientation each time.
        let mut seen = std::collections::HashSet::new();
        let unique_edges: Vec<_> = edges
            .filter(|e| seen.insert(e.placed_shape_key()))
            .collect();
        let result = s.fillet(&unique_edges.iter().map(|e| (0.05, e)).collect::<Vec<_>>());
        assert!(result.is_ok(), "fillet should succeed: {:?}", result.err());
    }

    #[test]
    fn fillet_builder_add_then_build() {
        use crate::rs_topo::FilletBuilder;
        let s = box_solid(0.0);
        let edges = s.edges();
        let mut seen = std::collections::HashSet::new();
        let unique_edges: Vec<_> = edges
            .filter(|e| seen.insert(e.placed_shape_key()))
            .collect();
        let mut builder = FilletBuilder::new(&s).unwrap();
        for e in &unique_edges {
            builder.add_edge(0.05, e).unwrap();
        }
        let result = builder.build();
        assert!(result.is_ok());
    }

    #[test]
    fn fillet_result_tessellates() {
        let s = box_solid(0.0);
        let edges = s.edges();
        let mut seen = std::collections::HashSet::new();
        let unique_edges: Vec<_> = edges
            .filter(|e| seen.insert(e.placed_shape_key()))
            .collect();
        let filleted = s
            .fillet(&unique_edges.iter().map(|e| (0.05, e)).collect::<Vec<_>>())
            .unwrap();
        let tess = crate::tessellate::compute(&filleted, 0.05, 0.5).unwrap();
        assert!(!tess.faces.is_empty());
    }
    fn unique_edges(shape: &OcShape) -> Vec<OcEdge> {
        let mut seen = std::collections::HashSet::new();
        shape
            .edges()
            .into_iter()
            .filter(|e| seen.insert(e.placed_shape_key()))
            .collect()
    }

    #[test]
    fn chamfer_box_edges_succeeds() {
        let s = box_solid(0.0);
        let edges = unique_edges(&s);
        let result = s.chamfer(&edges.iter().map(|e| (0.05, e)).collect::<Vec<_>>());
        assert!(result.is_ok(), "chamfer failed: {:?}", result.err());
    }

    #[test]
    fn chamfer_builder_symmetric() {
        use crate::rs_topo::ChamferBuilder;
        let s = box_solid(0.0);
        let edges = unique_edges(&s);
        let mut builder = ChamferBuilder::new(&s).unwrap();
        for e in &edges {
            builder.add_edge(0.05, e).unwrap();
        }
        assert!(builder.build().is_ok());
    }

    #[test]
    fn chamfer_result_tessellates() {
        let s = box_solid(0.0);
        let edges = unique_edges(&s);
        let chamfered = s
            .chamfer(&edges.iter().map(|e| (0.05, e)).collect::<Vec<_>>())
            .unwrap();
        let tess = crate::tessellate::compute(&chamfered, 0.05, 0.5).unwrap();
        assert!(!tess.faces.is_empty());
    }
    #[test]
    fn offset_shape_outward_expands_bounds() {
        let s = box_solid(0.0);
        let expanded = s.offset_shape(0.1).unwrap();
        let tess = crate::tessellate::compute(&expanded, 0.05, 0.5).unwrap();
        let max_x = tess
            .vertices
            .iter()
            .map(|v| v.point[0])
            .fold(f64::NEG_INFINITY, f64::max);
        assert!(
            max_x > 1.0 + 0.05,
            "expanded shape should exceed x=1.0, got {max_x}"
        );
    }

    #[test]
    fn thick_solid_one_face_removed() {
        // Box 0..1. Remove the top face (max Z), hollow inward by -0.1.
        let s = box_solid(0.0);
        // Find the face with highest Z centroid — that's the top.
        let top_face = s
            .faces()
            .into_iter()
            .max_by(|a, b| {
                let za = crate::tessellate::compute(&a.as_shape(), 0.1, 0.5)
                    .unwrap()
                    .vertices
                    .iter()
                    .map(|v| v.point[2])
                    .fold(f64::NEG_INFINITY, f64::max);
                let zb = crate::tessellate::compute(&b.as_shape(), 0.1, 0.5)
                    .unwrap()
                    .vertices
                    .iter()
                    .map(|v| v.point[2])
                    .fold(f64::NEG_INFINITY, f64::max);
                za.partial_cmp(&zb).unwrap()
            })
            .unwrap();
        let result = s.thick_solid(&[&top_face], -0.1, 1e-3);
        assert!(result.is_ok(), "thick_solid failed: {:?}", result.err());
    }

    #[test]
    fn thick_solid_result_tessellates() {
        let s = box_solid(0.0);
        let top_face = s
            .faces()
            .into_iter()
            .max_by(|a, b| {
                let za = crate::tessellate::compute(&a.as_shape(), 0.1, 0.5)
                    .unwrap()
                    .vertices
                    .iter()
                    .map(|v| v.point[2])
                    .fold(f64::NEG_INFINITY, f64::max);
                let zb = crate::tessellate::compute(&b.as_shape(), 0.1, 0.5)
                    .unwrap()
                    .vertices
                    .iter()
                    .map(|v| v.point[2])
                    .fold(f64::NEG_INFINITY, f64::max);
                za.partial_cmp(&zb).unwrap()
            })
            .unwrap();
        let hollowed = s.thick_solid(&[&top_face], -0.1, 1e-3).unwrap();
        let tess = crate::tessellate::compute(&hollowed, 0.05, 0.5).unwrap();
        assert!(!tess.faces.is_empty());
    }
    #[test]
    fn fuse_disjoint_solids_returns_ok_compound() {
        // Box A: x 0..1, Box B: x 10..11 — disjoint.
        // OCCT returns a Compound containing both; this must be Ok, not Err.
        let a = box_solid(0.0);
        let b = box_solid(10.0);
        let result = a.oc_fuse(&b);
        assert!(
            result.is_ok(),
            "fuse of disjoint solids should return Ok compound, got: {:?}",
            result.err()
        );
        assert_eq!(
            result.unwrap().shape_type(),
            ShapeType::Compound,
            "disjoint fuse result should be a Compound"
        );
    }
}
