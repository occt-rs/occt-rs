// occt_sys/topo/edge.hxx — TopoDS_Edge construction and vertex access.
//
// Only the straight-line vertex-to-vertex case is bound.
// BRepBuilderAPI_MakeEdge defers failure to IsDone()/Error() rather than
// throwing, so the builder is exposed as-is.
//
// TopExp::Vertices(edge, v1, v2) extracts the start and end vertices.
//
// Reference:
//   BRepBuilderAPI_MakeEdge  — https://dev.opencascade.org/doc/refman/html/class_b_rep_builder_a_p_i___make_edge.html
//   BRepBuilderAPI_EdgeError — https://dev.opencascade.org/doc/refman/html/BRepBuilderAPI__EdgeError_8hxx.html
//   TopExp                   — https://dev.opencascade.org/doc/refman/html/class_top_exp.html
//   TopoDS_Edge              — https://dev.opencascade.org/doc/refman/html/class_topo_d_s___edge.html
//
// Sourced from OCCT 7.9 documentation.
// No derivation from any other binding crate.

#pragma once

#include <memory>
#include <BRepBuilderAPI_EdgeError.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <TopoDS_Edge.hxx>
#include <TopExp.hxx>

#include "vertex.hxx"

struct MakeEdgeBuilder {
    BRepBuilderAPI_MakeEdge inner;
    MakeEdgeBuilder(const TopoDS_Vertex& v1, const TopoDS_Vertex& v2) : inner(v1, v2) {}
    bool is_done() const { return inner.IsDone(); }
    int  error()   const { return static_cast<int>(inner.Error()); }
    std::unique_ptr<TopoDS_Edge> edge() {
        return std::make_unique<TopoDS_Edge>(inner.Edge());
    }
};

inline std::unique_ptr<MakeEdgeBuilder> new_make_edge_builder(
    const TopoDS_Vertex& v1, const TopoDS_Vertex& v2)
{
    return std::make_unique<MakeEdgeBuilder>(v1, v2);
}

// TopoDS_Edge copy shares the underlying TShape handle (ref-counted).
inline std::unique_ptr<TopoDS_Edge> clone_edge(const TopoDS_Edge& e) {
    return std::make_unique<TopoDS_Edge>(e);
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
