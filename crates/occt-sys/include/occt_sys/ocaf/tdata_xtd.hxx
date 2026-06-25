// TDataXtd_Geometry tags a label with a geometric construction kind
// (ANY_GEOM, POINT, LINE, ...).  It qualifies the shape stored in the
// co-located TNaming_NamedShape but is a separate attribute with its own GUID.
//
// TDataXtd_Constraint records a constraint declaration on a label: its kind
// and references to 1–4 TNaming_NamedShape attributes (not TDataXtd_Geometry
// handles — the constraint binds topology, not the qualifier tag).
// Dimensional constraints additionally reference a TDataStd_Real attribute
// on a sub-label.
//
// Both types follow the standard Handle-in-shim pattern used throughout
// attributes.hxx: the shim struct owns the Handle by value; all functions
// are inline free functions returning UniquePtr<ShimT>.
//
// GUIDs stay on the C++ side: find helpers call FindAttribute(GetID(), ...)
// internally.
//
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___geometry.html
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___constraint.html
// Reference: https://dev.opencascade.org/doc/refman/html/group__enum__t_data_xtd.html
//
// Sourced from OCCT 8.0 documentation (API stable since 7.x for these types).
// No derivation from any other binding crate.

#pragma once

#include <memory>

#include <TDataXtd_Geometry.hxx>
#include <TDataXtd_GeometryEnum.hxx>
#include <TDataXtd_Constraint.hxx>
#include <TDataXtd_ConstraintEnum.hxx>
#include <TDataXtd_Position.hxx>
#include <TDataXtd_Point.hxx>
#include <TDataXtd_Axis.hxx>
#include <TDataXtd_Plane.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <TNaming_NamedShape.hxx>
#include <TDataStd_Real.hxx>
#include <TDF_Label.hxx>
#include <gp_Pnt.hxx>
#include <gp_Lin.hxx>
#include <gp_Pln.hxx>
#include <gp_Ax1.hxx>
#include <gp_Ax2.hxx>
#include <gp_Dir.hxx>
#include <gp_Pnt.hxx>
#include <TopoDS_Vertex.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>



#include "label.hxx"
#include "attributes.hxx"
#include "tnaming.hxx"
#include "../exception.hxx"
#include "rust/cxx.h"

// ── TDataXtd_Geometry ────────────────────────────────────────────────────────
//
// TDataXtd_Geometry is a TDF_Attribute that tags a label with a geometry kind
// (TDataXtd_GeometryEnum).  Attach it to the same label as a
// TNaming_NamedShape to declare what kind of geometry the stored shape
// represents.
//
// API notes (sourced from refman):
//   Set(label)        — static; finds or creates the attribute on label,
//                       type defaults to TDataXtd_ANY_GEOM.  Must be called
//                       inside an open command.  Returns Handle.
//   SetType(T)        — non-static; updates the geometry kind on an existing
//                       handle.  Must be called inside an open command.
//   GetType() const   — reads the geometry kind from a handle.
//   Type(label)       — static; reads the kind directly from the label.
//                       Raises if no attribute is present.
//   GetID()           — static GUID accessor; used by FindAttribute.
//
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___geometry.html

struct TDataXtdGeometryHandle {
    Handle(TDataXtd_Geometry) inner;
};

// Set(label, geom_type) — finds or creates a TDataXtd_Geometry attribute on
// label with the given type set atomically before AddAttribute.
//
// When creating a new attribute, SetType is called on the raw object before
// AddAttribute so the type is part of the single AddAttribute delta that undo
// can cleanly reverse.  Calling SetType after AddAttribute would call Backup()
// on a freshly-added but not-yet-committed attribute, producing a corrupt undo
// delta that leaves the attribute on the label after undo.
//
// When the attribute already exists, SetType is called on the existing handle;
// Backup() then operates correctly on the committed state.
//
// Must be called inside an open command scope.
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___geometry.html
inline std::unique_ptr<TDataXtdGeometryHandle> tdataxtd_geometry_set(
    const TdfLabel& label, int geom_type)
{
    try {
        auto result = std::make_unique<TDataXtdGeometryHandle>();
        Handle(TDataXtd_Geometry) A;
        if (!label.inner.FindAttribute(TDataXtd_Geometry::GetID(), A)) {
            A = new TDataXtd_Geometry();
            A->SetType(static_cast<TDataXtd_GeometryEnum>(geom_type));
            label.inner.AddAttribute(A);
        } else {
            A->SetType(static_cast<TDataXtd_GeometryEnum>(geom_type));
        }
        result->inner = A;
        return result;
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}

// SetType(T) — non-const instance method.
// Updates the geometry kind on an already-committed attribute.
// Safe to call on an existing attribute: Backup() operates on committed state.
// Do NOT call this on a freshly-created attribute before its first commit —
// use tdataxtd_geometry_set with the desired type instead.
// Must be called inside an open command scope.
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___geometry.html
inline void tdataxtd_geometry_set_type(
    TDataXtdGeometryHandle& h, int geom_type)
{
    h.inner->SetType(static_cast<TDataXtd_GeometryEnum>(geom_type));
}

// GetType() const — reads the geometry kind from a handle.
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___geometry.html
inline int tdataxtd_geometry_get_type(const TDataXtdGeometryHandle& h) {
    return static_cast<int>(h.inner->GetType());
}

// FindAttribute pattern — returns nullptr if attribute is absent.
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___geometry.html
inline std::unique_ptr<TDataXtdGeometryHandle> tdataxtd_geometry_find(
    const TdfLabel& label)
{
    Handle(TDataXtd_Geometry) attr;
    if (label.inner.FindAttribute(TDataXtd_Geometry::GetID(), attr)) {
        auto result = std::make_unique<TDataXtdGeometryHandle>();
        result->inner = attr;
        return result;
    }
    return nullptr;
}

// ForgetAttribute — removes TDataXtd_Geometry from label.
// Returns false if not present.  Must be inside an open command.
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_d_f___label.html
inline bool tdataxtd_geometry_forget(const TdfLabel& label) {
    return label.inner.ForgetAttribute(TDataXtd_Geometry::GetID()) == Standard_True;
}

// ── TDataXtd_Constraint ──────────────────────────────────────────────────────
//
// TDataXtd_Constraint records a constraint on a label.  Each geometry
// participant is a Handle(TNaming_NamedShape) — the topology attribute, NOT
// a TDataXtd_Geometry handle (the geometry qualifier tag is unrelated).
//
// API notes (sourced from refman 8.0 / stable since 7.x):
//   Set(type, ns1)            — instance method (non-static); 1-geometry form.
//   Set(type, ns1, ns2)       — 2-geometry form.
//   Set(type, ns1, ns2, ns3)  — 3-geometry form.
//   Set(type, ns1, ..., ns4)  — 4-geometry form.
//   All Set() overloads must be called inside an open command.
//
//   SetGeometry(index, ns)    — sets or replaces a geometry reference (1-based).
//   SetValue(real_handle)     — associates a TDataStd_Real as the dimension value.
//   SetType(ctr)              — updates the constraint kind.
//   GetType() const           — reads the constraint kind.
//   NbGeometries() const      — count of geometry references (1–4).
//   GetGeometry(index) const  — 1-based accessor; returns null handle if OOB.
//   IsDimension() const       — true when a value attribute is associated.
//   GetValue() const          — returns Handle(TDataStd_Real); null if not dimensional.
//   Verified() const / Verified(bool) — solver validity flag.
//   IsPlanar() const / GetPlane() const / SetPlane(ns) — 2D constraint support.
//   GetID()                   — static GUID accessor.
//
// We expose a unified tdataxtd_constraint_set_n(label, type, ns_array, count)
// shim to avoid four separate bridge functions for the C++ overload set.
//
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___constraint.html

struct TDataXtdConstraintHandle {
    Handle(TDataXtd_Constraint) inner;
};

// tdataxtd_constraint_ensure_handle — returns an existing TDataXtd_Constraint
// handle from label, or nullptr if absent.  Used by set shims to decide
// whether to create and AddAttribute a new one.
// The set shims call Set() on the raw object BEFORE AddAttribute so that
// Backup() inside Set() is a no-op (attribute has no label yet), and
// AddAttribute records the fully-initialized state as the single undo delta.
static inline Handle(TDataXtd_Constraint) tdataxtd_constraint_find_existing(
    const TDF_Label& label)
{
    Handle(TDataXtd_Constraint) attr;
    label.FindAttribute(TDataXtd_Constraint::GetID(), attr);
    return attr; // null handle if absent
}

// Set with one TNaming_NamedShape geometry reference.
// Must be called inside an open command scope.
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___constraint.html
inline std::unique_ptr<TDataXtdConstraintHandle> tdataxtd_constraint_set1(
    const TdfLabel& label,
    int constraint_type,
    const TnamingNamedShapeHandle& g1)
{
    try {
        auto result = std::make_unique<TDataXtdConstraintHandle>();
        Handle(TDataXtd_Constraint) attr =
            tdataxtd_constraint_find_existing(label.inner);
        if (attr.IsNull()) {
            attr = new TDataXtd_Constraint();
            attr->Set(static_cast<TDataXtd_ConstraintEnum>(constraint_type),
                      g1.inner);
            label.inner.AddAttribute(attr);
        } else {
            attr->Set(static_cast<TDataXtd_ConstraintEnum>(constraint_type),
                      g1.inner);
        }
        result->inner = attr;
        return result;
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}

// Set with two TNaming_NamedShape geometry references.
// Must be called inside an open command scope.
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___constraint.html
inline std::unique_ptr<TDataXtdConstraintHandle> tdataxtd_constraint_set2(
    const TdfLabel& label,
    int constraint_type,
    const TnamingNamedShapeHandle& g1,
    const TnamingNamedShapeHandle& g2)
{
    try {
        auto result = std::make_unique<TDataXtdConstraintHandle>();
        Handle(TDataXtd_Constraint) attr =
            tdataxtd_constraint_find_existing(label.inner);
        if (attr.IsNull()) {
            attr = new TDataXtd_Constraint();
            attr->Set(static_cast<TDataXtd_ConstraintEnum>(constraint_type),
                      g1.inner, g2.inner);
            label.inner.AddAttribute(attr);
        } else {
            attr->Set(static_cast<TDataXtd_ConstraintEnum>(constraint_type),
                      g1.inner, g2.inner);
        }
        result->inner = attr;
        return result;
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}

// Set with three TNaming_NamedShape geometry references.
// Must be called inside an open command scope.
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___constraint.html
inline std::unique_ptr<TDataXtdConstraintHandle> tdataxtd_constraint_set3(
    const TdfLabel& label,
    int constraint_type,
    const TnamingNamedShapeHandle& g1,
    const TnamingNamedShapeHandle& g2,
    const TnamingNamedShapeHandle& g3)
{
    try {
        auto result = std::make_unique<TDataXtdConstraintHandle>();
        Handle(TDataXtd_Constraint) attr =
            tdataxtd_constraint_find_existing(label.inner);
        if (attr.IsNull()) {
            attr = new TDataXtd_Constraint();
            attr->Set(static_cast<TDataXtd_ConstraintEnum>(constraint_type),
                      g1.inner, g2.inner, g3.inner);
            label.inner.AddAttribute(attr);
        } else {
            attr->Set(static_cast<TDataXtd_ConstraintEnum>(constraint_type),
                      g1.inner, g2.inner, g3.inner);
        }
        result->inner = attr;
        return result;
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}

// Set with four TNaming_NamedShape geometry references.
// Must be called inside an open command scope.
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___constraint.html
inline std::unique_ptr<TDataXtdConstraintHandle> tdataxtd_constraint_set4(
    const TdfLabel& label,
    int constraint_type,
    const TnamingNamedShapeHandle& g1,
    const TnamingNamedShapeHandle& g2,
    const TnamingNamedShapeHandle& g3,
    const TnamingNamedShapeHandle& g4)
{
    try {
        auto result = std::make_unique<TDataXtdConstraintHandle>();
        Handle(TDataXtd_Constraint) attr =
            tdataxtd_constraint_find_existing(label.inner);
        if (attr.IsNull()) {
            attr = new TDataXtd_Constraint();
            attr->Set(static_cast<TDataXtd_ConstraintEnum>(constraint_type),
                      g1.inner, g2.inner, g3.inner, g4.inner);
            label.inner.AddAttribute(attr);
        } else {
            attr->Set(static_cast<TDataXtd_ConstraintEnum>(constraint_type),
                      g1.inner, g2.inner, g3.inner, g4.inner);
        }
        result->inner = attr;
        return result;
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}

// SetGeometry(index, ns) — sets or replaces a geometry reference (1-based).
// Non-const; must be called inside an open command scope.
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___constraint.html
inline void tdataxtd_constraint_set_geometry(
    TDataXtdConstraintHandle& c,
    int index,
    const TnamingNamedShapeHandle& ns)
{
    c.inner->SetGeometry(index, ns.inner);
}

// SetValue(real_handle) — associates a TDataStd_Real as the dimension value.
// Non-const; must be called inside an open command scope.
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___constraint.html
inline void tdataxtd_constraint_set_value(
    TDataXtdConstraintHandle& c,
    const TDataStdRealHandle& val)
{
    c.inner->SetValue(val.inner);
}

// SetType(ctr) — updates the constraint kind.
// Non-const; must be called inside an open command scope.
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___constraint.html
inline void tdataxtd_constraint_set_type(
    TDataXtdConstraintHandle& c, int constraint_type)
{
    c.inner->SetType(static_cast<TDataXtd_ConstraintEnum>(constraint_type));
}

// GetType() const.
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___constraint.html
inline int tdataxtd_constraint_get_type(const TDataXtdConstraintHandle& c) {
    return static_cast<int>(c.inner->GetType());
}

// NbGeometries() const — count of geometry references (1–4).
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___constraint.html
inline int tdataxtd_constraint_nb_geometries(const TDataXtdConstraintHandle& c) {
    return c.inner->NbGeometries();
}

// GetGeometry(index) const — 1-based.
// Returns nullptr if index is out of range or the slot is null.
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___constraint.html
inline std::unique_ptr<TnamingNamedShapeHandle> tdataxtd_constraint_get_geometry(
    const TDataXtdConstraintHandle& c, int index)
{
    Handle(TNaming_NamedShape) ns = c.inner->GetGeometry(index);
    if (ns.IsNull()) return nullptr;
    auto result = std::make_unique<TnamingNamedShapeHandle>();
    result->inner = ns;
    return result;
}

// IsDimension() const.
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___constraint.html
inline bool tdataxtd_constraint_is_dimension(const TDataXtdConstraintHandle& c) {
    return c.inner->IsDimension();
}

// GetValue() const — returns nullptr when IsDimension() is false.
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___constraint.html
inline std::unique_ptr<TDataStdRealHandle> tdataxtd_constraint_get_value(
    const TDataXtdConstraintHandle& c)
{
    if (!c.inner->IsDimension()) return nullptr;
    Handle(TDataStd_Real) r = c.inner->GetValue();
    if (r.IsNull()) return nullptr;
    auto result = std::make_unique<TDataStdRealHandle>();
    result->inner = r;
    return result;
}

// Verified() const — solver validity flag.
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___constraint.html
inline bool tdataxtd_constraint_verified(const TDataXtdConstraintHandle& c) {
    return c.inner->Verified();
}

// Verified(bool) — sets solver validity.
// Non-const; must be called inside an open command scope.
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___constraint.html
inline void tdataxtd_constraint_set_verified(
    TDataXtdConstraintHandle& c, bool status)
{
    c.inner->Verified(status);
}

// IsPlanar() const — true when this is a 2D (planar) constraint.
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___constraint.html
inline bool tdataxtd_constraint_is_planar(const TDataXtdConstraintHandle& c) {
    return c.inner->IsPlanar();
}

// GetPlane() const — the TNaming_NamedShape of the constraint plane.
// Returns nullptr when IsPlanar() is false.
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___constraint.html
inline std::unique_ptr<TnamingNamedShapeHandle> tdataxtd_constraint_get_plane(
    const TDataXtdConstraintHandle& c)
{
    if (!c.inner->IsPlanar()) return nullptr;
    const Handle(TNaming_NamedShape)& ns = c.inner->GetPlane();
    if (ns.IsNull()) return nullptr;
    auto result = std::make_unique<TnamingNamedShapeHandle>();
    result->inner = ns;
    return result;
}

// SetPlane(ns) — sets the plane of a 2D constraint.
// Non-const; must be called inside an open command scope.
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___constraint.html
inline void tdataxtd_constraint_set_plane(
    TDataXtdConstraintHandle& c,
    const TnamingNamedShapeHandle& plane)
{
    c.inner->SetPlane(plane.inner);
}

// FindAttribute pattern — returns nullptr if attribute is absent.
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___constraint.html
inline std::unique_ptr<TDataXtdConstraintHandle> tdataxtd_constraint_find(
    const TdfLabel& label)
{
    Handle(TDataXtd_Constraint) attr;
    if (label.inner.FindAttribute(TDataXtd_Constraint::GetID(), attr)) {
        auto result = std::make_unique<TDataXtdConstraintHandle>();
        result->inner = attr;
        return result;
    }
    return nullptr;
}

// ForgetAttribute — removes TDataXtd_Constraint from label.
// Returns false if not present.  Must be inside an open command.
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_d_f___label.html
inline bool tdataxtd_constraint_forget(const TdfLabel& label) {
    return label.inner.ForgetAttribute(TDataXtd_Constraint::GetID()) == Standard_True;
}

// ── TDataXtd_Position ────────────────────────────────────────────────────────
//
// API notes (sourced from TDataXtd_Position.hxx / .cxx, local headers):
//   Set(label, gp_Pnt) — static void; finds or creates attribute, calls
//                         SetPosition(pnt).  SetPosition calls Backup().
//   Set(label)         — static; finds or creates with default pos (0,0,0).
//   Get(label, gp_Pnt&)— static bool; writes position into out-param.
//                         Returns false when attribute is absent.
//   GetPosition() const— instance method; reads stored gp_Pnt.
//   SetPosition(pnt)   — non-static; updates stored pnt, calls Backup().
//   GetID()            — static GUID accessor.
//
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___position.html
 
struct TDataXtdPositionHandle {
    Handle(TDataXtd_Position) inner;
};
 
// Set(label, x, y, z) — finds or creates a TDataXtd_Position attribute on
// label with the given position set atomically before AddAttribute.
//
// When creating a new attribute, SetPosition is called on the raw object
// before AddAttribute so the position is part of the single AddAttribute
// delta that undo can cleanly reverse.  When the attribute already exists,
// SetPosition is called on the committed handle; Backup() then operates
// correctly on the committed state.
//
// Must be called inside an open command scope.
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___position.html
inline std::unique_ptr<TDataXtdPositionHandle> tdataxtd_position_set(
    const TdfLabel& label, double x, double y, double z)
{
    try {
        auto result = std::make_unique<TDataXtdPositionHandle>();
        Handle(TDataXtd_Position) A;
        gp_Pnt pnt(x, y, z);
        if (!label.inner.FindAttribute(TDataXtd_Position::GetID(), A)) {
            A = new TDataXtd_Position();
            A->SetPosition(pnt);   // Backup() no-op on unattached object
            label.inner.AddAttribute(A);
        } else {
            A->SetPosition(pnt);   // Backup() on committed state — correct
        }
        result->inner = A;
        return result;
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}
 
// SetPosition(x, y, z) — updates position on an already-committed attribute.
// Calls Backup() on committed state; correct for use in a subsequent command.
// Must be called inside an open command scope.
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___position.html
inline void tdataxtd_position_set_position(
    TDataXtdPositionHandle& h, double x, double y, double z)
{
    h.inner->SetPosition(gp_Pnt(x, y, z));
}
 
// GetPosition() const — reads the stored gp_Pnt; decomposes to scalars so
// no gp type crosses the cxx bridge.
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___position.html
inline void tdataxtd_position_get_position(
    const TDataXtdPositionHandle& h,
    double& x, double& y, double& z)
{
    const gp_Pnt& p = h.inner->GetPosition();
    x = p.X(); y = p.Y(); z = p.Z();
}
 
// FindAttribute pattern — returns nullptr if attribute is absent.
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___position.html
inline std::unique_ptr<TDataXtdPositionHandle> tdataxtd_position_find(
    const TdfLabel& label)
{
    Handle(TDataXtd_Position) attr;
    if (label.inner.FindAttribute(TDataXtd_Position::GetID(), attr)) {
        auto result = std::make_unique<TDataXtdPositionHandle>();
        result->inner = attr;
        return result;
    }
    return nullptr;
}
 
// ForgetAttribute — removes TDataXtd_Position from label.
// Returns false if not present.  Must be inside an open command.
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_d_f___label.html
inline bool tdataxtd_position_forget(const TdfLabel& label) {
    return label.inner.ForgetAttribute(TDataXtd_Position::GetID()) == Standard_True;
}
// ── Shape constructors (gp → TopoDS) ─────────────────────────────────────────
//
// These are free functions used by the Option B safe API to build the shape
// before passing it to TnamingBuilderShim::generated_fresh.  They are not
// specific to TDataXtd but live here because they exist solely to support
// these three attribute types.
 
// make_vertex_shape — BRepBuilderAPI_MakeVertex from point coordinates.
// Returns a TopoDS_Vertex for use with generated_fresh.
// Reference: https://dev.opencascade.org/doc/refman/html/class_b_rep_builder_a_p_i___make_vertex.html
inline std::unique_ptr<TopoDS_Vertex> tdataxtd_make_vertex_shape(
    double x, double y, double z)
{
    try {
        auto result = std::make_unique<TopoDS_Vertex>();
        *result = BRepBuilderAPI_MakeVertex(gp_Pnt(x, y, z));
        return result;
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}
 
// make_infinite_edge_from_ax1 — BRepBuilderAPI_MakeEdge(gp_Lin) from Ax1
// scalars (origin + direction).  gp_Lin and gp_Ax1 are structurally
// identical; gp_Lin has a constructor from gp_Ax1.
// Returns a TopoDS_Edge for use with generated_fresh.
// Reference: https://dev.opencascade.org/doc/refman/html/class_b_rep_builder_a_p_i___make_edge.html
inline std::unique_ptr<TopoDS_Edge> tdataxtd_make_infinite_edge_from_ax1(
    double ox, double oy, double oz,
    double dx, double dy, double dz)
{
    try {
        auto result = std::make_unique<TopoDS_Edge>();
        gp_Ax1 ax1(gp_Pnt(ox, oy, oz), gp_Dir(dx, dy, dz));
        gp_Lin lin(ax1);
        *result = BRepBuilderAPI_MakeEdge(lin);
        return result;
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}
 
// make_face_from_ax2 — BRepBuilderAPI_MakeFace(gp_Pln) from Ax2 scalars
// (origin + normal + x_direction).  gp_Pln is constructed from gp_Ax3;
// gp_Ax3 has a constructor from gp_Ax2.
// Returns a TopoDS_Face for use with generated_fresh.
// Reference: https://dev.opencascade.org/doc/refman/html/class_b_rep_builder_a_p_i___make_face.html
inline std::unique_ptr<TopoDS_Face> tdataxtd_make_face_from_ax2(
    double ox, double oy, double oz,
    double nx, double ny, double nz,
    double xx, double xy, double xz)
{
    try {
        auto result = std::make_unique<TopoDS_Face>();
        gp_Ax2 ax2(gp_Pnt(ox, oy, oz), gp_Dir(nx, ny, nz), gp_Dir(xx, xy, xz));
        gp_Pln pln(ax2);
        *result = BRepBuilderAPI_MakeFace(pln);
        return result;
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}
 
// ── TDataXtd_Point ───────────────────────────────────────────────────────────
//
// Tag attribute: marks a label whose TNaming_NamedShape contains a vertex.
// Set(label) — static; finds or creates the marker.  No geometry here.
// API notes (sourced from TDataXtd_Point.hxx):
//   Set(label)      — static; finds or creates the marker attribute.
//   GetID()         — static GUID accessor.
//
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___point.html
 
struct TDataXtdPointHandle {
    Handle(TDataXtd_Point) inner;
};
 
// Set(label) — finds or creates the TDataXtd_Point tag attribute on label.
// Caller is responsible for having placed a vertex NamedShape on the label
// via TnamingBuilder::generated_fresh before or in the same command.
// Must be called inside an open command scope.
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___point.html
inline std::unique_ptr<TDataXtdPointHandle> tdataxtd_point_set(
    const TdfLabel& label)
{
    try {
        auto result = std::make_unique<TDataXtdPointHandle>();
        result->inner = TDataXtd_Point::Set(label.inner);
        return result;
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}
 
// FindAttribute pattern — returns nullptr if attribute is absent.
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___point.html
inline std::unique_ptr<TDataXtdPointHandle> tdataxtd_point_find(
    const TdfLabel& label)
{
    Handle(TDataXtd_Point) attr;
    if (label.inner.FindAttribute(TDataXtd_Point::GetID(), attr)) {
        auto result = std::make_unique<TDataXtdPointHandle>();
        result->inner = attr;
        return result;
    }
    return nullptr;
}
 
// ForgetAttribute — removes TDataXtd_Point from label.
// Returns false if not present.  Must be inside an open command.
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_d_f___label.html
inline bool tdataxtd_point_forget(const TdfLabel& label) {
    return label.inner.ForgetAttribute(TDataXtd_Point::GetID()) == Standard_True;
}
 
// ── TDataXtd_Axis ────────────────────────────────────────────────────────────
//
// Tag attribute: marks a label whose TNaming_NamedShape contains a linear edge.
// API notes (sourced from TDataXtd_Axis.hxx):
//   Set(label)      — static; finds or creates the marker attribute.
//   GetID()         — static GUID accessor.
//
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___axis.html
 
struct TDataXtdAxisHandle {
    Handle(TDataXtd_Axis) inner;
};
 
// Set(label) — finds or creates the TDataXtd_Axis tag attribute on label.
// Caller is responsible for having placed a linear edge NamedShape on the
// label via TnamingBuilder::generated_fresh before or in the same command.
// Must be called inside an open command scope.
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___axis.html
inline std::unique_ptr<TDataXtdAxisHandle> tdataxtd_axis_set(
    const TdfLabel& label)
{
    try {
        auto result = std::make_unique<TDataXtdAxisHandle>();
        result->inner = TDataXtd_Axis::Set(label.inner);
        return result;
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}
 
// FindAttribute pattern — returns nullptr if attribute is absent.
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___axis.html
inline std::unique_ptr<TDataXtdAxisHandle> tdataxtd_axis_find(
    const TdfLabel& label)
{
    Handle(TDataXtd_Axis) attr;
    if (label.inner.FindAttribute(TDataXtd_Axis::GetID(), attr)) {
        auto result = std::make_unique<TDataXtdAxisHandle>();
        result->inner = attr;
        return result;
    }
    return nullptr;
}
 
// ForgetAttribute — removes TDataXtd_Axis from label.
// Returns false if not present.  Must be inside an open command.
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_d_f___label.html
inline bool tdataxtd_axis_forget(const TdfLabel& label) {
    return label.inner.ForgetAttribute(TDataXtd_Axis::GetID()) == Standard_True;
}
 
// ── TDataXtd_Plane ───────────────────────────────────────────────────────────
//
// Tag attribute: marks a label whose TNaming_NamedShape contains a planar face.
// API notes (sourced from TDataXtd_Plane.hxx):
//   Set(label)      — static; finds or creates the marker attribute.
//   GetID()         — static GUID accessor.
//
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___plane.html
 
struct TDataXtdPlaneHandle {
    Handle(TDataXtd_Plane) inner;
};
 
// Set(label) — finds or creates the TDataXtd_Plane tag attribute on label.
// Caller is responsible for having placed a planar face NamedShape on the
// label via TnamingBuilder::generated_fresh before or in the same command.
// Must be called inside an open command scope.
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___plane.html
inline std::unique_ptr<TDataXtdPlaneHandle> tdataxtd_plane_set(
    const TdfLabel& label)
{
    try {
        auto result = std::make_unique<TDataXtdPlaneHandle>();
        result->inner = TDataXtd_Plane::Set(label.inner);
        return result;
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}
 
// FindAttribute pattern — returns nullptr if attribute is absent.
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___plane.html
inline std::unique_ptr<TDataXtdPlaneHandle> tdataxtd_plane_find(
    const TdfLabel& label)
{
    Handle(TDataXtd_Plane) attr;
    if (label.inner.FindAttribute(TDataXtd_Plane::GetID(), attr)) {
        auto result = std::make_unique<TDataXtdPlaneHandle>();
        result->inner = attr;
        return result;
    }
    return nullptr;
}
 
// ForgetAttribute — removes TDataXtd_Plane from label.
// Returns false if not present.  Must be inside an open command.
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_d_f___label.html
inline bool tdataxtd_plane_forget(const TdfLabel& label) {
    return label.inner.ForgetAttribute(TDataXtd_Plane::GetID()) == Standard_True;
}


