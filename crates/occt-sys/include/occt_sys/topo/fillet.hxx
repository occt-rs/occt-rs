// Builder pattern: construct with a base shape, register edges via add_edge,
// call build(), then extract the result via shape().
//
// Add() can throw Standard_NoSuchObject if the edge is not on the shape;
// Build() can throw on degenerate geometry. Both are wrapped.
//
// History: Modified() and Generated() return TopTools_ListOfShape — cannot
// cross the cxx bridge in the current architecture. Deferred to F2
// (ShapeHistory trait). Do not drop FilletBuilder before history is read
// when that work lands.
//
// Reference:
//   BRepFilletAPI_MakeFillet — https://dev.opencascade.org/doc/refman/html/class_b_rep_fillet_a_p_i___make_fillet.html
//
// Sourced from OCCT 7.9 documentation.
// No derivation from any other binding crate.

#pragma once

#include <memory>
#include <BRepFilletAPI_MakeFillet.hxx>
#include <Message_ProgressRange.hxx>
#include <TopoDS_Shape.hxx>

#include "solid.hxx"
#include "../exception.hxx"

struct MakeFilletBuilder {
    BRepFilletAPI_MakeFillet inner;

    MakeFilletBuilder(const TopoDS_Shape& shape) : inner(shape) {}

    // Add(Radius, E): registers a constant-radius fillet on the given edge.
    // Non-const. Throws Standard_NoSuchObject if edge is not on the shape.
    void add_edge(double radius, const TopoDS_Edge& edge) {
        try {
            inner.Add(radius, edge);
        } catch (const std::runtime_error&) { throw; }
        catch (...) { rethrow_occt_as_runtime_error(); }
    }

    // Build(): computes the fillet. Non-const. Check is_done() after.
    void build() {
        try {
            inner.Build(Message_ProgressRange());
        } catch (const std::runtime_error&) { throw; }
        catch (...) { rethrow_occt_as_runtime_error(); }
    }

    bool is_done() const { return inner.IsDone(); }

    // Shape() is non-const on BRepBuilderAPI_MakeShape. Call only when is_done().
    std::unique_ptr<TopoDS_Shape> shape() {
        return std::make_unique<TopoDS_Shape>(inner.Shape());
    }
};

inline std::unique_ptr<MakeFilletBuilder> new_make_fillet_builder(
    const TopoDS_Shape& shape)
{
    try {
        return std::make_unique<MakeFilletBuilder>(shape);
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}
