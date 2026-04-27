// build.rs for occt-sys
//
// OCCT discovery order
// --------------------
// 1. pkg-config: probes the toolkits we use.
// 2. OCCT_DIR fallback: reads the OCCT_DIR env var and emits link flags manually.
//
// Dynamic linking is mandatory — see DEVELOPMENT.md.
//
// Toolkits in use
// ---------------
// TKernel   — Standard_Failure and RTTI, Poly_Triangulation, TopLoc_Location
// TKMath    — gp_* geometry primitives
// TKBRep    — TopoDS_*, BRep_Tool, TopExp_Explorer
// TKTopAlgo — BRepBuilderAPI_Make*
// TKPrim    — BRepPrimAPI_MakePrism
// TKMesh    — BRepMesh_IncrementalMesh

use std::path::PathBuf;

const OCCT_TOOLKITS: &[&str] = &[
    "TKernel",
    "TKMath",
    "TKBRep",
    "TKTopAlgo",
    "TKPrim",
    "TKMesh",
    "TKBO",
    "TKFillet",
    "TKOffset",
    "TKCAF",
    "TKCDF",
    "TKDCAF",
    "TKLCAF",
];

fn main() {
    // Bridge source files.
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/gp.rs");
    println!("cargo:rerun-if-changed=src/topo.rs");
    // C++ headers — one entry per file so incremental rebuilds are precise.
    println!("cargo:rerun-if-changed=include/occt_sys.hxx");
    println!("cargo:rerun-if-changed=include/occt_sys/exception.hxx");
    println!("cargo:rerun-if-changed=include/occt_sys/gp.hxx");
    println!("cargo:rerun-if-changed=include/occt_sys/topo.hxx");
    println!("cargo:rerun-if-changed=include/occt_sys/topo/vertex.hxx");
    println!("cargo:rerun-if-changed=include/occt_sys/topo/edge.hxx");
    println!("cargo:rerun-if-changed=include/occt_sys/topo/wire.hxx");
    println!("cargo:rerun-if-changed=include/occt_sys/topo/face.hxx");
    println!("cargo:rerun-if-changed=include/occt_sys/topo/solid.hxx");
    println!("cargo:rerun-if-changed=include/occt_sys/topo/shape.hxx");
    println!("cargo:rerun-if-changed=include/occt_sys/topo/explorer.hxx");
    println!("cargo:rerun-if-changed=include/occt_sys/topo/mesh.hxx");
    println!("cargo:rerun-if-changed=include/occt_sys/ocaf/label.hxx");
    println!("cargo:rerun-if-changed=include/occt_sys/ocaf/document.hxx");
    println!("cargo:rerun-if-changed=include/occt_sys/ocaf/application.hxx");
    println!("cargo:rerun-if-changed=include/occt_sys/ocaf/attributes.hxx");
    // println!("cargo:rerun-if-changed=include/occt_sys/topo/edge_polygon.hxx");
    println!("cargo:rerun-if-env-changed=OCCT_DIR");

    let include_paths = discover_occt();

    cxx_build::bridges(["src/gp.rs", "src/topo.rs"])
        .flag_if_supported("-std=c++17")
        .includes(&include_paths)
        .include("include")
        .compile("occt-sys");
}

fn discover_occt() -> Vec<PathBuf> {
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
