//! Raw cxx bridge for OpenCASCADE Technology (OCCT).
//!
//! Two bridge modules mirror the `occt-rs` structure:
//!   - [`gp`]   — geometric primitive materialisers (`gp_Pnt`, `gp_Vec`, …)
//!   - [`topo`] — topological shape builders and inspectors
//!
//! The unified [`ffi`] re-export lets `occt-rs` continue using
//! `use occt_sys::ffi;` without change.
//!
//! Sourced from OCCT 7.9 documentation and cxx docs.
//! No derivation from any other binding crate.
//! See DEVELOPMENT.md for the IP hygiene policy.

pub mod gp;
pub mod topo;

/// Unified FFI namespace.
///
/// Re-exports every item from `gp::ffi` and `topo::ffi`.  The two modules
/// have no overlapping names, so glob re-export is unambiguous.
pub mod ffi {
    pub use crate::gp::ffi::*;
    pub use crate::topo::ffi::*;
}
