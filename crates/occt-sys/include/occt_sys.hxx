// occt_sys.hxx — C++ shim layer for the occt-sys cxx bridge.
//
// This file contains only what is needed to materialise gp_* objects for
// consumption by higher-level OCCT APIs.  Mathematical operations on these
// types are implemented in pure Rust in the occt-rs crate and never cross
// the FFI boundary.
//
// LLM generated with reference to the following documents:
//   OCCT 7.9 reference documentation — https://dev.opencascade.org/doc/refman/html/
//   cxx documentation              — https://cxx.rs/
//
// See DEVELOPMENT.md for the full IP hygiene policy.

#pragma once

#include <memory>
#include <stdexcept>
#include <string>

#include <Standard_Failure.hxx>
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

// ── gp_Dir materialisation ─────────────────────────────────────────────────
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
