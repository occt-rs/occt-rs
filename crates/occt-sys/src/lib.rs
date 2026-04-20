/// Raw cxx bridge for OpenCASCADE Technology (OCCT).
///
/// Sourced from:
///   - OCCT 7.9 reference documentation <https://dev.opencascade.org/doc/refman/html/>
///   - cxx documentation <https://cxx.rs/>
///
/// No derivation from any other binding crate.
/// See DEVELOPMENT.md for the IP hygiene policy.
///
/// # Design
///
/// The `gp_*` types are geometric primitives fully described by their
/// coordinates.  All mathematical operations on them (dot product, cross
/// product, angle, arithmetic) are implemented in pure Rust in `occt-rs`
/// without crossing the FFI boundary.
///
/// This bridge exposes only what is needed to materialise `gp_*` objects
/// when passing to higher-level OCCT APIs (BRep construction, surface
/// queries, etc.).  The opaque types are declared so that future bindings
/// can accept or return them directly when needed.

#[cxx::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("occt_sys.hxx");
        // ── TopoDS_Face ──────────────────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/class_topo_d_s___face.html
        #[cxx_name = "TopoDS_Face"]
        type TopdsFace;

        /// Copy-constructs a TopoDS_Face.  Called by OcFace's Clone impl.
        fn clone_face(f: &TopdsFace) -> UniquePtr<TopdsFace>;

        // ── MakeFaceBuilder ──────────────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/class_b_rep_builder_a_p_i___make_face.html
        type MakeFaceBuilder;

        /// Constructs a MakeFaceBuilder from a wire.
        /// `only_plane = true` restricts construction to planar faces.
        fn new_make_face_from_wire(w: &TopodsWire, only_plane: bool) -> UniquePtr<MakeFaceBuilder>;

        /// Returns true when the face was successfully constructed.
        fn is_done(self: &MakeFaceBuilder) -> bool;

        /// Returns the raw BRepBuilderAPI_FaceError value.
        fn error(self: &MakeFaceBuilder) -> i32;

        /// Returns the constructed face.  Only call when is_done() is true.
        fn face(self: Pin<&mut MakeFaceBuilder>) -> UniquePtr<TopdsFace>;

        // ── Face inspection ──────────────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/class_b_rep_tools.html
        /// Returns the outer boundary wire of a face via BRepTools::OuterWire.
        fn face_outer_wire(f: &TopdsFace) -> UniquePtr<TopodsWire>;

        // ── WireEdgeExplorer ─────────────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/class_b_rep_tools___wire_explorer.html
        type WireEdgeExplorer;

        fn new_wire_edge_explorer(w: &TopodsWire) -> UniquePtr<WireEdgeExplorer>;
        fn more(self: &WireEdgeExplorer) -> bool;
        fn next(self: Pin<&mut WireEdgeExplorer>);
        fn current_edge(self: &WireEdgeExplorer) -> UniquePtr<TopodsEdge>;

        // ── Edge vertex access ───────────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/class_top_exp.html
        fn edge_start_vertex(e: &TopodsEdge) -> UniquePtr<TopodsVertex>;
        fn edge_end_vertex(e: &TopodsEdge) -> UniquePtr<TopodsVertex>;
        #[cxx_name = "TopoDS_Wire"]
        type TopodsWire;

        fn clone_wire(w: &TopodsWire) -> UniquePtr<TopodsWire>;

        type MakeWireBuilder;

        fn new_make_wire_builder() -> UniquePtr<MakeWireBuilder>;
        fn add_edge(self: Pin<&mut MakeWireBuilder>, e: &TopodsEdge);
        fn is_done(self: &MakeWireBuilder) -> bool;
        fn error(self: &MakeWireBuilder) -> i32;
        fn wire(self: Pin<&mut MakeWireBuilder>) -> UniquePtr<TopodsWire>;
        // ── TopoDS_Edge ──────────────────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/class_topo_d_s___edge.html
        #[cxx_name = "TopoDS_Edge"]
        type TopodsEdge;

        /// Copy-constructs a TopoDS_Edge.  Called by OcEdge's Clone impl.
        fn clone_edge(e: &TopodsEdge) -> UniquePtr<TopodsEdge>;

        // ── MakeEdgeBuilder ──────────────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/class_b_rep_builder_a_p_i___make_edge.html
        type MakeEdgeBuilder;

        /// Constructs a MakeEdgeBuilder from two vertices.
        fn new_make_edge_builder(
            v1: &TopodsVertex,
            v2: &TopodsVertex,
        ) -> UniquePtr<MakeEdgeBuilder>;

        /// Returns true when the edge was successfully constructed.
        fn is_done(self: &MakeEdgeBuilder) -> bool;

        /// Returns the raw BRepBuilderAPI_EdgeError value.
        /// Map against constants in occt_rs::topo::edge::EdgeError.
        fn error(self: &MakeEdgeBuilder) -> i32;

        /// Returns the constructed edge.  Only call when is_done() is true.
        fn edge(self: Pin<&mut MakeEdgeBuilder>) -> UniquePtr<TopodsEdge>;

        // ── TopoDS_Vertex ────────────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/class_topo_d_s___vertex.html
        #[cxx_name = "TopoDS_Vertex"]
        type TopodsVertex;

        /// Constructs a TopoDS_Vertex from raw coordinates via
        /// BRepBuilderAPI_MakeVertex.
        fn make_vertex(x: f64, y: f64, z: f64) -> UniquePtr<TopodsVertex>;

        /// Copy-constructs a TopoDS_Vertex.  Called by OcVertex's Clone impl.
        fn clone_vertex(v: &TopodsVertex) -> UniquePtr<TopodsVertex>;

        /// Reads the X coordinate of the point embedded in a TopoDS_Vertex.
        fn vertex_pnt_x(v: &TopodsVertex) -> f64;
        /// Reads the Y coordinate of the point embedded in a TopoDS_Vertex.
        fn vertex_pnt_y(v: &TopodsVertex) -> f64;
        /// Reads the Z coordinate of the point embedded in a TopoDS_Vertex.
        fn vertex_pnt_z(v: &TopodsVertex) -> f64;

        // ── gp_Pnt ───────────────────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/classgp___pnt.html
        #[cxx_name = "gp_Pnt"]
        type GpPnt;

        /// Materialises a `gp_Pnt` for passing to an OCCT API.
        /// Called by `OcPnt::to_ffi()` in the safe layer.
        fn new_gp_pnt_xyz(x: f64, y: f64, z: f64) -> UniquePtr<GpPnt>;

        // ── gp_Vec ───────────────────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/classgp___vec.html
        #[cxx_name = "gp_Vec"]
        type GpVec;

        /// Materialises a `gp_Vec` for passing to an OCCT API.
        /// Called by `OcVec::to_ffi()` in the safe layer.
        fn new_gp_vec_xyz(x: f64, y: f64, z: f64) -> UniquePtr<GpVec>;

        // ── gp_Dir ───────────────────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/classgp___dir.html
        #[cxx_name = "gp_Dir"]
        type GpDir;

        /// Materialises a `gp_Dir` for passing to an OCCT API.
        /// Called by `OcDir::to_ffi()` in the safe layer.
        ///
        /// `OcDir` validates and normalises at construction in pure Rust, so
        /// this should never return `Err` in practice.  The `Result` is
        /// retained as a safety net against invariant violations.
        fn new_gp_dir_xyz(x: f64, y: f64, z: f64) -> Result<UniquePtr<GpDir>>;

        // ── gp_Ax1 ───────────────────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/classgp___ax1.html
        #[cxx_name = "gp_Ax1"]
        type GpAx1;

        /// Materialises a `gp_Ax1` for passing to an OCCT API.
        /// Called by `OcAx1::to_ffi()` in the safe layer.
        ///
        /// The direction components are guaranteed unit-magnitude (from `OcDir`),
        /// so this should never return `Err` in practice.  The `Result` is
        /// retained as a safety net against invariant violations.
        fn new_gp_ax1(
            px: f64,
            py: f64,
            pz: f64,
            dx: f64,
            dy: f64,
            dz: f64,
        ) -> Result<UniquePtr<GpAx1>>;

        // ── gp_Ax2 ───────────────────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/classgp___ax2.html
        #[cxx_name = "gp_Ax2"]
        type GpAx2;

        /// Materialises a `gp_Ax2` for passing to an OCCT API.
        /// Called by `OcAx2::to_ffi()` in the safe layer.
        ///
        /// By the time this is called, `OcAx2` has already validated that the
        /// main direction and X direction are non-parallel and has stored the
        /// corrected perpendicular X direction, so `ConstructionError` should
        /// never fire in practice.  The `Result` is retained as a safety net.
        fn new_gp_ax2(
            px: f64,
            py: f64,
            pz: f64,
            nx: f64,
            ny: f64,
            nz: f64,
            xx: f64,
            xy: f64,
            xz: f64,
        ) -> Result<UniquePtr<GpAx2>>;
    }
}
