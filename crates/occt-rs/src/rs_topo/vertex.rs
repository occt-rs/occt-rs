//! Topological vertex type.
//!
//! `OcVertex` is the simplest topological entity — a point in 3-D space
//! with an associated tolerance.  It wraps `TopoDS_Vertex` from OCCT.
//!
//! Reference: <https://dev.opencascade.org/doc/refman/html/class_b_rep_builder_a_p_i___make_vertex.html>

use crate::error::{OcctError, OcctErrorKind};
use crate::gp::OcPnt;
use crate::rs_topo::OcShape;
use occt_sys::ffi;
use std::marker::PhantomData;

/// A BRep topological vertex.
///
/// Wraps `TopoDS_Vertex`.  Internally reference-counted by OCCT, so `Clone`
/// is cheap — it shares the underlying `TShape` handle.
///
/// # Thread safety
///
/// OCCT's `Handle` reference-counting is not atomic.  `OcVertex` must not
/// be sent across thread boundaries.
pub struct OcVertex {
    inner: cxx::UniquePtr<ffi::TopodsVertex>,
    _not_send: PhantomData<*mut ()>,
}

impl std::fmt::Debug for OcVertex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OcVertex")
            .field("pnt", &self.pnt())
            .finish_non_exhaustive()
    }
}

impl OcVertex {
    /// Returns `None` if `inner` is a null `UniquePtr` or the `TopoDS_Vertex`
    /// has a null TShape handle (`IsNull()`).
    #[allow(dead_code)]
    pub(crate) fn from_ffi(inner: cxx::UniquePtr<ffi::TopodsVertex>) -> Option<Self> {
        if inner.is_null() {
            return None;
        }
        if ffi::topods_vertex_is_null(inner.as_ref().unwrap()) {
            return None;
        }
        Some(Self {
            inner,
            _not_send: PhantomData,
        })
    }

    /// Constructs an `OcVertex` without null checks.
    ///
    /// # Safety
    ///
    /// Caller guarantees that `inner` is a non-null `UniquePtr` wrapping a
    /// `TopoDS_Vertex` whose TShape handle is non-null (`!IsNull()`).
    pub(crate) unsafe fn from_ffi_unchecked(inner: cxx::UniquePtr<ffi::TopodsVertex>) -> Self {
        Self {
            inner,
            _not_send: PhantomData,
        }
    }

    /// Constructs a vertex at the given point with the default tolerance.
    pub fn from_pnt(p: &OcPnt) -> Self {
        // Safety: BRepBuilderAPI_MakeVertex always produces a valid vertex;
        // make_vertex shim wraps it in make_unique — non-null.
        unsafe { Self::from_ffi_unchecked(ffi::make_vertex(p.x, p.y, p.z)) }
    }

    /// Returns the 3-D point stored in this vertex.
    pub fn pnt(&self) -> OcPnt {
        OcPnt {
            x: ffi::vertex_pnt_x(&self.inner),
            y: ffi::vertex_pnt_y(&self.inner),
            z: ffi::vertex_pnt_z(&self.inner),
        }
    }

    /// Widens this vertex to a general [`OcShape`] for use with shape-level
    /// APIs such as tessellation.
    ///
    /// The conversion is a cheap TShape handle reference-count increment;
    /// no geometry is copied.
    pub fn as_shape(&self) -> OcShape {
        // Safety: vertex_as_shape is a zero-cost upcast; clone_shape is make_unique — non-null.
        unsafe { OcShape::from_ffi_unchecked(ffi::clone_shape(ffi::vertex_as_shape(&self.inner))) }
    }

    pub(crate) fn as_ffi(&self) -> &ffi::TopodsVertex {
        &self.inner
    }
}

impl Clone for OcVertex {
    fn clone(&self) -> Self {
        // Safety: clone_vertex is make_unique<TopoDS_Vertex>(v) — non-null.
        unsafe { Self::from_ffi_unchecked(ffi::clone_vertex(&self.inner)) }
    }
}

impl TryFrom<&OcShape> for OcVertex {
    type Error = OcctError;

    /// Shape -> Vertex downcast
    ///
    /// Calls `TopoDS::Vertex(const TopoDS_Shape&)`.
    /// Reference: <https://dev.opencascade.org/doc/refman/html/class_topo_d_s.html>
    ///
    /// Fails with `DomainError` if `shape` is not actually a vertex.
    fn try_from(shape: &OcShape) -> Result<Self, Self::Error> {
        // `shape_type` is needed before the actual FFI call because the underlying CPP api throws
        // an exception if the wrong object is provided. We avoid exceptions by guaranteeing at the
        // setting up a pre-condition at the first FFI boundary crossing that precludes a CPP
        // exception being thrown
        let actual = shape.shape_type();
        if actual != crate::rs_topo::ShapeType::Vertex {
            return Err(OcctError {
                kind: OcctErrorKind::DomainError,
                message: format!("expected TopAbs_VERTEX, found {actual:?}"),
            });
        }
        // Safety: shape_type() confirmed TopAbs_VERTEX above, so
        // shape_as_vertex's precondition holds and TopoDS::Vertex cannot
        // throw here. shape_as_vertex wraps the result in
        // make_unique<TopoDS_Vertex> — non-null.
        Ok(unsafe { Self::from_ffi_unchecked(ffi::shape_as_vertex(shape.as_ffi())) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_origin() {
        let p = OcPnt::new(0.0, 0.0, 0.0);
        let v = OcVertex::from_pnt(&p);
        assert_eq!(p, v.pnt());
    }

    #[test]
    fn round_trip_arbitrary() {
        let p = OcPnt::new(1.5, -2.25, 7.0);
        let v = OcVertex::from_pnt(&p);
        assert_eq!(p, v.pnt());
    }

    #[test]
    fn clone_shares_data() {
        let p = OcPnt::new(3.0, 4.0, 5.0);
        let v1 = OcVertex::from_pnt(&p);
        let v2 = v1.clone();
        assert_eq!(v1.pnt(), v2.pnt());
    }

    #[test]
    fn as_shape_widens() {
        let p = OcPnt::new(1.0, 2.0, 3.0);
        let v = OcVertex::from_pnt(&p);
        let _shape = v.as_shape();
    }

    #[test]
    fn try_from_matching_type_succeeds() {
        let v = OcVertex::from_pnt(&OcPnt::new(1.0, 2.0, 3.0));
        let shape = v.as_shape();
        assert!(OcVertex::try_from(&shape).is_ok());
    }

    #[test]
    fn try_from_mismatched_type_fails() {
        let e =
            crate::rs_topo::OcEdge::from_pnts(OcPnt::new(0.0, 0.0, 0.0), OcPnt::new(1.0, 0.0, 0.0))
                .unwrap();
        let shape = e.as_shape();
        let err = OcVertex::try_from(&shape).unwrap_err();
        assert_eq!(err.kind, crate::error::OcctErrorKind::DomainError);
    }
}
