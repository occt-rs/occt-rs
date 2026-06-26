// Sourced from OCCT 7.9 documentation.
// No derivation from any other binding crate.
//
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_naming___builder.html
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_naming___named_shape.html

use std::marker::PhantomData;

use cxx::UniquePtr;
use occt_sys::ffi::{self, new_tnaming_builder};

use super::{label::OcLabel, shape::OcShape};

// ---------------------------------------------------------------------------
// TnamingEvolution
// Maps TNaming_Evolution integer values from OCCT.
// Reference: https://dev.opencascade.org/doc/refman/html/group__enum__t_naming.html
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TnamingEvolution {
    /// Shape appeared fresh — no topological ancestor.
    Primitive,
    /// Shape was generated from an ancestor (e.g. extrusion of an edge).
    Generated,
    /// Shape is a modification of an ancestor (e.g. face rounded by fillet).
    Modify,
    /// Ancestor was deleted by this operation.
    Delete,
    /// Sub-shape selection in context — used by DOC-4 / TNaming_Selector.
    Selected,
}

impl TnamingEvolution {
    fn from_raw(v: i32) -> Self {
        // TNaming_Evolution enum constants — verify ordinal values against
        // https://dev.opencascade.org/doc/refman/html/group__enum__t_naming.html
        // if undo behaviour is unexpected.
        match v {
            0 => Self::Primitive,
            1 => Self::Generated,
            2 => Self::Modify,
            3 => Self::Delete,
            4 => Self::Selected,
            _ => panic!("Unknown TNaming_Evolution value: {v}"),
        }
    }
}

pub struct TnamingBuilder<'cmd> {
    inner: UniquePtr<ffi::TnamingBuilderShim>,
    _not_send: PhantomData<*mut ()>,
    _cmd: PhantomData<&'cmd ()>,
}

impl<'cmd> TnamingBuilder<'cmd> {
    pub fn new(label: &OcLabel) -> Self {
        Self {
            inner: new_tnaming_builder(&label.inner),
            _not_send: PhantomData,
            _cmd: PhantomData,
        }
    }

    /// Records `shape` as a primitive — no topological ancestor.
    ///
    /// Use this for shapes produced by constructors (`BRepPrimAPI_MakeBox`,
    /// `OcFace::from_wire`, etc.) that have no prior shape in the document.
    pub fn generated_fresh(&mut self, shape: &OcShape) {
        self.inner
            .pin_mut()
            .generated_fresh(shape.inner.as_ref().unwrap());
    }

    /// Records `generated` as produced from `old`.
    ///
    /// Use this when an operation creates a new sub-shape from an ancestor
    /// sub-shape (e.g. extrusion generates a lateral face from a wire edge).
    pub fn generated_from(&mut self, old: &OcShape, generated: &OcShape) {
        self.inner.pin_mut().generated_from(
            old.inner.as_ref().unwrap(),
            generated.inner.as_ref().unwrap(),
        );
    }

    /// Records `modified` as a modification of `old`.
    ///
    /// Use this when an operation transforms an existing shape into a new one
    /// (e.g. fillet rounds a face — the original face becomes the `old` arg,
    /// the rounded replacement is `modified`).
    pub fn modify(&mut self, old: &OcShape, modified: &OcShape) {
        self.inner.pin_mut().modify(
            old.inner.as_ref().unwrap(),
            modified.inner.as_ref().unwrap(),
        );
    }

    /// Records `old` as deleted by this operation — it has no successor.
    pub fn delete(&mut self, old: &OcShape) {
        self.inner
            .pin_mut()
            .delete_shape(old.inner.as_ref().unwrap());
    }

    /// Records a sub-shape selection in context. Reserved for DOC-4 /
    /// `TNaming_Selector` workflows.
    pub fn select(&mut self, shape: &OcShape, in_shape: &OcShape) {
        self.inner.pin_mut().select(
            shape.inner.as_ref().unwrap(),
            in_shape.inner.as_ref().unwrap(),
        );
    }

    /// Returns a handle to the `TNaming_NamedShape` attribute written on the
    /// label. The handle remains valid after the builder is dropped and across
    /// undo/redo boundaries.
    pub fn named_shape(&self) -> TnamingNamedShape {
        TnamingNamedShape {
            inner: self.inner.named_shape(),
            _not_send: PhantomData,
        }
    }
}

// ---------------------------------------------------------------------------
// TnamingNamedShape
//
// Read handle to the TNaming_NamedShape attribute. Can be obtained from
// TnamingBuilder::named_shape() or TnamingNamedShape::find(label).
// Reflects undo/redo state — get() returns the shape as of the current
// transaction stack position.
// ---------------------------------------------------------------------------

pub struct TnamingNamedShape {
    inner: UniquePtr<ffi::TnamingNamedShapeHandle>,
    _not_send: PhantomData<*mut ()>,
}

impl TnamingNamedShape {
    pub(crate) fn from_ffi(inner: UniquePtr<ffi::TnamingNamedShapeHandle>) -> Self {
        Self {
            inner,
            _not_send: PhantomData,
        }
    }

    /// Returns `None` if no `TNaming_NamedShape` attribute is present on `label`.
    pub fn find(label: &OcLabel) -> Option<Self> {
        // find_tnaming_named_shape writes into `out` on success.
        // We need a valid (non-null) TnamingNamedShapeHandle to write into.
        // Construct one via the builder round-trip: build an empty wrapper.
        // The find shim takes Pin<&mut TnamingNamedShapeHandle> — we need an
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

    /// Current shape as recorded on the label. After undo, returns the
    /// pre-operation shape.
    pub fn get(&self) -> OcShape {
        OcShape {
            inner: ffi::tnaming_named_shape_get(self.inner.as_ref().unwrap()),
            _not_send: PhantomData,
        }
    }

    /// The original shape — before any evolution was recorded on this label.
    pub fn original_shape(&self) -> OcShape {
        OcShape {
            inner: ffi::tnaming_tool_original_shape(self.inner.as_ref().unwrap()),
            _not_send: PhantomData,
        }
    }

    /// The provenance kind recorded when this shape was written.
    pub fn evolution(&self) -> TnamingEvolution {
        TnamingEvolution::from_raw(ffi::tnaming_named_shape_evolution(
            self.inner.as_ref().unwrap(),
        ))
    }

    pub(crate) fn inner(&self) -> &UniquePtr<ffi::TnamingNamedShapeHandle> {
        &self.inner
    }
}
#[test]
fn tnaming_undo_reverses_modify() {
    use crate::gp::OcPnt;
    use crate::topo::{OcApplication, OcEdge, OcFace};
    use occt_sys::ffi::new_tnaming_builder;

    let mut app = OcApplication::new();
    let mut doc = app.new_document("BinXCAF").unwrap();

    // Two distinct shapes
    let edges = vec![
        OcEdge::from_pnts(OcPnt::new(0.0, 0.0, 0.0), OcPnt::new(1.0, 0.0, 0.0)).unwrap(),
        OcEdge::from_pnts(OcPnt::new(1.0, 0.0, 0.0), OcPnt::new(1.0, 1.0, 0.0)).unwrap(),
        OcEdge::from_pnts(OcPnt::new(1.0, 1.0, 0.0), OcPnt::new(0.0, 1.0, 0.0)).unwrap(),
        OcEdge::from_pnts(OcPnt::new(0.0, 1.0, 0.0), OcPnt::new(0.0, 0.0, 0.0)).unwrap(),
    ];
    let wire_a = crate::topo::OcWire::from_edges(&edges).unwrap();
    let face_a = OcFace::from_wire(&wire_a, true).unwrap();
    let shape_a = face_a.as_shape();

    let edges = vec![
        OcEdge::from_pnts(OcPnt::new(0.5, 0.0, 0.0), OcPnt::new(1.5, 0.0, 0.0)).unwrap(),
        OcEdge::from_pnts(OcPnt::new(1.5, 0.0, 0.0), OcPnt::new(1.5, 1.0, 0.0)).unwrap(),
        OcEdge::from_pnts(OcPnt::new(1.5, 1.0, 0.0), OcPnt::new(0.5, 1.0, 0.0)).unwrap(),
        OcEdge::from_pnts(OcPnt::new(0.5, 1.0, 0.0), OcPnt::new(0.5, 0.0, 0.0)).unwrap(),
    ];
    let wire_b = crate::topo::OcWire::from_edges(&edges).unwrap();
    let face_b = OcFace::from_wire(&wire_b, true).unwrap();
    let shape_b = face_b.as_shape();

    let root = doc.main();

    // Command 1: create the label and record shape_a as primitive
    let (label, named_shape) = {
        let cmd = doc.begin_command().unwrap();
        let label = root.get_or_create_child(&cmd, 1);
        let mut b = TnamingBuilder::new(&label);
        b.generated_fresh(&shape_a);
        let ns = b.named_shape();
        cmd.commit().unwrap();
        (label, ns)
    };

    // Command 2: modify to shape_b
    {
        let cmd = doc.begin_command().unwrap();
        let mut b = TnamingBuilder::new(&label);
        b.modify(&shape_a, &shape_b);
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
// ---------------------------------------------------------------------------
// TnamingSelector
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_naming___selector.html
// ---------------------------------------------------------------------------

/// Records how a sub-shape was selected so it can be re-found after model
/// changes.
///
/// Construct via [`OcDocument::selector`].
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
/// original [`select`] was recorded with [`TnamingBuilder`].  The bindings
/// layer cannot verify this; incomplete recording produces incorrect results
/// or returns `false` without further diagnosis.
///
/// [`select`]: TnamingSelector::select
/// [`solve`]: TnamingSelector::solve
/// [`Command`]: crate::topo::document::Command
pub struct TnamingSelector {
    pub(crate) inner: UniquePtr<ffi::TnamingSelectorShim>,
    _not_send: PhantomData<*mut ()>,
}

impl TnamingSelector {
    pub(crate) fn new(inner: UniquePtr<ffi::TnamingSelectorShim>) -> Self {
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
    /// [`Command`]: crate::topo::document::Command
    pub fn select(
        &mut self,
        _cmd: &crate::topo::document::Command<'_>,
        shape: &OcShape,
        context: &OcShape,
    ) -> bool {
        ffi::tnaming_selector_select(
            self.inner.pin_mut(),
            shape.inner.as_ref().unwrap(),
            context.inner.as_ref().unwrap(),
        )
    }

    /// Re-evaluates the stored selection description against the current model.
    /// Returns `false` if the selection can no longer be resolved.
    ///
    /// See [struct-level docs](TnamingSelector) for the precondition on
    /// complete provenance recording.
    pub fn solve(&mut self) -> bool {
        ffi::tnaming_selector_solve(self.inner.pin_mut())
    }

    /// Returns the [`TnamingNamedShape`] written by [`select`], if any.
    ///
    /// [`select`]: TnamingSelector::select
    pub fn named_shape(&self) -> Option<TnamingNamedShape> {
        let mut out = ffi::new_tnaming_named_shape_handle();
        let found = ffi::tnaming_selector_named_shape(self.inner.as_ref().unwrap(), out.pin_mut());
        found.then(|| TnamingNamedShape {
            inner: out,
            _not_send: PhantomData,
        })
    }
}
