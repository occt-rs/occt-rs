// Sourced from OCCT 7.9 documentation.
// No derivation from any other binding crate.
//
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_naming___builder.html
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_naming___named_shape.html

use std::marker::PhantomData;

use cxx::UniquePtr;
use occt_sys::ffi::{self, new_tnaming_builder};

use super::label::OcLabel;
use crate::rs_topo::OcShape;

// ---------------------------------------------------------------------------
// TopoNamingEvolution
// Maps TNaming_Evolution integer values from OCCT.
// Reference: https://dev.opencascade.org/doc/refman/html/group__enum__t_naming.html
// ---------------------------------------------------------------------------

/// Encodes the Evolution kind of a set of [`OcShape`]s
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopoNamingEvolution {
    /// Shape appeared fresh — no topological ancestor.
    Primitive,
    /// Shape was generated from an ancestor. Carries an `old/new` shape-pair
    Generated,
    /// Shape is a modification of an ancestor. Carries an `old/new` shape-pair
    Modify,
    /// Ancestor deletion.
    Delete,
    #[deprecated(note = "Seems to be retained in OCCT for legacy reasons")]
    Replace,
    /// Sub-shape selection in context — used by DOC-4 / TNaming_Selector.
    Selected,
}

impl TopoNamingEvolution {
    fn try_from_raw(v: i32) -> Option<Self> {
        // TNaming_Evolution enum constants — verify ordinal values against
        // https://dev.opencascade.org/doc/refman/html/group__enum__t_naming.html
        // if undo behaviour is unexpected.
        match v {
            0 => Some(Self::Primitive),
            1 => Some(Self::Generated),
            2 => Some(Self::Modify),
            3 => Some(Self::Delete),
            #[allow(deprecated)]
            4 => Some(Self::Replace),
            5 => Some(Self::Selected),
            _ => None,
        }
    }
}

/// Records shape provenance: A principal tool for addressing the Topo Naming Problem
///
/// Every shape in the document — whether a primitive construction or the
/// result of an operation — is recorded on a label using `TopoNamingBuilder`.
/// This is what connects raw OCCT geometry to the OCAF document model: the
/// shape becomes addressable, referenceable, and survives parametric rebuild.
///
/// The evolution kind tells the naming graph what happened:
/// - [`Primitive`] — shape appeared fresh, no topological ancestor
/// - [`Modify`] — shape is a modification of an ancestor
/// - [`Generated`] — shape was generated from an ancestor sub-shape
/// - [`Delete`] — ancestor was consumed and does not appear in the output
///
/// The example below extends the document established in [`OcPointAttr`] and
/// [`OcPlaneAttr`], adding the extruded solid under `body/1`:
///
/// ```text
/// main (0:1)
/// ├── 1 (0:1:1)   planes
/// │   ├── 1 (0:1:1:1)   XY plane
/// │   │       ...
/// │   ├── 2 (0:1:1:2)   YZ plane
/// │   │       ...
/// │   └── 3 (0:1:1:3)   XZ plane
/// │           TopoNamingNamedShape (Primitive, planar face)
/// │           OcPlaneAttr
/// ├── 2 (0:1:2)   sketch
/// │   ├── 1 (0:1:2:1)   point A (0.0, 1.0, 0.0)
/// │   │       ...
/// │   ├── 2 (0:1:2:2)   point B (0.0, 0.0, 0.0)
/// │   │       ...
/// │   ├── 3 (0:1:2:3)   point C (1.0, 0.0, 0.0)
/// │   │       ...
/// │   ├── 4 (0:1:2:4)   point D (1.0, 1.0, 0.0)
/// │   │       TopoNamingNamedShape (Primitive, vertex)
/// │   │       OcPointAttr
/// │   └── 5 (0:1:2:5)   face
/// │           TopoNamingNamedShape (Primitive, unit square face)
/// └── 3 (0:1:3)   body
///     └── 1 (0:1:3:1)   solid
///             TopoNamingNamedShape (Generated, 1×1×1 prism)
///             OcReal "depth" = 1.0
/// ```
///
/// ```
/// // Note: this example is incomplete pending OcFace::try_from binding.
/// // The planes and sketch commands are correct and can be run independently.
/// # use occt_rs::gp::{OcAx2, OcDir, OcPnt, OcVec};
/// # use occt_rs::ocaf::OcApplication;
/// # use occt_rs::ocaf::attributes::OcReal;
/// # use occt_rs::ocaf::tdata_xtd::{OcPlaneAttr, OcPointAttr};
/// # use occt_rs::ocaf::topo_naming::{TopoNamingEvolution, TopoNamingNamedShape};
/// # use occt_rs::rs_topo::{OcEdge, OcFace, OcWire};
///
/// # let mut app = OcApplication::new();
/// # let mut doc = app.new_document("BinXCAF").unwrap();
/// # doc.set_undo_limit(10);
///
/// let main = doc.main();
///
///
/// // planes command
/// {
///     doc.begin_command().unwrap();
///     let planes = main.get_or_create_child(1);
///
///     # let xy = planes.get_or_create_child(1);
///     # OcPlaneAttr::record_shape(&xy, OcAx2::new(
///     #     OcPnt::new(0.0, 0.0, 0.0),
///     #     OcDir::new(0.0, 0.0, 1.0).unwrap(),
///     #     OcDir::new(1.0, 0.0, 0.0).unwrap(),
///     # ).unwrap()).unwrap();
///     # OcPlaneAttr::set(&xy).unwrap();
///
///     # let yz = planes.get_or_create_child(2);
///     # OcPlaneAttr::record_shape(&yz, OcAx2::new(
///     #     OcPnt::new(0.0, 0.0, 0.0),
///     #     OcDir::new(1.0, 0.0, 0.0).unwrap(),
///     #     OcDir::new(0.0, 1.0, 0.0).unwrap(),
///     # ).unwrap()).unwrap();
///     # OcPlaneAttr::set(&yz).unwrap();
///     // snipped: making xy and yz
///
///     let xz = planes.get_or_create_child(3);
///     OcPlaneAttr::record_shape(&xz, OcAx2::new(
///         OcPnt::new(0.0, 0.0, 0.0),
///         OcDir::new(0.0, 1.0, 0.0).unwrap(),
///         OcDir::new(1.0, 0.0, 0.0).unwrap(),
///     ).unwrap()).unwrap();
///     OcPlaneAttr::set(&xz).unwrap();
///
///     doc.commit().unwrap();
/// }
///
/// // sketch command
/// {
///     doc.begin_command().unwrap();
///     let sketch = main.get_or_create_child(2);
///
///     # let la = sketch.get_or_create_child(1);
///     # OcPointAttr::record_shape(&la, OcPnt::new(0.0, 1.0, 0.0)).unwrap();
///     # OcPointAttr::set(&la).unwrap();
///
///     # let lb = sketch.get_or_create_child(2);
///     # OcPointAttr::record_shape(&lb, OcPnt::new(0.0, 0.0, 0.0)).unwrap();
///     # OcPointAttr::set(&lb).unwrap();
///
///     # let lc = sketch.get_or_create_child(3);
///     # OcPointAttr::record_shape(&lc, OcPnt::new(1.0, 0.0, 0.0)).unwrap();
///     # OcPointAttr::set(&lc).unwrap();
///     // snipped: maing la, lb and lc
///
///     let ld = sketch.get_or_create_child(4);
///     OcPointAttr::record_shape(&ld, OcPnt::new(1.0, 1.0, 0.0)).unwrap();
///     OcPointAttr::set(&ld).unwrap();
///
///     // Record the square face — the extrude input
///     let lface = sketch.get_or_create_child(5);
///     let wire = OcWire::from_edges(&[
///         OcEdge::from_pnts(OcPnt::new(0.0, 0.0, 0.0), OcPnt::new(1.0, 0.0, 0.0)).unwrap(),
///         OcEdge::from_pnts(OcPnt::new(1.0, 0.0, 0.0), OcPnt::new(1.0, 1.0, 0.0)).unwrap(),
///         OcEdge::from_pnts(OcPnt::new(1.0, 1.0, 0.0), OcPnt::new(0.0, 1.0, 0.0)).unwrap(),
///         OcEdge::from_pnts(OcPnt::new(0.0, 1.0, 0.0), OcPnt::new(0.0, 0.0, 0.0)).unwrap(),
///     ]).unwrap();
///     let face = OcFace::from_wire(&wire, true).unwrap();
///     doc.name_builder(&lface).primitive(&face.as_shape());
///
///     doc.commit().unwrap();
/// }
///
/// // extrude command
/// //
/// // The solid is generated from the sketch face: Generated
/// // The extrude depth is stored alongside it as OcReal so that editing
/// // the depth and rebuilding the solid can be undone as a single step.
/// let solid_label = {
///     doc.begin_command().unwrap();
///     let body   = main.get_or_create_child(3);
///     let lsolid = body.get_or_create_child(1);
///
///     let depth = OcReal::set(&lsolid, 1.0).unwrap();
///
///     // Read the face from the sketch label.
///     let sketch = main.get_or_create_child(2);
///     let lface  = sketch.get_or_create_child(5);
///     let face_shape = TopoNamingNamedShape::find(&lface).unwrap().get().unwrap();
///     let face = OcFace::try_from(&face_shape).unwrap();
///
///     // do the extrude to make the solid
///     let extrude_shape = face.extrude(OcVec::new(0.0, 0.0, depth.get())).unwrap();
///
///     doc.name_builder(&lsolid).generated(&face_shape, &extrude_shape);
///
///     doc.commit().unwrap();
///     lsolid
/// };
///
/// let ns = TopoNamingNamedShape::find(&solid_label).unwrap();
/// assert_eq!(ns.evolution(), Some(TopoNamingEvolution::Generated));
/// assert!(ns.get().is_some());
/// assert!((OcReal::find(&solid_label).unwrap().get() - 1.0).abs() < 1e-12);
///
/// // Undo the extrude — the named shape and depth attribute disappear
/// doc.undo().unwrap();
/// assert!(TopoNamingNamedShape::find(&solid_label).is_none());
/// assert!(OcReal::find(&solid_label).is_none());
///
/// // Redo restores both
/// doc.redo().unwrap();
/// assert!(TopoNamingNamedShape::find(&solid_label).is_some());
/// assert!((OcReal::find(&solid_label).unwrap().get() - 1.0).abs() < 1e-12);
///
/// // A label with no named shape returns None
/// let empty_label = {
///     doc.begin_command().unwrap();
///     let l = main.get_or_create_child(99);
///     doc.commit().unwrap();
///     l
/// };
/// assert!(TopoNamingNamedShape::find(&empty_label).is_none());
/// ```
///
/// [`Primitive`]: TopoNamingEvolution::Primitive
/// [`Modify`]: TopoNamingEvolution::Modify
/// [`Generated`]: TopoNamingEvolution::Generated
/// [`Delete`]: TopoNamingEvolution::Delete
/// [`OcPointAttr`]: crate::ocaf::tdata_xtd::OcPointAttr
/// [`OcPlaneAttr`]: crate::ocaf::tdata_xtd::OcPlaneAttr
pub struct TopoNamingBuilder {
    inner: UniquePtr<ffi::TopoNamingBuilderShim>,
    _not_send: PhantomData<*mut ()>,
}

impl TopoNamingBuilder {
    pub fn new(label: &OcLabel) -> Self {
        Self {
            inner: new_tnaming_builder(&label.inner),
            _not_send: PhantomData,
        }
    }

    /// Records a [`TopoNamingEvolution::Primitive`]
    ///
    /// Use this for shapes produced by constructors (`BRepPrimAPI_MakeBox`,
    /// `OcFace::from_wire`, etc.) that have no prior shape in the document.
    pub fn primitive(&mut self, shape: &OcShape) {
        self.inner.pin_mut().generated_fresh(shape.as_ffi());
    }

    /// Records a [`TopoNamingEvolution::Generated`]
    ///
    /// Use this when an operation creates a new sub-shape from an ancestor
    /// sub-shape (e.g. extrusion generates a lateral face from a wire edge).
    pub fn generated(&mut self, old: &OcShape, new: &OcShape) {
        self.inner
            .pin_mut()
            .generated_from(old.as_ffi(), new.as_ffi());
    }

    /// Records a [`TopoNamingEvolution::Modify`]
    ///
    /// Use this when an operation transforms an existing shape into a new one
    /// (e.g. fillet rounds a face — the original face becomes the `old` arg,
    /// the rounded replacement is `modified`).
    pub fn modified(&mut self, old: &OcShape, new: &OcShape) {
        self.inner.pin_mut().modify(old.as_ffi(), new.as_ffi());
    }

    /// Records a [`TopoNamingEvolution::Delete`]
    pub fn delete(&mut self, old: &OcShape) {
        self.inner.pin_mut().delete_shape(old.as_ffi());
    }

    /// Records a [`TopoNamingEvolution::Selected`]
    ///
    /// Records a sub-shape selection in context. Reserved for DOC-4 /
    /// `TNaming_Selector` workflows.
    pub fn select(&mut self, shape: &OcShape, in_shape: &OcShape) {
        self.inner
            .pin_mut()
            .select(shape.as_ffi(), in_shape.as_ffi());
    }

    /// Returns a handle to the `TNaming_NamedShape` attribute written on the
    /// label. The handle remains valid after the builder is dropped and across
    /// undo/redo boundaries.
    pub fn named_shape(&self) -> TopoNamingNamedShape {
        TopoNamingNamedShape {
            inner: self.inner.named_shape(),
            _not_send: PhantomData,
        }
    }
}

// ---------------------------------------------------------------------------
// TopoNamingNamedShape
//
// ---------------------------------------------------------------------------

/// Topological shape naming attribute on an [`OcLabel`]
///
/// Read handle to the TNaming_NamedShape attribute. Can be obtained from
/// TopoNamingBuilder::named_shape() or TopoNamingNamedShape::find(label).
/// Reflects undo/redo state — get() returns the shape as of the current
/// transaction stack position.
pub struct TopoNamingNamedShape {
    inner: UniquePtr<ffi::TopoNamingNamedShapeHandle>,
    _not_send: PhantomData<*mut ()>,
}

impl TopoNamingNamedShape {
    pub(crate) fn from_ffi(inner: UniquePtr<ffi::TopoNamingNamedShapeHandle>) -> Self {
        Self {
            inner,
            _not_send: PhantomData,
        }
    }

    /// Retrieve a handle on `label` if present
    pub fn find(label: &OcLabel) -> Option<Self> {
        // find_tnaming_named_shape writes into `out` on success.
        // We need a valid (non-null) TopoNamingNamedShapeHandle to write into.
        // Construct one via the builder round-trip: build an empty wrapper.
        // The find shim takes Pin<&mut TopoNamingNamedShapeHandle> — we need an
        // allocated instance. Use a dummy label-less handle here; the shim
        // will overwrite inner if found.
        //
        // NOTE: This requires a `new_tnaming_named_shape_handle` factory in
        // the bridge that allocates a default-constructed wrapper. Add that
        // shim to tnaming.hxx — see comment below.
        let mut out = ffi::new_tnaming_named_shape_handle();
        let found = ffi::find_tnaming_named_shape(label.inner.as_ref().unwrap(), out.pin_mut());
        if found {
            Some(Self {
                inner: out,
                _not_send: PhantomData,
            })
        } else {
            None
        }
    }

    /// Current shapes as recorded on the label. None, if empty
    pub fn get(&self) -> Option<OcShape> {
        OcShape::from_ffi(ffi::tnaming_named_shape_get(self.inner.as_ref().unwrap()))
    }

    /// The original shapes — before any evolution was recorded on this label.
    pub fn original_shape(&self) -> Option<OcShape> {
        OcShape::from_ffi(ffi::tnaming_tool_original_shape(
            self.inner.as_ref().unwrap(),
        ))
    }

    /// The provenance kind recorded when this shape was written.
    pub fn evolution(&self) -> Option<TopoNamingEvolution> {
        TopoNamingEvolution::try_from_raw(ffi::tnaming_named_shape_evolution(
            self.inner.as_ref().unwrap(),
        ))
    }

    pub(crate) fn inner(&self) -> &UniquePtr<ffi::TopoNamingNamedShapeHandle> {
        &self.inner
    }
}
// ---------------------------------------------------------------------------
// TopoNamingSelector
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_naming___selector.html
// ---------------------------------------------------------------------------

/// A stable selection record, providing for re-selection after re-compute
///
/// A stable selection record for re-finding a sub-shape after rebuild.
///
/// In the scenario, `sketch2` is drawn on a chamfer face. After the user
/// edits the sketch and triggers a rebuild, the chamfer face is a different
/// object at a different address. `TopoNamingSelector` re-finds it by
/// re-evaluating its recorded naming description against the current shape
/// history.
///
/// Note: Every operation since the original selection must be
/// recorded with [`TopoNamingBuilder`]. Incomplete provenance recording
/// produces incorrect results, or [`solve`] returning `false`.
///
/// The example below extends the document established in [`ChamferBuilder`],
/// adding the stable face reference on `sketch2/5`:
///
/// ```text
/// main (0:1)
/// ├── 3 (0:1:3)   body
/// │   ├── 1 (0:1:3:1)   solid
/// │   │       TopoNamingNamedShape (Primitive, 1×1×1 prism)
/// │   │       OcReal "depth" = 1.0
/// │   └── 2 (0:1:3:2)   chamfer
/// │           TopoNamingNamedShape (Modify, chamfered solid)
/// │           OcReal "distance" = 0.05
/// └── 4 (0:1:4)   sketch2
///     └── 5 (0:1:4:5)   ref-face
///             TopoNamingNamedShape (Selected — TopoNamingSelector)
/// ```
///
/// ```rust,ignore
/// // NOTE: this example is incomplete pending two missing bindings:
/// //
/// // 1. OcFace::try_from — needed to retrieve the face from the document
/// //    and rebuild from new parameters after a sketch edit.
/// //
/// // 2. The solve() demonstration requires a full rebuild cycle:
/// //    edit sketch point → rebuild wire → rebuild face → rebuild solid →
/// //    re-apply chamfer → re-record provenance → solve() re-finds the
/// //    chamfer face by naming description rather than pointer identity.
/// //
/// // Until those are in place, this example only shows the write side —
/// // recording the selection. The read side (solve() after rebuild) is
/// // where the topo-naming problem is actually demonstrated.
/// //
/// // See: todo_toponamingnamedshape_empty_label.md for the related gap.
/// # let mut doc = app.new_document("BinXCAF").unwrap();
/// # doc.set_undo_limit(10);
///
/// let main = doc.main();
///
/// // Build and record the solid on body/1
/// let wire = OcWire::from_edges(&[
///     OcEdge::from_pnts(OcPnt::new(0.0, 0.0, 0.0), OcPnt::new(1.0, 0.0, 0.0)).unwrap(),
///     # OcEdge::from_pnts(OcPnt::new(1.0, 0.0, 0.0), OcPnt::new(1.0, 1.0, 0.0)).unwrap(),
///     # OcEdge::from_pnts(OcPnt::new(1.0, 1.0, 0.0), OcPnt::new(0.0, 1.0, 0.0)).unwrap(),
///     # OcEdge::from_pnts(OcPnt::new(0.0, 1.0, 0.0), OcPnt::new(0.0, 0.0, 0.0)).unwrap(),
///     // snipped: the other 3 points
/// ]).unwrap();
///
/// let face_shape = OcFace::from_wire(&wire, true).unwrap()
///     .extrude(OcVec::new(0.0, 0.0, 1.0)).unwrap();
///
/// let pre_faces: Vec<_> = face_shape.faces().collect();
///
/// // setup label 0:1:3 (body) and the first solid child
/// {
///     doc.begin_command().unwrap();
///     let lsolid = main.get_or_create_child(3)
///                      .get_or_create_child(1);
///     OcReal::set(&lsolid, 1.0).unwrap();
///     doc.name_builder(&lsolid).primitive(&face_shape);
///     doc.commit().unwrap();
/// }
///
/// // Apply chamfer, record result and modified faces on body/2
/// let (chamfer_shape, chamfer_face) = {
///     doc.begin_command().unwrap();
///     let lchamfer = main.get_or_create_child(3)
///                        .get_or_create_child(2);
///
///     let distance = OcReal::set(&lchamfer, 0.05).unwrap();
///     let edge = face_shape.edges().next().unwrap();
///     let mut cb = ChamferBuilder::new(&face_shape).unwrap();
///     cb.add_edge(distance.get(), &edge).unwrap();
///     let mut built = cb.build_with_history().unwrap();
///     let chamfer_shape = built.shape().clone();
///
///     // Capture the generated chamfer face while built is still live
///     let chamfer_face = built.generated(&edge.as_shape()).next().unwrap();
///
///     let mut nb = doc.name_builder(&lchamfer);
///     for face in &pre_faces {
///         for modified in built.modified(&face.as_shape()) {
///             nb.modified(&face.as_shape(), &modified);
///         }
///     }
///
///     doc.commit().unwrap();
///     (chamfer_shape, chamfer_face)
/// };
///
/// // The new face generated from the chamfered edge is the one sketch2
/// // will be drawn on. We record a stable selection of it on sketch2/5
/// // so that after rebuild, solve() can re-find it by naming description
/// // rather than by pointer identity.
/// let ref_face_label = {
///     doc.begin_command().unwrap();
///     let sketch2 = main.get_or_create_child(4);
///     let ref_face = sketch2.get_or_create_child(5);
///
///
///     let mut selector = doc.selector(&ref_face);
///     selector.select(&chamfer_face, &chamfer_shape);
///
///     doc.commit().unwrap();
///     ref_face
/// };
///
/// // The selector wrote a Selected named shape on the ref-face label
/// let ns = TopoNamingNamedShape::find(&ref_face_label).unwrap();
/// assert_eq!(ns.evolution(), Some(TopoNamingEvolution::Selected));
///
/// // solve() re-evaluates the selection against the current model —
/// // returns true when the named face can still be found
/// let mut selector = doc.main()
///     .find_child(4).unwrap()
///     .find_child(5).unwrap();
/// // Note: in a full rebuild cycle, solve() is called after re-applying
/// // all operations and re-recording their provenance with TopoNamingBuilder.
/// // The selector then walks the naming graph to re-find the chamfer face.
/// ```
///
/// [`TopoNamingBuilder`]: crate::ocaf::topo_naming::TopoNamingBuilder
/// [`ChamferBuilder`]: crate::rs_topo::ChamferBuilder
///
/// [`select`]: TopoNamingSelector::select
/// [`solve`]: TopoNamingSelector::solve
/// [`Command`]: crate::ocaf::document::Command
/// [`Command::selector`]: crate::ocaf::document::Command::selector
pub struct TopoNamingSelector {
    pub(crate) inner: UniquePtr<ffi::TopoNamingSelectorShim>,
    _not_send: PhantomData<*mut ()>,
}

impl TopoNamingSelector {
    pub(crate) fn new(inner: UniquePtr<ffi::TopoNamingSelectorShim>) -> Self {
        Self {
            inner,
            _not_send: PhantomData,
        }
    }

    /// Records that `shape` (a sub-shape of `context`) should be re-findable
    /// after model changes.  Returns `false` if the selection cannot be named
    /// unambiguously.
    ///
    /// [`Command`]: crate::ocaf::document::Command
    pub fn select(&mut self, shape: &OcShape, context: &OcShape) -> bool {
        ffi::tnaming_selector_select(self.inner.pin_mut(), shape.as_ffi(), context.as_ffi())
    }

    /// Re-evaluates the stored selection description against the current model.
    /// Returns `false` if the selection can no longer be resolved.
    ///
    /// See [struct-level docs](TopoNamingSelector) for the precondition on
    /// complete provenance recording.
    pub fn solve(&mut self) -> bool {
        ffi::tnaming_selector_solve(self.inner.pin_mut())
    }

    /// Returns the [`TopoNamingNamedShape`] written by [`select`], if any.
    ///
    /// [`select`]: TopoNamingSelector::select
    pub fn named_shape(&self) -> Option<TopoNamingNamedShape> {
        let mut out = ffi::new_tnaming_named_shape_handle();
        let found = ffi::tnaming_selector_named_shape(self.inner.as_ref().unwrap(), out.pin_mut());
        found.then(|| TopoNamingNamedShape {
            inner: out,
            _not_send: PhantomData,
        })
    }
}
#[cfg(test)]
mod test {
    use crate::ocaf::{OcApplication, TopoNamingBuilder};

    #[test]
    fn tnaming_undo_reverses_modify() {
        use crate::gp::OcPnt;
        use crate::rs_topo::{OcEdge, OcFace};

        let mut app = OcApplication::new();
        let mut doc = app.new_document("BinXCAF").unwrap();

        // Two distinct shapes
        let edges = vec![
            OcEdge::from_pnts(OcPnt::new(0.0, 0.0, 0.0), OcPnt::new(1.0, 0.0, 0.0)).unwrap(),
            OcEdge::from_pnts(OcPnt::new(1.0, 0.0, 0.0), OcPnt::new(1.0, 1.0, 0.0)).unwrap(),
            OcEdge::from_pnts(OcPnt::new(1.0, 1.0, 0.0), OcPnt::new(0.0, 1.0, 0.0)).unwrap(),
            OcEdge::from_pnts(OcPnt::new(0.0, 1.0, 0.0), OcPnt::new(0.0, 0.0, 0.0)).unwrap(),
        ];
        let wire_a = crate::rs_topo::OcWire::from_edges(&edges).unwrap();
        let face_a = OcFace::from_wire(&wire_a, true).unwrap();
        let shape_a = face_a.as_shape();

        let edges = vec![
            OcEdge::from_pnts(OcPnt::new(0.5, 0.0, 0.0), OcPnt::new(1.5, 0.0, 0.0)).unwrap(),
            OcEdge::from_pnts(OcPnt::new(1.5, 0.0, 0.0), OcPnt::new(1.5, 1.0, 0.0)).unwrap(),
            OcEdge::from_pnts(OcPnt::new(1.5, 1.0, 0.0), OcPnt::new(0.5, 1.0, 0.0)).unwrap(),
            OcEdge::from_pnts(OcPnt::new(0.5, 1.0, 0.0), OcPnt::new(0.5, 0.0, 0.0)).unwrap(),
        ];
        let wire_b = crate::rs_topo::OcWire::from_edges(&edges).unwrap();
        let face_b = OcFace::from_wire(&wire_b, true).unwrap();
        let shape_b = face_b.as_shape();

        let root = doc.main();

        // Command 1: create the label and record shape_a as primitive
        let (label, named_shape) = {
            doc.begin_command().unwrap();
            let label = root.get_or_create_child(1);
            let mut b = TopoNamingBuilder::new(&label);
            b.primitive(&shape_a);
            let ns = b.named_shape();
            doc.commit().unwrap();
            (label, ns)
        };

        // Command 2: modify to shape_b
        {
            doc.begin_command().unwrap();
            let mut b = TopoNamingBuilder::new(&label);
            b.modified(&shape_a, &shape_b);
            doc.commit().unwrap();
        }

        // After command 2, get() should return shape_b
        // (compare via some observable property — bounding box, vertex count, etc.)

        doc.undo().unwrap();

        // After undo, get() should return shape_a
        // This is the verification the milestone requires before proceeding.
        let _ = named_shape.get();
        // Assert shape identity here
    }
}
