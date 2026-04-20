/// Raw cxx bridge for OpenCASCADE Technology (OCCT).
///
/// LLM generated with reference from:
///   - OCCT 7.9 reference documentation <https://dev.opencascade.org/doc/refman/html/>
///   - cxx documentation <https://cxx.rs/>
///
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

        // ── TopoDS_Vertex ────────────────────────────────────────────────────
        // Reference (MakeVertex):
        //   https://dev.opencascade.org/doc/refman/html/class_b_rep_builder_a_p_i___make_vertex.html
        // Reference (BRep_Tool):
        //   https://dev.opencascade.org/doc/refman/html/class_b_rep___tool.html
        //
        // TopoDS_Vertex is a handle wrapper; copying shares the underlying
        // TShape (ref-counted).  OcVertex in occt-rs wraps UniquePtr<TopodsVertex>
        // and implements Clone via clone_vertex.

        #[cxx_name = "TopoDS_Vertex"]
        type TopodsVertex;

        /// Constructs a vertex from coordinates.  `to_ffi()` on `OcPnt` is
        /// not used here — the gp_Pnt is constructed inside the shim so the
        /// entire build is stack-allocated.
        fn make_vertex(x: f64, y: f64, z: f64) -> UniquePtr<TopodsVertex>;

        /// Copy-constructs a vertex.  Used by `OcVertex`'s `Clone` impl.
        fn clone_vertex(v: &TopodsVertex) -> UniquePtr<TopodsVertex>;

        /// `BRep_Tool::Pnt(v).X()` — reads back one coordinate without
        /// allocating a gp_Pnt on the heap.
        fn vertex_pnt_x(v: &TopodsVertex) -> f64;
        fn vertex_pnt_y(v: &TopodsVertex) -> f64;
        fn vertex_pnt_z(v: &TopodsVertex) -> f64;

        // ── gp_Pnt ──────────────────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/classgp___pnt.html
        #[cxx_name = "gp_Pnt"]
        type GpPnt;

        /// Materialises a `gp_Pnt` for passing to an OCCT API.
        /// Called by `OcPnt::to_ffi()` in the safe layer.
        fn new_gp_pnt_xyz(x: f64, y: f64, z: f64) -> UniquePtr<GpPnt>;

        // ── gp_Vec ──────────────────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/classgp___vec.html
        #[cxx_name = "gp_Vec"]
        type GpVec;

        /// Materialises a `gp_Vec` for passing to an OCCT API.
        /// Called by `OcVec::to_ffi()` in the safe layer.
        fn new_gp_vec_xyz(x: f64, y: f64, z: f64) -> UniquePtr<GpVec>;

        // ── gp_Dir ──────────────────────────────────────────────────────────
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
    }
}
