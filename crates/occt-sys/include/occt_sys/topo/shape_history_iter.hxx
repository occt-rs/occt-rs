// Shape history cursor iterator — snapshots a TopTools_ListOfShape from
// Modified() or Generated() and iterates without re-querying the builder.
//
// Used by MakeFillet/Chamfer/OffsetShape/ThickSolid history impls.
//
// Reference: https://dev.opencascade.org/doc/refman/html/class_top_tools___list_of_shape.html
// Sourced from OCCT 7.9 documentation.
// No derivation from any other binding crate.

#pragma once

#include <memory>
#include <TopTools_ListOfShape.hxx>
#include <TopoDS_Shape.hxx>

struct ShapeListIter {
    TopTools_ListOfShape snapshot;
    TopTools_ListOfShape::iterator cursor;
    TopTools_ListOfShape::iterator end;

    ShapeListIter(const TopTools_ListOfShape& lst)
        : snapshot(lst), cursor(snapshot.begin()), end(snapshot.end()) {}
};

inline std::unique_ptr<ShapeListIter>
shape_list_iter_new(const TopTools_ListOfShape& lst) {
    return std::make_unique<ShapeListIter>(lst);
}
inline bool shape_list_iter_more(const ShapeListIter& it) {
    return it.cursor != it.end;
}
inline void shape_list_iter_next(ShapeListIter& it) {
    ++it.cursor;
}
inline std::unique_ptr<TopoDS_Shape> shape_list_iter_value(const ShapeListIter& it) {
    return std::make_unique<TopoDS_Shape>(*it.cursor);
}
