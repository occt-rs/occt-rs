{
  description = "occt-rs development environment";

  inputs = {
    nixpkgs.url     = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs     = import nixpkgs { inherit system overlays; };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" "clippy" "rustfmt" ];
          # No wasm target — this is a native geometry kernel crate.
        };

        opencascade = pkgs.opencascade-occt;

      in {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            # Rust
            rustToolchain

            # C/C++ toolchain (cxx-build invokes cc)
            pkgs.gcc
            pkgs.gnumake
            pkgs.cmake       # needed by some OCCT install layouts; low cost to keep
            pkgs.pkg-config

            # OpenCASCADE — dynamically linked, as required by LGPL and DEVELOPMENT.md
            opencascade

            # Runtime deps that nixpkgs opencascade-occt links against.
            # We need these on LD_LIBRARY_PATH so test binaries can load TKMath.so.
            # (TKMath itself does not use fonts/GL directly, but TKernel may pull
            # them in transitively depending on the nixpkgs build configuration.)
            pkgs.freetype
            pkgs.fontconfig

            # X11 and OpenGL — opencascade-occt in nixpkgs is built with
            # visualisation support; its shared objects reference these even when
            # we only call into the headless geometry kernel.  Remove if you
            # switch to an OCCT build configured with -DOCCT_NO_OPENGL=ON.
            pkgs.libX11
            pkgs.libGL
          ];

          shellHook = ''
            # Primary discovery mechanism for build.rs (via pkg-config).
            export PKG_CONFIG_PATH="${opencascade}/lib/pkgconfig:$PKG_CONFIG_PATH"

            # Fallback discovery variable documented in README.md.
            export OCCT_DIR="${opencascade}"

            # Allow test binaries and the linker to find OCCT's shared objects.
            export LD_LIBRARY_PATH="${opencascade}/lib:${pkgs.freetype}/lib:${pkgs.fontconfig.lib}/lib:${pkgs.libX11}/lib:${pkgs.libGL}/lib:$LD_LIBRARY_PATH"

            # CMake 4.x compatibility (applies if any transitive build dep uses CMake).
            export CMAKE_POLICY_VERSION_MINIMUM=3.5
          '';
        };
      });
}
