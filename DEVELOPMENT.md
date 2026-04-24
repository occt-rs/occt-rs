# Development Guide

## IP Hygiene Policy

This document records the sourcing discipline applied to this codebase.
It exists to establish a clear, timestamped record of intent for licensing
and future foundation transfer purposes.

### Why this matters

`occt-rs` is dual-licensed: AGPL-3.0 publicly, and under a separate
proprietary license to the IP owner. The dual-license depends on the IP
owner holding clean, unencumbered rights to the entire codebase. A single
contribution derived from an LGPL source would permanently compromise the
ability to relicense — including any future transfer to a foundation.

### Banned dependencies

AGPL-licensed dependencies are permanently banned, as they would propagate
into the public license and conflict with the proprietary license.

All dependencies must be MIT, Apache-2.0, BSD, or LGPL. LGPL dependencies
(including OCCT itself) must be dynamically linked in all configurations.

### Existing OCCT binding crates

`occt-rs` is AGPL-licensed, with a strong CLA and dual-licencing. Deriving implementotions
from other binding crates must be carried out with strict scrutiny so as to
preserve the licencing hygiene. This includes copying types, method signatures, idioms, or structural
conventions. Not maintaining hygiene here would risk compromising the dual-license. This codebase must be clean of any such derivation.

### Required sourcing discipline

All OCCT binding code must be sourced from:
- OCCT official reference documentation: https://dev.opencascade.org/doc/refman/html/
- OCCT source headers, read directly for type and method signatures
- cxx documentation: https://cxx.rs/

When using AI assistance, prompts must reference OCCT documentation
explicitly. Prompts must not reference existing binding implementations.

Convergent implementations — where this codebase independently arrives at
the same structure as another binding crate — are legally defensible under
the idea/expression dichotomy, provided the sourcing discipline above is
followed consistently and documented.

### Contributions

All contributors must sign the CLA (see `CLA.md`) and include a
`Signed-off-by` line in every commit (`git commit -s`). Both are enforced
as required status checks on all pull requests. There are no exceptions.

A single un-CLA'd contribution permanently compromises relicensing rights.
The CLA bot is a hard gate, not a courtesy reminder.
