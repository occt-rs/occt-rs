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
// Node coordinates are stored in the face's LOCAL frame. The face's
// TopLoc_Location (BRep_Tool::Triangulation out-parameter) places them in
// world space and is surfaced via placement_is_identity()/placement_value();
// it is NOT applied to the node_* accessors, which stay local.
//
// Co-located faces (prism caps reuse one moved TShape) share a single local
// triangulation and differ only by this location — dropping it collapses them.
//
// placement_value() is a lossy 3x4 projection for presentation. The structured
// TopLoc_Location (for STEP/IGES assembly export) is not preserved here.
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
#include <BRepLib_ToolTriangulatedShape.hxx>
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
    bool                       loc_is_identity;
    double                     loc_values[12];  // row-major 3x4; same order as transform.hxx

    PolyTriangulationHandle(Handle(Poly_Triangulation) h, const TopLoc_Location& loc)
        : inner(std::move(h)),
          loc_is_identity(loc.IsIdentity())
    {
        const gp_Trsf& t = loc.Transformation();
        // rows 1-3, cols 1-4 — identical layout to the SetValues args in transform.hxx
        loc_values[0]  = t.Value(1,1); loc_values[1]  = t.Value(1,2);
        loc_values[2]  = t.Value(1,3); loc_values[3]  = t.Value(1,4);
        loc_values[4]  = t.Value(2,1); loc_values[5]  = t.Value(2,2);
        loc_values[6]  = t.Value(2,3); loc_values[7]  = t.Value(2,4);
        loc_values[8]  = t.Value(3,1); loc_values[9]  = t.Value(3,2);
        loc_values[10] = t.Value(3,3); loc_values[11] = t.Value(3,4);
    }

    bool is_null()      const { return inner.IsNull(); }
    int  nb_nodes()     const { return inner->NbNodes(); }
    int  nb_triangles() const { return inner->NbTriangles(); }

    // Raw node coordinate at 1-based index i (see file-level TODO re: location).
    double node_x(int i) const { return inner->Node(i).X(); }
    double node_y(int i) const { return inner->Node(i).Y(); }
    double node_z(int i) const { return inner->Node(i).Z(); }
    // Reference: https://dev.opencascade.org/doc/refman/html/class_poly___triangulation.html
    // Valid only when has_normals() is true; caller must guard.
    bool   has_normals()   const { return inner->HasNormals(); }
    double normal_x(int i) const { return inner->Normal(i).X(); }
    double normal_y(int i) const { return inner->Normal(i).Y(); }
    double normal_z(int i) const { return inner->Normal(i).Z(); }

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
    bool   placement_is_identity() const { return loc_is_identity; }
    double placement_value(int i) const  { return loc_values[i]; }
};

// Returns the triangulation for the given face.  The returned handle is null
// if BRep_Mesh_IncrementalMesh has not been called on the containing shape.
inline std::unique_ptr<PolyTriangulationHandle> face_triangulation(
    const TopoDS_Face& f)
{
    TopLoc_Location loc;
    Handle(Poly_Triangulation) tri = BRep_Tool::Triangulation(f, loc);
    return std::make_unique<PolyTriangulationHandle>(std::move(tri), loc);
}
// Reference: https://dev.opencascade.org/doc/refman/html/class_b_rep_lib___tool_triangulated_shape.html
// Populates per-node surface normals on the face's triangulation in place,
// evaluating the surface at the UV nodes BRepMesh stored. Best-effort: returns
// false (no abort) on a face with no triangulation or if computation throws.
// Idempotent — ComputeNormals does nothing if normals already present.
inline bool compute_face_normals(const TopoDS_Face& f) {
    try {
        TopLoc_Location loc;
        Handle(Poly_Triangulation) tri = BRep_Tool::Triangulation(f, loc);
        if (tri.IsNull()) return false;
        BRepLib_ToolTriangulatedShape::ComputeNormals(f, tri);
        return tri->HasNormals();
    } catch (...) {
        return false;
    }
}
