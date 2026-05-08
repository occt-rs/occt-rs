# occt-rs

Rust bindings for [OpenCASCADE Technology (OCCT)](https://dev.opencascade.org/), a geometric modelling kernel.

This is not an official project, or crate. This is an indipendant project.

Any concerns held by OpenCASCADE that can be shared publicly should be raised as an issue on Github. If you prefer to discuss privately, you can reach the project owner in the email provided in the root Cargo.toml


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


## A not on use of AI

Generally speaking, I consider AI, and its temptation to be overused, as problematic: It incurs cognitive debt, detaches the engineer from the code they are supposed to be resposnsible for, as well as a myriad of non-technical (politics and values) of issues.

With that said, binding crates to sys libraries written in CPP is perhaps one of the use cases where the utility of LLMs is particularly suited. Relatively heavy on the "boilerplate", formulaic, and not much need for engineering creativity (though it does come up at the safe-wrapper level).

I never let AI touch my code base directly. That will probably always be a hard-requirement for both myself, and those contributing to code-bases I'm responsible for. A good rule of thumb: Before AI there was stack-overflow. Bad engineers copy-pasted from there without thought. Good engineers would go there to research how similar problems to their own have come up, how they were addressed, and would learn from that. Maybe they would even use the code from SO in their own project. Don't use AI to solve the problem for you; use it to help you be a better engineer.

If you wish to contribute to this code base, make sure you are explicit about the degree to which you use AI. 


You will see a lot of AI signs in this code base, em-dashes, fancy commenting, etc., and it's not something I'm proud of. I won't hide my shame, though. I've accepted its heavy use in this project for reasons stated above.
