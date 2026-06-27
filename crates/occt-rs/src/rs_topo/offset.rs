//! Offset shape and thick solid builders.
//!
//! # Offset shape
//!
//! [`OffsetShapeBuilder`] wraps `BRepOffsetAPI_MakeOffsetShape::PerformBySimple`.
//! For a one-call convenience, use [`OcShape::offset_shape`].
//!
//! # Thick solid
//!
//! [`ThickSolidBuilder`] accumulates closing faces (the faces to remove when
//! hollowing out the solid), then calls `MakeThickSolidByJoin` at build time.
//! For a one-call convenience, use [`OcShape::thick_solid`].
//!
//! # History
//!
//! `Modified` / `Generated` deferred to F2.
//!
//! [`OcShape::offset_shape`]: crate::topo::OcShape::offset_shape
//! [`OcShape::thick_solid`]: crate::topo::OcShape::thick_solid

use std::marker::PhantomData;

use occt_sys::ffi;

use crate::error::{OcctError, OcctErrorKind};
use crate::rs_topo::{BuiltWithHistory, HistoryProvider, OcFace, OcShape};

// ── OffsetShapeBuilder ────────────────────────────────────────────────────────

/// Wraps `BRepOffsetAPI_MakeOffsetShape::PerformBySimple`.
pub struct OffsetShapeBuilder {
    inner: cxx::UniquePtr<ffi::MakeOffsetShapeBuilder>,
    _not_send: PhantomData<*mut ()>,
}

impl OffsetShapeBuilder {
    pub fn new() -> Self {
        Self {
            inner: ffi::new_make_offset_shape_builder(),
            _not_send: PhantomData,
        }
    }

    /// Performs the offset and returns the result.
    ///
    /// Positive `offset` expands outward; negative shrinks inward.
    pub fn perform(mut self, shape: &OcShape, offset: f64) -> Result<OcShape, OcctError> {
        self.inner
            .pin_mut()
            .perform(shape.as_ffi(), offset)
            .map_err(OcctError::from)?;
        if self.inner.is_done() {
            // Safety: MakeOffsetShapeBuilder::shape() returns make_unique<TopoDS_Shape>
            // on a completed builder — non-null.
            Ok(unsafe { OcShape::from_ffi_unchecked(self.inner.pin_mut().shape()) })
        } else {
            Err(OcctError {
                kind: OcctErrorKind::ConstructionError,
                message: "BRepOffsetAPI_MakeOffsetShape: IsDone() false".to_owned(),
            })
        }
    }
    pub fn build_with_history(mut self) -> Result<BuiltWithHistory<Self>, OcctError> {
        let shape = self.try_build()?;
        Ok(BuiltWithHistory::new(self, shape))
    }

    fn try_build(&mut self) -> Result<OcShape, OcctError> {
        todo!("move existing build/perform body here")
    }
}

impl Default for OffsetShapeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl HistoryProvider for OffsetShapeBuilder {
    fn modified_shapes(&mut self, input: &OcShape) -> Vec<OcShape> {
        let s = input.as_ffi();
        let count = self.inner.pin_mut().modified_count(s);
        (0..count)
            .map(|i| {
                // Safety: modified_at returns make_unique<TopoDS_Shape>(*it) over
                // a TopTools_ListOfShape populated by OCCT — non-null by OCCT contract.
                unsafe { OcShape::from_ffi_unchecked(self.inner.pin_mut().modified_at(s, i)) }
            })
            .collect()
    }

    fn generated_shapes(&mut self, input: &OcShape) -> Vec<OcShape> {
        let s = input.as_ffi();
        let count = self.inner.pin_mut().generated_count(s);
        (0..count)
            .map(|i| {
                // Safety: generated_at returns make_unique<TopoDS_Shape>(*it) over
                // a TopTools_ListOfShape populated by OCCT — non-null by OCCT contract.
                unsafe { OcShape::from_ffi_unchecked(self.inner.pin_mut().generated_at(s, i)) }
            })
            .collect()
    }

    fn is_shape_deleted(&mut self, input: &OcShape) -> bool {
        self.inner.pin_mut().is_deleted(input.as_ffi())
    }
}

// ── ThickSolidBuilder ─────────────────────────────────────────────────────────

/// Builds a hollow solid by removing closing faces and offsetting inward.
///
/// Add one or more faces to remove via [`add_closing_face`], then call
/// [`build`] with the base solid and offset parameters.
///
/// [`add_closing_face`]: ThickSolidBuilder::add_closing_face
/// [`build`]: ThickSolidBuilder::build
pub struct ThickSolidBuilder {
    inner: cxx::UniquePtr<ffi::MakeThickSolidBuilder>,
    _not_send: PhantomData<*mut ()>,
}

impl ThickSolidBuilder {
    pub fn new() -> Self {
        Self {
            inner: ffi::new_make_thick_solid_builder(),
            _not_send: PhantomData,
        }
    }

    /// Registers a face to remove (open face) when hollowing the solid.
    pub fn add_closing_face(&mut self, face: &OcFace) {
        self.inner.pin_mut().add_closing_face(face.as_ffi());
    }

    /// Hollows `shape` with the registered closing faces removed.
    ///
    /// `offset` is the wall thickness (typically negative to hollow inward).
    /// `tolerance` controls geometrical precision; `1e-3` is typical.
    pub fn build(
        mut self,
        shape: &OcShape,
        offset: f64,
        tolerance: f64,
    ) -> Result<OcShape, OcctError> {
        self.try_build(shape, offset, tolerance)
    }
    pub fn build_with_history(
        mut self,
        shape: &OcShape,
        offset: f64,
        tolerance: f64,
    ) -> Result<BuiltWithHistory<Self>, OcctError> {
        let shape = self.try_build(shape, offset, tolerance)?;
        Ok(BuiltWithHistory::new(self, shape))
    }

    fn try_build(
        &mut self,
        shape: &OcShape,
        offset: f64,
        tolerance: f64,
    ) -> Result<OcShape, OcctError> {
        self.inner
            .pin_mut()
            .build(shape.as_ffi(), offset, tolerance)
            .map_err(OcctError::from)?;
        if self.inner.is_done() {
            // Safety: MakeThickSolidBuilder::shape() returns make_unique<TopoDS_Shape>
            // on a completed builder — non-null.
            Ok(unsafe { OcShape::from_ffi_unchecked(self.inner.pin_mut().shape()) })
        } else {
            Err(OcctError {
                kind: OcctErrorKind::ConstructionError,
                message: "BRepOffsetAPI_MakeThickSolid: IsDone() false after build".to_owned(),
            })
        }
    }
}

impl Default for ThickSolidBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl HistoryProvider for ThickSolidBuilder {
    fn modified_shapes(&mut self, input: &OcShape) -> Vec<OcShape> {
        let s = input.as_ffi();
        let count = self.inner.pin_mut().modified_count(s);
        (0..count)
            .map(|i| {
                // Safety: modified_at returns make_unique<TopoDS_Shape>(*it) over
                // a TopTools_ListOfShape populated by OCCT — non-null by OCCT contract.
                unsafe { OcShape::from_ffi_unchecked(self.inner.pin_mut().modified_at(s, i)) }
            })
            .collect()
    }

    fn generated_shapes(&mut self, input: &OcShape) -> Vec<OcShape> {
        let s = input.as_ffi();
        let count = self.inner.pin_mut().generated_count(s);
        (0..count)
            .map(|i| {
                // Safety: generated_at returns make_unique<TopoDS_Shape>(*it) over
                // a TopTools_ListOfShape populated by OCCT — non-null by OCCT contract.
                unsafe { OcShape::from_ffi_unchecked(self.inner.pin_mut().generated_at(s, i)) }
            })
            .collect()
    }

    fn is_shape_deleted(&mut self, input: &OcShape) -> bool {
        self.inner.pin_mut().is_deleted(input.as_ffi())
    }
}
