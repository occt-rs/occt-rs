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
