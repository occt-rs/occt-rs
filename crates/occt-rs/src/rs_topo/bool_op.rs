//! Boolean-operation builders with history: Fuse, Cut, Common.
//!
//! Three distinct structs, not one generic type — `BRepAlgoAPI_Fuse`, `_Cut`,
//! and `_Common` share the SetArguments/SetTools/Build shape but have no
//! common instantiation point above the abstract `BRepAlgoAPI_BuilderAlgo`.
//!
//! For a one-call convenience with no history, use [`OcShape::oc_fuse`],
//! [`OcShape::oc_cut`], [`OcShape::oc_common`].
//!
//! [`OcShape::oc_fuse`]: crate::rs_topo::OcShape::oc_fuse
//! [`OcShape::oc_cut`]: crate::rs_topo::OcShape::oc_cut
//! [`OcShape::oc_common`]: crate::rs_topo::OcShape::oc_common

use std::marker::PhantomData;

use occt_sys::ffi;

use crate::error::{OcctError, OcctErrorKind};
use crate::rs_topo::shape_history_iter::ShapeListIter;
use crate::rs_topo::{BuiltWithHistory, HistoryProvider, OcShape};

/// Builder for `BRepAlgoAPI_Fuse` (binary union), preserving history.
pub struct FuseBuilder {
    inner: cxx::UniquePtr<ffi::MakeFuseBuilder>,
    _not_send: PhantomData<*mut ()>,
}

impl FuseBuilder {
    pub fn new() -> Self {
        Self {
            inner: ffi::new_make_fuse_builder(),
            _not_send: PhantomData,
        }
    }

    /// Fuses `s1` with `s2`. Consumes `self`.
    ///
    /// One build per instance is a load-bearing constraint, not an
    /// incidental consequence of `self` being consumed here: `myShape` on
    /// the C++ side is only reassigned on `BuildResult`'s success path and is
    /// never cleared on a subsequent failed `Build()` on the same instance.
    /// A hypothetical retry/rebuild path reusing one `BRepAlgoAPI_Fuse`
    /// instance would silently return the *first* call's result after a
    /// later failure. Do not add such a path without re-deriving this.
    pub fn build(mut self, s1: &OcShape, s2: &OcShape) -> Result<OcShape, OcctError> {
        self.try_build(s1, s2)
    }

    /// Fuses `s1` with `s2`, keeping the builder alive for shape history
    /// queries via [`BuiltWithHistory`]. One build per instance — see
    /// [`build`](Self::build).
    pub fn build_with_history(
        mut self,
        s1: &OcShape,
        s2: &OcShape,
    ) -> Result<BuiltWithHistory<Self>, OcctError> {
        let shape = self.try_build(s1, s2)?;
        Ok(BuiltWithHistory::new(self, shape))
    }

    fn try_build(&mut self, s1: &OcShape, s2: &OcShape) -> Result<OcShape, OcctError> {
        self.inner
            .pin_mut()
            .build(s1.as_ffi(), s2.as_ffi())
            .map_err(OcctError::from)?;
        // HasErrors() is BRepAlgoAPI_BuilderAlgo's own documented failure
        // signal (BOPAlgo_Options: "Error means that the algorithm has
        // failed") and is checked first for that reason. IsDone() is checked
        // too — Build() only calls Done() on the success path, so the two
        // are correlated on the path confirmed in BRepAlgoAPI_BuilderAlgo's
        // own Build()/BuildResult(), but Fuse's own .cxx hasn't been
        // individually confirmed to wire them together the same way, so both
        // are checked and reported distinctly rather than collapsed into one
        // generic message.
        if self.inner.has_errors() {
            return Err(OcctError {
                kind: OcctErrorKind::ConstructionError,
                message:
                    "BRepAlgoAPI_Fuse: HasErrors() true after Build() — boolean operation failed"
                        .to_owned(),
            });
        }
        if !self.inner.is_done() {
            return Err(OcctError {
                kind: OcctErrorKind::ConstructionError,
                message: "BRepAlgoAPI_Fuse: IsDone() false after Build() with HasErrors() false — unexpected incomplete state".to_owned(),
            });
        }
        // Safety: MakeFuseBuilder::shape() returns make_unique<TopoDS_Shape>
        // on a completed builder — non-null.
        Ok(unsafe { OcShape::from_ffi_unchecked(self.inner.pin_mut().shape()) })
    }
}

impl Default for FuseBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl HistoryProvider for FuseBuilder {
    fn modified_shapes(&mut self, input: &OcShape) -> impl Iterator<Item = OcShape> + '_ {
        ShapeListIter::new(ffi::fuse_modified_iter(
            self.inner.pin_mut(),
            input.as_ffi(),
        ))
    }
    fn generated_shapes(&mut self, input: &OcShape) -> impl Iterator<Item = OcShape> + '_ {
        ShapeListIter::new(ffi::fuse_generated_iter(
            self.inner.pin_mut(),
            input.as_ffi(),
        ))
    }
    fn is_shape_deleted(&mut self, input: &OcShape) -> bool {
        self.inner.pin_mut().is_deleted(input.as_ffi())
    }
}

/// Builder for `BRepAlgoAPI_Cut` (binary subtraction), preserving history.
///
/// `s1` is the "object" (left operand); `s2` is subtracted from it.
pub struct CutBuilder {
    inner: cxx::UniquePtr<ffi::MakeCutBuilder>,
    _not_send: PhantomData<*mut ()>,
}

impl CutBuilder {
    pub fn new() -> Self {
        Self {
            inner: ffi::new_make_cut_builder(),
            _not_send: PhantomData,
        }
    }

    /// Subtracts `s2` from `s1`. Consumes `self`.
    ///
    /// One build per instance — load-bearing, not incidental. See
    /// [`FuseBuilder::build`] for the full rationale (identical
    /// `BRepAlgoAPI_BuilderAlgo` lineage: `myShape` is never cleared on a
    /// subsequent failed `Build()` on the same instance).
    pub fn build(mut self, s1: &OcShape, s2: &OcShape) -> Result<OcShape, OcctError> {
        self.try_build(s1, s2)
    }

    /// One build per instance — see [`build`](Self::build).
    pub fn build_with_history(
        mut self,
        s1: &OcShape,
        s2: &OcShape,
    ) -> Result<BuiltWithHistory<Self>, OcctError> {
        let shape = self.try_build(s1, s2)?;
        Ok(BuiltWithHistory::new(self, shape))
    }

    fn try_build(&mut self, s1: &OcShape, s2: &OcShape) -> Result<OcShape, OcctError> {
        self.inner
            .pin_mut()
            .build(s1.as_ffi(), s2.as_ffi())
            .map_err(OcctError::from)?;
        // See FuseBuilder::try_build for why HasErrors() is checked first
        // and reported distinctly from IsDone().
        if self.inner.has_errors() {
            return Err(OcctError {
                kind: OcctErrorKind::ConstructionError,
                message:
                    "BRepAlgoAPI_Cut: HasErrors() true after Build() — boolean operation failed"
                        .to_owned(),
            });
        }
        if !self.inner.is_done() {
            return Err(OcctError {
                kind: OcctErrorKind::ConstructionError,
                message: "BRepAlgoAPI_Cut: IsDone() false after Build() with HasErrors() false — unexpected incomplete state".to_owned(),
            });
        }
        // Safety: MakeCutBuilder::shape() returns make_unique<TopoDS_Shape>
        // on a completed builder — non-null.
        Ok(unsafe { OcShape::from_ffi_unchecked(self.inner.pin_mut().shape()) })
    }
}

impl Default for CutBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl HistoryProvider for CutBuilder {
    fn modified_shapes(&mut self, input: &OcShape) -> impl Iterator<Item = OcShape> + '_ {
        ShapeListIter::new(ffi::cut_modified_iter(self.inner.pin_mut(), input.as_ffi()))
    }
    fn generated_shapes(&mut self, input: &OcShape) -> impl Iterator<Item = OcShape> + '_ {
        ShapeListIter::new(ffi::cut_generated_iter(
            self.inner.pin_mut(),
            input.as_ffi(),
        ))
    }
    fn is_shape_deleted(&mut self, input: &OcShape) -> bool {
        self.inner.pin_mut().is_deleted(input.as_ffi())
    }
}

/// Builder for `BRepAlgoAPI_Common` (binary intersection), preserving history.
pub struct CommonBuilder {
    inner: cxx::UniquePtr<ffi::MakeCommonBuilder>,
    _not_send: PhantomData<*mut ()>,
}

impl CommonBuilder {
    pub fn new() -> Self {
        Self {
            inner: ffi::new_make_common_builder(),
            _not_send: PhantomData,
        }
    }

    /// Intersects `s1` with `s2`. Consumes `self`.
    ///
    /// One build per instance — load-bearing, not incidental. See
    /// [`FuseBuilder::build`] for the full rationale.
    pub fn build(mut self, s1: &OcShape, s2: &OcShape) -> Result<OcShape, OcctError> {
        self.try_build(s1, s2)
    }

    /// One build per instance — see [`build`](Self::build).
    pub fn build_with_history(
        mut self,
        s1: &OcShape,
        s2: &OcShape,
    ) -> Result<BuiltWithHistory<Self>, OcctError> {
        let shape = self.try_build(s1, s2)?;
        Ok(BuiltWithHistory::new(self, shape))
    }

    fn try_build(&mut self, s1: &OcShape, s2: &OcShape) -> Result<OcShape, OcctError> {
        self.inner
            .pin_mut()
            .build(s1.as_ffi(), s2.as_ffi())
            .map_err(OcctError::from)?;
        // See FuseBuilder::try_build for why HasErrors() is checked first
        // and reported distinctly from IsDone().
        if self.inner.has_errors() {
            return Err(OcctError {
                kind: OcctErrorKind::ConstructionError,
                message:
                    "BRepAlgoAPI_Common: HasErrors() true after Build() — boolean operation failed"
                        .to_owned(),
            });
        }
        if !self.inner.is_done() {
            return Err(OcctError {
                kind: OcctErrorKind::ConstructionError,
                message: "BRepAlgoAPI_Common: IsDone() false after Build() with HasErrors() false — unexpected incomplete state".to_owned(),
            });
        }
        // Safety: MakeCommonBuilder::shape() returns make_unique<TopoDS_Shape>
        // on a completed builder — non-null.
        Ok(unsafe { OcShape::from_ffi_unchecked(self.inner.pin_mut().shape()) })
    }
}

impl Default for CommonBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl HistoryProvider for CommonBuilder {
    fn modified_shapes(&mut self, input: &OcShape) -> impl Iterator<Item = OcShape> + '_ {
        ShapeListIter::new(ffi::common_modified_iter(
            self.inner.pin_mut(),
            input.as_ffi(),
        ))
    }
    fn generated_shapes(&mut self, input: &OcShape) -> impl Iterator<Item = OcShape> + '_ {
        ShapeListIter::new(ffi::common_generated_iter(
            self.inner.pin_mut(),
            input.as_ffi(),
        ))
    }
    fn is_shape_deleted(&mut self, input: &OcShape) -> bool {
        self.inner.pin_mut().is_deleted(input.as_ffi())
    }
}
