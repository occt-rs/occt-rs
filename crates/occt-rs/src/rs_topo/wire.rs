//! Topological wire type.
//!
//! Reference: <https://dev.opencascade.org/doc/refman/html/class_b_rep_builder_a_p_i___make_wire.html>

use crate::error::{OcctError, OcctErrorKind};
use crate::rs_topo::{OcEdge, OcShape};
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
            // Safety: BRepTools_WireExplorer::Current() yields a valid edge
            // while More() is true; current_edge() wraps it in make_unique — non-null.
            result.push(unsafe { OcEdge::from_ffi_unchecked(explorer.current_edge()) });

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
        // Safety: MakeWireBuilder::wire() returns make_unique<TopoDS_Wire>
        // on a completed (IsDone()) builder — non-null.
        Ok(unsafe { Self::from_ffi_unchecked(builder.pin_mut().wire()) })
    }

    /// Widens this wire to a general [`OcShape`] for use with shape-level
    /// APIs such as tessellation.
    ///
    /// The conversion is a cheap TShape handle reference-count increment;
    /// no geometry is copied.
    pub fn as_shape(&self) -> OcShape {
        // Safety: wire_as_shape is a zero-cost upcast; clone_shape is make_unique — non-null.
        unsafe { OcShape::from_ffi_unchecked(ffi::clone_shape(ffi::wire_as_shape(&self.inner))) }
    }

    pub(crate) fn as_ffi(&self) -> &ffi::TopodsWire {
        &self.inner
    }

    /// Returns `None` if `inner` is a null `UniquePtr` or the `TopoDS_Wire`
    /// has a null TShape handle (`IsNull()`).
    pub(crate) fn from_ffi(inner: cxx::UniquePtr<ffi::TopodsWire>) -> Option<Self> {
        if inner.is_null() {
            return None;
        }
        if ffi::topods_wire_is_null(inner.as_ref().unwrap()) {
            return None;
        }
        Some(Self {
            inner,
            _not_send: PhantomData,
        })
    }

    /// Constructs an `OcWire` without null checks.
    ///
    /// # Safety
    ///
    /// Caller guarantees that `inner` is a non-null `UniquePtr` wrapping a
    /// `TopoDS_Wire` whose TShape handle is non-null (`!IsNull()`).
    pub(crate) unsafe fn from_ffi_unchecked(inner: cxx::UniquePtr<ffi::TopodsWire>) -> Self {
        Self {
            inner,
            _not_send: PhantomData,
        }
    }
}

impl Clone for OcWire {
    fn clone(&self) -> Self {
        // Safety: clone_wire is make_unique<TopoDS_Wire>(w) — non-null.
        unsafe { Self::from_ffi_unchecked(ffi::clone_wire(&self.inner)) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gp::OcPnt;

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
        assert_eq!(wire.edges().len(), 3);
    }

    #[test]
    fn as_shape_widens() {
        let edges = vec![
            OcEdge::from_pnts(OcPnt::new(0.0, 0.0, 0.0), OcPnt::new(1.0, 0.0, 0.0)).unwrap(),
            OcEdge::from_pnts(OcPnt::new(1.0, 0.0, 0.0), OcPnt::new(0.5, 1.0, 0.0)).unwrap(),
            OcEdge::from_pnts(OcPnt::new(0.5, 1.0, 0.0), OcPnt::new(0.0, 0.0, 0.0)).unwrap(),
        ];
        let wire = OcWire::from_edges(&edges).unwrap();
        let _shape = wire.as_shape();
    }
}
