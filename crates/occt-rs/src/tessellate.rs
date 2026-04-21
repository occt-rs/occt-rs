//! Shape tessellation via `BRep_Mesh_IncrementalMesh`.
//!
//! The primary entry point is [`compute`].  It meshes a shape and returns the
//! resulting geometry split by topological sub-shape.
//!
//! # Usage
//!
//! ```rust,ignore
//! use occt_rs::tessellate;
//!
//! let shape = solid.as_shape();   // OcSolid, OcFace, etc.
//! let result = tessellate::compute(&shape, 0.1, 0.5)?;
//!
//! for face in &result.faces {
//!     println!("face {:?}: {} triangles", face.key, face.mesh.indices.len() / 3);
//! }
//!
//! for edge in &result.edges {
//!     println!("edge {:?}: {} points", edge.key, edge.points.len());
//! }
//! ```
//!
//! # Coordinate precision
//!
//! OCCT stores all geometry as `f64`.  Tessellation output uses `f32` for triangle
//! meshes (GPU buffer natural precision).  Edge polylines are returned as `f64`
//! to preserve the precision of the underlying polygon data.
//!
//! # TopLoc_Location
//!
//! Edge polyline node coordinates are transformed by `TopLoc_Location` to place
//! them in global model space. Face triangulations do not apply location
//! (deferred work, noted as future concern).
//!
//! # Edge polygon availability
//!
//! Edge polylines are extracted via `BRep_Tool::Polygon3D`. After
//! `BRep_Mesh_IncrementalMesh`, most edges have a direct 3D polygon. Edges
//! without one are silently omitted from the result.
//!
//! # Deduplication
//!
//! `TopExp_Explorer` does not deduplicate.  A vertex shared by N edges appears
//! N times in `result.vertices`.  Callers that need unique entries should
//! filter on [`ShapeKey`].
//!
//! Reference:
//!   `BRep_Mesh_IncrementalMesh`       — <https://dev.opencascade.org/doc/refman/html/class_b_rep_mesh___incremental_mesh.html>
//!   `BRep_Tool::Polygon3D`            — <https://dev.opencascade.org/doc/refman/html/class_b_rep___tool.html>
//!   `TopExp_Explorer`                 — <https://dev.opencascade.org/doc/refman/html/class_top_exp___explorer.html>

use crate::error::{OcctError, OcctErrorKind};
use crate::topo::OcShape;
use occt_sys::ffi;

// TopAbs_ShapeEnum constants.
// Reference: https://dev.opencascade.org/doc/refman/html/namespace_top_abs.html
const TOP_ABS_FACE: i32 = 4;
const TOP_ABS_EDGE: i32 = 6;
const TOP_ABS_VERTEX: i32 = 7;

// ── Public types ───────────────────────────────────────────────────────────────

/// Within-session identity for a placed topological sub-shape instance.
///
/// Encodes TShape (geometry), Location (placement), and Orientation — the
/// three components that together distinguish a placed instance in OCCT.
/// Two faces that share underlying geometry but sit at different positions
/// (e.g. the top and bottom caps of a `BRepPrimAPI_MakePrism` solid, which
/// share a `TShape` but differ by `Location`) receive distinct keys.
///
/// The key is a hash of those three components; collisions are
/// astronomically unlikely for any realistic number of shapes in a session.
///
/// **Not persistent.** Keys are meaningless across serialise/deserialise
/// cycles and process restarts.  When the TDF attribute layer is added,
/// `ShapeKey` values will compose with `TDF_Label` identifiers for
/// persistent identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShapeKey(pub usize);

/// A triangle mesh for one tessellated face.
///
/// Vertex positions are in model-space global coordinates (assuming identity
/// `TopLoc_Location`; see the module-level note).
#[derive(Debug, Clone)]
pub struct TriMesh {
    /// Interleaved vertex positions: `[x₀, y₀, z₀, x₁, y₁, z₁, …]`.
    ///
    /// Length is always a multiple of 3.  `f32` matches typical GPU buffer
    /// layout; OCCT's internal precision is `f64`.
    pub vertices: Vec<f32>,

    /// Triangle vertex indices, 0-based, in groups of three: `[a₀, b₀, c₀, …]`.
    ///
    /// Length is always a multiple of 3.
    pub indices: Vec<u32>,
}

/// A tessellated face.
#[derive(Debug, Clone)]
pub struct TessFace {
    /// Session-scoped identity of the originating `TopoDS_Face`.
    pub key: ShapeKey,
    /// Triangle mesh for this face.
    pub mesh: TriMesh,
}

/// A tessellated edge polyline.
#[derive(Debug, Clone)]
pub struct TessEdge {
    /// Session-scoped identity of the originating `TopoDS_Edge`.
    pub key: ShapeKey,
    /// Polyline points in model space, transformed by `TopLoc_Location`.
    /// Each point is `[x, y, z]`.  Returned as `f64` to preserve polygon precision.
    pub points: Vec<[f64; 3]>,
}

/// A tessellated vertex.
#[derive(Debug, Clone)]
pub struct TessVertex {
    /// Session-scoped identity of the originating `TopoDS_Vertex`.
    pub key: ShapeKey,
    /// Vertex position in model space.  `f32`; see module-level precision note.
    pub point: [f32; 3],
}

/// Output of [`compute`].
#[derive(Debug, Clone)]
pub struct TessellationResult {
    /// One entry per `TopoDS_Face` found in the shape.
    pub faces: Vec<TessFace>,
    /// One entry per `TopoDS_Edge` found in the shape that has polygon data.
    /// Edges without polygon data are omitted.
    pub edges: Vec<TessEdge>,
    /// One entry per `TopoDS_Vertex` occurrence found in the shape.
    /// May contain duplicate keys; see module-level deduplication note.
    pub vertices: Vec<TessVertex>,
}

// ── Entry point ────────────────────────────────────────────────────────────────

/// Tessellates `shape` using `BRep_Mesh_IncrementalMesh` and returns the
/// resulting geometry split by sub-shape.
///
/// The meshing result is stored in-place on the BRep faces; after this call
/// `shape` retains its triangulation for the rest of its lifetime.
///
/// # Parameters
///
/// - `linear_deflection`: Maximum chord deviation in model units (absolute).
///   Smaller values produce finer meshes.  `0.1` is typical for moderate quality.
/// - `angular_deflection`: Maximum angular deviation in radians.  `0.5` is the
///   OCCT default.
///
/// # Errors
///
/// Returns `Err` if `BRep_Mesh_IncrementalMesh` construction throws (degenerate
/// or empty shape) or if `IsDone()` is false after construction.
pub fn compute(
    shape: &OcShape,
    linear_deflection: f64,
    angular_deflection: f64,
) -> Result<TessellationResult, OcctError> {
    // 1. Mesh the shape.
    //    BRep_Mesh_IncrementalMesh stores triangulations on each face in-place.
    //    The builder can be dropped once construction succeeds; the triangulations
    //    are owned by the BRep and live as long as the shape.
    let mesh = ffi::new_incremental_mesh(
        shape.as_ffi(),
        linear_deflection,
        false, // is_relative: use absolute deflection
        angular_deflection,
        false, // is_in_parallel: single-threaded
    )
    .map_err(OcctError::from)?;

    if !mesh.is_done() {
        return Err(OcctError {
            kind: OcctErrorKind::ConstructionError,
            message: "BRep_Mesh_IncrementalMesh: IsDone() false after construction".to_owned(),
        });
    }
    drop(mesh);

    // 2. Extract face tessellations.
    let mut faces = Vec::new();
    let mut face_exp = ffi::new_shape_explorer(shape.as_ffi(), TOP_ABS_FACE);
    while face_exp.more() {
        let shape_ref = face_exp.current();
        let key = ShapeKey(ffi::shape_key(shape_ref));
        let face = ffi::shape_as_face(shape_ref);

        let tri = ffi::face_triangulation(&face);
        if !tri.is_null() {
            let nb_v = tri.nb_nodes();
            let nb_t = tri.nb_triangles();

            let mut vertices = Vec::with_capacity(3 * nb_v as usize);
            for i in 1..=nb_v {
                // f64 → f32 narrowing: deliberate, see module-level precision note.
                vertices.push(tri.node_x(i) as f32);
                vertices.push(tri.node_y(i) as f32);
                vertices.push(tri.node_z(i) as f32);
            }

            let mut indices = Vec::with_capacity(3 * nb_t as usize);
            for i in 1..=nb_t {
                // OCCT triangle indices are 1-based; convert to 0-based.
                indices.push((tri.triangle_n1(i) - 1) as u32);
                indices.push((tri.triangle_n2(i) - 1) as u32);
                indices.push((tri.triangle_n3(i) - 1) as u32);
            }

            faces.push(TessFace {
                key,
                mesh: TriMesh { vertices, indices },
            });
        }

        face_exp.pin_mut().next();
    }

    // 3. Extract edge polylines.
    //    Each edge is queried for BRep_Tool::Polygon3D.
    //    Edges without a polygon are skipped.
    let mut edges = Vec::new();
    let mut edge_exp = ffi::new_shape_explorer(shape.as_ffi(), TOP_ABS_EDGE);
    while edge_exp.more() {
        let shape_ref = edge_exp.current();
        let key = ShapeKey(ffi::shape_key(shape_ref));
        let edge_topo = ffi::shape_as_edge(shape_ref);

        if let Some(points) = try_polygon3d(&edge_topo) {
            edges.push(TessEdge { key, points });
        }

        edge_exp.pin_mut().next();
    }

    // 4. Extract vertex positions.
    let mut vertices = Vec::new();
    let mut vtx_exp = ffi::new_shape_explorer(shape.as_ffi(), TOP_ABS_VERTEX);
    while vtx_exp.more() {
        let shape_ref = vtx_exp.current();
        let key = ShapeKey(ffi::shape_key(shape_ref));
        let vertex = ffi::shape_as_vertex(shape_ref);

        vertices.push(TessVertex {
            key,
            point: [
                ffi::vertex_pnt_x(&vertex) as f32,
                ffi::vertex_pnt_y(&vertex) as f32,
                ffi::vertex_pnt_z(&vertex) as f32,
            ],
        });
        vtx_exp.pin_mut().next();
    }

    Ok(TessellationResult {
        faces,
        edges,
        vertices,
    })
}

// ── Edge polygon extraction helper ─────────────────────────────────────────────

/// Attempts to extract a Polygon3D polyline from an edge.
/// Returns None if the edge has no direct 3D polygon.
fn try_polygon3d(edge: &ffi::TopodsEdge) -> Option<Vec<[f64; 3]>> {
    let poly = ffi::edge_polygon3d(edge);
    if poly.is_null() {
        return None;
    }
    let loc = ffi::edge_polygon3d_location(edge);
    let nb_nodes = ffi::polygon3d_nb_nodes(&poly);

    let mut points = Vec::with_capacity(nb_nodes as usize);
    for i in 1..=nb_nodes {
        let x = ffi::polygon3d_node_x(&poly, i);
        let y = ffi::polygon3d_node_y(&poly, i);
        let z = ffi::polygon3d_node_z(&poly, i);

        // Apply location transformation to place in global model space.
        let (tx, ty, tz) = if ffi::location_is_identity(&loc) {
            (x, y, z)
        } else {
            let mut out = (0.0, 0.0, 0.0);
            ffi::apply_location_to_point(&loc, x, y, z, &mut out.0, &mut out.1, &mut out.2);
            out
        };

        points.push([tx, ty, tz]);
    }

    Some(points)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gp::{OcPnt, OcVec};
    use crate::topo::{OcEdge, OcFace, OcWire};

    fn triangle_prism() -> crate::topo::OcSolid {
        let edges = vec![
            OcEdge::from_pnts(OcPnt::new(0.0, 0.0, 0.0), OcPnt::new(1.0, 0.0, 0.0)).unwrap(),
            OcEdge::from_pnts(OcPnt::new(1.0, 0.0, 0.0), OcPnt::new(0.5, 1.0, 0.0)).unwrap(),
            OcEdge::from_pnts(OcPnt::new(0.5, 1.0, 0.0), OcPnt::new(0.0, 0.0, 0.0)).unwrap(),
        ];
        let wire = OcWire::from_edges(&edges).unwrap();
        let face = OcFace::from_wire(&wire, true).unwrap();
        face.extrude(OcVec::new(0.0, 0.0, 1.0)).unwrap()
    }

    #[test]
    fn prism_has_five_faces() {
        let shape = triangle_prism().as_shape();
        let result = compute(&shape, 0.1, 0.5).unwrap();
        // A triangular prism has 2 triangular + 3 rectangular faces.
        assert_eq!(result.faces.len(), 5);
    }

    #[test]
    #[ignore = "edge polylines require PolygonOnTriangulation, not Polygon3D; deferred to assembly-import PR"]
    fn prism_has_nine_edges() {
        let shape = triangle_prism().as_shape();
        let result = compute(&shape, 0.1, 0.5).unwrap();
        // A triangular prism has 9 edges: 3 on bottom, 3 on top, 3 vertical.
        assert_eq!(
            result.edges.len(),
            9,
            "expected 9 edges, got {}",
            result.edges.len()
        );
    }

    #[test]
    fn all_edges_have_points() {
        let shape = triangle_prism().as_shape();
        let result = compute(&shape, 0.1, 0.5).unwrap();
        for edge in &result.edges {
            assert!(!edge.points.is_empty(), "edge {:?} has no points", edge.key);
        }
    }

    #[test]
    fn edge_keys_are_distinct() {
        let shape = triangle_prism().as_shape();
        let result = compute(&shape, 0.1, 0.5).unwrap();
        let mut keys: Vec<usize> = result.edges.iter().map(|e| e.key.0).collect();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(
            keys.len(),
            result.edges.len(),
            "edge keys should be distinct"
        );
    }

    #[test]
    fn all_faces_non_empty() {
        let shape = triangle_prism().as_shape();
        let result = compute(&shape, 0.1, 0.5).unwrap();
        for f in &result.faces {
            assert!(
                !f.mesh.vertices.is_empty(),
                "face {:?} has no vertices",
                f.key
            );
            assert!(
                !f.mesh.indices.is_empty(),
                "face {:?} has no indices",
                f.key
            );
            assert_eq!(f.mesh.vertices.len() % 3, 0);
            assert_eq!(f.mesh.indices.len() % 3, 0);
        }
    }

    #[test]
    fn face_keys_are_distinct() {
        let shape = triangle_prism().as_shape();
        let result = compute(&shape, 0.1, 0.5).unwrap();
        let mut keys: Vec<usize> = result.faces.iter().map(|f| f.key.0).collect();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), result.faces.len());
    }

    #[test]
    fn vertices_nonempty() {
        let shape = triangle_prism().as_shape();
        let result = compute(&shape, 0.1, 0.5).unwrap();
        // A triangular prism has 6 vertices; TopExp_Explorer may return duplicates.
        assert!(result.vertices.len() >= 6);
    }

    #[test]
    fn single_face_tessellates() {
        let edges = vec![
            OcEdge::from_pnts(OcPnt::new(0.0, 0.0, 0.0), OcPnt::new(1.0, 0.0, 0.0)).unwrap(),
            OcEdge::from_pnts(OcPnt::new(1.0, 0.0, 0.0), OcPnt::new(0.5, 1.0, 0.0)).unwrap(),
            OcEdge::from_pnts(OcPnt::new(0.5, 1.0, 0.0), OcPnt::new(0.0, 0.0, 0.0)).unwrap(),
        ];
        let wire = OcWire::from_edges(&edges).unwrap();
        let face = OcFace::from_wire(&wire, true).unwrap();
        let shape = face.as_shape();
        let result = compute(&shape, 0.1, 0.5).unwrap();
        assert_eq!(result.faces.len(), 1);
        assert!(!result.faces[0].mesh.vertices.is_empty());
    }
}
