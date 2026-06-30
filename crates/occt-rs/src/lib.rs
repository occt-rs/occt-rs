//! Safe Rust bindings for [OpenCASCADE Technology (OCCT)][occt] 7.9,
//! a geometry kernel and application framework for building CAD software.
//!
//! OCCT provides two major layers that this crate exposes:
//!
//! - **Geometry and topology** ([`gp`], [`rs_topo`]) — points, directions,
//!   edges, wires, faces, solids, and the operations that build and modify them.
//! - **Application framework** ([`ocaf`]) — a transactional document model
//!   with undo/redo, persistent labelled storage, and topological naming so
//!   that references to sub-shapes survive parametric rebuild.
//!
//! ## Running scenario
//!
//! Doc-comment examples throughout this crate follow a single running scenario:
//! building a simple parametric CAD application that lets a user sketch a
//! profile, extrude it into a solid, chamfer an edge, and draw a second sketch
//! on the resulting chamfer face.  Each module's examples show one step in
//! that workflow.  If an example assumes context you haven't seen yet, it links
//! back to where that context is established.
//!
//! The application maintains one OCAF document whose label tree looks like
//! this:
//!
//! ```text
//! main  (doc.main(), tag 1 under the framework root)
//! │
//! ├── 1: planes // This is part of initial creation
//! │   ├── 1: XY plane   TopoNamingNamedShape (Primitive, planar face)
//! │   │                 OcPlaneAttr
//! │   ├── 2: YZ plane   TopoNamingNamedShape (Primitive, planar face)
//! │   │                 OcPlaneAttr
//! │   └── 3: XZ plane   TopoNamingNamedShape (Primitive, planar face)
//! │                     OcPlaneAttr
//! │
//! ├── 2: sketch  // Representing the author drawing a square on a base-plane
//! │   ├── 1: point A (0,1,0)  TopoNamingNamedShape (Primitive, vertex)
//! │   │                       OcPointAttr
//! │   ├── 2: point B (0,0,0)  TopoNamingNamedShape (Primitive, vertex)
//! │   │                       OcPointAttr
//! │   ├── 3: point C (1,0,0)  TopoNamingNamedShape (Primitive, vertex)
//! │   │                       OcPointAttr
//! │   ├── 4: point D (1,1,0)  TopoNamingNamedShape (Primitive, vertex)
//! │   │                       OcPointAttr
//! │   └── 5: face             TopoNamingNamedShape (Primitive, unit square face)
//! │
//! ├── 3: body
//! │   │   // The first shape: made frome the extrude
//! │   ├── 1: solid    TopoNamingNamedShape (Primitive, 1×1×1 prism)
//! │   │               TDataStd_Real  "depth" = 1.0
//! │   │   // A new shape is crated upon chamfer action
//! │   ├── 2: chamfer  TopoNamingNamedShape (Modify, chamfered solid)
//! │   │               TDataStd_Real  "distance" = 0.05
//! │   │   // A new shape is crated upon the pocet from sketch 2
//! │   └── 3: pocket   [planned — requires BRepAlgoAPI_Cut, not yet bound]
//! │
//! │      // Drawing a sketch on the face made by the chamfer
//! └── 4: sketch2  (triangle on the chamfer face)
//!     ├── 1: point E  TopoNamingNamedShape (Primitive, vertex)
//!     │               OcPointAttr
//!     ├── 2: point F  TopoNamingNamedShape (Primitive, vertex)
//!     │               OcPointAttr
//!     ├── 3: point G  TopoNamingNamedShape (Primitive, vertex)
//!     │               OcPointAttr
//!     ├── 4: face     TopoNamingNamedShape (Primitive, triangle face)
//!     │      // The selection on the face at 0:3:2
//!     └── 5: ref-face TopoNamingNamedShape (Selected — TopoNamingSelector)
//! ```
//!
//! ## Reading order
//!
//! - [`gp`] — geometry primitives: points, directions, coordinate frames.
//!   Start here if you are new to OCCT geometry.
//! - [`rs_topo`] — topology: edges, wires, faces, solids, and operation
//!   builders (fillet, chamfer, offset).  This is where sketch profiles and
//!   extruded bodies are built.
//! - [`ocaf`] — the application framework: documents, labels, commands,
//!   attributes, and topological naming.  Start with [`ocaf::OcApplication`],
//!   then [`ocaf::OcDocument`].
//! - [`tessellate`] — mesh tessellation: convert a shape to a triangle mesh
//!   for rendering.
//!
//! [occt]: https://dev.opencascade.org/
//! [TopoNamingNamedShape]: crate::ocaf::topo_naming::TopoNamingNamedShape

pub mod app_util;
pub mod error;
pub mod gp;
pub mod ocaf;
pub mod rs_topo;
pub mod tessellate;
