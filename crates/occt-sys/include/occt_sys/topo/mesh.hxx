// occt_sys/topo/mesh.hxx — BRep_Mesh_IncrementalMesh and Poly_Triangulation access.
//
// IncrementalMeshBuilder wraps BRep_Mesh_IncrementalMesh.  The constructor
// performs meshing immediately (calls Perform() internally); check is_done()
// after construction.  The triangulation is stored on each face of the BRep
// in-place and survives the builder's lifetime.
//
// BRep_Mesh_IncrementalMesh inherits Standard_Transient; it is held here via
// Handle to follow the OCCT object-lifecycle convention.
//
// PolyTriangulationHandle wraps Handle(Poly_Triangulation) so the triangulation
// can cross the cxx FFI boundary.  Node and triangle indices are 1-based (OCCT
// convention); convert to 0-based on the Rust side.
//
// TODO: TopLoc_Location is not applied to node coordinates in this release.
// For shapes built with BRepBuilderAPI_* APIs (no STEP/IGES assembly placement),
// the location is always identity and node coordinates are already in global
// space.  Location support is deferred to the assembly-import PR.
//
// Reference:
//   BRep_Mesh_IncrementalMesh — https://dev.opencascade.org/doc/refman/html/class_b_rep_mesh___incremental_mesh.html
//   BRep_Tool::Triangulation  — https://dev.opencascade.org/doc/refman/html/class_b_rep___tool.html
//   Poly_Triangulation        — https://dev.opencascade.org/doc/refman/html/class_poly___triangulation.html
//   Poly_Triangle             — https://dev.opencascade.org/doc/refman/html/class_poly___triangle.html
//
// Sourced from OCCT 7.9 documentation.
// No derivation from any other binding crate.

#pragma once

#include <memory>

#include <BRepMesh_IncrementalMesh.hxx>
#include <BRep_Tool.hxx>
#include <Poly_Triangulation.hxx>
#include <TopLoc_Location.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Shape.hxx>

#include "../exception.hxx"

// ── IncrementalMeshBuilder ─────────────────────────────────────────────────────

struct IncrementalMeshBuilder {
    Handle(BRepMesh_IncrementalMesh) inner;

    // theLinDeflection : maximum chord deviation (model units; absolute unless
    //                    is_rel is true)
    // is_rel           : if true, deflection is relative to edge length
    // theAngDeflection : maximum angular deviation (radians)
    // is_par           : if true, run meshing in parallel (requires a
    //                    thread-safe OCCT build)
    //
    // The constructor calls Perform() automatically; IsDone() reflects the
    // result.
    IncrementalMeshBuilder(
        const TopoDS_Shape& shape,
        double lin_def,
        bool   is_rel,
        double ang_def,
        bool   is_par)
        : inner(new BRepMesh_IncrementalMesh(
            shape,
            lin_def,
            is_rel ? Standard_True : Standard_False,
            ang_def,
            is_par ? Standard_True : Standard_False))
    {}

    bool is_done() const { return inner->IsDone() == Standard_True; }
};

// Factory.  BRep_Mesh_IncrementalMesh can throw Standard_Failure for degenerate
// or empty shapes; wrapped and re-thrown as std::runtime_error.
inline std::unique_ptr<IncrementalMeshBuilder> new_incremental_mesh(
    const TopoDS_Shape& shape,
    double lin_def,
    bool   is_rel,
    double ang_def,
    bool   is_par)
{
    try {
        return std::make_unique<IncrementalMeshBuilder>(
            shape, lin_def, is_rel, ang_def, is_par);
    } catch (const std::runtime_error&) {
        throw;  // don't double-wrap already-marshalled exceptions
    } catch (...) {
        rethrow_occt_as_runtime_error();
    }
}

// ── PolyTriangulationHandle ────────────────────────────────────────────────────
//
// Callers must check is_null() before calling any other method.
// Behaviour on null dereference is undefined.

struct PolyTriangulationHandle {
    Handle(Poly_Triangulation) inner;

    explicit PolyTriangulationHandle(Handle(Poly_Triangulation) h)
        : inner(std::move(h)) {}

    bool is_null()      const { return inner.IsNull(); }
    int  nb_nodes()     const { return inner->NbNodes(); }
    int  nb_triangles() const { return inner->NbTriangles(); }

    // Raw node coordinate at 1-based index i (see file-level TODO re: location).
    double node_x(int i) const { return inner->Node(i).X(); }
    double node_y(int i) const { return inner->Node(i).Y(); }
    double node_z(int i) const { return inner->Node(i).Z(); }

    // Triangle vertex indices at 1-based index i (1-based OCCT convention).
    // Poly_Triangle::Get returns all three at once; three separate methods
    // minimise the shim surface while staying cxx-compatible.
    int triangle_n1(int i) const {
        Standard_Integer n1, n2, n3;
        inner->Triangle(i).Get(n1, n2, n3);
        return n1;
    }
    int triangle_n2(int i) const {
        Standard_Integer n1, n2, n3;
        inner->Triangle(i).Get(n1, n2, n3);
        return n2;
    }
    int triangle_n3(int i) const {
        Standard_Integer n1, n2, n3;
        inner->Triangle(i).Get(n1, n2, n3);
        return n3;
    }
};

// Returns the triangulation for the given face.  The returned handle is null
// if BRep_Mesh_IncrementalMesh has not been called on the containing shape.
//
// TopLoc_Location is retrieved but not applied (see file-level TODO).
inline std::unique_ptr<PolyTriangulationHandle> face_triangulation(
    const TopoDS_Face& f)
{
    TopLoc_Location loc;
    return std::make_unique<PolyTriangulationHandle>(
        BRep_Tool::Triangulation(f, loc));
}
