//! Topological wire type.
//!
//! Reference: <https://dev.opencascade.org/doc/refman/html/class_b_rep_builder_a_p_i___make_wire.html>

use crate::error::{OcctError, OcctErrorKind};
use crate::rs_topo::shape_explorer_iter::WireEdgeIter;
use crate::rs_topo::{OcEdge, OcShape};
use occt_sys::ffi;
use std::marker::PhantomData;

pub struct OcWire {
    inner: cxx::UniquePtr<ffi::TopodsWire>,
    _not_send: PhantomData<*mut ()>,
}

impl std::fmt::Debug for OcWire {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OcWire").finish_non_exhaustive()
    }
}

impl OcWire {
    pub fn edges(&self) -> impl Iterator<Item = OcEdge> {
        WireEdgeIter::new(ffi::new_wire_edge_explorer(&self.inner))
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

impl TryFrom<&OcShape> for OcWire {
    type Error = OcctError;

    /// Shape -> Wire downcast
    ///
    /// Calls `TopoDS::Wire(const TopoDS_Shape&)`.
    /// Reference: <https://dev.opencascade.org/doc/refman/html/class_topo_d_s.html>
    ///
    /// Fails with `DomainError` if `shape` is not actually a wire.
    fn try_from(shape: &OcShape) -> Result<Self, Self::Error> {
        // `shape_type` is needed before the actual FFI call because the underlying CPP api throws
        // an exception if the wrong object is provided. We avoid exceptions by guaranteeing at the
        // setting up a pre-condition at the first FFI boundary crossing that precludes a CPP
        // exception being thrown
        let actual = shape.shape_type();
        if actual != crate::rs_topo::ShapeType::Wire {
            return Err(OcctError {
                kind: OcctErrorKind::DomainError,
                message: format!("expected TopAbs_WIRE, found {actual:?}"),
            });
        }
        // Safety: shape_type() confirmed TopAbs_WIRE above, so shape_as_wire's
        // precondition holds and TopoDS::Wire cannot throw here. shape_as_wire
        // wraps the result in make_unique<TopoDS_Wire> — non-null.
        Ok(unsafe { Self::from_ffi_unchecked(ffi::shape_as_wire(shape.as_ffi())) })
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
        assert_eq!(wire.edges().collect::<Vec<_>>().len(), 3);
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

    #[test]
    fn try_from_matching_type_succeeds() {
        let edges = vec![
            OcEdge::from_pnts(OcPnt::new(0.0, 0.0, 0.0), OcPnt::new(1.0, 0.0, 0.0)).unwrap(),
            OcEdge::from_pnts(OcPnt::new(1.0, 0.0, 0.0), OcPnt::new(0.5, 1.0, 0.0)).unwrap(),
            OcEdge::from_pnts(OcPnt::new(0.5, 1.0, 0.0), OcPnt::new(0.0, 0.0, 0.0)).unwrap(),
        ];
        let wire = OcWire::from_edges(&edges).unwrap();
        let shape = wire.as_shape();
        assert!(OcWire::try_from(&shape).is_ok());
    }

    #[test]
    fn try_from_mismatched_type_fails() {
        let e = OcEdge::from_pnts(OcPnt::new(0.0, 0.0, 0.0), OcPnt::new(1.0, 0.0, 0.0)).unwrap();
        let shape = e.as_shape();
        let err = OcWire::try_from(&shape).unwrap_err();
        assert_eq!(err.kind, crate::error::OcctErrorKind::DomainError);
    }
}
