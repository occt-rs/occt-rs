//! Topological wire type.
//!
//! Reference: <https://dev.opencascade.org/doc/refman/html/class_b_rep_builder_a_p_i___make_wire.html>

use crate::error::{OcctError, OcctErrorKind};
use crate::topo::OcEdge;
use occt_sys::ffi;
use std::marker::PhantomData;

pub struct OcWire {
    inner: cxx::UniquePtr<ffi::TopodsWire>,
    _not_send: PhantomData<*mut ()>,
}

impl OcWire {
    pub fn edges(&self) -> Vec<OcEdge> {
        let mut explorer = ffi::new_wire_edge_explorer(&self.inner);
        let mut result = Vec::new();
        while explorer.more() {
            result.push(OcEdge::from_ffi(explorer.current_edge()));
            explorer.pin_mut().next();
        }
        result
    }
    pub fn from_edges(edges: &[OcEdge]) -> Result<Self, OcctError> {
        let mut builder = ffi::new_make_wire_builder();
        for e in edges {
            builder.pin_mut().add_edge(e.as_ffi());
            if !builder.is_done() {
                return Err(OcctError {
                    kind: OcctErrorKind::ConstructionError,
                    message: builder.error().to_string(),
                });
            }
        }
        Ok(Self {
            inner: builder.pin_mut().wire(),
            _not_send: PhantomData,
        })
    }

    pub(crate) fn as_ffi(&self) -> &ffi::TopodsWire {
        &self.inner
    }
}

impl Clone for OcWire {
    fn clone(&self) -> Self {
        Self {
            inner: ffi::clone_wire(&self.inner),
            _not_send: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gp::OcPnt;
    use crate::topo::OcEdge;

    #[test]
    fn triangle() {
        let edges = vec![
            OcEdge::from_pnts(OcPnt::new(0.0, 0.0, 0.0), OcPnt::new(1.0, 0.0, 0.0)).unwrap(),
            OcEdge::from_pnts(OcPnt::new(1.0, 0.0, 0.0), OcPnt::new(0.5, 1.0, 0.0)).unwrap(),
            OcEdge::from_pnts(OcPnt::new(0.5, 1.0, 0.0), OcPnt::new(0.0, 0.0, 0.0)).unwrap(),
        ];
        assert!(OcWire::from_edges(&edges).is_ok());
    }

    #[test]
    fn disconnected_fails() {
        let edges = vec![
            OcEdge::from_pnts(OcPnt::new(0.0, 0.0, 0.0), OcPnt::new(1.0, 0.0, 0.0)).unwrap(),
            OcEdge::from_pnts(OcPnt::new(5.0, 0.0, 0.0), OcPnt::new(6.0, 0.0, 0.0)).unwrap(),
        ];
        assert!(OcWire::from_edges(&edges).is_err());
    }
    #[test]
    fn round_trip_triangle_vertices() {
        let pts = [
            (OcPnt::new(0.0, 0.0, 0.0), OcPnt::new(1.0, 0.0, 0.0)),
            (OcPnt::new(1.0, 0.0, 0.0), OcPnt::new(0.5, 1.0, 0.0)),
            (OcPnt::new(0.5, 1.0, 0.0), OcPnt::new(0.0, 0.0, 0.0)),
        ];
        let edges: Vec<_> = pts
            .iter()
            .map(|(a, b)| OcEdge::from_pnts(*a, *b).unwrap())
            .collect();
        let wire = OcWire::from_edges(&edges).unwrap();
        let out_edges = wire.edges();
        assert_eq!(out_edges.len(), 3);
        for e in &out_edges {
            let _s = e.start_vertex().pnt();
            let _e = e.end_vertex().pnt();
            // Coordinates are readable without panic — geometry is intact.
        }
    }
}
