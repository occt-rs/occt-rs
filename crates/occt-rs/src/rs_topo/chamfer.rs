//! Constant and asymmetric chamfer builder.
//!
//! Three edge-registration modes:
//! - [`add_edge`] — symmetric chamfer (equal distance both sides)
//! - [`add_edge_asymmetric`] — two-distance chamfer
//! - [`add_edge_dist_angle`] — distance-angle chamfer
//!
//! For symmetric chamfers on all edges, prefer [`OcShape::chamfer`].
//!
//! History (`Modified`, `Generated`) deferred to F2.
//!
//! Reference: <https://dev.opencascade.org/doc/refman/html/class_b_rep_fillet_a_p_i___make_chamfer.html>
//!
//! [`add_edge`]: ChamferBuilder::add_edge
//! [`add_edge_asymmetric`]: ChamferBuilder::add_edge_asymmetric
//! [`add_edge_dist_angle`]: ChamferBuilder::add_edge_dist_angle

use std::marker::PhantomData;

use occt_sys::ffi;

use crate::error::{OcctError, OcctErrorKind};
use crate::rs_topo::shape_history_iter::ShapeListIter;
use crate::rs_topo::{BuiltWithHistory, HistoryProvider, OcEdge, OcFace, OcShape};

/// Builder for chamfer operations on a solid.
///
/// In the scenario, a chamfer is applied to one edge of the extruded solid.
/// The chamfer produces a new planar face where that edge was — `sketch2`
/// is then drawn on this face. The modified-face information from
/// [`build_with_history`](ChamferBuilder::build_with_history) feeds directly
/// into [`TopoNamingBuilder::modified`] so the document knows which faces
/// changed.
///
/// A chamfer is used here rather than a fillet because the chamfer produces
/// a flat face — easier to reason about when learning the naming scheme.
///
/// The example below extends the document established in
/// [`TopoNamingNamedShape`], adding the chamfer result under `body/2`:
///
/// ```text
/// main (0:1)
/// └── 3 (0:1:3)   body
///     ├── 1 (0:1:3:1)   cube
///     │       TopoNamingNamedShape (Primitive, 1×1×1 prism)
///     │       OcReal "depth" = 1.0
///     └── 2 (0:1:3:2)   chamfered cube
///             TopoNamingNamedShape (Modify, chamfered solid)
///             OcReal "distance" = 0.05
/// ```
///
/// ```
/// use occt_rs::gp::{OcPnt, OcVec};
/// use occt_rs::ocaf::OcApplication;
/// use occt_rs::ocaf::attributes::OcReal;
/// use occt_rs::ocaf::topo_naming::{TopoNamingEvolution, TopoNamingNamedShape};
/// use occt_rs::rs_topo::{ChamferBuilder, OcEdge, OcFace, OcWire};
///
/// let mut app = OcApplication::new();
/// let mut doc = app.new_document("BinXCAF").unwrap();
/// doc.set_undo_limit(10);
///
/// let main = doc.main();
///
/// // Build and record the solid on body/1
/// let wire = OcWire::from_edges(&[
///     OcEdge::from_pnts(OcPnt::new(0.0, 0.0, 0.0), OcPnt::new(1.0, 0.0, 0.0)).unwrap(),
///     OcEdge::from_pnts(OcPnt::new(1.0, 0.0, 0.0), OcPnt::new(1.0, 1.0, 0.0)).unwrap(),
///     OcEdge::from_pnts(OcPnt::new(1.0, 1.0, 0.0), OcPnt::new(0.0, 1.0, 0.0)).unwrap(),
///     OcEdge::from_pnts(OcPnt::new(0.0, 1.0, 0.0), OcPnt::new(0.0, 0.0, 0.0)).unwrap(),
/// ]).unwrap();
/// let solid_shape = OcFace::from_wire(&wire, true).unwrap()
///     .extrude(OcVec::new(0.0, 0.0, 1.0)).unwrap();
///
/// let (solid_label, pre_faces) = {
///     doc.begin_command().unwrap();
///     let body   = main.get_or_create_child(3);
///     let lsolid = body.get_or_create_child(1);
///
///     OcReal::set(&lsolid, 1.0).unwrap();
///     doc.name_builder(&lsolid).primitive(&solid_shape);
///
///     doc.commit().unwrap();
///
///     let faces: Vec<_> = solid_shape.faces().collect();
///     (lsolid, faces)
/// };
///
/// // ── command: chamfer one edge, record result on body/2 ────────────────────
/// //
/// // The chamfer distance is stored first as the authoritative parameter.
/// // The result is recorded as Modify — it evolved from the solid at body/1.
/// let chamfer_label = {
///     doc.begin_command().unwrap();
///     let body     = main.get_or_create_child(3);
///     let lchamfer = body.get_or_create_child(2);
///
///     // Store the chamfer distance — the authoritative parameter
///     let distance = OcReal::set(&lchamfer, 0.05).unwrap();
///
///     // Apply the chamfer using the stored distance
///     // In this case, we are using naive edge selection. If done properly, the edge selection
///     // here would be done using topo-naming solutions.
///     let selected_edge = solid_shape.edges().next().unwrap();
///     let mut cb = ChamferBuilder::new(&solid_shape).unwrap();
///     cb.add_edge(distance.get(), &selected_edge).unwrap();
///     let mut built = cb.build_with_history().unwrap();
///
///     // Record the result and which faces were modified
///     let mut nb = doc.name_builder(&lchamfer);
///     for face in &pre_faces {
///         for modified in built.modified(&face.as_shape()) {
///             nb.modified(&face.as_shape(), &modified);
///         }
///     }
///
///     doc.commit().unwrap();
///     lchamfer
/// };
///
/// let ns = TopoNamingNamedShape::find(&chamfer_label).unwrap();
/// assert_eq!(ns.evolution(), Some(TopoNamingEvolution::Modify));
/// assert!((OcReal::find(&chamfer_label).unwrap().get() - 0.05).abs() < 1e-12);
///
/// // Undo removes the chamfer — solid is still present
/// doc.undo().unwrap();
/// assert!(TopoNamingNamedShape::find(&chamfer_label).is_none());
/// assert!(TopoNamingNamedShape::find(&solid_label).is_some());
///
/// // Redo restores it
/// doc.redo().unwrap();
/// assert!(TopoNamingNamedShape::find(&chamfer_label).is_some());
/// ```
///
/// [`TopoNamingNamedShape`]: crate::ocaf::topo_naming::TopoNamingNamedShape
/// [`TopoNamingBuilder::modified`]: crate::ocaf::topo_naming::TopoNamingBuilder::modified
/// [`FilletBuilder`]: crate::rs_topo::FilletBuilder
pub struct ChamferBuilder {
    pub(crate) inner: cxx::UniquePtr<ffi::MakeChamferBuilder>,
    _not_send: PhantomData<*mut ()>,
}

impl ChamferBuilder {
    /// Constructs a chamfer builder on `shape`.
    pub fn new(shape: &OcShape) -> Result<Self, OcctError> {
        let inner = ffi::new_make_chamfer_builder(shape.as_ffi()).map_err(OcctError::from)?;
        Ok(Self {
            inner,
            _not_send: PhantomData,
        })
    }

    /// Registers a symmetric chamfer on `edge` (equal distance on both sides).
    pub fn add_edge(&mut self, dis: f64, edge: &OcEdge) -> Result<(), OcctError> {
        self.inner
            .pin_mut()
            .add_edge(dis, edge.as_ffi())
            .map_err(OcctError::from)
    }

    /// Registers an asymmetric two-distance chamfer on `edge`.
    ///
    /// `face` selects which side receives `dis1`; `dis2` is applied to the
    /// opposite side.  `face` must be one of the two faces adjacent to `edge`.
    pub fn add_edge_asymmetric(
        &mut self,
        dis1: f64,
        dis2: f64,
        edge: &OcEdge,
        face: &OcFace,
    ) -> Result<(), OcctError> {
        self.inner
            .pin_mut()
            .add_edge_asymmetric(dis1, dis2, edge.as_ffi(), face.as_ffi())
            .map_err(OcctError::from)
    }

    /// Registers a distance-angle chamfer on `edge`.
    ///
    /// `face` selects which side receives `dis`; `angle` (radians) is applied
    /// to the opposite side.  `face` must be one of the two faces adjacent to
    /// `edge`.
    pub fn add_edge_dist_angle(
        &mut self,
        dis: f64,
        angle: f64,
        edge: &OcEdge,
        face: &OcFace,
    ) -> Result<(), OcctError> {
        self.inner
            .pin_mut()
            .add_edge_dist_angle(dis, angle, edge.as_ffi(), face.as_ffi())
            .map_err(OcctError::from)
    }

    /// Computes the chamfer operation and returns the resulting shape.
    ///
    /// Consumes `self`.
    pub fn build(mut self) -> Result<OcShape, OcctError> {
        self.try_build()
    }
    pub fn build_with_history(mut self) -> Result<BuiltWithHistory<Self>, OcctError> {
        let shape = self.try_build()?;
        Ok(BuiltWithHistory::new(self, shape))
    }

    fn try_build(&mut self) -> Result<OcShape, OcctError> {
        self.inner.pin_mut().build().map_err(OcctError::from)?;
        if self.inner.is_done() {
            // Safety: MakeChamferBuilder::shape() returns make_unique<TopoDS_Shape>
            // on a completed builder — non-null.
            Ok(unsafe { OcShape::from_ffi_unchecked(self.inner.pin_mut().shape()) })
        } else {
            Err(OcctError {
                kind: OcctErrorKind::ConstructionError,
                message: "BRepFilletAPI_MakeChamfer: IsDone() false after Build()".to_owned(),
            })
        }
    }
}
impl HistoryProvider for ChamferBuilder {
    fn modified_shapes(&mut self, input: &OcShape) -> impl Iterator<Item = OcShape> + '_ {
        ShapeListIter::new(ffi::chamfer_modified_iter(
            self.inner.pin_mut(),
            input.as_ffi(),
        ))
    }
    fn generated_shapes(&mut self, input: &OcShape) -> impl Iterator<Item = OcShape> + '_ {
        ShapeListIter::new(ffi::chamfer_generated_iter(
            self.inner.pin_mut(),
            input.as_ffi(),
        ))
    }
    fn is_shape_deleted(&mut self, input: &OcShape) -> bool {
        self.inner.pin_mut().is_deleted(input.as_ffi())
    }
}
