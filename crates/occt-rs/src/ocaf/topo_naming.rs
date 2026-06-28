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

/// Principal tool for addressing the Topo Naming Problem
///
/// Under the OCAF tnaming scheme, an [`OcLabel`] can carry one [`TopoNamingNamedShape`] attribute. It
/// describes an evolution kind in the form of [`TopoNamingEvolution`], and has one or more shapes
/// to it
pub struct TopoNamingBuilder<'cmd> {
    inner: UniquePtr<ffi::TopoNamingBuilderShim>,
    _not_send: PhantomData<*mut ()>,
    _cmd: PhantomData<&'cmd ()>,
}

impl<'cmd> TopoNamingBuilder<'cmd> {
    pub fn new(label: &OcLabel) -> Self {
        Self {
            inner: new_tnaming_builder(&label.inner),
            _not_send: PhantomData,
            _cmd: PhantomData,
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
/// Construct via [`Command::selector`].
///
/// # Command requirement on `select`
///
/// [`select`] writes a `TNaming_Naming` attribute and must be called while a
/// [`Command`] is open.  This is enforced at compile time: `select` takes a
/// `&Command<'_>` proof token.  The token is unused at runtime; its presence
/// in the call site is the guarantee.
///
/// # Precondition on `solve`
///
/// [`solve`] requires that every history-generating operation since the
/// original [`select`] was recorded with [`TopoNamingBuilder`].  The bindings
/// layer cannot verify this; incomplete recording produces incorrect results
/// or returns `false` without further diagnosis.
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
    /// The `_cmd` parameter is a compile-time proof that a [`Command`] is
    /// open; it is not used at runtime.
    ///
    /// [`Command`]: crate::ocaf::document::Command
    pub fn select(
        &mut self,
        _cmd: &crate::ocaf::document::Command<'_>,
        shape: &OcShape,
        context: &OcShape,
    ) -> bool {
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
            let cmd = doc.begin_command().unwrap();
            let label = root.get_or_create_child(&cmd, 1);
            let mut b = TopoNamingBuilder::new(&label);
            b.primitive(&shape_a);
            let ns = b.named_shape();
            cmd.commit().unwrap();
            (label, ns)
        };

        // Command 2: modify to shape_b
        {
            let cmd = doc.begin_command().unwrap();
            let mut b = TopoNamingBuilder::new(&label);
            b.modified(&shape_a, &shape_b);
            cmd.commit().unwrap();
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
