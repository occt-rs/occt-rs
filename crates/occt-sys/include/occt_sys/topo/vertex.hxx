// occt_sys/topo/vertex.hxx — TopoDS_Vertex construction and point read-back.
//
// BRepBuilderAPI_MakeVertex(const gp_Pnt&) builds a TopoDS_Vertex with
// Precision::Confusion() as the default tolerance.
//
// BRep_Tool::Pnt returns gp_Pnt by value.  Three thin shims extract one
// coordinate each, keeping the gp_Pnt stack-allocated and avoiding heap
// allocation on the read-back path.
//
// Reference:
//   BRepBuilderAPI_MakeVertex — https://dev.opencascade.org/doc/refman/html/class_b_rep_builder_a_p_i___make_vertex.html
//   BRep_Tool                 — https://dev.opencascade.org/doc/refman/html/class_b_rep___tool.html
//   TopoDS_Vertex             — https://dev.opencascade.org/doc/refman/html/class_topo_d_s___vertex.html
//
// Sourced from OCCT 7.9 documentation.
// No derivation from any other binding crate.

#pragma once

#include <memory>
#include <BRep_Tool.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <TopoDS_Vertex.hxx>
#include <gp_Pnt.hxx>

inline std::unique_ptr<TopoDS_Vertex> make_vertex(double x, double y, double z) {
    return std::make_unique<TopoDS_Vertex>(
        BRepBuilderAPI_MakeVertex(gp_Pnt(x, y, z)).Vertex()
    );
}

// TopoDS_Vertex copy shares the underlying TShape handle (ref-counted).
inline std::unique_ptr<TopoDS_Vertex> clone_vertex(const TopoDS_Vertex& v) {
    return std::make_unique<TopoDS_Vertex>(v);
}

inline double vertex_pnt_x(const TopoDS_Vertex& v) { return BRep_Tool::Pnt(v).X(); }
inline double vertex_pnt_y(const TopoDS_Vertex& v) { return BRep_Tool::Pnt(v).Y(); }
inline double vertex_pnt_z(const TopoDS_Vertex& v) { return BRep_Tool::Pnt(v).Z(); }
