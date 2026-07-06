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

/// Holds a build with its output shape. The build can be queried foc topo-naming provenance
///
/// The query methods feed directly into [`TopoNamingBuilder`]:
///
/// - retreive modified topology with [`TopoNamingBuilder::modified`]
/// - retreive generated topology with [`TopoNamingBuilder::generated`]
///
/// This is the mechanism that keeps the naming graph stable. With everything recorded,
/// [`TopoNamingSelector::solve`] can re-find sub-shapes after rebuild.
///
/// # Why `&mut self` on history queries
///
/// `Modified`, `Generated`, and `IsDeleted` are non-const in OCCT. The
/// `&mut self` is a fidelity artefact of the cxx bridge, not a mutation
/// signal — results are stable after the build is complete.
///
/// # Example
///
/// Here we extend the example written out in [`ChamferBuilder`], doc-tests
/// showing all three query methods and how their results feed into the
/// naming record:
///
/// ```text
/// main (0:1)
/// └── 3 (0:1:3)   body
///     ├── 1 (0:1:3:1)   solid
///     │       TopoNamingNamedShape (Primitive, 1×1×1 prism)
///     │       OcReal "depth" = 1.0
///     └── 2 (0:1:3:2)   chamfer
///             TopoNamingNamedShape (Modify, chamfered solid)
///             OcReal "distance" = 0.05
/// ```
///
/// ```
/// # use occt_rs::gp::{OcPnt, OcVec};
/// # use occt_rs::ocaf::OcApplication;
/// # use occt_rs::ocaf::attributes::OcReal;
/// # use occt_rs::ocaf::topo_naming::{TopoNamingEvolution, TopoNamingNamedShape};
/// # use occt_rs::rs_topo::{ChamferBuilder, OcEdge, OcFace, OcWire};
///
/// # let mut app = OcApplication::new();
/// # let mut doc = app.new_document("BinXCAF").unwrap();
///
/// let main = doc.main();
///
/// let wire = OcWire::from_edges(&[
///     // <snipped edge creation>
///     # OcEdge::from_pnts(OcPnt::new(0.0, 0.0, 0.0), OcPnt::new(1.0, 0.0, 0.0)).unwrap(),
///     # OcEdge::from_pnts(OcPnt::new(1.0, 0.0, 0.0), OcPnt::new(1.0, 1.0, 0.0)).unwrap(),
///     # OcEdge::from_pnts(OcPnt::new(1.0, 1.0, 0.0), OcPnt::new(0.0, 1.0, 0.0)).unwrap(),
///     # OcEdge::from_pnts(OcPnt::new(0.0, 1.0, 0.0), OcPnt::new(0.0, 0.0, 0.0)).unwrap(),
/// ]).unwrap();
/// let solid_shape = OcFace::from_wire(&wire, true).unwrap()
///     .extrude(OcVec::new(0.0, 0.0, 1.0)).unwrap();
/// let initial_faces: Vec<_> = solid_shape.faces().collect();
/// let initial_edges: Vec<_> = solid_shape.edges().collect();
///
/// // Record the solid on body/1
/// {
///     # doc.begin_command().unwrap();
///     # let lsolid = main.get_or_create_child(3)
///     #                  .get_or_create_child(1);
///     # OcReal::set(&lsolid, 1.0).unwrap();
///     # doc.name_builder(&lsolid).primitive(&solid_shape);
///     doc.commit().unwrap();
/// }
///
/// // Apply the chamfer and query history
/// doc.begin_command().unwrap();
/// let lchamfer = main.get_or_create_child(3)
///                    .get_or_create_child(2);
///
/// let distance = OcReal::set(&lchamfer, 0.05).unwrap();
/// let edge = solid_shape.edges().next().unwrap();
/// let mut cb = ChamferBuilder::new(&solid_shape).unwrap();
/// cb.add_edge(distance.get(), &edge).unwrap();
/// let mut built = cb.build_with_history().unwrap();
///
/// // modified() — which original faces were modified by the chamfer.
/// // Feed these into TopoNamingBuilder::modified so the naming graph
/// // records what changed.
/// let mut nb = doc.name_builder(&lchamfer);
/// let mut modified_count = 0;
/// for face in &initial_faces {
///     for modified in built.modified(&face.as_shape()) {
///         nb.modified(&face.as_shape(), &modified);
///         modified_count += 1;
///     }
/// }
/// assert!(modified_count > 0);
///
/// // generated() — the new chamfer face itself, produced from the edge.
/// // This is the face sketch2 will be drawn on.
/// let generated: Vec<_> = built.generated(&edge.as_shape()).collect();
/// assert!(!generated.is_empty());
///
/// // is_deleted() — chamfer modifies faces, it does not delete them.
/// for face in &initial_faces {
///     assert!(!built.is_deleted(&face.as_shape()));
/// }
///
/// doc.commit().unwrap();
///
/// assert_eq!(
///     TopoNamingNamedShape::find(&lchamfer).unwrap().evolution(),
///     Some(TopoNamingEvolution::Modify),
/// );
/// ```
///
/// [`TopoNamingBuilder`]: crate::ocaf::topo_naming::TopoNamingBuilder
/// [`TopoNamingBuilder::modified`]: crate::ocaf::topo_naming::TopoNamingBuilder::modified
/// [`TopoNamingBuilder::generated`]: crate::ocaf::topo_naming::TopoNamingBuilder::generated
/// [`TopoNamingSelector::solve`]: crate::ocaf::topo_naming::TopoNamingSelector::solve
/// [`ChamferBuilder`]: crate::rs_topo::ChamferBuilder
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
