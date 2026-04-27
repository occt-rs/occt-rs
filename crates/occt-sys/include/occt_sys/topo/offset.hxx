// MakeOffsetShape: PerformBySimple(shape, offset) only. No join type exposed.
//
// MakeThickSolid: uses a builder that accumulates closing faces into a
// TopTools_ListOfShape on the C++ side, then calls MakeThickSolidByJoin.
// TopTools_ListOfShape never crosses the cxx bridge.
// Defaults: Mode=BRepOffset_Skin, Intersection=false, SelfInter=false,
//           Join=GeomAbs_Arc, RemoveIntEdges=false.
//
// History: Modified() / Generated() deferred to F2.
//
// Reference:
//   BRepOffsetAPI_MakeOffsetShape — https://dev.opencascade.org/doc/refman/html/class_b_rep_offset_a_p_i___make_offset_shape.html
//   BRepOffsetAPI_MakeThickSolid  — https://dev.opencascade.org/doc/refman/html/class_b_rep_offset_a_p_i___make_thick_solid.html
//
// Sourced from OCCT 7.9 documentation.
// No derivation from any other binding crate.

#pragma once

#include <memory>
#include <BRepOffsetAPI_MakeOffsetShape.hxx>
#include <BRepOffsetAPI_MakeThickSolid.hxx>
#include <TopTools_ListOfShape.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Shape.hxx>
#include <Message_ProgressRange.hxx>

#include "solid.hxx"
#include "../exception.hxx"

// ── MakeOffsetShape ───────────────────────────────────────────────────────────

struct MakeOffsetShapeBuilder {
    BRepOffsetAPI_MakeOffsetShape inner;

    // PerformBySimple does not throw — it sets error status internally.
    void perform(const TopoDS_Shape& shape, double offset) {
        try {
            inner.PerformBySimple(shape, offset);
        } catch (const std::runtime_error&) { throw; }
        catch (...) { rethrow_occt_as_runtime_error(); }
    }

    bool is_done() const { return inner.IsDone(); }

    std::unique_ptr<TopoDS_Shape> shape() {
        return std::make_unique<TopoDS_Shape>(inner.Shape());
    }
};

inline std::unique_ptr<MakeOffsetShapeBuilder> new_make_offset_shape_builder() {
    return std::make_unique<MakeOffsetShapeBuilder>();
}

// ── MakeThickSolid ────────────────────────────────────────────────────────────

struct MakeThickSolidBuilder {
    BRepOffsetAPI_MakeThickSolid inner;
    TopTools_ListOfShape closing_faces;

    void add_closing_face(const TopoDS_Face& face) {
        closing_faces.Append(face);
    }

    // MakeThickSolidByJoin with OCCT-recommended defaults.
    void build(const TopoDS_Shape& shape, double offset, double tol) {
        try {
            inner.MakeThickSolidByJoin(
                shape,
                closing_faces,
                offset,
                tol,
                BRepOffset_Skin,      // Mode
                Standard_False,       // Intersection
                Standard_False,       // SelfInter
                GeomAbs_Arc,          // Join
                Standard_False        // RemoveIntEdges
            );
        } catch (const std::runtime_error&) { throw; }
        catch (...) { rethrow_occt_as_runtime_error(); }
    }

    bool is_done() const { return inner.IsDone(); }

    std::unique_ptr<TopoDS_Shape> shape() {
        return std::make_unique<TopoDS_Shape>(inner.Shape());
    }
};

inline std::unique_ptr<MakeThickSolidBuilder> new_make_thick_solid_builder() {
    return std::make_unique<MakeThickSolidBuilder>();
}
