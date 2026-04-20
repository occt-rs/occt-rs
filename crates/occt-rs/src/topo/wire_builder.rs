//! Key-based wire builder.
//!
//! Constructs an `OcWire` from a sequence of edges specified as key pairs.
//! Vertices are created once per key and reused, guaranteeing topological
//! connectivity at shared keys.

use crate::error::OcctError;
use crate::gp::OcPnt;
use crate::topo::{OcEdge, OcVertex, OcWire};
use std::collections::HashMap;
use std::hash::Hash;

/// Builds a wire from edges specified as `(start_key, end_key, start_pnt, end_pnt)` tuples.
///
/// A single `OcVertex` is created per unique key.  Edges sharing a key share
/// the same vertex handle, giving OCCT direct topological connectivity.
pub struct KeyedWireBuilder<K> {
    vertices: HashMap<K, OcVertex>,
    edges: Vec<OcEdge>,
}

impl<K: Eq + Hash> KeyedWireBuilder<K> {
    pub fn new() -> Self {
        Self {
            vertices: HashMap::new(),
            edges: Vec::new(),
        }
    }

    /// Adds an edge between two keyed vertices.
    ///
    /// If a vertex for a key already exists its point is ignored — the
    /// existing vertex is reused.  Returns `Err` if the two keys resolve
    /// to coincident vertices.
    pub fn add_edge(
        &mut self,
        start_key: K,
        start_pnt: OcPnt,
        end_key: K,
        end_pnt: OcPnt,
    ) -> Result<(), OcctError> {
        let v1 = self
            .vertices
            .entry(start_key)
            .or_insert_with(|| OcVertex::from_pnt(&start_pnt))
            .clone();
        let v2 = self
            .vertices
            .entry(end_key)
            .or_insert_with(|| OcVertex::from_pnt(&end_pnt))
            .clone();
        let edge = OcEdge::from_vertices(&v1, &v2)?;
        self.edges.push(edge);
        Ok(())
    }

    /// Consumes the builder and produces an `OcWire`.
    pub fn build(self) -> Result<OcWire, OcctError> {
        OcWire::from_edges(&self.edges)
    }
}

impl<K: Eq + Hash> Default for KeyedWireBuilder<K> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triangle_by_key() {
        let mut b = KeyedWireBuilder::new();
        b.add_edge(
            0u32,
            OcPnt::new(0.0, 0.0, 0.0),
            1,
            OcPnt::new(1.0, 0.0, 0.0),
        )
        .unwrap();
        b.add_edge(
            1u32,
            OcPnt::new(1.0, 0.0, 0.0),
            2,
            OcPnt::new(0.5, 1.0, 0.0),
        )
        .unwrap();
        b.add_edge(
            2u32,
            OcPnt::new(0.5, 1.0, 0.0),
            0,
            OcPnt::new(0.0, 0.0, 0.0),
        )
        .unwrap();
        assert!(b.build().is_ok());
    }

    #[test]
    fn shared_vertex_reused() {
        // Key 1 appears as end of first edge and start of second.
        // The point supplied the second time is different but should be ignored.
        let mut b = KeyedWireBuilder::new();
        b.add_edge(
            0u32,
            OcPnt::new(0.0, 0.0, 0.0),
            1,
            OcPnt::new(1.0, 0.0, 0.0),
        )
        .unwrap();
        b.add_edge(
            1u32,
            OcPnt::new(99.0, 99.0, 99.0),
            2,
            OcPnt::new(0.5, 1.0, 0.0),
        )
        .unwrap();
        b.add_edge(
            2u32,
            OcPnt::new(0.5, 1.0, 0.0),
            0,
            OcPnt::new(0.0, 0.0, 0.0),
        )
        .unwrap();
        assert!(b.build().is_ok());
    }
}
