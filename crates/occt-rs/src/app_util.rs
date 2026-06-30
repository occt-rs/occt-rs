//! Quarantine for application-layer logic that does not belong in the binding.
//!
//! Items here compose safe wrappers and add policy/workflow; they cross no FFI
//! boundary and are NOT part of the unopinionated binding surface. They are
//! parked here pending extraction to the application/utility layer. Do not add
//! new items here to "find a home" for them — that decision is the extraction,
//! not this module.

pub mod proximity_wire_builder;
pub mod sketch_polyline_builder;
pub mod wire_builder;

pub use proximity_wire_builder::ProximityWireBuilder;
pub use sketch_polyline_builder::SketchPolylineBuilder;
pub use wire_builder::KeyedWireBuilder;
