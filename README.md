# occt-rs

Rust bindings for [OpenCASCADE Technology (OCCT)](https://dev.opencascade.org/), a geometric modelling kernel.

> **Status:** Early development. No stable API yet.

## Building

OCCT must be installed and dynamically linked. Discovery is attempted via `pkg-config` first, then the `OCCT_DIR` environment variable:

```sh
OCCT_DIR=/opt/opencascade-7.8.0 cargo build
```

Static linking is not supported and will not be added — see [Licensing](#licensing).

## Licensing

This crate is licensed under **AGPL-3.0** for public use.

The CLA (see `CLA.md`) includes a proprietary relicensing clause. The practical effect is that the copyright holder - currently the project maintainer - can use this crate in proprietary software they distribute. The motivation is to keep the project's IP profile clean for a future transfer to a foundation, not to enable proprietary capture of the parametric modelling tooling ecosystem. If you don't trust the copyright holder to stay true to the spirit of FOSS, raise a GitHub issue and we can discuss.

OCCT itself is [LGPL 2.1](https://dev.opencascade.org/doc/overview/html/intro_license.html), which is why dynamic linking is required in all configurations — end users must be able to relink against a modified version of OCCT.

## Contributing

All contributors must:

1. Sign the CLA — the CLA Assistant bot will prompt you when you open a pull request.
2. Sign off every commit: `git commit -s`

Both are enforced as required status checks. See `DEVELOPMENT.md` for the IP hygiene policy that governs sourcing decisions in this codebase.
