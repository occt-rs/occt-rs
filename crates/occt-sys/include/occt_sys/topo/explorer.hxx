// occt_sys/topo/explorer.hxx — TopExp_Explorer wrapper.
//
// ShapeExplorer iterates over all sub-shapes of a given type within a shape.
// The shape type is specified as an integer matching TopAbs_ShapeEnum:
//
//   TopAbs_COMPOUND  = 0   TopAbs_COMPSOLID = 1
//   TopAbs_SOLID     = 2   TopAbs_SHELL     = 3
//   TopAbs_FACE      = 4   TopAbs_WIRE      = 5
//   TopAbs_EDGE      = 6   TopAbs_VERTEX    = 7
//   TopAbs_SHAPE     = 8
//
// current() returns a const reference valid until next() is called or the
// explorer is destroyed.  This maps to a Rust lifetime tied to &self.
//
// TopExp_Explorer does not deduplicate: a vertex shared by N edges is returned
// N times when exploring TopAbs_VERTEX.  Callers that need unique sub-shapes
// should track seen TShape pointers (via shape_tshape_ptr) on the Rust side.
//
// Reference:
//   TopExp_Explorer  — https://dev.opencascade.org/doc/refman/html/class_top_exp___explorer.html
//   TopAbs_ShapeEnum — https://dev.opencascade.org/doc/refman/html/namespace_top_abs.html
//
// Sourced from OCCT 7.9 documentation.
// No derivation from any other binding crate.

#pragma once

#include <memory>

#include <TopAbs_ShapeEnum.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS_Shape.hxx>

struct ShapeExplorer {
    TopExp_Explorer inner;

    ShapeExplorer(const TopoDS_Shape& shape, int shape_enum)
        : inner(shape, static_cast<TopAbs_ShapeEnum>(shape_enum)) {}

    bool more() const { return inner.More() == Standard_True; }

    // Non-const: advances the iterator to the next matching sub-shape.
    void next() { inner.Next(); }

    // Returns a reference to the current sub-shape.  The reference is valid
    // until next() is called or the explorer is destroyed.
    const TopoDS_Shape& current() const { return inner.Current(); }
};

inline std::unique_ptr<ShapeExplorer> new_shape_explorer(
    const TopoDS_Shape& shape, int shape_enum)
{
    return std::make_unique<ShapeExplorer>(shape, shape_enum);
}
