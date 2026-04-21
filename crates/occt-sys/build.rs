// build.rs for occt-sys
//
// OCCT discovery order
// --------------------
// 1. pkg-config: probes the libraries we use.  Works automatically when
//    PKG_CONFIG_PATH includes OCCT's pkgconfig dir (set by the dev flake).
// 2. OCCT_DIR fallback: reads the OCCT_DIR env var and emits link flags manually.
//
// Dynamic linking is mandatory — see DEVELOPMENT.md for the licensing rationale.
//
// Toolkits in use
// ---------------
// TKernel    — Standard_Failure and RTTI (used by all OCCT code)
// TKMath     — gp_* geometry primitives
// TKBRep     — TopoDS_* topology types, BRep_Tool
// TKTopAlgo  — BRepBuilderAPI_MakeVertex and the wider BRepBuilderAPI package

use std::path::PathBuf;

// All OCCT toolkits this crate links against.  Order matters for static
// linking (not used here, but kept consistent): dependencies before dependents.
const OCCT_TOOLKITS: &[&str] = &["TKernel", "TKMath", "TKBRep", "TKTopAlgo", "TKPrim"];

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
    // Probe each toolkit individually.  pkg-config emits link flags for each.
    // We use TKMath's include paths as representative (all toolkits share the
    // same header directory in a standard OCCT install).
    let mut include_paths: Option<Vec<PathBuf>> = None;
    for tk in OCCT_TOOLKITS {
        match pkg_config::Config::new()
            .atleast_version("7.6")
            .cargo_metadata(true)
            .probe(tk)
        {
            Ok(lib) if include_paths.is_none() && !lib.include_paths.is_empty() => {
                include_paths = Some(lib.include_paths);
            }
            _ => {}
        }
    }
    if let Some(paths) = include_paths {
        return paths;
    }

    // ── OCCT_DIR fallback ─────────────────────────────────────────────────
    let dir = std::env::var("OCCT_DIR").unwrap_or_else(|_| {
        panic!(
            "OCCT not found.\n\
             Either ensure pkg-config can find the OCCT toolkits \
             (set PKG_CONFIG_PATH to point at OCCT's pkgconfig directory)\n\
             or set OCCT_DIR=/path/to/your/occt/installation."
        )
    });

    println!("cargo:rustc-link-search=native={dir}/lib");
    for tk in OCCT_TOOLKITS {
        println!("cargo:rustc-link-lib=dylib={tk}");
    }

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
