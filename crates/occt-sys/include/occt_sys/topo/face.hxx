// occt_sys/topo/face.hxx — TopoDS_Face construction and outer wire access.
//
// Constructor: BRepBuilderAPI_MakeFace(const TopoDS_Wire&, Standard_Boolean)
//   only_plane = true  → rejects non-planar wires
//   only_plane = false → OCCT attempts to find a fitting surface
//
// Face() is non-const in BRepBuilderAPI_MakeFace; the accessor therefore
// takes a mutable reference (Pin<&mut MakeFaceBuilder> on the Rust side).
//
// Reference:
//   BRepBuilderAPI_MakeFace  — https://dev.opencascade.org/doc/refman/html/class_b_rep_builder_a_p_i___make_face.html
//   BRepBuilderAPI_FaceError — https://dev.opencascade.org/doc/refman/html/BRepBuilderAPI__FaceError_8hxx.html
//   BRepTools                — https://dev.opencascade.org/doc/refman/html/class_b_rep_tools.html
//   TopoDS_Face              — https://dev.opencascade.org/doc/refman/html/class_topo_d_s___face.html
//
// Sourced from OCCT 7.9 documentation.
// No derivation from any other binding crate.

#pragma once

#include <memory>
#include <BRepBuilderAPI_FaceError.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepTools.hxx>
#include <TopoDS_Face.hxx>
#include <gp_Pln.hxx>

#include "wire.hxx"
#include "../exception.hxx"

struct MakeFaceBuilder {
    BRepBuilderAPI_MakeFace inner;
    MakeFaceBuilder(const TopoDS_Wire& w, bool only_plane)
        : inner(w, only_plane ? Standard_True : Standard_False) {}
    // BRepBuilderAPI_MakeFace(const gp_Pln&, const TopoDS_Wire&, Standard_Boolean Inside)
    // Inside=Standard_True: wire is the outer boundary; face is the bounded interior.
    // Reference: https://dev.opencascade.org/doc/refman/html/class_b_rep_builder_a_p_i___make_face.html
    MakeFaceBuilder(const gp_Pln& pln, const TopoDS_Wire& w)
        : inner(pln, w, Standard_True) {}
    bool is_done() const { return inner.IsDone(); }
    int  error()   const { return static_cast<int>(inner.Error()); }
    std::unique_ptr<TopoDS_Face> face() {
        return std::make_unique<TopoDS_Face>(inner.Face());
    }
};

inline std::unique_ptr<MakeFaceBuilder> new_make_face_from_wire(
    const TopoDS_Wire& w, bool only_plane)
{
    return std::make_unique<MakeFaceBuilder>(w, only_plane);
}

// Constructs a face on an explicitly provided plane.
// px/py/pz: a point on the plane.  nx/ny/nz: the plane normal (need not be unit).
// gp_Dir normalises the components; throws Standard_ConstructionError on zero magnitude.
// gp_Pln reference: https://dev.opencascade.org/doc/refman/html/classgp___pln.html
inline std::unique_ptr<MakeFaceBuilder> new_make_face_from_plane_and_wire(
    double px, double py, double pz,
    double nx, double ny, double nz,
    const TopoDS_Wire& w)
{
    try {
        gp_Pln pln(gp_Pnt(px, py, pz), gp_Dir(nx, ny, nz));
        return std::make_unique<MakeFaceBuilder>(pln, w);
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}

// TopoDS_Face copy shares the underlying TShape handle (ref-counted).
inline std::unique_ptr<TopoDS_Face> clone_face(const TopoDS_Face& f) {
    return std::make_unique<TopoDS_Face>(f);
}

inline std::unique_ptr<TopoDS_Wire> face_outer_wire(const TopoDS_Face& f) {
    return std::make_unique<TopoDS_Wire>(BRepTools::OuterWire(f));
}
