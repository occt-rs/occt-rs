//! OCAF attribute bindings for `TDataXtd_Geometry` and `TDataXtd_Constraint`.
//!
//! These two attributes work together to express sketch geometry and
//! constraints in a `TDF_Data` document:
//!
//! - [`OcGeometryAttr`] — a qualifier tag that labels a `TNaming_NamedShape`
//!   with a geometry kind (point, line, circle, …).  It lives on the **same
//!   label** as the named shape and adds no data of its own beyond the kind
//!   enum.
//!
//! - [`OcConstraintAttr`] — records a constraint between 1–4 geometry
//!   participants.  Each participant is referenced by its
//!   [`TnamingNamedShape`](crate::topo::tnaming::TnamingNamedShape) handle —
//!   the topology record — not by an [`OcGeometryAttr`] handle.  Dimensional
//!   constraints additionally carry an [`OcReal`](crate::topo::attributes::OcReal)
//!   value attached via a sub-label.
//!
//! ## Constraint kind
//!
//! `TDataXtd_ConstraintEnum` is intentionally narrow (24 values covering
//! basic OCAF primitives).  Application-level constraint semantics beyond
//! that set should be managed at the application layer using a GCS of your
//! choice.  This binding exposes the enum as [`ConstraintKind`] but you are
//! free to ignore it and store application constraint kinds via
//! [`OcInteger`](crate::topo::attributes::OcInteger) on a sub-label.
//!
//! ## Sourcing
//!
//! Sourced from OCCT 8.0 documentation (API stable since 7.x).
//! No derivation from any other binding crate.
//!
//! Reference: <https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___geometry.html>
//! Reference: <https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___constraint.html>
//! Reference: <https://dev.opencascade.org/doc/refman/html/group__enum__t_data_xtd.html>

use std::marker::PhantomData;

use occt_sys::ffi;

use crate::error::OcctError;
use crate::topo::attributes::OcReal;
use crate::topo::document::Command;
use crate::topo::label::OcLabel;
use crate::topo::tnaming::TnamingNamedShape;

// ── GeometryKind ─────────────────────────────────────────────────────────────

/// The kind of geometric construction a label represents.
///
/// Maps `TDataXtd_GeometryEnum`.  Attached to a label alongside a
/// [`TnamingNamedShape`] to qualify what kind of geometry the stored shape is.
///
/// Reference: <https://dev.opencascade.org/doc/refman/html/group__enum__t_data_xtd.html>
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GeometryKind {
    Any = 0,
    Point = 1,
    Line = 2,
    Circle = 3,
    Ellipse = 4,
    Spline = 5,
    Plane = 6,
    Cylinder = 7,
}

impl GeometryKind {
    fn from_raw(v: i32) -> Self {
        match v {
            0 => Self::Any,
            1 => Self::Point,
            2 => Self::Line,
            3 => Self::Circle,
            4 => Self::Ellipse,
            5 => Self::Spline,
            6 => Self::Plane,
            7 => Self::Cylinder,
            _ => panic!("unknown TDataXtd_GeometryEnum value: {v}"),
        }
    }
}

// ── OcGeometryAttr ───────────────────────────────────────────────────────────

/// A `TDataXtd_Geometry` attribute handle — tags a label with a geometry kind.
///
/// Attach to the **same label** as a [`TnamingNamedShape`] to declare what
/// kind of geometry the stored shape represents.  The attribute carries no
/// geometry data itself; the shape lives in the named-shape attribute.
///
/// # Usage pattern
///
/// ```ignore
/// // Inside an open command:
/// let mut geom = OcGeometryAttr::set(&cmd, &label)?;
/// geom.set_type(&cmd, GeometryKind::Circle);
/// ```
///
/// `set` and `set_type` are separated to match the OCCT API, where
/// `TDataXtd_Geometry::Set(label)` creates the attribute with `ANY_GEOM` and
/// `SetType(T)` updates it.  Both may be called in the same command.
pub struct OcGeometryAttr {
    inner: cxx::UniquePtr<ffi::TDataXtdGeometryHandle>,
    _not_send: PhantomData<*mut ()>,
}

impl OcGeometryAttr {
    /// Finds or creates a `TDataXtd_Geometry` attribute on `label` with the
    /// given `kind` set atomically.
    ///
    /// The type is applied before the attribute is registered with the label,
    /// so the single `AddAttribute` operation is the complete undo delta.
    /// This means undo cleanly removes the attribute rather than reverting it
    /// to a default state.
    ///
    /// Must be called inside an open [`Command`] scope.
    pub fn set(_cmd: &Command<'_>, label: &OcLabel, kind: GeometryKind) -> Result<Self, OcctError> {
        let inner =
            ffi::tdataxtd_geometry_set(&label.inner, kind as i32).map_err(OcctError::from)?;
        Ok(Self {
            inner,
            _not_send: PhantomData,
        })
    }

    /// Updates the geometry kind on an already-committed attribute.
    ///
    /// Safe only on an attribute that has already been committed in a prior
    /// command — `Backup()` inside OCCT's `SetType` requires a committed state
    /// to snapshot correctly.  For new attributes, pass the kind to [`set`]
    /// directly.
    ///
    /// Must be called inside an open [`Command`] scope.
    ///
    /// [`set`]: Self::set
    pub fn set_type(&mut self, _cmd: &Command<'_>, kind: GeometryKind) {
        ffi::tdataxtd_geometry_set_type(self.inner.pin_mut(), kind as i32);
    }

    /// Reads the geometry kind from this handle.
    pub fn kind(&self) -> GeometryKind {
        GeometryKind::from_raw(ffi::tdataxtd_geometry_get_type(&self.inner))
    }

    /// Probes for a `TDataXtd_Geometry` attribute on `label`.
    ///
    /// Returns `None` when the attribute is not present.
    pub fn find(label: &OcLabel) -> Option<Self> {
        let inner = ffi::tdataxtd_geometry_find(&label.inner);
        if inner.is_null() {
            None
        } else {
            Some(Self {
                inner,
                _not_send: PhantomData,
            })
        }
    }

    /// Removes the `TDataXtd_Geometry` attribute from `label`.
    ///
    /// Returns `false` if the attribute was not present.
    /// Must be called inside an open [`Command`] scope.
    pub fn forget(_cmd: &Command<'_>, label: &OcLabel) -> bool {
        ffi::tdataxtd_geometry_forget(&label.inner)
    }
}

impl std::fmt::Debug for OcGeometryAttr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OcGeometryAttr")
            .field("kind", &self.kind())
            .finish()
    }
}

// ── ConstraintKind ───────────────────────────────────────────────────────────

/// The kind of constraint recorded by a `TDataXtd_Constraint` attribute.
///
/// Maps `TDataXtd_ConstraintEnum`.  This enumeration covers the primitive
/// constraint vocabulary recognised by OCCT's OCAF layer.  Application-level
/// constraint semantics that go beyond this set should be managed at the
/// application layer.
///
/// Reference: <https://dev.opencascade.org/doc/refman/html/group__enum__t_data_xtd.html>
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConstraintKind {
    Radius = 0,
    Diameter = 1,
    MinorRadius = 2,
    MajorRadius = 3,
    Tangent = 4,
    Parallel = 5,
    Perpendicular = 6,
    Concentric = 7,
    Coincident = 8,
    Distance = 9,
    Angle = 10,
    EqualRadius = 11,
    Through = 12,
    Symmetric = 13,
    Midpoint = 14,
    EqualDistance = 15,
    Fix = 16,
    Rigid = 17,
    From = 18,
    Axis = 19,
    Mate = 20,
    FaceDistance = 21,
    Round = 22,
    Offset = 23,
}

impl ConstraintKind {
    fn from_raw(v: i32) -> Self {
        match v {
            0 => Self::Radius,
            1 => Self::Diameter,
            2 => Self::MinorRadius,
            3 => Self::MajorRadius,
            4 => Self::Tangent,
            5 => Self::Parallel,
            6 => Self::Perpendicular,
            7 => Self::Concentric,
            8 => Self::Coincident,
            9 => Self::Distance,
            10 => Self::Angle,
            11 => Self::EqualRadius,
            12 => Self::Through,
            13 => Self::Symmetric,
            14 => Self::Midpoint,
            15 => Self::EqualDistance,
            16 => Self::Fix,
            17 => Self::Rigid,
            18 => Self::From,
            19 => Self::Axis,
            20 => Self::Mate,
            21 => Self::FaceDistance,
            22 => Self::Round,
            23 => Self::Offset,
            _ => panic!("unknown TDataXtd_ConstraintEnum value: {v}"),
        }
    }
}

// ── OcConstraintAttr ─────────────────────────────────────────────────────────

/// A `TDataXtd_Constraint` attribute handle.
///
/// Records a constraint on a label: its kind, references to 1–4
/// [`TnamingNamedShape`] topology attributes, and optionally an associated
/// [`OcReal`] dimension value.
///
/// # Geometry participants
///
/// Each geometry slot holds a `Handle(TNaming_NamedShape)`.  You pass
/// [`TnamingNamedShape`] handles — **not** [`OcGeometryAttr`] handles.
/// `TDataXtd_Geometry` is a qualifier tag on a label; `TDataXtd_Constraint`
/// binds to the shape topology.
///
/// # Dimensional constraints
///
/// For constraints with a numeric value (distance, angle, radius, …):
/// 1. Create an [`OcReal`] on a child label of the constraint label inside an
///    open command.
/// 2. Call [`Self::set_value`] with the resulting handle.
pub struct OcConstraintAttr {
    inner: cxx::UniquePtr<ffi::TDataXtdConstraintHandle>,
    _not_send: PhantomData<*mut ()>,
}

impl OcConstraintAttr {
    /// Attaches a constraint referencing 1–4 [`TnamingNamedShape`] geometry
    /// participants.
    ///
    /// Must be called inside an open [`Command`] scope.
    ///
    /// # Panics
    ///
    /// Panics if `geoms` is empty or has more than 4 entries.
    pub fn set(
        _cmd: &Command<'_>,
        label: &OcLabel,
        kind: ConstraintKind,
        geoms: &[&TnamingNamedShape],
    ) -> Result<Self, OcctError> {
        let inner = match geoms {
            [g1] => ffi::tdataxtd_constraint_set1(&label.inner, kind as i32, g1.inner()),
            [g1, g2] => {
                ffi::tdataxtd_constraint_set2(&label.inner, kind as i32, g1.inner(), g2.inner())
            }
            [g1, g2, g3] => ffi::tdataxtd_constraint_set3(
                &label.inner,
                kind as i32,
                g1.inner(),
                g2.inner(),
                g3.inner(),
            ),
            [g1, g2, g3, g4] => ffi::tdataxtd_constraint_set4(
                &label.inner,
                kind as i32,
                g1.inner(),
                g2.inner(),
                g3.inner(),
                g4.inner(),
            ),
            [] => panic!("OcConstraintAttr::set: geoms must not be empty"),
            _ => panic!("OcConstraintAttr::set: at most 4 geometry references supported"),
        }
        .map_err(OcctError::from)?;
        Ok(Self {
            inner,
            _not_send: PhantomData,
        })
    }

    /// Sets or replaces a geometry reference at `index` (1-based).
    ///
    /// Must be called inside an open [`Command`] scope.
    pub fn set_geometry(&mut self, _cmd: &Command<'_>, index: i32, ns: &TnamingNamedShape) {
        ffi::tdataxtd_constraint_set_geometry(self.inner.pin_mut(), index, ns.inner());
    }

    /// Associates an [`OcReal`] attribute as the dimension value.
    ///
    /// The `OcReal` must already be set on a label inside an open command.
    /// Must be called inside an open [`Command`] scope.
    pub fn set_value(&mut self, _cmd: &Command<'_>, val: &OcReal) {
        ffi::tdataxtd_constraint_set_value(self.inner.pin_mut(), val.inner());
    }

    /// Updates the constraint kind.
    ///
    /// Must be called inside an open [`Command`] scope.
    pub fn set_kind(&mut self, _cmd: &Command<'_>, kind: ConstraintKind) {
        ffi::tdataxtd_constraint_set_type(self.inner.pin_mut(), kind as i32);
    }

    /// The constraint kind.
    pub fn kind(&self) -> ConstraintKind {
        ConstraintKind::from_raw(ffi::tdataxtd_constraint_get_type(&self.inner))
    }

    /// Number of geometry references (1–4).
    pub fn nb_geometries(&self) -> i32 {
        ffi::tdataxtd_constraint_nb_geometries(&self.inner)
    }

    /// Returns the geometry reference at `index` (1-based).
    ///
    /// Returns `None` when `index` is out of range or the slot is null.
    pub fn geometry(&self, index: i32) -> Option<TnamingNamedShape> {
        let inner = ffi::tdataxtd_constraint_get_geometry(&self.inner, index);
        if inner.is_null() {
            None
        } else {
            Some(TnamingNamedShape::from_ffi(inner))
        }
    }

    /// Returns `true` when this constraint has an associated dimension value.
    pub fn is_dimension(&self) -> bool {
        ffi::tdataxtd_constraint_is_dimension(&self.inner)
    }

    /// Returns the associated dimension value attribute, if any.
    ///
    /// Returns `None` when [`Self::is_dimension`] is false.
    pub fn value(&self) -> Option<OcReal> {
        let inner = ffi::tdataxtd_constraint_get_value(&self.inner);
        if inner.is_null() {
            None
        } else {
            Some(OcReal::from_ffi(inner))
        }
    }

    /// Returns `true` when the solver has marked this constraint as satisfied.
    pub fn verified(&self) -> bool {
        ffi::tdataxtd_constraint_verified(&self.inner)
    }

    /// Sets the solver-validity flag.
    ///
    /// Must be called inside an open [`Command`] scope.
    pub fn set_verified(&mut self, _cmd: &Command<'_>, status: bool) {
        ffi::tdataxtd_constraint_set_verified(self.inner.pin_mut(), status);
    }

    /// Returns `true` when this is a 2D (planar) constraint.
    pub fn is_planar(&self) -> bool {
        ffi::tdataxtd_constraint_is_planar(&self.inner)
    }

    /// Returns the plane's named-shape reference for a 2D constraint.
    ///
    /// Returns `None` when [`Self::is_planar`] is false.
    pub fn plane(&self) -> Option<TnamingNamedShape> {
        let inner = ffi::tdataxtd_constraint_get_plane(&self.inner);
        if inner.is_null() {
            None
        } else {
            Some(TnamingNamedShape::from_ffi(inner))
        }
    }

    /// Sets the plane of a 2D constraint.
    ///
    /// Must be called inside an open [`Command`] scope.
    pub fn set_plane(&mut self, _cmd: &Command<'_>, plane: &TnamingNamedShape) {
        ffi::tdataxtd_constraint_set_plane(self.inner.pin_mut(), plane.inner());
    }

    /// Probes for a `TDataXtd_Constraint` attribute on `label`.
    ///
    /// Returns `None` when the attribute is not present.
    pub fn find(label: &OcLabel) -> Option<Self> {
        let inner = ffi::tdataxtd_constraint_find(&label.inner);
        if inner.is_null() {
            None
        } else {
            Some(Self {
                inner,
                _not_send: PhantomData,
            })
        }
    }

    /// Removes the `TDataXtd_Constraint` attribute from `label`.
    ///
    /// Returns `false` if the attribute was not present.
    /// Must be called inside an open [`Command`] scope.
    pub fn forget(_cmd: &Command<'_>, label: &OcLabel) -> bool {
        ffi::tdataxtd_constraint_forget(&label.inner)
    }
}

impl std::fmt::Debug for OcConstraintAttr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OcConstraintAttr")
            .field("kind", &self.kind())
            .field("nb_geometries", &self.nb_geometries())
            .field("is_dimension", &self.is_dimension())
            .field("verified", &self.verified())
            .finish()
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gp::OcPnt;
    use crate::topo::application::OcApplication;
    use crate::topo::document::OcDocument;
    use crate::topo::tnaming::TnamingBuilder;
    use crate::topo::{OcEdge, OcFace, OcWire};
    use occt_sys::ffi::new_tnaming_builder;

    fn new_doc() -> (OcApplication, OcDocument) {
        let mut app = OcApplication::new();
        let doc = app.new_document("BinXCAF").unwrap();
        (app, doc)
    }

    /// Build a unit square face and record it as a primitive named shape on
    /// `label` using the already-open `cmd`.  Returns the `TnamingNamedShape`
    /// handle.  Does not open or close any command.
    fn record_named_shape(cmd: &Command<'_>, label: &OcLabel) -> TnamingNamedShape {
        let edges = [
            OcEdge::from_pnts(OcPnt::new(0.0, 0.0, 0.0), OcPnt::new(1.0, 0.0, 0.0)).unwrap(),
            OcEdge::from_pnts(OcPnt::new(1.0, 0.0, 0.0), OcPnt::new(1.0, 1.0, 0.0)).unwrap(),
            OcEdge::from_pnts(OcPnt::new(1.0, 1.0, 0.0), OcPnt::new(0.0, 1.0, 0.0)).unwrap(),
            OcEdge::from_pnts(OcPnt::new(0.0, 1.0, 0.0), OcPnt::new(0.0, 0.0, 0.0)).unwrap(),
        ];
        let wire = OcWire::from_edges(&edges).unwrap();
        let face = OcFace::from_wire(&wire, true).unwrap();
        let shape = face.as_shape();
        let mut builder = TnamingBuilder::new(new_tnaming_builder(&label.inner));
        builder.generated_fresh(&shape);
        builder.named_shape()
    }

    #[test]
    fn geometry_set_diagnostic() {
        let (_app, mut doc) = new_doc();

        let label = {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let l = main.get_or_create_child(&cmd, 1);
            cmd.commit().unwrap();
            l
        };

        assert!(
            OcGeometryAttr::find(&label).is_none(),
            "attribute should not exist before cmd2"
        );

        {
            let cmd = doc.begin_command().unwrap();
            let result = OcGeometryAttr::set(&cmd, &label, GeometryKind::Line);
            assert!(result.is_ok(), "set returned Err: {:?}", result.err());
            assert!(
                OcGeometryAttr::find(&label).is_some(),
                "find inside open command returned None"
            );
            cmd.commit().unwrap();
        }

        assert!(
            OcGeometryAttr::find(&label).is_some(),
            "attribute not found after commit"
        );

        {
            let cmd = doc.begin_command().unwrap();
            let forgotten = OcGeometryAttr::forget(&cmd, &label);
            assert!(forgotten, "forget returned false");
            cmd.commit().unwrap();
        }
        assert!(
            OcGeometryAttr::find(&label).is_none(),
            "attribute still present after forget"
        );

        let undo_forget = doc.undo();
        println!("undo-of-forget result: {:?}", undo_forget);
        assert!(
            OcGeometryAttr::find(&label).is_some(),
            "attribute not restored after undo-of-forget"
        );

        let undo_set = doc.undo();
        println!("undo-of-set result: {:?}", undo_set);
        assert!(
            OcGeometryAttr::find(&label).is_none(),
            "attribute still present after undo-of-set"
        );
    }

    #[test]
    fn geometry_set_and_find() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            OcGeometryAttr::set(&cmd, &label, GeometryKind::Circle).unwrap();
            cmd.commit().unwrap();
        }
        let found = OcGeometryAttr::find(&label).expect("attribute should be present");
        assert_eq!(found.kind(), GeometryKind::Circle);
    }

    #[test]
    fn geometry_set_defaults_to_any() {
        let (_app, mut doc) = new_doc();
        let main = doc.main();
        let cmd = doc.begin_command().unwrap();
        let label = main.get_or_create_child(&cmd, 1);
        let attr = OcGeometryAttr::set(&cmd, &label, GeometryKind::Any).unwrap();
        assert_eq!(attr.kind(), GeometryKind::Any);
        cmd.commit().unwrap();
    }

    #[test]
    fn geometry_forget_removes() {
        let (_app, mut doc) = new_doc();
        let label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            label = main.get_or_create_child(&cmd, 1);
            OcGeometryAttr::set(&cmd, &label, GeometryKind::Any).unwrap();
            cmd.commit().unwrap();
        }
        {
            let cmd = doc.begin_command().unwrap();
            assert!(OcGeometryAttr::forget(&cmd, &label));
            cmd.commit().unwrap();
        }
        assert!(OcGeometryAttr::find(&label).is_none());
    }

    #[test]
    fn geometry_undo_restores() {
        // Label created in cmd1 so it survives undo of cmd2.
        // Matches the two-command pattern used in attributes.rs undo tests.
        let (_app, mut doc) = new_doc();
        let label = {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let l = main.get_or_create_child(&cmd, 1);
            cmd.commit().unwrap();
            l
        };
        {
            let cmd = doc.begin_command().unwrap();
            OcGeometryAttr::set(&cmd, &label, GeometryKind::Any).unwrap();
            cmd.commit().unwrap();
        }
        assert!(OcGeometryAttr::find(&label).is_some());
        doc.undo().unwrap();
        assert!(OcGeometryAttr::find(&label).is_none());
    }

    // ── OcConstraintAttr ──────────────────────────────────────────────────

    #[test]
    fn constraint_set_one_geom_and_find() {
        let (_app, mut doc) = new_doc();
        let c_label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let geom_label = main.get_or_create_child(&cmd, 1);
            c_label = main.get_or_create_child(&cmd, 2);
            let ns = record_named_shape(&cmd, &geom_label);
            OcConstraintAttr::set(&cmd, &c_label, ConstraintKind::Fix, &[&ns]).unwrap();
            cmd.commit().unwrap();
        }
        let found = OcConstraintAttr::find(&c_label).expect("constraint should be present");
        assert_eq!(found.kind(), ConstraintKind::Fix);
        assert_eq!(found.nb_geometries(), 1);
    }

    #[test]
    fn constraint_set_two_geoms_nb_geometries() {
        let (_app, mut doc) = new_doc();
        let c_label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let l1 = main.get_or_create_child(&cmd, 1);
            let l2 = main.get_or_create_child(&cmd, 2);
            c_label = main.get_or_create_child(&cmd, 3);
            let ns1 = record_named_shape(&cmd, &l1);
            let ns2 = record_named_shape(&cmd, &l2);
            OcConstraintAttr::set(&cmd, &c_label, ConstraintKind::Parallel, &[&ns1, &ns2]).unwrap();
            cmd.commit().unwrap();
        }
        let found = OcConstraintAttr::find(&c_label).unwrap();
        assert_eq!(found.nb_geometries(), 2);
        assert_eq!(found.kind(), ConstraintKind::Parallel);
    }

    #[test]
    fn constraint_geometry_accessor_in_range() {
        let (_app, mut doc) = new_doc();
        let c_label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let l1 = main.get_or_create_child(&cmd, 1);
            c_label = main.get_or_create_child(&cmd, 2);
            let ns = record_named_shape(&cmd, &l1);
            OcConstraintAttr::set(&cmd, &c_label, ConstraintKind::Fix, &[&ns]).unwrap();
            cmd.commit().unwrap();
        }
        let found = OcConstraintAttr::find(&c_label).unwrap();
        assert!(found.geometry(1).is_some());
    }

    #[test]
    fn constraint_geometry_oob_returns_none() {
        let (_app, mut doc) = new_doc();
        let c_label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let l1 = main.get_or_create_child(&cmd, 1);
            c_label = main.get_or_create_child(&cmd, 2);
            let ns = record_named_shape(&cmd, &l1);
            OcConstraintAttr::set(&cmd, &c_label, ConstraintKind::Fix, &[&ns]).unwrap();
            cmd.commit().unwrap();
        }
        let found = OcConstraintAttr::find(&c_label).unwrap();
        assert!(found.geometry(2).is_none());
    }

    #[test]
    fn constraint_dimensional_round_trip() {
        let (_app, mut doc) = new_doc();
        let c_label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let geom_label = main.get_or_create_child(&cmd, 1);
            c_label = main.get_or_create_child(&cmd, 2);
            let val_label = c_label.get_or_create_child(&cmd, 1);
            let ns = record_named_shape(&cmd, &geom_label);
            let real = OcReal::set(&cmd, &val_label, 42.0).unwrap();
            let mut c =
                OcConstraintAttr::set(&cmd, &c_label, ConstraintKind::Radius, &[&ns]).unwrap();
            c.set_value(&cmd, &real);
            cmd.commit().unwrap();
        }
        let found = OcConstraintAttr::find(&c_label).unwrap();
        assert!(found.is_dimension());
        let val = found.value().expect("value should be present");
        assert!((val.get() - 42.0).abs() < 1e-12);
    }

    #[test]
    fn constraint_forget_removes() {
        let (_app, mut doc) = new_doc();
        let c_label;
        {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let l1 = main.get_or_create_child(&cmd, 1);
            c_label = main.get_or_create_child(&cmd, 2);
            let ns = record_named_shape(&cmd, &l1);
            OcConstraintAttr::set(&cmd, &c_label, ConstraintKind::Fix, &[&ns]).unwrap();
            cmd.commit().unwrap();
        }
        {
            let cmd = doc.begin_command().unwrap();
            assert!(OcConstraintAttr::forget(&cmd, &c_label));
            cmd.commit().unwrap();
        }
        assert!(OcConstraintAttr::find(&c_label).is_none());
    }

    #[test]
    fn constraint_undo_restores() {
        // Label and named shape created in cmd1; constraint set in cmd2.
        // Undo of cmd2 removes the constraint while the label survives.
        let (_app, mut doc) = new_doc();
        let (l1, c_label, ns) = {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let l1 = main.get_or_create_child(&cmd, 1);
            let cl = main.get_or_create_child(&cmd, 2);
            let ns = record_named_shape(&cmd, &l1);
            cmd.commit().unwrap();
            (l1, cl, ns)
        };
        {
            let cmd = doc.begin_command().unwrap();
            OcConstraintAttr::set(&cmd, &c_label, ConstraintKind::Fix, &[&ns]).unwrap();
            cmd.commit().unwrap();
        }
        assert!(OcConstraintAttr::find(&c_label).is_some());
        doc.undo().unwrap();
        assert!(OcConstraintAttr::find(&c_label).is_none());
    }
}
