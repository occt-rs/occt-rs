//! # occt-rs
//!
//! Safe Rust bindings for OpenCASCADE Technology (OCCT) 7.9.3.
//!
//! ## Access convention
//!
//! Types are accessed by module path. There are no crate-root re-exports of
//! primary types; import from the relevant submodule:
//!
//! - `occt_rs::gp` — geometry primitives (`OcPnt`, `OcDir`, `OcVec`, …)
//! - `occt_rs::rs_topo` — BRep topology (`OcShape`, `OcEdge`, `OcFace`, …) and operation builders
//! - `occt_rs::ocaf` — Application framework and utilitities
//! - `occt_rs::tessellate` — mesh tessellation (`compute`)
//! - `occt_rs::error` — error types (`OcctError`, `OcctErrorKind`, …)

pub mod app_util;
pub mod error;
pub mod gp;
pub mod ocaf;
pub mod rs_topo;
pub mod tessellate;
