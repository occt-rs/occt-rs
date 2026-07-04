//! Application Framework: start here
//!
//! OCAF: OpenCascade Application Framework
//!
//! In brief, OCAF provides the tooling framework for implementing CAD systems. Major features
//! include:
//!  - Topological Naming
//!  - Undo/Redo
//!  - export to standard formats
//!  - Parametric rebuilding
//!  - Transactional app-data management.
//!
//! To get a feel for using this library, start with the [`application`] module, then look at the
//! [`document`] module. Explore how nodes in the document data-tree are managed, and the role of
//! the [`Command`] binding in setting up a transaction
//!
//! Presently, documentation is sparse. The plan is to doc-comment with example code that provides
//! you with the bread-crumb trail through examples to equip you to make a parametric modelling
//! application in Rust using OCCT + OCAF
//!
//! For canonical documentation of the bound library, visit <https://dev.opencascade.org/doc/overview/html/occt_user_guides__ocaf.html>
//!
//! [`Command`]: document::Command

pub mod application;
pub mod attributes;
pub mod document;
pub mod label;
pub mod tdata_xtd;
pub mod topo_naming;
pub use application::OcApplication;
pub use attributes::{OcInteger, OcName, OcReal};
pub use document::OcDocument;
pub use label::{OcChildIterator, OcLabel};
pub use tdata_xtd::{
    ConstraintKind, GeometryKind, OcAxisAttr, OcConstraintAttr, OcGeometryAttr, OcPlaneAttr,
    OcPointAttr, OcPositionAttr,
};
pub use topo_naming::{
    TopoNamingBuilder, TopoNamingEvolution, TopoNamingNamedShape, TopoNamingSelector,
};
