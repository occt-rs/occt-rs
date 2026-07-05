// occt_sys/topo/shape.hxx — TopoDS_Shape utilities: clone, placed-instance
// identity key, up-casts (specific → shape), and down-casts (shape → specific).
//
// Up-casts are zero-cost reference casts (static_cast; no allocation).
// Down-casts use TopoDS::Face / TopoDS::Vertex / etc. which preserve
// orientation; they throw Standard_TypeMismatch on type mismatch.
// Callers must only invoke down-casts on shapes obtained from a TopExp_Explorer
// or other context that guarantees the correct shape type.  On mismatch, the
// cxx bridge terminates the process (no UB).
//
// shape_key encodes TShape identity + Location + Orientation.
// shape_tshape_ptr encodes TShape identity only (geometry deduplication).
//
// Why shape_key is needed:
//   BRepPrimAPI_MakePrism creates the top face of a swept solid by calling
//   TopoDS_Shape::Move() on the input (bottom) face — same TShape pointer,
//   different Location.  Using the TShape pointer alone as identity incorrectly
//   equates the top and bottom faces.  shape_key combines all three components
//   so that every distinct placed instance receives a distinct key.
//
// Reference:
//   TopoDS_Shape     — https://dev.opencascade.org/doc/refman/html/class_topo_d_s___shape.html
//   TopLoc_Location  — https://dev.opencascade.org/doc/refman/html/class_top_loc___location.html
//   gp_Trsf          — https://dev.opencascade.org/doc/refman/html/classgp___trsf.html
//   TopoDS           — https://dev.opencascade.org/doc/refman/html/class_topo_d_s.html
//
// Sourced from OCCT 7.9 documentation.
// No derivation from any other binding crate.

#pragma once

#include <cstddef>
#include <cstdint>
#include <cstring>
#include <memory>

#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Shape.hxx>
#include <TopoDS_Solid.hxx>
#include <TopoDS_Vertex.hxx>
#include <TopoDS_Wire.hxx>
#include <gp_Trsf.hxx>

// ── Null predicates ───────────────────────────────────────────────────────────
// TopoDS_Shape::IsNull() returns true when the TShape handle is null.
// Reference: https://dev.opencascade.org/doc/refman/html/class_topo_d_s___shape.html

inline bool topods_shape_is_null(const TopoDS_Shape& s) {
    return s.IsNull();
}
inline bool topods_face_is_null(const TopoDS_Face& s) {
    return s.IsNull();
}
inline bool topods_edge_is_null(const TopoDS_Edge& s) {
    return s.IsNull();
}
inline bool topods_wire_is_null(const TopoDS_Wire& s) {
    return s.IsNull();
}
inline bool topods_vertex_is_null(const TopoDS_Vertex& s) {
    return s.IsNull();
}
inline bool topods_solid_is_null(const TopoDS_Solid& s) {
    return s.IsNull();
}
// ── Clone ─────────────────────────────────────────────────────────────────────

// Copy-constructs a TopoDS_Shape.  The underlying TShape handle is shared
// (ref-counted by OCCT); no geometry is copied.
inline std::unique_ptr<TopoDS_Shape> clone_shape(const TopoDS_Shape& s) {
    return std::make_unique<TopoDS_Shape>(s);
}

// ── Placed-instance identity key ──────────────────────────────────────────────

// Returns a within-session identity key for a placed shape instance, encoding
// TShape (geometry), Location (placement), and Orientation.
//
// Two shapes that are the "same face" in different positions (e.g. the top and
// bottom of a prism) have the same TShape pointer but different Locations and
// will receive different keys from this function.
//
// The key is a hash; collisions are astronomically unlikely in practice for
// any reasonable number of shapes in a session.
inline std::size_t shape_key(const TopoDS_Shape& s) {
    // TShape + Location, via OCCT's own hash.
    std::size_t h = std::hash<TopoDS_Shape>{}(s);

    // Combine in Orientation using the same combine primitive
    // std::hash<TopoDS_Shape> itself uses for TShape+Location
    // (see TopoDS_Shape.hxx's std::hash specialization).
    TopAbs_Orientation orient = s.Orientation();
    return opencascade::MurmurHash::hash_combine(&orient, sizeof(orient), h);
}
inline bool face_is_reversed(const TopoDS_Face& f) {
    return f.Orientation() == TopAbs_REVERSED;
}

// Returns the raw TShape pointer as a size_t.
// Useful for geometry-level deduplication (e.g. detecting that two placed
// instances share the same underlying surface).  Not suitable as a unique
// key for placed instances — use shape_key() for that.
inline std::size_t shape_tshape_ptr(const TopoDS_Shape& s) {
    return reinterpret_cast<std::size_t>(s.TShape().get());
}

// ── Up-casts (zero-cost; return const reference) ──────────────────────────────

inline const TopoDS_Shape& face_as_shape(const TopoDS_Face& f) {
    return static_cast<const TopoDS_Shape&>(f);
}

inline const TopoDS_Shape& solid_as_shape(const TopoDS_Solid& s) {
    return static_cast<const TopoDS_Shape&>(s);
}

inline const TopoDS_Shape& edge_as_shape(const TopoDS_Edge& e) {
    return static_cast<const TopoDS_Shape&>(e);
}

inline const TopoDS_Shape& wire_as_shape(const TopoDS_Wire& w) {
    return static_cast<const TopoDS_Shape&>(w);
}

inline const TopoDS_Shape& vertex_as_shape(const TopoDS_Vertex& v) {
    return static_cast<const TopoDS_Shape&>(v);
}

// ── Down-casts ────────────────────────────────────────────────────────────────
// Precondition: s must be the declared shape type.  TopoDS::Face / Vertex / etc.
// cast with orientation preservation.  Violation throws Standard_TypeMismatch
// which the cxx bridge catches and converts to a process abort.

inline std::unique_ptr<TopoDS_Face> shape_as_face(const TopoDS_Shape& s) {
    return std::make_unique<TopoDS_Face>(TopoDS::Face(s));
}

inline std::unique_ptr<TopoDS_Vertex> shape_as_vertex(const TopoDS_Shape& s) {
    return std::make_unique<TopoDS_Vertex>(TopoDS::Vertex(s));
}

inline std::unique_ptr<TopoDS_Edge> shape_as_edge(const TopoDS_Shape& s) {
    return std::make_unique<TopoDS_Edge>(TopoDS::Edge(s));
}

inline std::unique_ptr<TopoDS_Wire> shape_as_wire(const TopoDS_Shape& s) {
    return std::make_unique<TopoDS_Wire>(TopoDS::Wire(s));
}

inline std::unique_ptr<TopoDS_Solid> shape_as_solid(const TopoDS_Shape& s) {
    return std::make_unique<TopoDS_Solid>(TopoDS::Solid(s));
}
