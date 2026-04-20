// build.rs for occt-sys
//
// OCCT discovery order
// --------------------
// 1. pkg-config: probes TKMath (and TKernel as a dependency).
//    Works automatically when PKG_CONFIG_PATH includes OCCT's pkgconfig dir,
//    which the development flake sets up via shellHook.
// 2. OCCT_DIR fallback: if pkg-config fails, reads the OCCT_DIR environment
//    variable and constructs -I / -L flags manually.
//
// Dynamic linking is mandatory — see DEVELOPMENT.md for the licensing rationale.

use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=include/occt_sys.hxx");
    println!("cargo:rerun-if-env-changed=OCCT_DIR");

    let include_paths = discover_occt();

    cxx_build::bridge("src/lib.rs")
        .flag_if_supported("-std=c++17")
        .includes(&include_paths)
        // Our shim header lives here; resolved as `#include "occt_sys.hxx"`.
        .include("include")
        .compile("occt-sys");
}

/// Returns the OCCT header include paths and emits the necessary
/// `cargo:rustc-link-*` directives.
fn discover_occt() -> Vec<PathBuf> {
    // ── pkg-config path ───────────────────────────────────────────────────
    // probe() emits cargo:rustc-link-lib and cargo:rustc-link-search for us.
    let tkmath = pkg_config::Config::new()
        .atleast_version("7.6")
        .cargo_metadata(true)
        .probe("TKMath");

    if let Ok(lib) = tkmath {
        // TKernel is pulled in transitively by TKMath's pkg-config but probe
        // it explicitly so we get its include paths too.
        let _ = pkg_config::Config::new()
            .atleast_version("7.6")
            .cargo_metadata(true)
            .probe("TKernel");

        if !lib.include_paths.is_empty() {
            return lib.include_paths;
        }
        // pkg-config found the library but reported no include paths (some
        // OCCT installs omit Cflags from their .pc files).  Fall through to
        // OCCT_DIR to pick up the header location.
    }

    // ── OCCT_DIR fallback ─────────────────────────────────────────────────
    let dir = std::env::var("OCCT_DIR").unwrap_or_else(|_| {
        panic!(
            "OCCT not found.\n\
             Either ensure pkg-config can find TKMath \
             (set PKG_CONFIG_PATH to point at OCCT's pkgconfig directory)\n\
             or set OCCT_DIR=/path/to/your/occt/installation."
        )
    });

    // Emit link directives for the two core libraries.
    println!("cargo:rustc-link-search=native={dir}/lib");
    println!("cargo:rustc-link-lib=dylib=TKMath");
    println!("cargo:rustc-link-lib=dylib=TKernel");

    // OCCT headers can be under include/opencascade/ (typical upstream
    // install) or directly under include/ (some distro/Nix layouts).
    for candidate in ["include/opencascade", "include"] {
        let p = PathBuf::from(&dir).join(candidate);
        if p.is_dir() {
            return vec![p];
        }
    }

    panic!(
        "OCCT headers not found.\n\
         Looked in:\n  {dir}/include/opencascade\n  {dir}/include\n\
         Verify that OCCT_DIR points to the installation root."
    );
}
