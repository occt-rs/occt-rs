// occt_sys/topo/solid.hxx — TopoDS_Solid and BRepPrimAPI_MakePrism.
//
// BRepPrimAPI_MakePrism sweeps a shape along a vector to produce a solid.
// Unlike BRepBuilderAPI_* builders it computes immediately in the constructor
// and throws Standard_Failure on failure — construction must be wrapped in
// try/catch.
//
// Shape() is non-const in BRepBuilderAPI_MakeShape; the solid() accessor
// therefore takes a mutable reference (Pin<&mut MakePrismBuilder> on the
// Rust side).
//
// History: MakePrism exposes Modified(), Generated(), and IsDeleted().
// The builder must not be dropped before history is queried.
//
// Reference:
//   BRepPrimAPI_MakePrism — https://dev.opencascade.org/doc/refman/html/class_b_rep_prim_a_p_i___make_prism.html
//   TopoDS_Solid          — https://dev.opencascade.org/doc/refman/html/class_topo_d_s___solid.html
//   TopoDS (downcast)     — https://dev.opencascade.org/doc/refman/html/class_topo_d_s.html
//
// Sourced from OCCT 7.9 documentation.
// No derivation from any other binding crate.

#pragma once

#include <memory>
#include <BRepPrimAPI_MakePrism.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Solid.hxx>
#include <TopoDS_Shape.hxx>
#include <gp_Vec.hxx>

#include "face.hxx"
#include "../exception.hxx"

struct MakePrismBuilder {
    BRepPrimAPI_MakePrism inner;
    // TopoDS_Face upcasts implicitly to TopoDS_Shape as required by MakePrism.
    MakePrismBuilder(const TopoDS_Face& face, double vx, double vy, double vz)
        : inner(face, gp_Vec(vx, vy, vz)) {}
    bool is_done() const { return inner.IsDone(); }
    std::unique_ptr<TopoDS_Solid> solid() {
        return std::make_unique<TopoDS_Solid>(TopoDS::Solid(inner.Shape()));
    }
};

// Reference: https://dev.opencascade.org/doc/refman/html/class_topo_d_s___shape.html
// TopAbs_ShapeEnum is a C++ enum; cast to int so cxx can cross it.
// Sourced from OCCT 7.9 documentation. No derivation from any other binding crate.
inline int topods_shape_type(const TopoDS_Shape& shape) {
    return static_cast<int>(shape.ShapeType());
}

// Factory.  Wraps construction in try/catch because MakePrism throws
// rather than deferring failure to IsDone().
inline std::unique_ptr<MakePrismBuilder> new_make_prism_from_face(
    const TopoDS_Face& face, double vx, double vy, double vz)
{
    try {
        return std::make_unique<MakePrismBuilder>(face, vx, vy, vz);
    } catch (const std::runtime_error&) {
        throw;  // don't double-wrap an already-marshalled exception
    } catch (...) {
        rethrow_occt_as_runtime_error();
    }
}

// TopoDS_Solid copy shares the underlying TShape handle (ref-counted).
inline std::unique_ptr<TopoDS_Solid> clone_solid(const TopoDS_Solid& s) {
    return std::make_unique<TopoDS_Solid>(s);
}
