//! Proximity-based wire builder.
//!
//! Constructs an `OcWire` from a sequence of point pairs.  Vertices within
//! `tolerance` of each other are merged into a single `OcVertex` before any
//! FFI call, guaranteeing topological connectivity for near-coincident points.
//!
//! The default tolerance mirrors `Precision::Confusion()` in OCCT (1e-7).
//! Use a wider tolerance when input coordinates may have accumulated
//! floating-point error beyond that threshold.

use crate::error::OcctError;
use crate::gp::OcPnt;
use crate::topo::{OcEdge, OcVertex, OcWire};

/// Default tolerance matching `Precision::Confusion()` in OCCT.
pub const PRECISION_CONFUSION: f64 = 1e-7;

/// Builds a wire from point pairs, merging vertices within a tolerance.
pub struct ProximityWireBuilder {
    tolerance: f64,
    pairs: Vec<(OcPnt, OcPnt)>,
}

impl ProximityWireBuilder {
    pub fn new() -> Self {
        Self {
            tolerance: PRECISION_CONFUSION,
            pairs: Vec::new(),
        }
    }

    pub fn with_tolerance(tolerance: f64) -> Self {
        Self {
            tolerance,
            pairs: Vec::new(),
        }
    }

    pub fn add_edge(&mut self, start: OcPnt, end: OcPnt) {
        self.pairs.push((start, end));
    }

    /// Consumes the builder and produces an `OcWire`.
    ///
    /// Vertex deduplication is performed in Rust before any FFI call.
    /// Points within `tolerance` of a previously seen point reuse that
    /// point's `OcVertex`.
    pub fn build(self) -> Result<OcWire, OcctError> {
        // Collect all endpoints, dedup by proximity.
        let mut unique: Vec<(OcPnt, OcVertex)> = Vec::new();

        let mut resolve = |p: OcPnt| -> OcVertex {
            let tol_sq = self.tolerance * self.tolerance;
            if let Some((_, v)) = unique.iter().find(|(q, _)| p.square_distance(q) <= tol_sq) {
                v.clone()
            } else {
                let v = OcVertex::from_pnt(&p);
                unique.push((p, v.clone()));
                v
            }
        };

        let mut edges = Vec::with_capacity(self.pairs.len());
        for (start, end) in self.pairs {
            let v1 = resolve(start);
            let v2 = resolve(end);
            edges.push(OcEdge::from_vertices(&v1, &v2)?);
        }

        OcWire::from_edges(&edges)
    }
}

impl Default for ProximityWireBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triangle_exact() {
        let mut b = ProximityWireBuilder::new();
        b.add_edge(OcPnt::new(0.0, 0.0, 0.0), OcPnt::new(1.0, 0.0, 0.0));
        b.add_edge(OcPnt::new(1.0, 0.0, 0.0), OcPnt::new(0.5, 1.0, 0.0));
        b.add_edge(OcPnt::new(0.5, 1.0, 0.0), OcPnt::new(0.0, 0.0, 0.0));
        assert!(b.build().is_ok());
    }

    #[test]
    fn triangle_within_tolerance() {
        // Endpoints differ by less than PRECISION_CONFUSION — should merge.
        let eps = PRECISION_CONFUSION * 0.1;
        let mut b = ProximityWireBuilder::new();
        b.add_edge(OcPnt::new(0.0, 0.0, 0.0), OcPnt::new(1.0, 0.0, 0.0));
        b.add_edge(OcPnt::new(1.0 + eps, 0.0, 0.0), OcPnt::new(0.5, 1.0, 0.0));
        b.add_edge(OcPnt::new(0.5, 1.0 + eps, 0.0), OcPnt::new(0.0, 0.0, 0.0));
        assert!(b.build().is_ok());
    }

    #[test]
    fn disconnected_beyond_tolerance_fails() {
        let mut b = ProximityWireBuilder::new();
        b.add_edge(OcPnt::new(0.0, 0.0, 0.0), OcPnt::new(1.0, 0.0, 0.0));
        b.add_edge(OcPnt::new(5.0, 0.0, 0.0), OcPnt::new(6.0, 0.0, 0.0));
        assert!(b.build().is_err());
    }

    #[test]
    fn wide_tolerance_merges_distant_points() {
        let mut b = ProximityWireBuilder::with_tolerance(1.0);
        b.add_edge(OcPnt::new(0.0, 0.0, 0.0), OcPnt::new(2.0, 0.0, 0.0));
        // End of first edge and start of second are 0.5 apart — within tol=1.0.
        b.add_edge(OcPnt::new(2.5, 0.0, 0.0), OcPnt::new(0.5, 0.0, 0.0));
        assert!(b.build().is_ok());
    }
}
