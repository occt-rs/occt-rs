//! Topological vertex type.
//!
//! `OcVertex` is the simplest topological entity — a point in 3-D space
//! with an associated tolerance.  It wraps `TopoDS_Vertex` from OCCT.
//!
//! Reference: <https://dev.opencascade.org/doc/refman/html/class_b_rep_builder_a_p_i___make_vertex.html>

use crate::gp::OcPnt;
use crate::topo::OcShape;
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

impl OcVertex {
    pub(crate) fn from_ffi(inner: cxx::UniquePtr<ffi::TopodsVertex>) -> Self {
        Self {
            inner,
            _not_send: PhantomData,
        }
    }

    /// Constructs a vertex at the given point with the default tolerance.
    pub fn from_pnt(p: &OcPnt) -> Self {
        Self {
            inner: ffi::make_vertex(p.x, p.y, p.z),
            _not_send: PhantomData,
        }
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
        OcShape::from_ffi(ffi::clone_shape(ffi::vertex_as_shape(&self.inner)))
    }

    pub(crate) fn as_ffi(&self) -> &ffi::TopodsVertex {
        &self.inner
    }
}

impl Clone for OcVertex {
    fn clone(&self) -> Self {
        Self {
            inner: ffi::clone_vertex(&self.inner),
            _not_send: PhantomData,
        }
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
}
