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
use crate::rs_topo::shape_history_iter::ShapeListIter;
use crate::rs_topo::{BuiltWithHistory, HistoryProvider, OcEdge, OcShape};

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
        self.try_build()
    }
    /// Perform the fillet and return the result, keeping the builder alive
    /// for shape history queries via [`BuiltWithHistory`].
    ///
    /// Use [`OcShape::fillet`] or `build()` when history is not needed.
    pub fn build_with_history(mut self) -> Result<BuiltWithHistory<Self>, OcctError> {
        let shape = self.try_build()?;
        Ok(BuiltWithHistory::new(self, shape))
    }

    /// Shared build body used by both `build()` and `build_with_history()`.
    ///
    /// # Integration note
    /// Move the body of the existing `build()` method here verbatim.
    /// Replace `build()` body with `self.try_build()`.
    fn try_build(&mut self) -> Result<OcShape, OcctError> {
        self.inner.pin_mut().build().map_err(OcctError::from)?;
        if self.inner.is_done() {
            // Safety: MakeFilletBuilder::shape() returns make_unique<TopoDS_Shape>
            // on a completed builder — non-null.
            Ok(unsafe { OcShape::from_ffi_unchecked(self.inner.pin_mut().shape()) })
        } else {
            Err(OcctError {
                kind: OcctErrorKind::ConstructionError,
                message: "BRepFilletAPI_MakeFillet: IsDone() false after Build()".to_owned(),
            })
        }
    }
}
impl HistoryProvider for FilletBuilder {
    fn modified_shapes(&mut self, input: &OcShape) -> impl Iterator<Item = OcShape> + '_ {
        ShapeListIter::new(ffi::fillet_modified_iter(
            self.inner.pin_mut(),
            input.as_ffi(),
        ))
    }
    fn generated_shapes(&mut self, input: &OcShape) -> impl Iterator<Item = OcShape> + '_ {
        ShapeListIter::new(ffi::fillet_generated_iter(
            self.inner.pin_mut(),
            input.as_ffi(),
        ))
    }
    fn is_shape_deleted(&mut self, input: &OcShape) -> bool {
        self.inner.pin_mut().is_deleted(input.as_ffi())
    }
}
#[cfg(test)]
mod history_tests {
    use super::*;
    use crate::{
        app_util::KeyedWireBuilder,
        gp::{OcDir, OcPnt, OcVec},
        rs_topo::OcFace,
    };

    /// Fillet a box, confirm that original faces appear as modified in output,
    /// and that filleted edges report generated faces.
    #[test]
    fn fillet_history_modified_and_generated() {
        // Build a 1×1×1 box
        let face =
            OcFace::from_wire_on_plane(OcPnt::origin(), OcDir::new(0.0, 0.0, 1.0).unwrap(), &{
                use KeyedWireBuilder;
                let mut b: KeyedWireBuilder<u32> = KeyedWireBuilder::new();
                b.add_edge(0, OcPnt::new(0.0, 0.0, 0.0), 1, OcPnt::new(1.0, 0.0, 0.0))
                    .unwrap();
                b.add_edge(1, OcPnt::new(1.0, 0.0, 0.0), 2, OcPnt::new(1.0, 1.0, 0.0))
                    .unwrap();
                b.add_edge(2, OcPnt::new(1.0, 1.0, 0.0), 3, OcPnt::new(0.0, 1.0, 0.0))
                    .unwrap();
                b.add_edge(3, OcPnt::new(0.0, 1.0, 0.0), 0, OcPnt::new(0.0, 0.0, 0.0))
                    .unwrap();
                b.build().unwrap()
            })
            .unwrap();
        let solid_shape = face.extrude(OcVec::new(0.0, 0.0, 1.0)).unwrap();

        // Collect unique faces and edges before the operation
        let pre_faces: Vec<_> = {
            let all = solid_shape.faces();
            let mut seen = std::collections::HashSet::new();
            all.into_iter()
                .filter(|f| seen.insert(f.shape_key()))
                .collect()
        };
        let pre_edges: Vec<_> = {
            let all = solid_shape.edges();
            let mut seen = std::collections::HashSet::new();
            all.into_iter()
                .filter(|e| seen.insert(e.shape_key()))
                .collect()
        };

        // Fillet all edges with radius 0.05
        let mut builder = FilletBuilder::new(&solid_shape).unwrap();
        for edge in &pre_edges {
            let _ = builder.add_edge(0.05, edge); // ignore individual errors
        }
        let mut built = builder.build_with_history().unwrap();

        // Every original face should appear as modified (its bounds changed)
        // or be reported unchanged — either is valid. At minimum, `modified`
        // must not panic and must return shapes belonging to the output.
        let mut any_modified = false;
        for face in &pre_faces {
            let fshape = &face.as_shape();
            let mods = built.modified(fshape);
            for m in mods {
                any_modified = true;
                assert_eq!(m.shape_type(), crate::rs_topo::ShapeType::Face);
            }
        }
        assert!(
            any_modified,
            "expected at least some original faces to be reported as modified after filleting"
        );

        // Filleted edges generate faces (the fillet surface faces)
        let mut any_generated = false;
        for edge in &pre_edges {
            if built.generated(&edge.as_shape()).next().is_some() {
                any_generated = true;
            }
        }
        assert!(
            any_generated,
            "expected some edges to generate fillet surface faces"
        );

        // No original face should be reported as deleted
        // (fillet modifies faces; it does not delete them)
        for face in &pre_faces {
            assert!(
                !built.is_deleted(&face.as_shape()),
                "fillet should not delete any original face"
            );
        }
    }
}
