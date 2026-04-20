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
          # rust-analyzer is included here so all contributors get it via the
          # flake rather than relying on a system or editor-managed install.
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

            # Project cargo tooling
            pkgs.bacon           # background compile/test/check loop
            pkgs.cargo-nextest   # faster test runner; used in CI
            pkgs.cargo-deny      # licence + advisory auditing; enforces IP hygiene policy
            pkgs.clang-tools     # clangd for C++ shim editing (cxx bridge)

            # OpenCASCADE — dynamically linked, as required by LGPL and DEVELOPMENT.md
            opencascade
            # Runtime deps that nixpkgs opencascade-occt links against.
            # We need these on LD_LIBRARY_PATH so test binaries can load TKMath.so.
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
            export PKG_CONFIG_PATH="${opencascade}/lib/pkgconfig:$PKG_CONFIG_PATH"
            export OCCT_DIR="${opencascade}"
            export LD_LIBRARY_PATH="${opencascade}/lib:${pkgs.freetype}/lib:${pkgs.fontconfig.lib}/lib:${pkgs.libX11}/lib:${pkgs.libGL}/lib:$LD_LIBRARY_PATH"
            export CMAKE_POLICY_VERSION_MINIMUM=3.5
            setup-git-signing() {
              local key="''${1:-''$HOME/.ssh/id_ed25519.pub}"
              git config --local gpg.format ssh
              git config --local commit.gpgsign true
              git config --local user.signingkey "$key"
              echo "Commit signing configured (local). Key: $key"
            }

            echo ""
            echo "occt-rs dev shell"
            echo "-----------------"
            echo "  bacon             — background check/test loop"
            echo "  cargo nextest run — run tests"
            echo "  cargo deny check  — audit licences and advisories (IP hygiene)"
            echo ""
            echo "Commit discipline:"
            echo "  setup-git-signing  — configure commit signing for this repo (local only)"
            echo "  git commit -s      — DCO sign-off required on every commit"
            echo "  See DEVELOPMENT.md for IP hygiene policy and signing setup."
            echo ""
          '';
        };
      });
}
