use crate::rs_topo::{OcEdge, OcFace};
use occt_sys::ffi;
use std::marker::PhantomData;

// ── ShapeFaceIter ─────────────────────────────────────────────────────────────

pub struct ShapeFaceIter {
    inner: cxx::UniquePtr<ffi::ShapeExplorer>,
    _not_send: PhantomData<*mut ()>,
}

impl ShapeFaceIter {
    pub(crate) fn new(inner: cxx::UniquePtr<ffi::ShapeExplorer>) -> Self {
        Self {
            inner,
            _not_send: PhantomData,
        }
    }
}

impl Iterator for ShapeFaceIter {
    type Item = OcFace;
    fn next(&mut self) -> Option<OcFace> {
        if !self.inner.more() {
            return None;
        }
        // Safety: current_owned returns make_unique<TopoDS_Shape>(inner.Current())
        // while more() is true; shape_as_face downcasts a shape already confirmed
        // to be a face by TopExp_Explorer: non-null by OCCT contract.
        let face = unsafe {
            OcFace::from_ffi_unchecked(ffi::shape_as_face(
                self.inner.current_owned().as_ref().unwrap(),
            ))
        };
        self.inner.pin_mut().next();
        Some(face)
    }
}

// ── ShapeEdgeIter ─────────────────────────────────────────────────────────────

pub struct ShapeEdgeIter {
    inner: cxx::UniquePtr<ffi::ShapeExplorer>,
    _not_send: PhantomData<*mut ()>,
}

impl ShapeEdgeIter {
    pub(crate) fn new(inner: cxx::UniquePtr<ffi::ShapeExplorer>) -> Self {
        Self {
            inner,
            _not_send: PhantomData,
        }
    }
}

impl Iterator for ShapeEdgeIter {
    type Item = OcEdge;
    fn next(&mut self) -> Option<OcEdge> {
        if !self.inner.more() {
            return None;
        }
        // Safety: current_owned returns make_unique<TopoDS_Shape>(inner.Current())
        // while more() is true; shape_as_edge downcasts a shape already confirmed
        // to be an edge by TopExp_Explorer — non-null by OCCT contract.
        let edge = unsafe {
            OcEdge::from_ffi_unchecked(ffi::shape_as_edge(
                self.inner.current_owned().as_ref().unwrap(),
            ))
        };
        self.inner.pin_mut().next();
        Some(edge)
    }
}

// ── WireEdgeIter ──────────────────────────────────────────────────────────────

pub struct WireEdgeIter {
    inner: cxx::UniquePtr<ffi::WireEdgeExplorer>,
    _not_send: PhantomData<*mut ()>,
}

impl WireEdgeIter {
    pub(crate) fn new(inner: cxx::UniquePtr<ffi::WireEdgeExplorer>) -> Self {
        Self {
            inner,
            _not_send: PhantomData,
        }
    }
}

impl Iterator for WireEdgeIter {
    type Item = OcEdge;
    fn next(&mut self) -> Option<OcEdge> {
        if !self.inner.more() {
            return None;
        }
        // Safety: current_edge returns make_unique<TopoDS_Edge> while more() is
        // true — non-null by BRepTools_WireExplorer contract.
        let edge = unsafe { OcEdge::from_ffi_unchecked(self.inner.current_edge()) };
        self.inner.pin_mut().next();
        Some(edge)
    }
}
