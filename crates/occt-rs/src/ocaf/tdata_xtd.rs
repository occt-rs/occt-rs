//! Extended attribute bindings.
//!
//!
//! - [`OcGeometryAttr`] — a qualifier tag that labels a `TNaming_NamedShape`
//!   with a geometry kind (point, line, circle, …).  It lives on the **same
//!   label** as the named shape and adds no data of its own beyond the kind
//!   enum.
//!
//! - [`OcConstraintAttr`] — records a constraint between 1–4 geometry
//!   participants.  Each participant is referenced by its
//!   [`TopoNamingNamedShape`] handle —
//!   the topology record — not by an [`OcGeometryAttr`] handle.  Dimensional
//!   constraints additionally carry an [`OcReal`]
//!   value attached via a sub-label.
//!
//! ## Constraint kind
//!
//! `TDataXtd_ConstraintEnum` is intentionally narrow (24 values covering
//! basic OCAF primitives).  Application-level constraint semantics beyond
//! that set should be managed at the application layer using a GCS of your
//! choice.  This binding exposes the enum as [`ConstraintKind`] but you are
//! free to ignore it and store application constraint kinds via
//! [`OcInteger`](crate::ocaf::attributes::OcInteger) on a sub-label.
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
use crate::gp::{OcAx1, OcAx2, OcPnt};
use crate::ocaf::attributes::OcReal;
use crate::ocaf::document::Command;
use crate::ocaf::label::OcLabel;
use crate::ocaf::topo_naming::TopoNamingBuilder;
use crate::ocaf::topo_naming::TopoNamingNamedShape;
use crate::rs_topo::OcShape;

// ── GeometryKind ─────────────────────────────────────────────────────────────

/// The kind of geometric construction a label represents.
///
/// Maps `TDataXtd_GeometryEnum`.  Attached to a label alongside a
/// [`TopoNamingNamedShape`] to qualify what kind of geometry the stored shape is.
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

/// Extends [`TopoNamingNamedShape`] to also include geometry kind
///
/// Binds to `TDataXtd_Geometry`
///
/// Add to an [`OcLabel`] that already has [`TopoNamingNamedShape`] to declare what
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
pub struct OcGeometryAttr {
    inner: cxx::UniquePtr<ffi::TDataXtdGeometryHandle>,
    _not_send: PhantomData<*mut ()>,
}

impl OcGeometryAttr {
    /// Upsert-get this value-attribute on `label` with the value provided
    ///
    /// The kind is applied before the attribute is registered with the label,
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
    /// TODO: make invalid usage unrepresentable somehow. Maybe require usage of the existing
    /// committed attribute
    ///
    /// Must be called inside an open [`Command`] scope.
    ///
    /// [`set`]: Self::set
    pub fn set_type(&mut self, _cmd: &Command<'_>, kind: GeometryKind) {
        ffi::tdataxtd_geometry_set_type(self.inner.pin_mut(), kind as i32);
    }

    /// Reads the kind-value from this attribute handle
    pub fn kind(&self) -> GeometryKind {
        GeometryKind::from_raw(ffi::tdataxtd_geometry_get_type(&self.inner))
    }

    /// Probes for this attribute on `label`.
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

    /// Removes this attribute from `label`.
    ///
    /// Returns `false` if the attribute was not present.
    /// Must be called inside an open [`Command`] scope.
    pub fn forget(_cmd: &Command<'_>, label: &OcLabel) -> bool {
        ffi::tdataxtd_geometry_forget(&label.inner)
    }
    /// Infers the geometry kind of `label` by inspecting its
    /// [`TopoNamingNamedShape`] topology.
    ///
    /// This is the OCCT-prescribed read path for Point, Axis, and Plane labels:
    /// rather than reading the `TDataXtd_Geometry` qualifier attribute, it
    /// inspects the actual shape topology — vertex → [`GeometryKind::Point`],
    /// linear edge → [`GeometryKind::Line`], planar face →
    /// [`GeometryKind::Plane`], and so on.
    ///
    /// Returns `Err` when [`TopoNamingNamedShape`] is present on the label.
    ///
    /// Reference: <https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___geometry.html>
    pub fn type_on_label(label: &OcLabel) -> Result<GeometryKind, OcctError> {
        ffi::tdataxtd_geometry_type_on_label(&label.inner)
            .map(GeometryKind::from_raw)
            .map_err(OcctError::from)
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

/// `TDataXtd_Constraint` attribute handle.
///
/// Records a constraint on a label: its kind, references to 1–4
/// [`TopoNamingNamedShape`] topology attributes, and optionally an associated
/// [`OcReal`] dimension value.
///
/// # Geometry participants
///
/// Each geometry slot holds a `Handle(TNaming_NamedShape)`.  You pass
/// [`TopoNamingNamedShape`] handles — **not** [`OcGeometryAttr`] handles.
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
    /// Attaches a constraint referencing 1–4 [`TopoNamingNamedShape`] geometry
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
        geoms: &[&TopoNamingNamedShape],
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
    pub fn set_geometry(&mut self, _cmd: &Command<'_>, index: i32, ns: &TopoNamingNamedShape) {
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
    pub fn geometry(&self, index: i32) -> Option<TopoNamingNamedShape> {
        let inner = ffi::tdataxtd_constraint_get_geometry(&self.inner, index);
        if inner.is_null() {
            None
        } else {
            Some(TopoNamingNamedShape::from_ffi(inner))
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
    pub fn plane(&self) -> Option<TopoNamingNamedShape> {
        let inner = ffi::tdataxtd_constraint_get_plane(&self.inner);
        if inner.is_null() {
            None
        } else {
            Some(TopoNamingNamedShape::from_ffi(inner))
        }
    }

    /// Sets the plane of a 2D constraint.
    ///
    /// Must be called inside an open [`Command`] scope.
    pub fn set_plane(&mut self, _cmd: &Command<'_>, plane: &TopoNamingNamedShape) {
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
// ── OcPositionAttr ───────────────────────────────────────────────────────────

/// A `TDataXtd_Position` attribute handle — stores a 3-D point on a label.
///
/// Unlike [`OcGeometryAttr`] (which only tags a label), `OcPositionAttr`
/// owns its `gp_Pnt` data directly inside the OCCT attribute.  The stored
/// point can be read back without inspecting any co-located `TNaming_NamedShape`.
///
/// # Undo ordering
///
/// [`Self::set`] applies the position before `AddAttribute`, so the single
/// `AddAttribute` operation is the complete undo delta.  [`Self::set_position`] is
/// for updating a position that was committed in a **prior** command; calling
/// it on a freshly-created, not-yet-committed attribute is unsound.
///
/// # Usage pattern
///
/// ```ignore
/// // Inside an open command:
/// let attr = OcPositionAttr::set(&cmd, &label, OcPnt::new(1.0, 2.0, 3.0))?;
/// assert_eq!(attr.position(), OcPnt::new(1.0, 2.0, 3.0));
/// ```
///
/// Reference: <https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___position.html>
pub struct OcPositionAttr {
    inner: cxx::UniquePtr<ffi::TDataXtdPositionHandle>,
    _not_send: PhantomData<*mut ()>,
}

impl OcPositionAttr {
    /// Finds or creates a `TDataXtd_Position` attribute on `label` with the
    /// given point set atomically before `AddAttribute`.
    ///
    /// The position is applied to the raw attribute object before it is
    /// registered with the label, so `AddAttribute` is the sole undo delta.
    /// Undo therefore cleanly removes the attribute rather than reverting to
    /// a default state.
    ///
    /// Must be called inside an open [`Command`] scope.
    pub fn set(_cmd: &Command<'_>, label: &OcLabel, pos: OcPnt) -> Result<Self, OcctError> {
        let inner = ffi::tdataxtd_position_set(&label.inner, pos.x, pos.y, pos.z)
            .map_err(OcctError::from)?;
        Ok(Self {
            inner,
            _not_send: PhantomData,
        })
    }

    /// Updates the stored position on an already-committed attribute.
    ///
    /// Safe only when this attribute was committed in a prior command —
    /// `SetPosition` inside OCCT calls `Backup()`, which requires a committed
    /// state to snapshot correctly.  For new attributes, pass the position
    /// to [`set`] directly.
    ///
    /// Must be called inside an open [`Command`] scope.
    ///
    /// [`set`]: Self::set
    pub fn set_position(&mut self, _cmd: &Command<'_>, pos: OcPnt) {
        ffi::tdataxtd_position_set_position(self.inner.pin_mut(), pos.x, pos.y, pos.z);
    }

    /// Reads the stored position from this handle.
    pub fn position(&self) -> OcPnt {
        let mut x = 0.0_f64;
        let mut y = 0.0_f64;
        let mut z = 0.0_f64;
        ffi::tdataxtd_position_get_position(&self.inner, &mut x, &mut y, &mut z);
        OcPnt::new(x, y, z)
    }

    /// Probes for a `TDataXtd_Position` attribute on `label`.
    ///
    /// Returns `None` when the attribute is not present.
    pub fn find(label: &OcLabel) -> Option<Self> {
        let inner = ffi::tdataxtd_position_find(&label.inner);
        if inner.is_null() {
            None
        } else {
            Some(Self {
                inner,
                _not_send: PhantomData,
            })
        }
    }

    /// Removes the `TDataXtd_Position` attribute from `label`.
    ///
    /// Returns `false` if the attribute was not present.
    /// Must be called inside an open [`Command`] scope.
    pub fn forget(_cmd: &Command<'_>, label: &OcLabel) -> bool {
        ffi::tdataxtd_position_forget(&label.inner)
    }
}

impl std::fmt::Debug for OcPositionAttr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OcPositionAttr")
            .field("position", &self.position())
            .finish()
    }
}

/// Marker attribute. Indicates a [`TopoNamingNamedShape`] is attached to an [`OcPnt`]
///
/// OCAF provides the means to fold a [`OcPnt`] geometry primitive, such as a user-authored sketch
/// point, into the topo-naming system. For this to happen:
///
/// - [`TopoNamingBuilder::primitive`] wraps the [`OcPnt`] coordinate in a
///   [`TopoNamingNamedShape`] vertex gives it a place in the naming graph.
///   This provides rebuild-stability
/// - [`OcPointAttr`] tags the label as a point: Constraint solvers, and other
///   topo-naming machinery can identify it by role without inspecting the
///   shape topology.
/// - Both live on the same label, in the same [`Command`]: All info (coordinate,
///   naming record, and semantic tag ) are grouped to a single undo/redo.
///
/// Effectively: [`OcPnt`] can participate as a TopoNaming entity, the point.
/// It can be referenced as a geometry participant in [`OcConstraintAttr`], and
/// survives topology operations through the OCAF topo-naming-problem machinery.
///
/// Use [`OcPositionAttr`] instead when you only need a raw coordinate that
/// participates in undo/redo but not in topological naming.
///
/// # Example
///
/// We will create a tree that looks as follows:
///
/// ```text
/// main (0:1)
/// └── 1 (0:1:1)   sketch
///     ├── 1 (0:1:1:1)   point A (0.0, 1.0, 0.0)
///     │       TopoNamingNamedShape (Primitive, vertex_a)
///     │       OcPointAttr
///     ├── 2 (0:1:1:2)   point B (0.0, 0.0, 0.0)
///     │       TopoNamingNamedShape (Primitive, vertex_b)
///     │       OcPointAttr
///     ├── 3 (0:1:1:3)   point C (1.0, 0.0, 0.0)
///     │       TopoNamingNamedShape (Primitive, vertex_c)
///     │       OcPointAttr
///     └── 4 (0:1:1:4)   point D (1.0, 1.0, 0.0)
///     │       TopoNamingNamedShape (Primitive, vertex_d)
///     │       OcPointAttr
///     └── 5 (0:1:1:5)   point A edited
///             TopoNamingNamedShape (Modify, (vertex_a, vertex_e))
///             OcPointAttr
/// ```
///
/// You can think of the example as an application handling a user adding the points while doing a
/// sketch
///
/// ```
/// use occt_rs::gp::OcPnt;
/// use occt_rs::ocaf::OcApplication;
/// use occt_rs::ocaf::tdata_xtd::OcPointAttr;
/// use occt_rs::ocaf::topo_naming::{TopoNamingEvolution, TopoNamingNamedShape};
///
/// let mut app = OcApplication::new();
/// let mut doc = app.new_document("BinXCAF").unwrap();
/// doc.set_undo_limit(10);
///
/// let main = doc.main();
/// let (pt_a, pt_b, pt_c, pt_d) = {
///     let cmd = doc.begin_command().unwrap();
///     let sketch = main.get_or_create_child(&cmd, 1);
///
///     let la = sketch.get_or_create_child(&cmd, 1);
///     OcPointAttr::record_shape(&cmd, &la, OcPnt::new(0.0, 1.0, 0.0)).unwrap();
///     OcPointAttr::set(&cmd, &la).unwrap();
///
///     let lb = sketch.get_or_create_child(&cmd, 2);
///     OcPointAttr::record_shape(&cmd, &lb, OcPnt::new(0.0, 0.0, 0.0)).unwrap();
///     OcPointAttr::set(&cmd, &lb).unwrap();
///
///     let lc = sketch.get_or_create_child(&cmd, 3);
///     OcPointAttr::record_shape(&cmd, &lc, OcPnt::new(1.0, 0.0, 0.0)).unwrap();
///     OcPointAttr::set(&cmd, &lc).unwrap();
///
///     let ld = sketch.get_or_create_child(&cmd, 4);
///     OcPointAttr::record_shape(&cmd, &ld, OcPnt::new(1.0, 1.0, 0.0)).unwrap();
///     OcPointAttr::set(&cmd, &ld).unwrap();
///
///     cmd.commit().unwrap();
///     (la, lb, lc, ld)
/// };
///
/// // Coordinates round-trip through the document
/// let p = OcPointAttr::get(&pt_a).unwrap().unwrap();
/// assert!((p.x - 0.0).abs() < 1e-12);
/// assert!((p.y - 1.0).abs() < 1e-12);
/// assert!((p.z - 0.0).abs() < 1e-12);
///
/// // And let's use the topo-naming system to make a new point that evolved from an old one:
///
/// let pt_a_edited = {
///     let cmd = doc.begin_command().unwrap();
///     let sketch = main.get_or_create_child(&cmd, 1);
///     let l = sketch.get_or_create_child(&cmd, 5);
///
///     // Retrieve the current shape before overwriting it
///     let old_shape = TopoNamingNamedShape::find(&pt_a).unwrap().get().unwrap();
///
///     // Record the new coordinate
///     OcPointAttr::record_shape(&cmd, &l, OcPnt::new(0.5, 1.0, 0.0)).unwrap();
///     OcPointAttr::set(&cmd, &l).unwrap();
///
///     // Retrieve the new shape and record the evolution explicitly
///     let new_shape = TopoNamingNamedShape::find(&l).unwrap().get().unwrap();
///     cmd.name_builder(&l).modified(&old_shape, &new_shape);
///
///     cmd.commit().unwrap();
///     l
/// };
///
/// // The original label is unchanged
/// assert_eq!(
///     TopoNamingNamedShape::find(&pt_a).unwrap().evolution(),
///     Some(TopoNamingEvolution::Primitive),
/// );
/// assert!((OcPointAttr::get(&pt_a).unwrap().unwrap().x - 0.0).abs() < 1e-12);
///
/// // The new label records the modification
/// assert_eq!(
///     TopoNamingNamedShape::find(&pt_a_edited).unwrap().evolution(),
///     Some(TopoNamingEvolution::Modify),
/// );
/// assert!((OcPointAttr::get(&pt_a_edited).unwrap().unwrap().x - 0.5).abs() < 1e-12);
///
/// // Undo removes the edit label's attributes
/// doc.undo().unwrap();
/// assert!(TopoNamingNamedShape::find(&pt_a_edited).is_none());
/// ```
///
/// Reference: <https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___point.html>
pub struct OcPointAttr {
    inner: cxx::UniquePtr<ffi::TDataXtdPointHandle>,
    _not_send: PhantomData<*mut ()>,
}

impl OcPointAttr {
    pub fn from_ffi(inner: cxx::UniquePtr<ffi::TDataXtdPointHandle>) -> Self {
        Self {
            inner,
            _not_send: PhantomData,
        }
    }
    pub fn as_ffi(self) -> cxx::UniquePtr<ffi::TDataXtdPointHandle> {
        self.inner
    }

    /// Records a vertex at `pos` as a generated shape on `label` via
    /// `TNaming_Builder`.
    ///
    /// This is the shape half of the Option B pattern.  Call this first, then
    /// call [`set`] to attach the semantic tag in the same command.  The
    /// returned [`TopoNamingNamedShape`] can be passed directly to
    /// [`OcConstraintAttr::set`].
    ///
    /// Must be called inside an open [`Command`] scope.
    ///
    /// [`set`]: Self::set
    pub fn record_shape(
        _cmd: &Command<'_>,
        label: &OcLabel,
        pos: OcPnt,
    ) -> Result<TopoNamingNamedShape, OcctError> {
        let vertex =
            ffi::tdataxtd_make_vertex_shape(pos.x, pos.y, pos.z).map_err(OcctError::from)?;
        // Safety: vertex_as_shape is a zero-cost upcast; clone_shape is
        // make_unique<TopoDS_Shape> — non-null.
        let shape =
            unsafe { OcShape::from_ffi_unchecked(ffi::clone_shape(ffi::vertex_as_shape(&vertex))) };
        let mut builder = TopoNamingBuilder::new(label);
        builder.primitive(&shape);
        Ok(builder.named_shape())
    }

    /// Finds or creates the `TDataXtd_Point` tag attribute on `label`.
    ///
    /// The vertex [`TopoNamingNamedShape`] must already be present on the label —
    /// call [`record_shape`] first in the same command.
    ///
    /// Must be called inside an open [`Command`] scope.
    ///
    /// [`record_shape`]: Self::record_shape
    pub fn set(_cmd: &Command<'_>, label: &OcLabel) -> Result<Self, OcctError> {
        let inner = ffi::tdataxtd_point_set(&label.inner).map_err(OcctError::from)?;
        Ok(Self {
            inner,
            _not_send: PhantomData,
        })
    }

    /// Returns the position of the vertex on `label`.
    ///
    /// Uses [`OcGeometryAttr::type_on_label`] to verify the shape is a point,
    /// then extracts coordinates via `BRep_Tool::Pnt`.
    ///
    /// Returns `None` when no `TNaming_NamedShape` is present.
    /// Returns `Err` when the shape is present but is not a vertex.
    pub fn get(label: &OcLabel) -> Result<Option<OcPnt>, OcctError> {
        match OcGeometryAttr::type_on_label(label) {
            Err(_) | Ok(GeometryKind::Any) => return Ok(None),
            Ok(GeometryKind::Point) => {}
            Ok(other) => {
                return Err(OcctError {
                    kind: crate::error::OcctErrorKind::DomainError,
                    message: format!(
                        "OcPointAttr::get: expected Point geometry, found {:?}",
                        other
                    ),
                })
            }
        }
        let ns = TopoNamingNamedShape::find(label).ok_or_else(|| OcctError {
            kind: crate::error::OcctErrorKind::DomainError,
            message: "OcPointAttr::get: TNaming_NamedShape absent despite type_on_label succeeding"
                .to_owned(),
        })?;
        // Safety: tnaming_named_shape_get wraps TNaming_NamedShape::Get() via
        // make_unique; the named shape is present (find succeeded above) — non-null.
        let shape =
            unsafe { OcShape::from_ffi_unchecked(ffi::tnaming_named_shape_get(ns.inner())) };
        let vertex = ffi::shape_as_vertex(shape.as_ffi());
        Ok(Some(OcPnt::new(
            ffi::vertex_pnt_x(&vertex),
            ffi::vertex_pnt_y(&vertex),
            ffi::vertex_pnt_z(&vertex),
        )))
    }

    /// Probes for a `TDataXtd_Point` attribute on `label`.
    ///
    /// Returns `None` when the attribute is not present.
    pub fn find(label: &OcLabel) -> Option<Self> {
        let inner = ffi::tdataxtd_point_find(&label.inner);
        if inner.is_null() {
            None
        } else {
            Some(Self {
                inner,
                _not_send: PhantomData,
            })
        }
    }

    /// Removes the `TDataXtd_Point` attribute from `label`.
    ///
    /// Returns `false` if the attribute was not present.
    /// Note: this removes the tag only, not the co-located `TNaming_NamedShape`.
    /// Must be called inside an open [`Command`] scope.
    pub fn forget(_cmd: &Command<'_>, label: &OcLabel) -> bool {
        ffi::tdataxtd_point_forget(&label.inner)
    }
}

impl std::fmt::Debug for OcPointAttr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OcPointAttr").finish()
    }
}

// ── OcAxisAttr ───────────────────────────────────────────────────────────────

/// A `TDataXtd_Axis` attribute handle — semantic tag marking a label as an axis.
///
/// The geometry lives in a [`TopoNamingNamedShape`] on the **same label** as an
/// infinite linear edge produced by [`TopoNamingBuilder::generated`].
///
/// The input to [`record_shape`] is an [`OcAx1`] (origin + direction).
/// Internally this constructs a `gp_Lin` (structurally identical to `gp_Ax1`)
/// and passes it to `BRepBuilderAPI_MakeEdge`, producing an infinite edge in
/// the TNaming graph.
///
/// [`record_shape`]: Self::record_shape
///
/// Reference: <https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___axis.html>
pub struct OcAxisAttr {
    inner: cxx::UniquePtr<ffi::TDataXtdAxisHandle>,
    _not_send: PhantomData<*mut ()>,
}

impl OcAxisAttr {
    pub fn from_ffi(inner: cxx::UniquePtr<ffi::TDataXtdAxisHandle>) -> Self {
        Self {
            inner,
            _not_send: PhantomData,
        }
    }
    pub fn as_ffi(self) -> cxx::UniquePtr<ffi::TDataXtdAxisHandle> {
        self.inner
    }

    /// Records an infinite linear edge defined by `axis` as a generated shape
    /// on `label` via `TNaming_Builder`.
    ///
    /// Call this first, then call [`set`] to attach the semantic tag in the
    /// same command.
    ///
    /// Must be called inside an open [`Command`] scope.
    ///
    /// [`set`]: Self::set
    pub fn record_shape(
        _cmd: &Command<'_>,
        label: &OcLabel,
        axis: OcAx1,
    ) -> Result<TopoNamingNamedShape, OcctError> {
        let loc = axis.location();
        let dir = axis.direction();
        let edge = ffi::tdataxtd_make_infinite_edge_from_ax1(
            loc.x,
            loc.y,
            loc.z,
            dir.x(),
            dir.y(),
            dir.z(),
        )
        .map_err(OcctError::from)?;
        // Safety: edge_as_shape is a zero-cost upcast; clone_shape is
        // make_unique<TopoDS_Shape> — non-null.
        let shape =
            unsafe { OcShape::from_ffi_unchecked(ffi::clone_shape(ffi::edge_as_shape(&edge))) };
        let mut builder = TopoNamingBuilder::new(label);
        builder.primitive(&shape);
        Ok(builder.named_shape())
    }

    /// Finds or creates the `TDataXtd_Axis` tag attribute on `label`.
    ///
    /// The linear edge [`TopoNamingNamedShape`] must already be present on the
    /// label — call [`record_shape`] first in the same command.
    ///
    /// Must be called inside an open [`Command`] scope.
    ///
    /// [`record_shape`]: Self::record_shape
    pub fn set(_cmd: &Command<'_>, label: &OcLabel) -> Result<Self, OcctError> {
        let inner = ffi::tdataxtd_axis_set(&label.inner).map_err(OcctError::from)?;
        Ok(Self::from_ffi(inner))
    }

    /// Returns the geometry kind and named shape of the axis on `label`.
    ///
    /// Returns the [`GeometryKind`] inferred from the shape topology (Line,
    /// Circle, Ellipse, Spline, etc.) alongside the [`TopoNamingNamedShape`]
    /// handle.  The caller uses the kind to decide how to interpret the shape —
    /// for a [`GeometryKind::Line`] they might extract an `OcAx1` via
    /// BRep_Tool; for a circle they would extract the axis differently.
    ///
    /// This follows the OCCT prescription: `TDataXtd_Geometry::Type(label)`
    /// identifies what the shape is; the caller does the geometry extraction.
    ///
    /// Returns `None` when no `TNaming_NamedShape` is present.
    pub fn get(label: &OcLabel) -> Result<Option<(GeometryKind, TopoNamingNamedShape)>, OcctError> {
        let kind = match OcGeometryAttr::type_on_label(label) {
            Err(_) | Ok(GeometryKind::Any) => return Ok(None),
            Ok(k) => k,
        };
        let ns = TopoNamingNamedShape::find(label)
            .expect("NamedShape must be present: type_on_label returned non-Any");
        Ok(Some((kind, ns)))
    }

    /// Probes for a `TDataXtd_Axis` attribute on `label`.
    ///
    /// Returns `None` when the attribute is not present.
    pub fn find(label: &OcLabel) -> Option<Self> {
        let inner = ffi::tdataxtd_axis_find(&label.inner);
        if inner.is_null() {
            None
        } else {
            Some(Self {
                inner,
                _not_send: PhantomData,
            })
        }
    }

    /// Removes the `TDataXtd_Axis` attribute from `label`.
    ///
    /// Returns `false` if the attribute was not present.
    /// Note: this removes the tag only, not the co-located `TNaming_NamedShape`.
    /// Must be called inside an open [`Command`] scope.
    pub fn forget(_cmd: &Command<'_>, label: &OcLabel) -> bool {
        ffi::tdataxtd_axis_forget(&label.inner)
    }
}

impl std::fmt::Debug for OcAxisAttr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OcAxisAttr").finish()
    }
}

/// A `TDataXtd_Plane` attribute handle — semantic tag marking a label as a plane.
///
/// The geometry lives in a [`TopoNamingNamedShape`] on the **same label** as an
/// infinite planar face produced by [`TopoNamingBuilder::generated`].
///
/// The input to [`record_shape`] is an [`OcAx2`] (origin + normal + X direction).
/// Internally this constructs a `gp_Pln` via `gp_Ax3(gp_Ax2)` and passes it
/// to `BRepBuilderAPI_MakeFace`.  The X direction determines the orientation
/// of the plane's local frame, which is relevant for interpreting "horizontal"
/// and "vertical" in 2D sketch constraints.
///
/// [`record_shape`]: Self::record_shape
///
/// Reference: <https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___plane.html>
pub struct OcPlaneAttr {
    inner: cxx::UniquePtr<ffi::TDataXtdPlaneHandle>,
    _not_send: PhantomData<*mut ()>,
}

impl OcPlaneAttr {
    pub fn as_ffi(self) -> cxx::UniquePtr<ffi::TDataXtdPlaneHandle> {
        self.inner
    }

    /// Records an infinite planar face defined by `frame` as a generated shape
    /// on `label` via `TNaming_Builder`.
    ///
    /// Call this first, then call [`set`] to attach the semantic tag in the
    /// same command.
    ///
    /// Must be called inside an open [`Command`] scope.
    ///
    /// [`set`]: Self::set
    pub fn record_shape(
        _cmd: &Command<'_>,
        label: &OcLabel,
        frame: OcAx2,
    ) -> Result<TopoNamingNamedShape, OcctError> {
        let loc = frame.location();
        let n = frame.direction();
        let x = frame.x_direction();
        let face = ffi::tdataxtd_make_face_from_ax2(
            loc.x,
            loc.y,
            loc.z,
            n.x(),
            n.y(),
            n.z(),
            x.x(),
            x.y(),
            x.z(),
        )
        .map_err(OcctError::from)?;
        // Safety: face_as_shape is a zero-cost upcast; clone_shape is
        // make_unique<TopoDS_Shape> — non-null.
        let shape =
            unsafe { OcShape::from_ffi_unchecked(ffi::clone_shape(ffi::face_as_shape(&face))) };
        let mut builder = TopoNamingBuilder::new(label);
        builder.primitive(&shape);
        Ok(builder.named_shape())
    }

    /// Finds or creates the `TDataXtd_Plane` tag attribute on `label`.
    ///
    /// The planar face [`TopoNamingNamedShape`] must already be present on the
    /// label — call [`record_shape`] first in the same command.
    ///
    /// Must be called inside an open [`Command`] scope.
    ///
    /// [`record_shape`]: Self::record_shape
    pub fn set(_cmd: &Command<'_>, label: &OcLabel) -> Result<Self, OcctError> {
        let inner = ffi::tdataxtd_plane_set(&label.inner).map_err(OcctError::from)?;
        Ok(Self::from_ffi(inner))
    }

    /// Returns the geometry kind and named shape of the plane on `label`.
    ///
    /// Returns the [`GeometryKind`] inferred from the shape topology alongside
    /// the [`TopoNamingNamedShape`] handle.  For a well-formed plane label the
    /// kind will be [`GeometryKind::Plane`]; the caller extracts the `gp_Pln`
    /// frame via BRep_Tool on the face in the named shape.
    ///
    /// Returns `None` when no `TNaming_NamedShape` is present.
    pub fn get(label: &OcLabel) -> Result<Option<(GeometryKind, TopoNamingNamedShape)>, OcctError> {
        let kind = match OcGeometryAttr::type_on_label(label) {
            Err(_) | Ok(GeometryKind::Any) => return Ok(None),
            Ok(k) => k,
        };
        let ns = TopoNamingNamedShape::find(label)
            .expect("NamedShape must be present: type_on_label returned non-Any");
        Ok(Some((kind, ns)))
    }

    /// Probes for a `TDataXtd_Plane` attribute on `label`.
    ///
    /// Returns `None` when the attribute is not present.
    pub fn find(label: &OcLabel) -> Option<Self> {
        let inner = ffi::tdataxtd_plane_find(&label.inner);
        if inner.is_null() {
            None
        } else {
            Some(Self::from_ffi(inner))
        }
    }

    /// Removes the `TDataXtd_Plane` attribute from `label`.
    ///
    /// Returns `false` if the attribute was not present.
    /// Note: this removes the tag only, not the co-located `TNaming_NamedShape`.
    /// Must be called inside an open [`Command`] scope.
    pub fn forget(_cmd: &Command<'_>, label: &OcLabel) -> bool {
        ffi::tdataxtd_plane_forget(&label.inner)
    }

    fn from_ffi(inner: cxx::UniquePtr<ffi::TDataXtdPlaneHandle>) -> Self {
        Self {
            inner,
            _not_send: PhantomData,
        }
    }
}

impl std::fmt::Debug for OcPlaneAttr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OcPlaneAttr").finish()
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gp::{OcDir, OcPnt};
    use crate::ocaf::application::OcApplication;
    use crate::ocaf::document::OcDocument;
    use crate::ocaf::topo_naming::TopoNamingBuilder;
    use crate::rs_topo::{OcEdge, OcFace, OcWire};

    fn new_doc() -> (OcApplication, OcDocument) {
        let mut app = OcApplication::new();
        let doc = app.new_document("BinXCAF").unwrap();
        (app, doc)
    }

    /// Build a unit square face and record it as a primitive named shape on
    /// `label` using the already-open `cmd`.  Returns the `TopoNamingNamedShape`
    /// handle.  Does not open or close any command.
    fn record_named_shape(_cmd: &Command<'_>, label: &OcLabel) -> TopoNamingNamedShape {
        let edges = [
            OcEdge::from_pnts(OcPnt::new(0.0, 0.0, 0.0), OcPnt::new(1.0, 0.0, 0.0)).unwrap(),
            OcEdge::from_pnts(OcPnt::new(1.0, 0.0, 0.0), OcPnt::new(1.0, 1.0, 0.0)).unwrap(),
            OcEdge::from_pnts(OcPnt::new(1.0, 1.0, 0.0), OcPnt::new(0.0, 1.0, 0.0)).unwrap(),
            OcEdge::from_pnts(OcPnt::new(0.0, 1.0, 0.0), OcPnt::new(0.0, 0.0, 0.0)).unwrap(),
        ];
        let wire = OcWire::from_edges(&edges).unwrap();
        let face = OcFace::from_wire(&wire, true).unwrap();
        let shape = face.as_shape();
        let mut builder = TopoNamingBuilder::new(&label);
        builder.primitive(&shape);
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
        let (_l1, c_label, ns) = {
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
    mod position_tests {
        use super::*;

        fn new_doc() -> (
            crate::ocaf::application::OcApplication,
            crate::ocaf::document::OcDocument,
        ) {
            let mut app = crate::ocaf::application::OcApplication::new();
            let doc = app.new_document("BinXCAF").unwrap();
            (app, doc)
        }

        #[test]
        fn position_set_and_find() {
            let (_app, mut doc) = new_doc();
            let label = {
                let main = doc.main();
                let cmd = doc.begin_command().unwrap();
                let l = main.get_or_create_child(&cmd, 1);
                OcPositionAttr::set(&cmd, &l, OcPnt::new(1.0, 2.0, 3.0)).unwrap();
                cmd.commit().unwrap();
                l
            };
            let found = OcPositionAttr::find(&label).expect("attribute should be present");
            let p = found.position();
            assert!((p.x - 1.0).abs() < 1e-12);
            assert!((p.y - 2.0).abs() < 1e-12);
            assert!((p.z - 3.0).abs() < 1e-12);
        }

        #[test]
        fn position_set_position_updates() {
            // Two-command pattern: create in cmd1, update in cmd2.
            let (_app, mut doc) = new_doc();
            let label = {
                let main = doc.main();
                let cmd = doc.begin_command().unwrap();
                let l = main.get_or_create_child(&cmd, 1);
                OcPositionAttr::set(&cmd, &l, OcPnt::new(0.0, 0.0, 0.0)).unwrap();
                cmd.commit().unwrap();
                l
            };
            {
                let cmd = doc.begin_command().unwrap();
                let mut attr = OcPositionAttr::find(&label).unwrap();
                attr.set_position(&cmd, OcPnt::new(4.0, 5.0, 6.0));
                cmd.commit().unwrap();
            }
            let found = OcPositionAttr::find(&label).unwrap();
            let p = found.position();
            assert!((p.x - 4.0).abs() < 1e-12);
            assert!((p.y - 5.0).abs() < 1e-12);
            assert!((p.z - 6.0).abs() < 1e-12);
        }

        #[test]
        fn position_forget_removes() {
            let (_app, mut doc) = new_doc();
            let label = {
                let main = doc.main();
                let cmd = doc.begin_command().unwrap();
                let l = main.get_or_create_child(&cmd, 1);
                OcPositionAttr::set(&cmd, &l, OcPnt::new(1.0, 0.0, 0.0)).unwrap();
                cmd.commit().unwrap();
                l
            };
            {
                let cmd = doc.begin_command().unwrap();
                assert!(OcPositionAttr::forget(&cmd, &label));
                cmd.commit().unwrap();
            }
            assert!(OcPositionAttr::find(&label).is_none());
        }

        #[test]
        fn position_undo_restores() {
            // Label created in cmd1 so it survives undo of cmd2.
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
                OcPositionAttr::set(&cmd, &label, OcPnt::new(7.0, 8.0, 9.0)).unwrap();
                cmd.commit().unwrap();
            }
            assert!(OcPositionAttr::find(&label).is_some());
            doc.undo().unwrap();
            assert!(OcPositionAttr::find(&label).is_none());
        }

        #[test]
        fn position_undo_set_position_restores() {
            // Two-command pattern: create in cmd1, update in cmd2, undo cmd2.
            // After undo, position should revert to cmd1 value.
            let (_app, mut doc) = new_doc();
            let label = {
                let main = doc.main();
                let cmd = doc.begin_command().unwrap();
                let l = main.get_or_create_child(&cmd, 1);
                OcPositionAttr::set(&cmd, &l, OcPnt::new(1.0, 2.0, 3.0)).unwrap();
                cmd.commit().unwrap();
                l
            };
            {
                let cmd = doc.begin_command().unwrap();
                let mut attr = OcPositionAttr::find(&label).unwrap();
                attr.set_position(&cmd, OcPnt::new(9.0, 9.0, 9.0));
                cmd.commit().unwrap();
            }
            doc.undo().unwrap();
            let found =
                OcPositionAttr::find(&label).expect("attribute should survive undo of update");
            let p = found.position();
            assert!(
                (p.x - 1.0).abs() < 1e-12,
                "x should revert to 1.0, got {}",
                p.x
            );
            assert!(
                (p.y - 2.0).abs() < 1e-12,
                "y should revert to 2.0, got {}",
                p.y
            );
            assert!(
                (p.z - 3.0).abs() < 1e-12,
                "z should revert to 3.0, got {}",
                p.z
            );
        }
    }
    // ── OcPointAttr ──────────────────────────────────────────────────────────

    #[test]
    fn point_set_and_find() {
        let (_app, mut doc) = new_doc();
        let label = {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let l = main.get_or_create_child(&cmd, 1);
            OcPointAttr::record_shape(&cmd, &l, OcPnt::new(1.0, 2.0, 3.0)).unwrap();
            OcPointAttr::set(&cmd, &l).unwrap();
            cmd.commit().unwrap();
            l
        };
        assert!(OcPointAttr::find(&label).is_some());
    }

    #[test]
    fn point_record_shape_returns_named_shape() {
        // The returned TopoNamingNamedShape must be non-null and usable as a
        // constraint geometry participant.
        let (_app, mut doc) = new_doc();
        let label = {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let l = main.get_or_create_child(&cmd, 1);
            let ns = OcPointAttr::record_shape(&cmd, &l, OcPnt::new(0.0, 0.0, 0.0)).unwrap();
            // Verify the handle is non-null by checking it can be used in a constraint.
            let c_label = main.get_or_create_child(&cmd, 2);
            OcConstraintAttr::set(&cmd, &c_label, ConstraintKind::Fix, &[&ns]).unwrap();
            OcPointAttr::set(&cmd, &l).unwrap();
            cmd.commit().unwrap();
            l
        };
        assert!(OcPointAttr::find(&label).is_some());
    }

    #[test]
    fn point_forget_removes_tag() {
        let (_app, mut doc) = new_doc();
        let label = {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let l = main.get_or_create_child(&cmd, 1);
            OcPointAttr::record_shape(&cmd, &l, OcPnt::new(1.0, 0.0, 0.0)).unwrap();
            OcPointAttr::set(&cmd, &l).unwrap();
            cmd.commit().unwrap();
            l
        };
        {
            let cmd = doc.begin_command().unwrap();
            assert!(OcPointAttr::forget(&cmd, &label));
            cmd.commit().unwrap();
        }
        assert!(OcPointAttr::find(&label).is_none());
    }

    #[test]
    fn point_undo_restores() {
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
            OcPointAttr::record_shape(&cmd, &label, OcPnt::new(5.0, 0.0, 0.0)).unwrap();
            OcPointAttr::set(&cmd, &label).unwrap();
            cmd.commit().unwrap();
        }
        assert!(OcPointAttr::find(&label).is_some());
        doc.undo().unwrap();
        assert!(OcPointAttr::find(&label).is_none());
    }

    // ── OcAxisAttr ───────────────────────────────────────────────────────────

    #[test]
    fn axis_set_and_find() {
        let (_app, mut doc) = new_doc();
        let label = {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let l = main.get_or_create_child(&cmd, 1);
            let axis = OcAx1::new(OcPnt::origin(), OcDir::new(0.0, 0.0, 1.0).unwrap());
            OcAxisAttr::record_shape(&cmd, &l, axis).unwrap();
            OcAxisAttr::set(&cmd, &l).unwrap();
            cmd.commit().unwrap();
            l
        };
        assert!(OcAxisAttr::find(&label).is_some());
    }

    #[test]
    fn axis_record_shape_usable_as_constraint_geom() {
        let (_app, mut doc) = new_doc();
        let label = {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let l = main.get_or_create_child(&cmd, 1);
            let axis = OcAx1::new(OcPnt::origin(), OcDir::new(1.0, 0.0, 0.0).unwrap());
            let ns = OcAxisAttr::record_shape(&cmd, &l, axis).unwrap();
            let c_label = main.get_or_create_child(&cmd, 2);
            OcConstraintAttr::set(&cmd, &c_label, ConstraintKind::Fix, &[&ns]).unwrap();
            OcAxisAttr::set(&cmd, &l).unwrap();
            cmd.commit().unwrap();
            l
        };
        assert!(OcAxisAttr::find(&label).is_some());
    }

    #[test]
    fn axis_forget_removes_tag() {
        let (_app, mut doc) = new_doc();
        let label = {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let l = main.get_or_create_child(&cmd, 1);
            let axis = OcAx1::new(OcPnt::origin(), OcDir::new(0.0, 1.0, 0.0).unwrap());
            OcAxisAttr::record_shape(&cmd, &l, axis).unwrap();
            OcAxisAttr::set(&cmd, &l).unwrap();
            cmd.commit().unwrap();
            l
        };
        {
            let cmd = doc.begin_command().unwrap();
            assert!(OcAxisAttr::forget(&cmd, &label));
            cmd.commit().unwrap();
        }
        assert!(OcAxisAttr::find(&label).is_none());
    }

    #[test]
    fn axis_undo_restores() {
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
            let axis = OcAx1::new(OcPnt::origin(), OcDir::new(0.0, 0.0, 1.0).unwrap());
            OcAxisAttr::record_shape(&cmd, &label, axis).unwrap();
            OcAxisAttr::set(&cmd, &label).unwrap();
            cmd.commit().unwrap();
        }
        assert!(OcAxisAttr::find(&label).is_some());
        doc.undo().unwrap();
        assert!(OcAxisAttr::find(&label).is_none());
    }

    // ── OcPlaneAttr ──────────────────────────────────────────────────────────

    #[test]
    fn plane_set_and_find() {
        let (_app, mut doc) = new_doc();
        let label = {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let l = main.get_or_create_child(&cmd, 1);
            let frame = OcAx2::new(
                OcPnt::origin(),
                OcDir::new(0.0, 0.0, 1.0).unwrap(),
                OcDir::new(1.0, 0.0, 0.0).unwrap(),
            )
            .unwrap();
            OcPlaneAttr::record_shape(&cmd, &l, frame).unwrap();
            OcPlaneAttr::set(&cmd, &l).unwrap();
            cmd.commit().unwrap();
            l
        };
        assert!(OcPlaneAttr::find(&label).is_some());
    }

    #[test]
    fn plane_record_shape_usable_as_constraint_plane() {
        // The returned NamedShape can be used as the plane of a 2D constraint.
        let (_app, mut _doc) = new_doc();
        let (_app, mut doc) = new_doc();
        let label = {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let plane_label = main.get_or_create_child(&cmd, 1);
            let geom_label = main.get_or_create_child(&cmd, 2);
            let c_label = main.get_or_create_child(&cmd, 3);

            let frame = OcAx2::new(
                OcPnt::origin(),
                OcDir::new(0.0, 0.0, 1.0).unwrap(),
                OcDir::new(1.0, 0.0, 0.0).unwrap(),
            )
            .unwrap();
            let plane_ns = OcPlaneAttr::record_shape(&cmd, &plane_label, frame).unwrap();
            OcPlaneAttr::set(&cmd, &plane_label).unwrap();

            let geom_ns =
                OcPointAttr::record_shape(&cmd, &geom_label, OcPnt::new(1.0, 0.0, 0.0)).unwrap();
            OcPointAttr::set(&cmd, &geom_label).unwrap();

            let mut c =
                OcConstraintAttr::set(&cmd, &c_label, ConstraintKind::Fix, &[&geom_ns]).unwrap();
            c.set_plane(&cmd, &plane_ns);

            cmd.commit().unwrap();
            plane_label
        };
        assert!(OcPlaneAttr::find(&label).is_some());
    }

    #[test]
    fn plane_forget_removes_tag() {
        let (_app, mut doc) = new_doc();
        let label = {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let l = main.get_or_create_child(&cmd, 1);
            let frame = OcAx2::new(
                OcPnt::origin(),
                OcDir::new(0.0, 0.0, 1.0).unwrap(),
                OcDir::new(1.0, 0.0, 0.0).unwrap(),
            )
            .unwrap();
            OcPlaneAttr::record_shape(&cmd, &l, frame).unwrap();
            OcPlaneAttr::set(&cmd, &l).unwrap();
            cmd.commit().unwrap();
            l
        };
        {
            let cmd = doc.begin_command().unwrap();
            assert!(OcPlaneAttr::forget(&cmd, &label));
            cmd.commit().unwrap();
        }
        assert!(OcPlaneAttr::find(&label).is_none());
    }

    #[test]
    fn plane_undo_restores() {
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
            let frame = OcAx2::new(
                OcPnt::origin(),
                OcDir::new(0.0, 0.0, 1.0).unwrap(),
                OcDir::new(1.0, 0.0, 0.0).unwrap(),
            )
            .unwrap();
            OcPlaneAttr::record_shape(&cmd, &label, frame).unwrap();
            OcPlaneAttr::set(&cmd, &label).unwrap();
            cmd.commit().unwrap();
        }
        assert!(OcPlaneAttr::find(&label).is_some());
        doc.undo().unwrap();
        assert!(OcPlaneAttr::find(&label).is_none());
    }

    #[test]
    fn type_on_label_point() {
        let (_app, mut doc) = new_doc();
        let label = {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let l = main.get_or_create_child(&cmd, 1);
            OcPointAttr::record_shape(&cmd, &l, OcPnt::new(1.0, 2.0, 3.0)).unwrap();
            OcPointAttr::set(&cmd, &l).unwrap();
            cmd.commit().unwrap();
            l
        };
        assert_eq!(
            OcGeometryAttr::type_on_label(&label).unwrap(),
            GeometryKind::Point
        );
    }

    #[test]
    fn type_on_label_line() {
        let (_app, mut doc) = new_doc();
        let label = {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let l = main.get_or_create_child(&cmd, 1);
            let axis = OcAx1::new(OcPnt::origin(), OcDir::new(0.0, 0.0, 1.0).unwrap());
            OcAxisAttr::record_shape(&cmd, &l, axis).unwrap();
            OcAxisAttr::set(&cmd, &l).unwrap();
            cmd.commit().unwrap();
            l
        };
        assert_eq!(
            OcGeometryAttr::type_on_label(&label).unwrap(),
            GeometryKind::Line
        );
    }

    #[test]
    fn type_on_label_plane() {
        let (_app, mut doc) = new_doc();
        let label = {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let l = main.get_or_create_child(&cmd, 1);
            let frame = OcAx2::new(
                OcPnt::origin(),
                OcDir::new(0.0, 0.0, 1.0).unwrap(),
                OcDir::new(1.0, 0.0, 0.0).unwrap(),
            )
            .unwrap();
            OcPlaneAttr::record_shape(&cmd, &l, frame).unwrap();
            OcPlaneAttr::set(&cmd, &l).unwrap();
            cmd.commit().unwrap();
            l
        };
        assert_eq!(
            OcGeometryAttr::type_on_label(&label).unwrap(),
            GeometryKind::Plane
        );
    }

    #[test]
    fn type_on_label_without_named_shape_returns_any() {
        let (_app, mut doc) = new_doc();
        let label = {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let l = main.get_or_create_child(&cmd, 1);
            cmd.commit().unwrap();
            l
        };
        assert_eq!(
            OcGeometryAttr::type_on_label(&label).unwrap(),
            GeometryKind::Any
        );
    }

    #[test]
    fn point_get_round_trips() {
        let (_app, mut doc) = new_doc();
        let label = {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let l = main.get_or_create_child(&cmd, 1);
            OcPointAttr::record_shape(&cmd, &l, OcPnt::new(3.0, 4.0, 5.0)).unwrap();
            OcPointAttr::set(&cmd, &l).unwrap();
            cmd.commit().unwrap();
            l
        };
        let p = OcPointAttr::get(&label)
            .unwrap()
            .expect("should be present");
        assert!((p.x - 3.0).abs() < 1e-12);
        assert!((p.y - 4.0).abs() < 1e-12);
        assert!((p.z - 5.0).abs() < 1e-12);
    }

    #[test]
    fn axis_get_returns_kind_and_named_shape() {
        let (_app, mut doc) = new_doc();
        let label = {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let l = main.get_or_create_child(&cmd, 1);
            let axis = OcAx1::new(OcPnt::origin(), OcDir::new(0.0, 0.0, 1.0).unwrap());
            OcAxisAttr::record_shape(&cmd, &l, axis).unwrap();
            OcAxisAttr::set(&cmd, &l).unwrap();
            cmd.commit().unwrap();
            l
        };
        let (kind, _ns) = OcAxisAttr::get(&label).unwrap().expect("should be present");
        assert_eq!(kind, GeometryKind::Line);
    }

    #[test]
    fn plane_get_returns_kind_and_named_shape() {
        let (_app, mut doc) = new_doc();
        let label = {
            let main = doc.main();
            let cmd = doc.begin_command().unwrap();
            let l = main.get_or_create_child(&cmd, 1);
            let frame = OcAx2::new(
                OcPnt::origin(),
                OcDir::new(0.0, 0.0, 1.0).unwrap(),
                OcDir::new(1.0, 0.0, 0.0).unwrap(),
            )
            .unwrap();
            OcPlaneAttr::record_shape(&cmd, &l, frame).unwrap();
            OcPlaneAttr::set(&cmd, &l).unwrap();
            cmd.commit().unwrap();
            l
        };
        let (kind, _ns) = OcPlaneAttr::get(&label)
            .unwrap()
            .expect("should be present");
        assert_eq!(kind, GeometryKind::Plane);
    }
}
