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

#include "edge_polygon.hxx"
#include "wire.hxx"
#include "Poly_Triangulation.hxx"

struct MakeFaceBuilder {
    BRepBuilderAPI_MakeFace inner;
    MakeFaceBuilder(const TopoDS_Wire& w, bool only_plane)
        : inner(w, only_plane ? Standard_True : Standard_False) {}
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

// TopoDS_Face copy shares the underlying TShape handle (ref-counted).
inline std::unique_ptr<TopoDS_Face> clone_face(const TopoDS_Face& f) {
    return std::make_unique<TopoDS_Face>(f);
}

inline std::unique_ptr<TopoDS_Wire> face_outer_wire(const TopoDS_Face& f) {
    return std::make_unique<TopoDS_Wire>(BRepTools::OuterWire(f));
}
// ── Triangulation extraction ──────────────────────────────────────────────
// Get the triangulation from a face as an opaque PolyTriangulation.
// Returns nullptr if the face has no triangulation.
// Reference: https://dev.opencascade.org/doc/refman/html/class_b_rep___tool.html
inline std::unique_ptr<PolyTriangulation> face_triangulation_polygon(const TopoDS_Face& f) {
    TopLoc_Location loc;
    Handle(Poly_Triangulation) tri = BRep_Tool::Triangulation(f, loc);
    if (tri.IsNull()) {
        return nullptr;
    }
    auto result = std::make_unique<PolyTriangulation>();
    result->inner = tri;
    return result;
}
