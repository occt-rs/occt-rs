// occt_sys/topo/wire.hxx — TopoDS_Wire construction and edge exploration.
//
// MakeWireBuilder wraps BRepBuilderAPI_MakeWire; edges are added one at a
// time and each addition is checkable via IsDone().
//
// WireEdgeExplorer wraps BRepTools_WireExplorer, which iterates edges in
// oriented order consistent with the wire's orientation.
//
// Reference:
//   BRepBuilderAPI_MakeWire  — https://dev.opencascade.org/doc/refman/html/class_b_rep_builder_a_p_i___make_wire.html
//   BRepBuilderAPI_WireError — https://dev.opencascade.org/doc/refman/html/BRepBuilderAPI__WireError_8hxx.html
//   BRepTools_WireExplorer   — https://dev.opencascade.org/doc/refman/html/class_b_rep_tools___wire_explorer.html
//   TopoDS_Wire              — https://dev.opencascade.org/doc/refman/html/class_topo_d_s___wire.html
//
// Sourced from OCCT 7.9 documentation.
// No derivation from any other binding crate.

#pragma once

#include <memory>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRepBuilderAPI_WireError.hxx>
#include <BRepTools_WireExplorer.hxx>
#include <TopoDS_Wire.hxx>

#include "edge.hxx"

struct MakeWireBuilder {
    BRepBuilderAPI_MakeWire inner;
    void add_edge(const TopoDS_Edge& e) { inner.Add(e); }
    bool is_done() const { return inner.IsDone(); }
    int  error()   const { return static_cast<int>(inner.Error()); }
    std::unique_ptr<TopoDS_Wire> wire() {
        return std::make_unique<TopoDS_Wire>(inner.Wire());
    }
};

inline std::unique_ptr<MakeWireBuilder> new_make_wire_builder() {
    return std::make_unique<MakeWireBuilder>();
}

// TopoDS_Wire copy shares the underlying TShape handle (ref-counted).
inline std::unique_ptr<TopoDS_Wire> clone_wire(const TopoDS_Wire& w) {
    return std::make_unique<TopoDS_Wire>(w);
}

struct WireEdgeExplorer {
    BRepTools_WireExplorer inner;
    WireEdgeExplorer(const TopoDS_Wire& w) : inner(w) {}
    bool more() const { return inner.More(); }
    void next()       { inner.Next(); }
    std::unique_ptr<TopoDS_Edge> current_edge() const {
        return std::make_unique<TopoDS_Edge>(inner.Current());
    }
};

inline std::unique_ptr<WireEdgeExplorer> new_wire_edge_explorer(const TopoDS_Wire& w) {
    return std::make_unique<WireEdgeExplorer>(w);
}
