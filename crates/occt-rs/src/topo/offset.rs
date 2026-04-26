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
use crate::topo::{OcFace, OcShape};

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
            Ok(OcShape::from_ffi(self.inner.pin_mut().shape()))
        } else {
            Err(OcctError {
                kind: OcctErrorKind::ConstructionError,
                message: "BRepOffsetAPI_MakeOffsetShape: IsDone() false".to_owned(),
            })
        }
    }
}

impl Default for OffsetShapeBuilder {
    fn default() -> Self {
        Self::new()
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
        self.inner
            .pin_mut()
            .build(shape.as_ffi(), offset, tolerance)
            .map_err(OcctError::from)?;
        if self.inner.is_done() {
            Ok(OcShape::from_ffi(self.inner.pin_mut().shape()))
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
