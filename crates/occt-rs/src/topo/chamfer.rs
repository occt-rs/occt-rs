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
use crate::topo::{OcEdge, OcFace, OcShape};

/// Builder for chamfer operations on a solid.
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
        self.inner.pin_mut().build().map_err(OcctError::from)?;
        if self.inner.is_done() {
            Ok(OcShape::from_ffi(self.inner.pin_mut().shape()))
        } else {
            Err(OcctError {
                kind: OcctErrorKind::ConstructionError,
                message: "BRepFilletAPI_MakeChamfer: IsDone() false after Build()".to_owned(),
            })
        }
    }
}
