//! Sequential polyline accumulator for interactive sketch workflows.
//!
//! [`SketchPolylineBuilder`] accumulates points one at a time, maintaining
//! topological connectivity between consecutive edges by reusing the same
//! [`OcVertex`] handle at each shared endpoint.  This is the same vertex-sharing
//! principle as [`KeyedWireBuilder`] but driven by sequential point addition
//! rather than explicit key pairs.
//!
//! # Probe methods
//!
//! [`current_open_wire`] and [`possible_closing_face`] are non-destructive probes
//! — they do not modify the builder state and may be called at any point.
//!
//! [`current_open_wire`]: SketchPolylineBuilder::current_open_wire
//! [`possible_closing_face`]: SketchPolylineBuilder::possible_closing_face

use crate::gp::OcPnt;
use crate::topo::{OcEdge, OcFace, OcVertex, OcWire};

/// Accumulates a polyline as a sequence of topologically connected edges.
///
/// Each call to [`add_point`] appends a vertex.  Edges are formed lazily
/// between consecutive vertices when queried.  Because the same [`OcVertex`]
/// handle is reused at each shared endpoint, no proximity tolerance is required
/// to achieve topological connectivity — adjacent edges are connected by
/// construction.
///
/// # Coincident points
///
/// [`add_point`] is infallible.  If two consecutive points are coincident,
/// [`edges`] will silently truncate at that pair (an edge between coincident
/// vertices is invalid in OCCT), and the probe methods will return `None`.
/// The caller is responsible for ensuring consecutive points are distinct.
///
/// [`add_point`]: SketchPolylineBuilder::add_point
/// [`edges`]: SketchPolylineBuilder::edges
pub struct SketchPolylineBuilder {
    /// One vertex per accumulated point, in insertion order.
    /// Adjacent edges share a vertex handle — topological connectivity
    /// is guaranteed without any proximity check.
    vertices: Vec<OcVertex>,
}

impl SketchPolylineBuilder {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
        }
    }

    /// Appends a point to the polyline.
    ///
    /// Infallible.  Consecutive coincident points are permitted at this stage
    /// but will cause [`edges`] to truncate; see the struct-level note.
    ///
    /// [`edges`]: SketchPolylineBuilder::edges
    pub fn add_point(&mut self, p: OcPnt) {
        self.vertices.push(OcVertex::from_pnt(&p));
    }

    /// Number of points accumulated so far.
    pub fn point_count(&self) -> usize {
        self.vertices.len()
    }

    /// The accumulated vertices in insertion order.
    pub fn vertices(&self) -> &[OcVertex] {
        &self.vertices
    }

    /// The open-polyline edges in insertion order.
    ///
    /// Returns edges for consecutive vertex pairs up to but not including the
    /// first coincident pair.  Returns an empty `Vec` when fewer than 2 points
    /// have been added.
    pub fn edges(&self) -> Vec<OcEdge> {
        self.vertices
            .windows(2)
            .map(|w| OcEdge::from_vertices(&w[0], &w[1]))
            .take_while(|r| r.is_ok())
            .map(|r| r.unwrap())
            .collect()
    }

    /// Snapshots the current in-progress polyline as an open [`OcWire`].
    ///
    /// Returns `None` if fewer than 2 points have been added, or if any
    /// consecutive point pair is coincident (edge construction would fail).
    ///
    /// Does not modify the builder state.
    pub fn current_open_wire(&self) -> Option<OcWire> {
        if self.vertices.len() < 2 {
            return None;
        }
        let edges = self.build_edges_all_or_none()?;
        OcWire::from_edges(&edges).ok()
    }

    /// Probes whether the current polyline can be closed into a planar face.
    ///
    /// Attempts to add a closing edge from the last accumulated point back to
    /// the first, build the resulting wire, and construct a planar face from it.
    /// Returns the face if successful, `None` otherwise.
    ///
    /// Failure cases include: fewer than 3 points, coincident points, a
    /// non-planar point sequence, or a degenerate closing edge (last point
    /// coincident with first).
    ///
    /// Does not modify the builder state.
    pub fn possible_closing_face(&self) -> Option<OcFace> {
        if self.vertices.len() < 3 {
            return None;
        }
        let mut edges = self.build_edges_all_or_none()?;
        // Closing edge: last vertex → first vertex.
        let closing = OcEdge::from_vertices(
            self.vertices.last().unwrap(),
            self.vertices.first().unwrap(),
        )
        .ok()?;
        edges.push(closing);
        let wire = OcWire::from_edges(&edges).ok()?;
        // only_plane: true — if the points are not coplanar, OCCT rejects the
        // wire and we return None rather than silently producing a warped face.
        OcFace::from_wire(&wire, true).ok()
    }

    /// Builds edges for all consecutive vertex pairs, returning `None` on the
    /// first failure (coincident vertices).
    fn build_edges_all_or_none(&self) -> Option<Vec<OcEdge>> {
        self.vertices
            .windows(2)
            .map(|w| OcEdge::from_vertices(&w[0], &w[1]).ok())
            .collect()
    }
}

impl Default for SketchPolylineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triangle_points() -> [OcPnt; 3] {
        [
            OcPnt::new(0.0, 0.0, 0.0),
            OcPnt::new(1.0, 0.0, 0.0),
            OcPnt::new(0.5, 1.0, 0.0),
        ]
    }

    #[test]
    fn empty_builder_has_no_edges_or_wire() {
        let b = SketchPolylineBuilder::new();
        assert_eq!(b.point_count(), 0);
        assert!(b.edges().is_empty());
        assert!(b.current_open_wire().is_none());
        assert!(b.possible_closing_face().is_none());
    }

    #[test]
    fn single_point_has_no_edges_or_wire() {
        let mut b = SketchPolylineBuilder::new();
        b.add_point(OcPnt::new(0.0, 0.0, 0.0));
        assert_eq!(b.point_count(), 1);
        assert!(b.edges().is_empty());
        assert!(b.current_open_wire().is_none());
        assert!(b.possible_closing_face().is_none());
    }

    #[test]
    fn two_points_gives_one_edge_and_open_wire() {
        let mut b = SketchPolylineBuilder::new();
        b.add_point(OcPnt::new(0.0, 0.0, 0.0));
        b.add_point(OcPnt::new(1.0, 0.0, 0.0));
        assert_eq!(b.edges().len(), 1);
        assert!(b.current_open_wire().is_some());
        // Two points cannot form a face.
        assert!(b.possible_closing_face().is_none());
    }

    #[test]
    fn three_coplanar_points_close_to_face() {
        let pts = triangle_points();
        let mut b = SketchPolylineBuilder::new();
        for p in &pts {
            b.add_point(*p);
        }
        assert_eq!(b.edges().len(), 2);
        assert!(b.current_open_wire().is_some());
        assert!(b.possible_closing_face().is_some());
    }

    #[test]
    fn possible_closing_face_is_non_destructive() {
        // Calling possible_closing_face does not change point_count or edges.
        let pts = triangle_points();
        let mut b = SketchPolylineBuilder::new();
        for p in &pts {
            b.add_point(*p);
        }
        let _ = b.possible_closing_face();
        assert_eq!(b.point_count(), 3);
        assert_eq!(b.edges().len(), 2);
    }

    #[test]
    fn current_open_wire_is_non_destructive() {
        let mut b = SketchPolylineBuilder::new();
        b.add_point(OcPnt::new(0.0, 0.0, 0.0));
        b.add_point(OcPnt::new(1.0, 0.0, 0.0));
        let _ = b.current_open_wire();
        assert_eq!(b.point_count(), 2);
    }

    #[test]
    fn nonplanar_points_cannot_close_to_face() {
        // Four non-coplanar points: a tetrahedron vertex arrangement.
        let mut b = SketchPolylineBuilder::new();
        b.add_point(OcPnt::new(0.0, 0.0, 0.0));
        b.add_point(OcPnt::new(1.0, 0.0, 0.0));
        b.add_point(OcPnt::new(0.5, 1.0, 0.0));
        b.add_point(OcPnt::new(0.5, 0.5, 1.0)); // lifts off the plane
        assert!(b.possible_closing_face().is_none());
    }

    #[test]
    fn four_coplanar_points_close_to_face() {
        // Square in XY plane.
        let mut b = SketchPolylineBuilder::new();
        b.add_point(OcPnt::new(0.0, 0.0, 0.0));
        b.add_point(OcPnt::new(1.0, 0.0, 0.0));
        b.add_point(OcPnt::new(1.0, 1.0, 0.0));
        b.add_point(OcPnt::new(0.0, 1.0, 0.0));
        assert_eq!(b.edges().len(), 3);
        assert!(b.possible_closing_face().is_some());
    }

    #[test]
    fn closing_face_can_be_extruded() {
        let pts = triangle_points();
        let mut b = SketchPolylineBuilder::new();
        for p in &pts {
            b.add_point(*p);
        }
        let face = b.possible_closing_face().unwrap();
        let solid = face.extrude(crate::gp::OcVec::new(0.0, 0.0, 1.0));
        assert!(solid.is_ok());
    }
}
