//! Topological vertex type.
//!
//! `OcVertex` is the simplest topological entity — a point in 3-D space
//! with an associated tolerance.  It wraps `TopoDS_Vertex` from OCCT.
//!
//! Reference: <https://dev.opencascade.org/doc/refman/html/class_b_rep_builder_a_p_i___make_vertex.html>

use crate::gp::OcPnt;
use occt_sys::ffi;
use std::marker::PhantomData;

/// A BRep topological vertex.
///
/// Wraps `TopoDS_Vertex`.  Internally reference-counted by OCCT, so `Clone`
/// is cheap — it shares the underlying `TShape` handle.
///
/// Corresponds to the result of `BRepBuilderAPI_MakeVertex(gp_Pnt)`.
///
/// # Thread safety
///
/// OCCT's `Handle` reference-counting is not atomic.  `OcVertex` must not
/// be sent across thread boundaries.
pub struct OcVertex {
    inner: cxx::UniquePtr<ffi::TopodsVertex>,
    /// Makes OcVertex !Send + !Sync without nightly negative_impls.
    _not_send: PhantomData<*mut ()>,
}

impl OcVertex {
    /// Constructs a vertex at the given point with the default tolerance
    /// (`Precision::Confusion()`).
    pub fn from_pnt(p: &OcPnt) -> Self {
        Self {
            inner: ffi::make_vertex(p.x, p.y, p.z),
            _not_send: PhantomData,
        }
    }

    /// Returns the 3-D point stored in this vertex.
    ///
    /// Calls `BRep_Tool::Pnt` via three coordinate shims.
    /// No heap allocation; the `gp_Pnt` is stack-allocated inside each shim.
    pub fn pnt(&self) -> OcPnt {
        OcPnt {
            x: ffi::vertex_pnt_x(&self.inner),
            y: ffi::vertex_pnt_y(&self.inner),
            z: ffi::vertex_pnt_z(&self.inner),
        }
    }

    /// Returns a reference to the underlying `TopoDS_Vertex` for use by
    /// other OCCT API bindings within this crate.
    pub(crate) fn as_ffi(&self) -> &ffi::TopodsVertex {
        &self.inner
    }
}

impl Clone for OcVertex {
    /// Clones by copy-constructing the `TopoDS_Vertex` handle.
    /// The underlying `TShape` is shared (ref-counted); this is O(1).
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
        let q = v.pnt();
        assert_eq!(p, q);
    }

    #[test]
    fn round_trip_arbitrary() {
        let p = OcPnt::new(1.5, -2.25, 7.0);
        let v = OcVertex::from_pnt(&p);
        let q = v.pnt();
        // Exact equality is expected: coordinates pass through OCCT without
        // any transformation or rounding.
        assert_eq!(p, q);
    }

    #[test]
    fn clone_shares_data_and_round_trips() {
        let p = OcPnt::new(3.0, 4.0, 5.0);
        let v1 = OcVertex::from_pnt(&p);
        let v2 = v1.clone();
        assert_eq!(v1.pnt(), v2.pnt());
    }

    #[test]
    fn to_ffi_is_exercised() {
        // This is the first test that actually calls OcPnt::to_ffi() indirectly
        // via make_vertex.  If the materialisation path is broken, this fails.
        let p = OcPnt::new(10.0, 20.0, 30.0);
        let v = OcVertex::from_pnt(&p);
        let back = v.pnt();
        assert!((back.x - 10.0).abs() < 1e-15);
        assert!((back.y - 20.0).abs() < 1e-15);
        assert!((back.z - 30.0).abs() < 1e-15);
    }
}
