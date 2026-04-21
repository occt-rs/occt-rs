// occt_sys/gp.hxx — Materialisation shims for gp_* geometric primitives.
//
// Called exclusively by Oc*::to_ffi() in the safe occt-rs layer.
// Mathematical operations on gp_* types live in pure Rust and never cross
// the FFI boundary.
//
// Reference:
//   gp_Pnt — https://dev.opencascade.org/doc/refman/html/classgp___pnt.html
//   gp_Vec — https://dev.opencascade.org/doc/refman/html/classgp___vec.html
//   gp_Dir — https://dev.opencascade.org/doc/refman/html/classgp___dir.html
//   gp_Ax1 — https://dev.opencascade.org/doc/refman/html/classgp___ax1.html
//   gp_Ax2 — https://dev.opencascade.org/doc/refman/html/classgp___ax2.html
//
// Sourced from OCCT 7.9 documentation.
// No derivation from any other binding crate.

#pragma once

#include <memory>
#include <gp_Ax1.hxx>
#include <gp_Ax2.hxx>
#include <gp_Dir.hxx>
#include <gp_Pnt.hxx>
#include <gp_Vec.hxx>

#include "exception.hxx"

inline std::unique_ptr<gp_Pnt> new_gp_pnt_xyz(double x, double y, double z) {
    return std::make_unique<gp_Pnt>(x, y, z);
}

inline std::unique_ptr<gp_Vec> new_gp_vec_xyz(double x, double y, double z) {
    return std::make_unique<gp_Vec>(x, y, z);
}

// gp_Dir raises Standard_ConstructionError on null magnitude.
// OcDir pre-validates, so this should never fire in practice.
inline std::unique_ptr<gp_Dir> new_gp_dir_xyz(double x, double y, double z) {
    try {
        return std::make_unique<gp_Dir>(x, y, z);
    } catch (...) {
        rethrow_occt_as_runtime_error();
    }
}

// Direction components are guaranteed unit-magnitude by OcDir/OcAx1.
inline std::unique_ptr<gp_Ax1> new_gp_ax1(
    double px, double py, double pz,
    double dx, double dy, double dz)
{
    try {
        return std::make_unique<gp_Ax1>(gp_Pnt(px, py, pz), gp_Dir(dx, dy, dz));
    } catch (...) {
        rethrow_occt_as_runtime_error();
    }
}

// OcAx2 pre-validates non-parallelism and stores the corrected X direction.
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
