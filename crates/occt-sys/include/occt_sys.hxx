// occt_sys.hxx — C++ shim layer for the occt-sys cxx bridge.
//
// This file contains only what is needed to materialise gp_* objects for
// consumption by higher-level OCCT APIs.  Mathematical operations on these
// types are implemented in pure Rust in the occt-rs crate and never cross
// the FFI boundary.
//
// Sourced from:
//   OCCT 7.9 reference documentation — https://dev.opencascade.org/doc/refman/html/
//   cxx documentation              — https://cxx.rs/
//
// No derivation from opencascade-rs or any other binding crate.
// See DEVELOPMENT.md for the full IP hygiene policy.

#pragma once

#include <memory>
#include <stdexcept>
#include <string>

#include <BRep_Tool.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <Standard_Failure.hxx>
#include <TopoDS_Vertex.hxx>
#include <gp_Ax1.hxx>
#include <gp_Ax2.hxx>
#include <gp_Dir.hxx>
#include <gp_Pnt.hxx>
#include <gp_Vec.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_EdgeError.hxx>
#include <TopoDS_Edge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_FaceError.hxx>
#include <TopoDS_Face.hxx>
#include <BRepTools.hxx>
#include <TopoDS_Solid.hxx>
#include <BRepPrimAPI_MakePrism.hxx>
#include <TopoDS.hxx>

// ── Exception protocol ──────────────────────────────────────────────────────
//
// Retained for gp_Dir construction, which raises Standard_ConstructionError
// on null magnitude, and for future OCCT APIs that can raise.
//
// Every fallible shim catches Standard_Failure and rethrows as
// std::runtime_error with what() = "OCCT:<DynamicTypeName>:<message>".
// occt_rs::OcctError parses this format.
[[noreturn]] inline void rethrow_occt_as_runtime_error() {
    try {
        throw;
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(
            std::string("OCCT:") + e.DynamicType()->Name() + ":" + e.GetMessageString()
        );
    } catch (const std::exception& e) {
        throw std::runtime_error(std::string("OCCT:Other:") + e.what());
    } catch (...) {
        throw std::runtime_error("OCCT:Other:unknown C++ exception");
    }
}
// ── BRepPrimAPI_MakePrism ─────────────────────────────────────────────────
// Reference: https://dev.opencascade.org/doc/refman/html/class_b_rep_prim_a_p_i___make_prism.html
// Reference: https://dev.opencascade.org/doc/refman/html/class_topo_d_s.html
//
// BRepPrimAPI_MakePrism sweeps a shape along a vector to produce a solid.
// Unlike BRepBuilderAPI_* builders, MakePrism computes immediately in the
// constructor and throws Standard_Failure on failure — the factory shim
// must wrap construction in try/catch.
//
// Shape() is non-const in BRepBuilderAPI_MakeShape (it advances internal
// state); the solid() accessor is therefore exposed as a mutable method.
//
// History note: BRepPrimAPI_MakePrism exposes Modified(), Generated(), and
// IsDeleted(). This builder must not be dropped before history is queried.
// History access is deferred; do not destroy MakePrismBuilder before
// extracting the solid AND any required history.


struct MakePrismBuilder {
    BRepPrimAPI_MakePrism inner;

    // face is passed as TopoDS_Face; it upcasts implicitly to TopoDS_Shape
    // in C++, as required by BRepPrimAPI_MakePrism(const TopoDS_Shape&, ...).
    MakePrismBuilder(const TopoDS_Face& face, double vx, double vy, double vz)
        : inner(face, gp_Vec(vx, vy, vz)) {}

    bool is_done() const { return inner.IsDone(); }

    // Downcasts the resulting TopoDS_Shape to TopoDS_Solid via TopoDS::Solid.
    // Only call when is_done() is true.
    std::unique_ptr<TopoDS_Solid> solid() {
        return std::make_unique<TopoDS_Solid>(TopoDS::Solid(inner.Shape()));
    }
};

// Factory. Wraps BRepPrimAPI_MakePrism construction in try/catch because
// MakePrism computes immediately and throws on failure — it does not defer
// failure to an IsDone()/Error() pair.
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
// ── TopoDS_Solid ──────────────────────────────────────────────────────────────
// Reference: https://dev.opencascade.org/doc/refman/html/class_topo_d_s___solid.html
//
// No construction path yet — OcSolid is surfaced as a type-only placeholder
// so that MakePrism (TKPrim, next step) has a typed output to return.
//
// TopoDS_Solid copy shares the underlying TShape handle (ref-counted).
 
inline std::unique_ptr<TopoDS_Solid> clone_solid(const TopoDS_Solid& s) {
    return std::make_unique<TopoDS_Solid>(s);
}

// ── gp_Pnt materialisation ─────────────────────────────────────────────────
// Called by OcPnt::to_ffi() when passing to an OCCT API.

inline std::unique_ptr<gp_Pnt> new_gp_pnt_xyz(double x, double y, double z) {
    return std::make_unique<gp_Pnt>(x, y, z);
}

// ── gp_Vec materialisation ─────────────────────────────────────────────────
// Called by OcVec::to_ffi() when passing to an OCCT API.

inline std::unique_ptr<gp_Vec> new_gp_vec_xyz(double x, double y, double z) {
    return std::make_unique<gp_Vec>(x, y, z);
}
#include <BRepTools_WireExplorer.hxx>
#include <TopExp.hxx>

// ── Wire edge exploration ─────────────────────────────────────────────────
// Reference (WireExplorer): https://dev.opencascade.org/doc/refman/html/class_b_rep_tools___wire_explorer.html
// Reference (TopExp):       https://dev.opencascade.org/doc/refman/html/class_top_exp.html

struct WireEdgeExplorer {
    BRepTools_WireExplorer inner;
    WireEdgeExplorer(const TopoDS_Wire& w) : inner(w) {}
    bool more() const { return inner.More(); }
    void next() { inner.Next(); }
    std::unique_ptr<TopoDS_Edge> current_edge() const {
        return std::make_unique<TopoDS_Edge>(inner.Current());
    }
};

inline std::unique_ptr<WireEdgeExplorer> new_wire_edge_explorer(const TopoDS_Wire& w) {
    return std::make_unique<WireEdgeExplorer>(w);
}

inline std::unique_ptr<TopoDS_Vertex> edge_start_vertex(const TopoDS_Edge& e) {
    TopoDS_Vertex v1, v2;
    TopExp::Vertices(e, v1, v2);
    return std::make_unique<TopoDS_Vertex>(v1);
}

inline std::unique_ptr<TopoDS_Vertex> edge_end_vertex(const TopoDS_Edge& e) {
    TopoDS_Vertex v1, v2;
    TopExp::Vertices(e, v1, v2);
    return std::make_unique<TopoDS_Vertex>(v2);
}
// ── TopoDS_Edge construction ───────────────────────────────────────────────
// Reference (MakeEdge): https://dev.opencascade.org/doc/refman/html/class_b_rep_builder_a_p_i___make_edge.html
// Reference (EdgeError): https://dev.opencascade.org/doc/refman/html/BRepBuilderAPI__EdgeError_8hxx.html
//
// Exposed as an opaque builder so the Rust side can inspect IsDone() and
// Error() directly.  No manipulation of error values in the shim.

struct MakeEdgeBuilder {
    BRepBuilderAPI_MakeEdge inner;

    MakeEdgeBuilder(const TopoDS_Vertex& v1, const TopoDS_Vertex& v2)
        : inner(v1, v2) {}

    bool is_done() const { return inner.IsDone(); }

    // Returns the raw BRepBuilderAPI_EdgeError enum value as int.
    // Callers map against BRepBuilderAPI_EdgeError.hxx constants.
    int error() const { return static_cast<int>(inner.Error()); }

    std::unique_ptr<TopoDS_Edge> edge() {
        return std::make_unique<TopoDS_Edge>(inner.Edge());
    }
};

inline std::unique_ptr<MakeEdgeBuilder> new_make_edge_builder(
    const TopoDS_Vertex& v1, const TopoDS_Vertex& v2)
{
    return std::make_unique<MakeEdgeBuilder>(v1, v2);
}

inline std::unique_ptr<TopoDS_Edge> clone_edge(const TopoDS_Edge& e) {
    return std::make_unique<TopoDS_Edge>(e);
}
// ── Wire builder ──────────────────────────────
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRepBuilderAPI_WireError.hxx>
#include <TopoDS_Wire.hxx>

struct MakeWireBuilder {
    BRepBuilderAPI_MakeWire inner;

    void add_edge(const TopoDS_Edge& e) { inner.Add(e); }
    bool is_done() const { return inner.IsDone(); }
    int error() const { return static_cast<int>(inner.Error()); }
    std::unique_ptr<TopoDS_Wire> wire() { return std::make_unique<TopoDS_Wire>(inner.Wire()); }
};

inline std::unique_ptr<MakeWireBuilder> new_make_wire_builder() {
    return std::make_unique<MakeWireBuilder>();
}

inline std::unique_ptr<TopoDS_Wire> clone_wire(const TopoDS_Wire& w) {
    return std::make_unique<TopoDS_Wire>(w);
}
// ── TopoDS_Vertex construction and inspection ──────────────────────────────
// Reference (MakeVertex): https://dev.opencascade.org/doc/refman/html/class_b_rep_builder_a_p_i___make_vertex.html
// Reference (BRep_Tool):  https://dev.opencascade.org/doc/refman/html/class_b_rep___tool.html
//
// BRepBuilderAPI_MakeVertex(const gp_Pnt&) builds a TopoDS_Vertex with
// Precision::Confusion() as its default tolerance.  The result is a small
// handle wrapper (TopoDS_Vertex contains a Handle(TopoDS_TShape) and is
// internally reference-counted); copying it is cheap and correct.
//
// BRep_Tool::Pnt(const TopoDS_Vertex&) returns gp_Pnt by value — a stack
// object.  Rather than returning that across the FFI boundary, we expose
// three thin shims that each extract one coordinate, keeping the gp_Pnt
// stack-allocated and avoiding any heap allocation on the read-back path.

/// Constructs a TopoDS_Vertex from raw coordinates.
/// The gp_Pnt and BRepBuilderAPI_MakeVertex live on the C++ stack.
inline std::unique_ptr<TopoDS_Vertex> make_vertex(double x, double y, double z) {
    return std::make_unique<TopoDS_Vertex>(
        BRepBuilderAPI_MakeVertex(gp_Pnt(x, y, z)).Vertex()
    );
}

/// Copy-constructs a TopoDS_Vertex.  Used to implement Clone on OcVertex.
/// TopoDS_Vertex copy shares the underlying TShape handle (ref-counted).
inline std::unique_ptr<TopoDS_Vertex> clone_vertex(const TopoDS_Vertex& v) {
    return std::make_unique<TopoDS_Vertex>(v);
}

/// Reads back the X coordinate of the vertex's 3-D point.
/// gp_Pnt is stack-allocated inside the shim; no heap allocation.
inline double vertex_pnt_x(const TopoDS_Vertex& v) {
    return BRep_Tool::Pnt(v).X();
}
inline double vertex_pnt_y(const TopoDS_Vertex& v) {
    return BRep_Tool::Pnt(v).Y();
}
inline double vertex_pnt_z(const TopoDS_Vertex& v) {
    return BRep_Tool::Pnt(v).Z();
}

// ── gp_Dir materialisation ─────────────────────────────────────────────────
// Reference: https://dev.opencascade.org/doc/refman/html/classgp___dir.html
//
// OcDir validates and normalises at construction in pure Rust.  By the time
// to_ffi() is called the coordinates are guaranteed unit magnitude, so this
// shim should never raise in practice.  The Result return is retained as a
// safety net; a panic in to_ffi() indicates a bug in OcDir's invariant.

inline std::unique_ptr<gp_Dir> new_gp_dir_xyz(double x, double y, double z) {
    try {
        return std::make_unique<gp_Dir>(x, y, z);
    } catch (...) {
        rethrow_occt_as_runtime_error();
    }
}

// ── gp_Ax1 materialisation ─────────────────────────────────────────────────
// Reference: https://dev.opencascade.org/doc/refman/html/classgp___ax1.html
//
// Called by OcAx1::to_ffi() when passing to an OCCT API.
// The direction components are guaranteed unit-magnitude (from OcDir), so
// the gp_Dir constructor should never raise in practice.  The try/catch is
// retained as a safety net against invariant violations.

inline std::unique_ptr<gp_Ax1> new_gp_ax1(
    double px, double py, double pz,
    double dx, double dy, double dz)
{
    try {
        return std::make_unique<gp_Ax1>(
            gp_Pnt(px, py, pz),
            gp_Dir(dx, dy, dz));
    } catch (...) {
        rethrow_occt_as_runtime_error();
    }
}

// ── gp_Ax2 materialisation ─────────────────────────────────────────────────
// Reference: https://dev.opencascade.org/doc/refman/html/classgp___ax2.html
//
// Called by OcAx2::to_ffi() when passing to an OCCT API.
// (nx,ny,nz) is the main/"Z" direction; (xx,xy,xz) is the X direction.
// By the time to_ffi() is called, OcAx2's constructor has already verified
// non-parallelism and stored the corrected, mutually-perpendicular directions
// (computed as (N ^ Vx) ^ N per the OCCT gp_Ax2 documentation), so
// ConstructionError should never fire in practice.  The try/catch is
// retained as a safety net against invariant violations.

inline std::unique_ptr<gp_Ax2> new_gp_ax2(
    double px, double py, double pz,
    double nx, double ny, double nz,
    double xx, double xy, double xz)
{
    try {
        return std::make_unique<gp_Ax2>(
            gp_Pnt(px, py, pz),
            gp_Dir(nx, ny, nz),
            gp_Dir(xx, xy, xz));
    } catch (...) {
        rethrow_occt_as_runtime_error();
    }
}

// ── TopoDS_Face construction ──────────────────────────────────────────────────
// Reference (MakeFace): https://dev.opencascade.org/doc/refman/html/class_b_rep_builder_a_p_i___make_face.html
// Reference (FaceError): https://dev.opencascade.org/doc/refman/html/BRepBuilderAPI__FaceError_8hxx.html
//
// Exposed as an opaque builder so the Rust side can inspect IsDone() and
// Error() directly.  No manipulation of error values in the shim.
//
// Constructor used: BRepBuilderAPI_MakeFace(const TopoDS_Wire&, Standard_Boolean)
//   only_plane = true  → fails if the wire is not planar
//   only_plane = false → OCCT attempts to find a fitting surface (default)
//
// Face() is non-const in BRepBuilderAPI_MakeFace (confirmed from OCCT header:
// it calls BRepBuilderAPI_MakeShape::Shape() which is non-const in the builder
// hierarchy).  The result accessor therefore takes a mutable reference.

struct MakeFaceBuilder {
    BRepBuilderAPI_MakeFace inner;

    MakeFaceBuilder(const TopoDS_Wire& w, bool only_plane)
        : inner(w, only_plane ? Standard_True : Standard_False) {}

    bool is_done() const { return inner.IsDone(); }

    // Returns the raw BRepBuilderAPI_FaceError enum value as int.
    // Callers map against BRepBuilderAPI_FaceError.hxx constants.
    int error() const { return static_cast<int>(inner.Error()); }

    std::unique_ptr<TopoDS_Face> face() {
        return std::make_unique<TopoDS_Face>(inner.Face());
    }
};

inline std::unique_ptr<MakeFaceBuilder> new_make_face_from_wire(
    const TopoDS_Wire& w, bool only_plane)
{
    return std::make_unique<MakeFaceBuilder>(w, only_plane);
}

/// Copy-constructs a TopoDS_Face.  Used to implement Clone on OcFace.
/// TopoDS_Face copy shares the underlying TShape handle (ref-counted).
inline std::unique_ptr<TopoDS_Face> clone_face(const TopoDS_Face& f) {
    return std::make_unique<TopoDS_Face>(f);
}

// ── Face inspection ───────────────────────────────────────────────────────────
// Reference (BRepTools): https://dev.opencascade.org/doc/refman/html/class_b_rep_tools.html
//
// BRepTools::OuterWire returns the outer boundary wire of a face.
// The result is a TopoDS_Wire by value; we heap-allocate it for UniquePtr
// ownership on the Rust side.

inline std::unique_ptr<TopoDS_Wire> face_outer_wire(const TopoDS_Face& f) {
    return std::make_unique<TopoDS_Wire>(BRepTools::OuterWire(f));
}
