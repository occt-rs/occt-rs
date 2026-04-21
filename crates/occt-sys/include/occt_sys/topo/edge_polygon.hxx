// Extracts polyline data from edges via BRep_Tool::Polygon3D (direct 3D polyline).
// Applies TopLoc_Location transformation to node coordinates in model space.
//
// Reference:
//   BRep_Tool                         — https://dev.opencascade.org/doc/refman/html/class_b_rep___tool.html
//   Poly_Polygon3D                    — https://dev.opencascade.org/doc/refman/html/class_poly___polygon3_d.html
//   TopLoc_Location                   — https://dev.opencascade.org/doc/refman/html/class_top_loc___location.html
//
// Sourced from OCCT 7.9 documentation.
// No derivation from any other binding crate.

#pragma once

#include <memory>
#include <NCollection_Handle.hxx>
#include <BRep_Tool.hxx>
#include <Poly_Polygon3D.hxx>
#include <TopoDS_Edge.hxx>
#include <TopLoc_Location.hxx>
#include <gp_Pnt.hxx>

// Opaque wrapper for Poly_Polygon3D.
struct PolyPolygon3D {
    Handle(Poly_Polygon3D) inner;
};

// Opaque wrapper for TopLoc_Location.
struct TopLocLocation {
    TopLoc_Location inner;
};

// ── Polygon3D extraction ──────────────────────────────────────────────────────

// Attempts to extract a Poly_Polygon3D from an edge.
// Returns nullptr if the edge has no direct 3D polygon.
inline std::unique_ptr<PolyPolygon3D> edge_polygon3d(const TopoDS_Edge& e) {
    TopLoc_Location loc;
    Handle(Poly_Polygon3D) poly = BRep_Tool::Polygon3D(e, loc);
    if (poly.IsNull()) {
        return nullptr;
    }
    auto result = std::make_unique<PolyPolygon3D>();
    result->inner = poly;
    return result;
}

// Gets the location associated with an edge's Polygon3D.
inline std::unique_ptr<TopLocLocation> edge_polygon3d_location(const TopoDS_Edge& e) {
    TopLoc_Location loc;
    BRep_Tool::Polygon3D(e, loc);
    auto result = std::make_unique<TopLocLocation>();
    result->inner = loc;
    return result;
}

// ── Polygon3D node access ─────────────────────────────────────────────────────

// Number of nodes in a Poly_Polygon3D.
inline int polygon3d_nb_nodes(const PolyPolygon3D& p) {
    return p.inner->NbNodes();
}

// Get node i (1-based indexing, as per OCCT convention).
// Returns coordinates as separate accessors to avoid crossing FFI boundary
// with a gp_Pnt value.
inline double polygon3d_node_x(const PolyPolygon3D& p, int i) {
    return p.inner->Nodes()(i).X();
}

inline double polygon3d_node_y(const PolyPolygon3D& p, int i) {
    return p.inner->Nodes()(i).Y();
}

inline double polygon3d_node_z(const PolyPolygon3D& p, int i) {
    return p.inner->Nodes()(i).Z();
}

// ── TopLoc_Location transformation ────────────────────────────────────────────

// Apply a TopLoc_Location to a point and return the transformed coordinates.
// The transformation is: point' = location.Transformation() * point
inline void apply_location_to_point(
    const TopLocLocation& loc,
    double x, double y, double z,
    double& out_x, double& out_y, double& out_z)
{
    gp_Pnt p(x, y, z);
    gp_Pnt transformed = p.Transformed(loc.inner.Transformation());
    out_x = transformed.X();
    out_y = transformed.Y();
    out_z = transformed.Z();
}

// Determine if a location is identity (no-op transformation).
inline bool location_is_identity(const TopLocLocation& loc) {
    return loc.inner.IsIdentity();
}
