// Three Add overloads are bound:
//   Add(dis, edge)               — symmetric (equal distance both sides)
//   Add(dis1, dis2, edge, face)  — asymmetric two-distance; face selects which
//                                   side receives dis1
//   AddDA(dis, angle, edge, face) — distance-angle; face selects the dis side
//
// History: Modified() and Generated() deferred to F2 (same blocker as fillet).
//
// Reference:
//   BRepFilletAPI_MakeChamfer — https://dev.opencascade.org/doc/refman/html/class_b_rep_fillet_a_p_i___make_chamfer.html
//
// Sourced from OCCT 7.9 documentation.
// No derivation from any other binding crate.

#pragma once

#include <memory>
#include <BRepFilletAPI_MakeChamfer.hxx>
#include <Message_ProgressRange.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Shape.hxx>

#include "solid.hxx"
#include "../exception.hxx"

struct MakeChamferBuilder {
    BRepFilletAPI_MakeChamfer inner;

    MakeChamferBuilder(const TopoDS_Shape& shape) : inner(shape) {}

    // Add(Dis, E) — symmetric chamfer.
    void add_edge(double dis, const TopoDS_Edge& edge) {
        try {
            inner.Add(dis, edge);
        } catch (const std::runtime_error&) { throw; }
        catch (...) { rethrow_occt_as_runtime_error(); }
    }

    // Add(Dis1, Dis2, E, F) — asymmetric two-distance chamfer.
    // F selects the face on which Dis1 is measured.
    void add_edge_asymmetric(double dis1, double dis2,
                              const TopoDS_Edge& edge, const TopoDS_Face& face) {
        try {
            inner.Add(dis1, dis2, edge, face);
        } catch (const std::runtime_error&) { throw; }
        catch (...) { rethrow_occt_as_runtime_error(); }
    }

    // AddDA(Dis, Angle, E, F) — distance-angle chamfer.
    // F selects the face on which Dis is measured.
    void add_edge_dist_angle(double dis, double angle,
                              const TopoDS_Edge& edge, const TopoDS_Face& face) {
        try {
            inner.AddDA(dis, angle, edge, face);
        } catch (const std::runtime_error&) { throw; }
        catch (...) { rethrow_occt_as_runtime_error(); }
    }

    void build() {
        try {
            inner.Build(Message_ProgressRange());
        } catch (const std::runtime_error&) { throw; }
        catch (...) { rethrow_occt_as_runtime_error(); }
    }

    bool is_done() const { return inner.IsDone(); }

    std::unique_ptr<TopoDS_Shape> shape() {
        return std::make_unique<TopoDS_Shape>(inner.Shape());
    }
};

inline std::unique_ptr<MakeChamferBuilder> new_make_chamfer_builder(
    const TopoDS_Shape& shape)
{
    try {
        return std::make_unique<MakeChamferBuilder>(shape);
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}
