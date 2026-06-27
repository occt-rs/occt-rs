// In history.rs or a new shape_history_iter.rs:

use std::marker::PhantomData;

use occt_sys::sys_topo::ffi;

use crate::rs_topo::OcShape;

pub struct ShapeListIter {
    inner: cxx::UniquePtr<ffi::ShapeListIter>,
    _not_send: PhantomData<*mut ()>,
}

impl ShapeListIter {
    pub(crate) fn new(inner: cxx::UniquePtr<ffi::ShapeListIter>) -> Self {
        Self {
            inner,
            _not_send: PhantomData,
        }
    }
}

impl Iterator for ShapeListIter {
    type Item = OcShape;
    fn next(&mut self) -> Option<Self::Item> {
        if !ffi::shape_list_iter_more(&self.inner) {
            return None;
        }
        // Safety: shape_list_iter_value returns make_unique<TopoDS_Shape>(*cursor)
        // while more() is true — non-null by OCCT contract.
        let shape = unsafe { OcShape::from_ffi_unchecked(ffi::shape_list_iter_value(&self.inner)) };
        ffi::shape_list_iter_next(self.inner.pin_mut());
        Some(shape)
    }
}
