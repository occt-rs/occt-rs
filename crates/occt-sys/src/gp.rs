//! cxx bridge for OCCT geometric primitive materialisers.
//!
//! All mathematical operations on `gp_*` types are implemented in pure Rust
//! in `occt-rs` without crossing the FFI boundary.  This bridge exposes
//! only what is needed to materialise values when passing to higher-level
//! OCCT APIs (BRep construction, surface queries, etc.).
//!
//! Sourced from:
//!   - OCCT 7.9 reference: <https://dev.opencascade.org/doc/refman/html/>
//!   - cxx docs: <https://cxx.rs/>
//!
//! No derivation from any other binding crate.

#[allow(clippy::too_many_arguments)]
#[cxx::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("occt_sys/gp.hxx");

        // ── gp_Pnt ───────────────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/classgp___pnt.html
        #[cxx_name = "gp_Pnt"]
        type GpPnt;

        /// Called by `OcPnt::to_ffi()`.
        fn new_gp_pnt_xyz(x: f64, y: f64, z: f64) -> UniquePtr<GpPnt>;

        // ── gp_Vec ───────────────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/classgp___vec.html
        #[cxx_name = "gp_Vec"]
        type GpVec;

        /// Called by `OcVec::to_ffi()`.
        fn new_gp_vec_xyz(x: f64, y: f64, z: f64) -> UniquePtr<GpVec>;

        // ── gp_Dir ───────────────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/classgp___dir.html
        #[cxx_name = "gp_Dir"]
        type GpDir;

        /// Called by `OcDir::to_ffi()`.  `OcDir` pre-validates, so `Err`
        /// here indicates an invariant violation in the Rust layer.
        fn new_gp_dir_xyz(x: f64, y: f64, z: f64) -> Result<UniquePtr<GpDir>>;

        // ── gp_Ax1 ───────────────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/classgp___ax1.html
        #[cxx_name = "gp_Ax1"]
        type GpAx1;

        /// Called by `OcAx1::to_ffi()`.
        fn new_gp_ax1(
            px: f64,
            py: f64,
            pz: f64,
            dx: f64,
            dy: f64,
            dz: f64,
        ) -> Result<UniquePtr<GpAx1>>;

        // ── gp_Ax2 ───────────────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/classgp___ax2.html
        #[cxx_name = "gp_Ax2"]
        type GpAx2;

        /// Called by `OcAx2::to_ffi()`.
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
        // ── GpTrsf ───────────────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/classgp___trsf.html
        //
        // gp_Trsf is held opaquely: its internal form enum is OCCT-managed state
        // that cannot be faithfully represented as a plain Rust struct.
        // Stored as UniquePtr<GpTrsf>; crossed via as_ffi() when passed to
        // higher-level OCCT APIs (BRepBuilderAPI_Transform, etc.).
        type GpTrsf;

        fn new_gp_trsf_identity() -> UniquePtr<GpTrsf>;
        fn clone_gp_trsf(t: &GpTrsf) -> UniquePtr<GpTrsf>;

        fn new_gp_trsf_translation(vx: f64, vy: f64, vz: f64) -> UniquePtr<GpTrsf>;
        fn new_gp_trsf_rotation(
            px: f64,
            py: f64,
            pz: f64,
            dx: f64,
            dy: f64,
            dz: f64,
            angle: f64,
        ) -> Result<UniquePtr<GpTrsf>>;
        fn new_gp_trsf_mirror_point(x: f64, y: f64, z: f64) -> UniquePtr<GpTrsf>;
        fn new_gp_trsf_mirror_axis(
            px: f64,
            py: f64,
            pz: f64,
            dx: f64,
            dy: f64,
            dz: f64,
        ) -> Result<UniquePtr<GpTrsf>>;
        fn new_gp_trsf_mirror_plane(
            px: f64,
            py: f64,
            pz: f64,
            nx: f64,
            ny: f64,
            nz: f64,
            xx: f64,
            xy: f64,
            xz: f64,
        ) -> Result<UniquePtr<GpTrsf>>;
        fn new_gp_trsf_scale(px: f64, py: f64, pz: f64, s: f64) -> UniquePtr<GpTrsf>;

        // Const methods.
        fn value(self: &GpTrsf, row: i32, col: i32) -> f64;
        fn is_negative(self: &GpTrsf) -> bool;
        fn multiplied(self: &GpTrsf, other: &GpTrsf) -> UniquePtr<GpTrsf>;
        fn inverted(self: &GpTrsf) -> Result<UniquePtr<GpTrsf>>;
    }
}
