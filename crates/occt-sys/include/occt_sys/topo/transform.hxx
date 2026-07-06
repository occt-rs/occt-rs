// Applies a gp_Trsf to a shape, producing a new independent shape.
// The transform is passed as 12 matrix entries (rows 1-3, cols 1-4) to
// avoid cross-bridge type sharing between the gp.rs and topo.rs bridges.
// Reconstructing gp_Trsf from matrix entries via SetValues is correct for
// BRepBuilderAPI_Transform; the builder applies the full 3×4 matrix to all
// curves and surfaces regardless of gp_TrsfForm.
//
// History methods (Modified, Generated, IsDeleted) are present on
// BRepBuilderAPI_Transform but not yet bound.  They require TopTools_ListOfShape
// to cross the cxx bridge — deferred to Milestone F (TransformBuilder type).
//
// Reference:
//   BRepBuilderAPI_Transform — https://dev.opencascade.org/doc/refman/html/class_b_rep_builder_a_p_i___transform.html
//   gp_Trsf::SetValues       — https://dev.opencascade.org/doc/refman/html/classgp___trsf.html
//
// Sourced from OCCT 7.9 documentation.
// No derivation from any other binding crate.

#pragma once

#include <memory>
#include <BRepBuilderAPI_Transform.hxx>
#include <gp_Trsf.hxx>
#include <TopoDS_Shape.hxx>

#include "solid.hxx"
#include "../exception.hxx"

// Sourced from OCCT 7.9 documentation.
// No derivation from any other binding crate.
//
// Reference: BRepBuilderAPI_Transform — https://dev.opencascade.org/doc/refman/html/class_b_rep_builder_a_p_i___transform.html
//
// MakeTransformBuilder: persistent counterpart to transform_shape (above).
// BRepBuilderAPI_Transform computes the transform in its own constructor —
// there is no separate Build() step to call after construction; is_done()
// reflects the result immediately. Modified()/Generated()/IsDeleted() read
// history populated during that same construction and require the instance
// to stay alive — see file-level lifetime note in sys_topo.rs.
struct MakeTransformBuilder {
    BRepBuilderAPI_Transform inner;

    MakeTransformBuilder(
        const TopoDS_Shape& shape,
        double r11, double r12, double r13, double t1,
        double r21, double r22, double r23, double t2,
        double r31, double r32, double r33, double t3,
        bool copy)
        : inner(shape,
                [&]{
                    gp_Trsf trsf;
                    trsf.SetValues(r11, r12, r13, t1,
                                   r21, r22, r23, t2,
                                   r31, r32, r33, t3);
                    return trsf;
                }(),
                copy ? Standard_True : Standard_False)
    {}

    bool is_done() const { return inner.IsDone(); }

    std::unique_ptr<TopoDS_Shape> shape() {
        return std::make_unique<TopoDS_Shape>(inner.Shape());
    }
    bool is_deleted(const TopoDS_Shape& s) {
        return inner.IsDeleted(s) == Standard_True;
    }
};

inline std::unique_ptr<ShapeListIter>
transform_modified_iter(MakeTransformBuilder& b, const TopoDS_Shape& s) {
    return shape_list_iter_new(b.inner.Modified(s));
}
inline std::unique_ptr<ShapeListIter>
transform_generated_iter(MakeTransformBuilder& b, const TopoDS_Shape& s) {
    return shape_list_iter_new(b.inner.Generated(s));
}

// Construction itself performs the transform and can throw
// (Standard_ConstructionError on a degenerate Trsf, etc.) — wrapped here,
// same as new_make_fillet_builder / new_make_chamfer_builder.
inline std::unique_ptr<MakeTransformBuilder> new_make_transform_builder(
    const TopoDS_Shape& shape,
    double r11, double r12, double r13, double t1,
    double r21, double r22, double r23, double t2,
    double r31, double r32, double r33, double t3,
    bool copy)
{
    try {
        return std::make_unique<MakeTransformBuilder>(
            shape, r11, r12, r13, t1, r21, r22, r23, t2, r31, r32, r33, t3, copy);
    } catch (const std::runtime_error&) {
        throw;
    } catch (...) {
        rethrow_occt_as_runtime_error();
    }
}
// transform_shape: applies the transform given as 12 matrix entries to `shape`.
//
// Entries: (r11..r34) are Value(row, col) from gp_Trsf, rows 1-3, cols 1-4.
// SetValues reconstructs the gp_Trsf; see file-level note on gp_TrsfForm.
//
// Throws std::runtime_error (OCCT: wire format) if BRepBuilderAPI_Transform
// raises or IsDone() is false after construction.
inline std::unique_ptr<TopoDS_Shape> transform_shape(
    const TopoDS_Shape& shape,
    double r11, double r12, double r13, double t1,
    double r21, double r22, double r23, double t2,
    double r31, double r32, double r33, double t3)
{
    try {
        gp_Trsf trsf;
        trsf.SetValues(r11, r12, r13, t1,
                       r21, r22, r23, t2,
                       r31, r32, r33, t3);

        BRepBuilderAPI_Transform builder(shape, trsf, Standard_False);

        if (!builder.IsDone()) {
            throw std::runtime_error(
                "OCCT:BRepBuilderAPI_Transform:IsDone() false after construction");
        }
        // Shape() is non-const on BRepBuilderAPI_MakeShape; called here on the
        // C++ stack builder, not across the bridge.
        return std::make_unique<TopoDS_Shape>(builder.Shape());
    } catch (const std::runtime_error&) {
        throw;
    } catch (...) {
        rethrow_occt_as_runtime_error();
    }
}
