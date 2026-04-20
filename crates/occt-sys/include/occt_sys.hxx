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
