//! Persistent transform builder — geometric transform with history.
//!
//! `BRepBuilderAPI_Transform` computes the transform inside its own
//! constructor; there is no separate build step. `Modified`/`Generated`/
//! `IsDeleted` read history populated at that same construction, so the
//! builder must stay alive until those queries are done — see
//! [`HistoryProvider`]/[`BuiltWithHistory`].
//!
//! For a one-call convenience with no history, use [`OcShape::transformed`].
//!
//! [`OcShape::transformed`]: crate::rs_topo::OcShape::transformed
//!
//! Reference: <https://dev.opencascade.org/doc/refman/html/class_b_rep_builder_a_p_i___transform.html>

use std::marker::PhantomData;

use occt_sys::ffi;

use crate::error::{OcctError, OcctErrorKind};
use crate::gp::OcTrsf;
use crate::rs_topo::shape_history_iter::ShapeListIter;
use crate::rs_topo::{BuiltWithHistory, HistoryProvider, OcShape};

pub struct TransformBuilder {
    inner: cxx::UniquePtr<ffi::MakeTransformBuilder>,
    _not_send: PhantomData<*mut ()>,
}

impl TransformBuilder {
    /// Constructs a transform builder, applying `trsf` to `shape` immediately.
    ///
    /// `copy`: per OCCT semantics, if `false` and `trsf` is a direct isometry,
    /// the result shares `shape`'s TShape with a new `Location`. If `true`,
    /// the transform is applied to a duplicate and the result is fully
    /// independent. `OcShape::transformed` always passes `false`; this
    /// constructor exposes the choice rather than deciding it.
    pub fn new(shape: &OcShape, trsf: &OcTrsf, copy: bool) -> Result<Self, OcctError> {
        let inner = ffi::new_make_transform_builder(
            shape.as_ffi(),
            trsf.value(1, 1).unwrap(),
            trsf.value(1, 2).unwrap(),
            trsf.value(1, 3).unwrap(),
            trsf.value(1, 4).unwrap(),
            trsf.value(2, 1).unwrap(),
            trsf.value(2, 2).unwrap(),
            trsf.value(2, 3).unwrap(),
            trsf.value(2, 4).unwrap(),
            trsf.value(3, 1).unwrap(),
            trsf.value(3, 2).unwrap(),
            trsf.value(3, 3).unwrap(),
            trsf.value(3, 4).unwrap(),
            copy,
        )
        .map_err(OcctError::from)?;
        Ok(Self {
            inner,
            _not_send: PhantomData,
        })
    }

    /// Returns the transformed shape. Consumes `self`.
    ///
    /// One build per instance is load-bearing, not incidental: `Shape()` is
    /// not overridden on `BRepBuilderAPI_Transform` (plain
    /// `BRepBuilderAPI_MakeShape` base), and the transform already ran once
    /// in the constructor — there is no repeat-build path to guard, but a
    /// future refactor that added one would inherit the same stale-`myShape`
    /// hazard documented on [`FuseBuilder::build`](crate::rs_topo::FuseBuilder::build).
    pub fn build(mut self) -> Result<OcShape, OcctError> {
        self.try_build()
    }

    /// Returns the transformed shape, keeping the builder alive for shape
    /// history queries via [`BuiltWithHistory`]. One build per instance —
    /// see [`build`](Self::build).
    pub fn build_with_history(mut self) -> Result<BuiltWithHistory<Self>, OcctError> {
        let shape = self.try_build()?;
        Ok(BuiltWithHistory::new(self, shape))
    }

    fn try_build(&mut self) -> Result<OcShape, OcctError> {
        if self.inner.is_done() {
            // Safety: MakeTransformBuilder::shape() returns make_unique<TopoDS_Shape>
            // on a completed builder — non-null.
            Ok(unsafe { OcShape::from_ffi_unchecked(self.inner.pin_mut().shape()) })
        } else {
            Err(OcctError {
                kind: OcctErrorKind::ConstructionError,
                message: "BRepBuilderAPI_Transform: IsDone() false after construction".to_owned(),
            })
        }
    }
}

impl HistoryProvider for TransformBuilder {
    fn modified_shapes(&mut self, input: &OcShape) -> impl Iterator<Item = OcShape> + '_ {
        ShapeListIter::new(ffi::transform_modified_iter(
            self.inner.pin_mut(),
            input.as_ffi(),
        ))
    }
    fn generated_shapes(&mut self, input: &OcShape) -> impl Iterator<Item = OcShape> + '_ {
        ShapeListIter::new(ffi::transform_generated_iter(
            self.inner.pin_mut(),
            input.as_ffi(),
        ))
    }
    fn is_shape_deleted(&mut self, input: &OcShape) -> bool {
        self.inner.pin_mut().is_deleted(input.as_ffi())
    }
}
