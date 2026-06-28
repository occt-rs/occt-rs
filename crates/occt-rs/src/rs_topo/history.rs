//! Shape history for post-build operation queries.
//!
//! `HistoryProvider` is implemented by operation builders that wrap a
//! `BRepBuilderAPI_MakeShape` descendant.  `BuiltWithHistory<B>` keeps the
//! builder alive after `build_with_history()` so that history remains valid.
//!
//! # Why `&mut self` on history queries
//!
//! `BRepBuilderAPI_MakeShape::Modified`, `::Generated`, and `::IsDeleted` are
//! all declared non-const in OCCT.  The cxx bridge maps non-const methods to
//! `Pin<&mut>`, which flows through to `&mut self` on the Rust trait.  Results
//! are stable after the build is complete; the `&mut` is a fidelity artefact,
//! not a mutation signal.
//!
//! Reference: <https://dev.opencascade.org/doc/refman/html/class_b_rep_builder_a_p_i___make_shape.html>

use crate::rs_topo::shape::OcShape;

/// Implemented by builders that expose post-build shape history.
///
/// The three methods map directly to the OCCT `BRepBuilderAPI_MakeShape`
/// virtual interface.  Callers typically use [`BuiltWithHistory`] rather than
/// calling these methods directly.
pub trait HistoryProvider {
    /// Shapes in the output that are modifications of `input`.
    fn modified_shapes(&mut self, input: &OcShape) -> impl Iterator<Item = OcShape> + '_;

    /// Shapes in the output that were generated from `input`.
    fn generated_shapes(&mut self, input: &OcShape) -> impl Iterator<Item = OcShape> + '_;

    /// True if `input` does not appear in the output in any form.
    fn is_shape_deleted(&mut self, input: &OcShape) -> bool;
}

/// Holds a completed builder alongside its output shape, keeping the builder
/// alive so that history queries remain valid.
///
/// Produced by `build_with_history()` on each operation builder.
///
/// # Mutability
///
/// History query methods require `&mut self` because the underlying OCCT
/// methods are non-const.  Declare the binding as `mut` at the call site:
///
/// ```rust,ignore
/// let mut built = FilletBuilder::new(&shape)?
///     .add_edge(0.1, &edge)?
///     .build_with_history()?;
///
/// let shape = built.shape().clone();
/// let modified: Vec<OcShape> = built.modified(&original_face);
/// ```
pub struct BuiltWithHistory<B: HistoryProvider> {
    builder: B,
    shape: OcShape,
}

impl<B: HistoryProvider> BuiltWithHistory<B> {
    /// Construct from a builder that has already performed its build and
    /// produced `shape`.  Called by each builder's `build_with_history()`.
    pub(crate) fn new(builder: B, shape: OcShape) -> Self {
        Self { builder, shape }
    }

    /// The result shape produced by the operation.
    ///
    /// Returns a reference.  Clone if you need an owned copy while also
    /// holding a mutable borrow for history queries.
    pub fn shape(&self) -> &OcShape {
        &self.shape
    }

    /// Shapes in the output that are modifications of `input`.
    ///
    /// An empty `Vec` means `input` was not modified (it may have been deleted
    /// or left unchanged; check [`is_deleted`][Self::is_deleted]).
    pub fn modified<'a>(&'a mut self, input: &'a OcShape) -> impl Iterator<Item = OcShape> + 'a {
        self.builder.modified_shapes(input)
    }

    /// Shapes in the output generated from `input`.
    pub fn generated<'a>(&'a mut self, input: &'a OcShape) -> impl Iterator<Item = OcShape> + 'a {
        self.builder.generated_shapes(input)
    }

    /// True if `input` does not appear in the output in any form.
    pub fn is_deleted(&mut self, input: &OcShape) -> bool {
        self.builder.is_shape_deleted(input)
    }
}
