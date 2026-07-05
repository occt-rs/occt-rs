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
// TopoDS_Shape identity is not one relation, it's three, all defined on the
// class itself:
//   IsPartner — same TShape only. Locations and Orientations may differ.
//   IsSame    — same TShape + Location. Orientations may differ.
//   IsEqual   — same TShape + Location + Orientation (this is operator==).
//
// same_shape / same_placed_shape / same_oriented_shape below are direct 1:1
// mirrors of those three, in that order. same_placed_shape_key and
// same_oriented_shape_key are hash keys for the two tiers that have actual
// consumers (Partner has none — predicate only, no key type).
//
// Why the placed (IsSame) tier matters independently of the oriented tier:
//   BRepPrimAPI_MakePrism creates the top face of a swept solid by calling
//   TopoDS_Shape::Move() on the input (bottom) face — same TShape pointer,
//   different Location. same_placed_shape_key still distinguishes them
//   (different Location), so it remains safe for that case. Separately, a
//   solid's internal edges are shared by exactly two faces which read that
//   edge in opposite directions by ordinary BRep convention — same TShape,
//   same Location, different Orientation. Dedup keyed on the oriented tier
//   will not collapse these; dedup keyed on the placed tier will.
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

// ── Identity tiers ─────────────────────────────────────────────────────────────

// IsPartner: same TShape only. Predicate only — no key type, since there is
// no current consumer that needs to hash at this tier. Add one only when a
// real caller needs it.
inline bool same_shape(const TopoDS_Shape& a, const TopoDS_Shape& b) {
    return a.IsPartner(b);
}

// IsSame: same TShape + Location. Orientation may differ.
inline bool same_placed_shape(const TopoDS_Shape& a, const TopoDS_Shape& b) {
    return a.IsSame(b);
}

// IsEqual (operator==): same TShape + Location + Orientation.
inline bool same_oriented_shape(const TopoDS_Shape& a, const TopoDS_Shape& b) {
    return a.IsEqual(b);
}

// Hash key for the IsSame (placed) tier. Body is exactly std::hash<TopoDS_Shape>,
// which OCCT itself defines over TShape+Location (see TopoDS_Shape.hxx) — the
// same primitive TopTools_ShapeMapHasher uses internally. No independent
// hashing scheme is implemented here.
//
// The key is a hash; collisions are astronomically unlikely in practice for
// any reasonable number of shapes in a session.
inline std::size_t same_placed_shape_key(const TopoDS_Shape& s) {
    return std::hash<TopoDS_Shape>{}(s);
}

// Hash key for the IsEqual (oriented) tier. Renamed from shape_key(); logic
// unchanged. TShape+Location comes from std::hash<TopoDS_Shape>; Orientation
// is combined on top via the same MurmurHash::hash_combine primitive
// std::hash<TopoDS_Shape> uses internally for TShape+Location.
inline std::size_t same_oriented_shape_key(const TopoDS_Shape& s) {
    std::size_t h = std::hash<TopoDS_Shape>{}(s);
    TopAbs_Orientation orient = s.Orientation();
    return opencascade::MurmurHash::hash_combine(&orient, sizeof(orient), h);
}

inline bool face_is_reversed(const TopoDS_Face& f) {
    return f.Orientation() == TopAbs_REVERSED;
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
