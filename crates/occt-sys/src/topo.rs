//! cxx bridge for OCCT topological shape builders and inspectors.
//!
//! The topo types form a dependency chain (Vertex → Edge → Wire → Face →
//! Solid), so all are declared in a single bridge to avoid cross-bridge
//! `ExternType` forwarding boilerplate.
//!
//! # Builder lifetime
//!
//! Builder types that expose `Modified()`, `Generated()`, or `IsDeleted()`
//! (currently `MakePrismBuilder`) must not be dropped before shape history
//! queries are complete.
//!
//! Generated using LLMs from informatin in:
//!   - OCCT 7.9 reference: <https://dev.opencascade.org/doc/refman/html/>
//!   - cxx docs: <https://cxx.rs/>
//!
//! No derivation from any other binding crate.

#[cxx::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("occt_sys/topo.hxx");

        // ── TopoDS_Vertex ─────────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/class_topo_d_s___vertex.html
        #[cxx_name = "TopoDS_Vertex"]
        type TopodsVertex;

        fn make_vertex(x: f64, y: f64, z: f64) -> UniquePtr<TopodsVertex>;
        fn clone_vertex(v: &TopodsVertex) -> UniquePtr<TopodsVertex>;
        fn vertex_pnt_x(v: &TopodsVertex) -> f64;
        fn vertex_pnt_y(v: &TopodsVertex) -> f64;
        fn vertex_pnt_z(v: &TopodsVertex) -> f64;

        // ── TopoDS_Edge ───────────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/class_topo_d_s___edge.html
        #[cxx_name = "TopoDS_Edge"]
        type TopodsEdge;

        fn clone_edge(e: &TopodsEdge) -> UniquePtr<TopodsEdge>;
        fn edge_start_vertex(e: &TopodsEdge) -> UniquePtr<TopodsVertex>;
        fn edge_end_vertex(e: &TopodsEdge) -> UniquePtr<TopodsVertex>;

        // ── MakeEdgeBuilder ───────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/class_b_rep_builder_a_p_i___make_edge.html
        type MakeEdgeBuilder;

        fn new_make_edge_builder(
            v1: &TopodsVertex,
            v2: &TopodsVertex,
        ) -> UniquePtr<MakeEdgeBuilder>;
        fn is_done(self: &MakeEdgeBuilder) -> bool;
        fn error(self: &MakeEdgeBuilder) -> i32;
        fn edge(self: Pin<&mut MakeEdgeBuilder>) -> UniquePtr<TopodsEdge>;

        // ── TopoDS_Wire ───────────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/class_topo_d_s___wire.html
        #[cxx_name = "TopoDS_Wire"]
        type TopodsWire;

        fn clone_wire(w: &TopodsWire) -> UniquePtr<TopodsWire>;

        // ── MakeWireBuilder ───────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/class_b_rep_builder_a_p_i___make_wire.html
        type MakeWireBuilder;

        fn new_make_wire_builder() -> UniquePtr<MakeWireBuilder>;
        fn add_edge(self: Pin<&mut MakeWireBuilder>, e: &TopodsEdge);
        fn is_done(self: &MakeWireBuilder) -> bool;
        fn error(self: &MakeWireBuilder) -> i32;
        fn wire(self: Pin<&mut MakeWireBuilder>) -> UniquePtr<TopodsWire>;

        // ── WireEdgeExplorer ──────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/class_b_rep_tools___wire_explorer.html
        type WireEdgeExplorer;

        fn new_wire_edge_explorer(w: &TopodsWire) -> UniquePtr<WireEdgeExplorer>;
        fn more(self: &WireEdgeExplorer) -> bool;
        fn next(self: Pin<&mut WireEdgeExplorer>);
        fn current_edge(self: &WireEdgeExplorer) -> UniquePtr<TopodsEdge>;

        // ── TopoDS_Face ───────────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/class_topo_d_s___face.html
        #[cxx_name = "TopoDS_Face"]
        type TopdsFace;

        fn clone_face(f: &TopdsFace) -> UniquePtr<TopdsFace>;
        fn face_outer_wire(f: &TopdsFace) -> UniquePtr<TopodsWire>;

        // ── MakeFaceBuilder ───────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/class_b_rep_builder_a_p_i___make_face.html
        type MakeFaceBuilder;

        fn new_make_face_from_wire(w: &TopodsWire, only_plane: bool) -> UniquePtr<MakeFaceBuilder>;
        fn is_done(self: &MakeFaceBuilder) -> bool;
        fn error(self: &MakeFaceBuilder) -> i32;
        fn face(self: Pin<&mut MakeFaceBuilder>) -> UniquePtr<TopdsFace>;

        // ── TopoDS_Solid ──────────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/class_topo_d_s___solid.html
        #[cxx_name = "TopoDS_Solid"]
        type TopdsSolid;

        fn clone_solid(s: &TopdsSolid) -> UniquePtr<TopdsSolid>;

        // ── MakePrismBuilder ──────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/class_b_rep_prim_a_p_i___make_prism.html
        //
        // Returns Result because MakePrism computes immediately in its
        // constructor and throws on failure rather than deferring to IsDone().
        type MakePrismBuilder;

        fn new_make_prism_from_face(
            face: &TopdsFace,
            vx: f64,
            vy: f64,
            vz: f64,
        ) -> Result<UniquePtr<MakePrismBuilder>>;
        fn is_done(self: &MakePrismBuilder) -> bool;
        fn solid(self: Pin<&mut MakePrismBuilder>) -> UniquePtr<TopdsSolid>;
    }
}
