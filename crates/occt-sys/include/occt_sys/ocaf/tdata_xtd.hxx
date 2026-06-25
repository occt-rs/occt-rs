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
#include <TNaming_NamedShape.hxx>
#include <TDataStd_Real.hxx>
#include <TDF_Label.hxx>

#include "label.hxx"
#include "attributes.hxx"
#include "tdata_xtd.hxx"
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

// Set(label) — static.
// Finds or creates a TDataXtd_Geometry attribute on label.
// Type defaults to TDataXtd_ANY_GEOM; call tdataxtd_geometry_set_type to
// update it within the same command.
// Must be called inside an open command scope.
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___geometry.html
inline std::unique_ptr<TDataXtdGeometryHandle> tdataxtd_geometry_set(
    const TdfLabel& label)
{
    try {
        auto result = std::make_unique<TDataXtdGeometryHandle>();
        result->inner = TDataXtd_Geometry::Set(label.inner);
        return result;
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}

// SetType(T) — non-const instance method.
// Updates the geometry kind on an existing handle.
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

// Type(label) — static; reads the geometry kind from a label directly.
// NOTE: raises Standard_NoSuchObject (rethrown as runtime_error) if no
// TDataXtd_Geometry attribute is present on the label.  Callers should probe
// with tdataxtd_geometry_find before calling this if presence is uncertain.
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___geometry.html
inline int tdataxtd_geometry_type_on_label(const TdfLabel& label) {
    try {
        return static_cast<int>(TDataXtd_Geometry::Type(label.inner));
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
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

// tdataxtd_constraint_find_or_create — finds or creates the constraint
// attribute on label via FindAttribute / new TDataXtd_Constraint.
// Used internally by the set shims to obtain the handle to call Set() on.
// (TDataXtd_Constraint::Set() is a non-static member in OCCT 8.0 / 7.9.)
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___constraint.html
static inline Handle(TDataXtd_Constraint) tdataxtd_constraint_find_or_create(
    const TDF_Label& label)
{
    Handle(TDataXtd_Constraint) attr;
    if (!label.FindAttribute(TDataXtd_Constraint::GetID(), attr)) {
        attr = new TDataXtd_Constraint();
        label.AddAttribute(attr);
    }
    return attr;
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
        result->inner = tdataxtd_constraint_find_or_create(label.inner);
        result->inner->Set(
            static_cast<TDataXtd_ConstraintEnum>(constraint_type),
            g1.inner);
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
        result->inner = tdataxtd_constraint_find_or_create(label.inner);
        result->inner->Set(
            static_cast<TDataXtd_ConstraintEnum>(constraint_type),
            g1.inner, g2.inner);
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
        result->inner = tdataxtd_constraint_find_or_create(label.inner);
        result->inner->Set(
            static_cast<TDataXtd_ConstraintEnum>(constraint_type),
            g1.inner, g2.inner, g3.inner);
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
        result->inner = tdataxtd_constraint_find_or_create(label.inner);
        result->inner->Set(
            static_cast<TDataXtd_ConstraintEnum>(constraint_type),
            g1.inner, g2.inner, g3.inner, g4.inner);
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
