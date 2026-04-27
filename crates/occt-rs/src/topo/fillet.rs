//! Constant-radius fillet builder.
//!
//! # Usage
//!
//! ```rust,ignore
//! use occt_rs::topo::{FilletBuilder, OcShape};
//!
//! let edges = solid.as_shape().edges();
//! let mut builder = FilletBuilder::new(&solid.as_shape())?;
//! for edge in &edges {
//!     builder.add_edge(0.1, edge)?;
//! }
//! let filleted: OcShape = builder.build()?;
//! ```
//!
//! For simple cases, prefer [`OcShape::fillet`].
//!
//! # History
//!
//! `BRepFilletAPI_MakeFillet` exposes `Modified` and `Generated` for shape
//! history queries.  These are deferred to Milestone F (`ShapeHistory` trait).
//! Do not drop `FilletBuilder` before history is read when that work lands.
//!
//! Reference: <https://dev.opencascade.org/doc/refman/html/class_b_rep_fillet_a_p_i___make_fillet.html>

use std::marker::PhantomData;

use occt_sys::ffi;

use crate::error::{OcctError, OcctErrorKind};
use crate::topo::{OcEdge, OcShape};

/// Builder for constant-radius fillets on a solid.
///
/// Constructed from a base shape; edges are registered via [`add_edge`];
/// [`build`] consumes the builder and returns the filleted shape.
///
/// [`add_edge`]: FilletBuilder::add_edge
/// [`build`]: FilletBuilder::build
pub struct FilletBuilder {
    pub(crate) inner: cxx::UniquePtr<ffi::MakeFilletBuilder>,
    _not_send: PhantomData<*mut ()>,
}

impl FilletBuilder {
    /// Constructs a fillet builder on `shape`.
    ///
    /// `shape` should be a solid or shell. Passing a lower-dimensional shape
    /// is accepted by OCCT but will produce no filleted edges.
    pub fn new(shape: &OcShape) -> Result<Self, OcctError> {
        let inner = ffi::new_make_fillet_builder(shape.as_ffi()).map_err(OcctError::from)?;
        Ok(Self {
            inner,
            _not_send: PhantomData,
        })
    }

    /// Registers a constant-radius fillet on `edge`.
    ///
    /// `edge` must be an edge of the shape passed to [`new`].
    ///
    /// Returns `Err` if OCCT raises (e.g. the edge is not on the shape).
    ///
    /// [`new`]: FilletBuilder::new
    pub fn add_edge(&mut self, radius: f64, edge: &OcEdge) -> Result<(), OcctError> {
        self.inner
            .pin_mut()
            .add_edge(radius, edge.as_ffi())
            .map_err(OcctError::from)
    }

    /// Computes the fillet operation and returns the resulting shape.
    ///
    /// Consumes `self`. Returns `Err` if OCCT raises during `Build()` or if
    /// `IsDone()` is false after construction.
    pub fn build(mut self) -> Result<OcShape, OcctError> {
        self.inner.pin_mut().build().map_err(OcctError::from)?;
        if self.inner.is_done() {
            Ok(OcShape::from_ffi(self.inner.pin_mut().shape()))
        } else {
            Err(OcctError {
                kind: OcctErrorKind::ConstructionError,
                message: "BRepFilletAPI_MakeFillet: IsDone() false after Build()".to_owned(),
            })
        }
    }
}
