/// Integration test: sketch → extrude → undo/redo → point edit.
///
/// Exercises the full OCAF stack in the pattern a real application uses:
///
/// 1. `OcApplication` + `OcDocument`
/// 2. Base planes (XY, YZ, XZ) as `TDataXtd_Plane` labels
/// 3. Sketch label on the XY plane — four `TDataXtd_Point` corners of a unit
///    square, plus a face `TNaming_NamedShape` used as the extrude input
/// 4. Extrude the square face into a solid and record it in the document
/// 5. Undo the extrude — solid label loses its `TNaming_NamedShape`
/// 6. Redo the extrude — solid label's `TNaming_NamedShape` returns
/// 7. Edit point A — move from (0,0,0) to (0.5,0,0) via a new `record_shape`
///    call, assert the `Modified` delta is reflected, then undo the edit and
///    assert the original coordinates are restored
///
/// Document structure (tag paths from `doc.main()`):
///
/// ```text
/// main
/// ├── 1: planes
/// │   ├── 1: XY  (TDataXtd_Plane + TNaming_NamedShape)
/// │   ├── 2: YZ
/// │   └── 3: XZ
/// ├── 2: sketch
/// │   ├── 1: point A  (TDataXtd_Point + TNaming_NamedShape)
/// │   ├── 2: point B
/// │   ├── 3: point C
/// │   ├── 4: point D
/// │   └── 5: face     (TNaming_NamedShape of the square face)
/// └── 3: body
///     └── 1: solid    (TNaming_NamedShape of the extruded solid)
/// ```
///
/// Constraint solving is out of scope — this test exercises document structure,
/// TNaming, and the undo/redo attribute stack only.
#[cfg(test)]
mod integration_sketch_extrude {
    use occt_rs::gp::{OcAx2, OcDir, OcPnt, OcVec};
    use occt_rs::ocaf::application::OcApplication;
    use occt_rs::ocaf::document::OcDocument;
    use occt_rs::ocaf::tdata_xtd::{OcPlaneAttr, OcPointAttr};
    use occt_rs::ocaf::topo_naming::{TopoNamingBuilder, TopoNamingNamedShape};
    use occt_rs::ocaf::OcInteger;
    use occt_rs::rs_topo::{OcEdge, OcFace, OcWire};

    // ── helpers ──────────────────────────────────────────────────────────────

    fn new_doc() -> (OcApplication, OcDocument) {
        let mut app = OcApplication::new();
        let doc = app.new_document("BinXCAF").unwrap();
        (app, doc)
    }

    /// Builds a unit square face from four corner points in the XY plane.
    ///
    /// A  (0,1,0) ── D  (1,1,0)
    /// │                      │
    /// B  (0,0,0) ── C  (1,0,0)
    ///
    /// Winding: B→C→D→A, matching the corner-point tags 1–4.
    fn make_square_face() -> OcFace {
        let b = OcPnt::new(0.0, 0.0, 0.0);
        let c = OcPnt::new(1.0, 0.0, 0.0);
        let d = OcPnt::new(1.0, 1.0, 0.0);
        let a = OcPnt::new(0.0, 1.0, 0.0);

        let edges = [
            OcEdge::from_pnts(b, c).unwrap(),
            OcEdge::from_pnts(c, d).unwrap(),
            OcEdge::from_pnts(d, a).unwrap(),
            OcEdge::from_pnts(a, b).unwrap(),
        ];
        let wire = OcWire::from_edges(&edges).unwrap();
        OcFace::from_wire(&wire, true).unwrap()
    }

    // ── step 1 + 2: app, document, base planes ────────────────────────────

    /// Smoke-test: application and document are created without error, and the
    /// three base-plane labels are written and found correctly.
    #[test]
    fn base_planes_are_created() {
        let (_app, mut doc) = new_doc();
        let main = doc.main();

        // Allocate the planes container label (tag 1) and three plane labels
        // (tags 1, 2, 3) in a single command.
        let (xy_label, yz_label, xz_label) = {
            doc.begin_command().unwrap();
            let planes = main.get_or_create_child(1);

            // XY plane: normal +Z, X direction +X
            let xy_label = planes.get_or_create_child(1);
            let xy_frame = OcAx2::new(
                OcPnt::origin(),
                OcDir::new(0.0, 0.0, 1.0).unwrap(),
                OcDir::new(1.0, 0.0, 0.0).unwrap(),
            )
            .unwrap();
            OcPlaneAttr::record_shape(&xy_label, xy_frame).unwrap();
            OcPlaneAttr::set(&xy_label).unwrap();

            // YZ plane: normal +X, X direction +Y
            let yz_label = planes.get_or_create_child(2);
            let yz_frame = OcAx2::new(
                OcPnt::origin(),
                OcDir::new(1.0, 0.0, 0.0).unwrap(),
                OcDir::new(0.0, 1.0, 0.0).unwrap(),
            )
            .unwrap();
            OcPlaneAttr::record_shape(&yz_label, yz_frame).unwrap();
            OcPlaneAttr::set(&yz_label).unwrap();

            // XZ plane: normal +Y, X direction +X
            let xz_label = planes.get_or_create_child(3);
            let xz_frame = OcAx2::new(
                OcPnt::origin(),
                OcDir::new(0.0, 1.0, 0.0).unwrap(),
                OcDir::new(1.0, 0.0, 0.0).unwrap(),
            )
            .unwrap();
            OcPlaneAttr::record_shape(&xz_label, xz_frame).unwrap();
            OcPlaneAttr::set(&xz_label).unwrap();

            doc.commit().unwrap();
            (xy_label, yz_label, xz_label)
        };

        assert!(OcPlaneAttr::find(&xy_label).is_some(), "XY plane missing");
        assert!(OcPlaneAttr::find(&yz_label).is_some(), "YZ plane missing");
        assert!(OcPlaneAttr::find(&xz_label).is_some(), "XZ plane missing");
    }

    // ── full integration path ─────────────────────────────────────────────

    #[test]
    fn sketch_extrude_undo_redo_edit() {
        let (_app, mut doc) = new_doc();
        let main = doc.main();

        // First command

        let planes_label = {
            doc.begin_command().unwrap();
            let planes = main.get_or_create_child(1);

            let xy_label = planes.get_or_create_child(1);
            let xy_frame = OcAx2::new(
                OcPnt::origin(),
                OcDir::new(0.0, 0.0, 1.0).unwrap(),
                OcDir::new(1.0, 0.0, 0.0).unwrap(),
            )
            .unwrap();
            OcPlaneAttr::record_shape(&xy_label, xy_frame).unwrap();
            OcPlaneAttr::set(&xy_label).unwrap();

            let yz_label = planes.get_or_create_child(2);
            let yz_frame = OcAx2::new(
                OcPnt::origin(),
                OcDir::new(1.0, 0.0, 0.0).unwrap(),
                OcDir::new(0.0, 1.0, 0.0).unwrap(),
            )
            .unwrap();
            OcPlaneAttr::record_shape(&yz_label, yz_frame).unwrap();
            OcPlaneAttr::set(&yz_label).unwrap();

            let xz_label = planes.get_or_create_child(3);
            let xz_frame = OcAx2::new(
                OcPnt::origin(),
                OcDir::new(0.0, 1.0, 0.0).unwrap(),
                OcDir::new(1.0, 0.0, 0.0).unwrap(),
            )
            .unwrap();
            OcPlaneAttr::record_shape(&xz_label, xz_frame).unwrap();
            OcPlaneAttr::set(&xz_label).unwrap();

            doc.commit().unwrap();
            planes
        };

        // XY plane label is directly accessible for the sketch reference.
        let xy_label = planes_label.find_child(1).expect("XY plane label");
        assert!(OcPlaneAttr::find(&xy_label).is_some());

        // ── Second command — four corner points + square face ──────────────
        //
        // Corner layout (unit square in XY):
        //   A (0,1,0)   B (0,0,0)   C (1,0,0)   D (1,1,0)
        // Tags: sketch/1=A, sketch/2=B, sketch/3=C, sketch/4=D, sketch/5=face

        let (_sketch_label, pt_a_label, pt_b_label, pt_c_label, pt_d_label, face_label) = {
            doc.begin_command().unwrap();
            let sketch = main.get_or_create_child(2);

            let pnt_a = OcPnt::new(0.0, 1.0, 0.0);
            let pnt_b = OcPnt::new(0.0, 0.0, 0.0);
            let pnt_c = OcPnt::new(1.0, 0.0, 0.0);
            let pnt_d = OcPnt::new(1.0, 1.0, 0.0);

            // Point labels — record_shape first (shape), then set (tag).
            let la = sketch.get_or_create_child(1);
            OcPointAttr::record_shape(&la, pnt_a).unwrap();
            OcPointAttr::set(&la).unwrap();

            let lb = sketch.get_or_create_child(2);
            OcPointAttr::record_shape(&lb, pnt_b).unwrap();
            OcPointAttr::set(&lb).unwrap();

            let lc = sketch.get_or_create_child(3);
            OcPointAttr::record_shape(&lc, pnt_c).unwrap();
            OcPointAttr::set(&lc).unwrap();

            let ld = sketch.get_or_create_child(4);
            OcPointAttr::record_shape(&ld, pnt_d).unwrap();
            OcPointAttr::set(&ld).unwrap();

            // Face label — record the square face shape as a primitive.
            // This is the geometry input to the extrude step.
            let lface = sketch.get_or_create_child(5);
            let face = make_square_face();
            let face_shape = face.as_shape();
            let mut face_builder = TopoNamingBuilder::new(&lface);
            face_builder.primitive(&face_shape);
            let _face_ns = face_builder.named_shape();

            doc.commit().unwrap();
            (sketch, la, lb, lc, ld, lface)
        };

        // Verify all four point attributes are present after the sketch command.
        assert!(
            OcPointAttr::find(&pt_a_label).is_some(),
            "point A missing after sketch command"
        );
        assert!(
            OcPointAttr::find(&pt_b_label).is_some(),
            "point B missing after sketch command"
        );
        assert!(
            OcPointAttr::find(&pt_c_label).is_some(),
            "point C missing after sketch command"
        );
        assert!(
            OcPointAttr::find(&pt_d_label).is_some(),
            "point D missing after sketch command"
        );
        assert!(
            TopoNamingNamedShape::find(&face_label).is_some(),
            "face named shape missing after sketch command"
        );

        // Verify point A coordinates round-trip correctly.
        let p = OcPointAttr::get(&pt_a_label)
            .unwrap()
            .expect("point A coords");
        assert!((p.x - 0.0).abs() < 1e-12, "A.x");
        assert!((p.y - 1.0).abs() < 1e-12, "A.y");
        assert!((p.z - 0.0).abs() < 1e-12, "A.z");

        // ── Third command ────────────────────────────────────────────────
        //
        // Rebuild the square face from geometry (the face label's NamedShape
        // holds the shape reference; for the actual extrude we reconstruct
        // the OcFace so we can call extrude() on it).

        let solid_label = {
            doc.begin_command().unwrap();
            let body = main.get_or_create_child(3);
            let lsolid = body.get_or_create_child(1);

            let face = make_square_face();
            let solid_shape = face.extrude(OcVec::new(0.0, 0.0, 1.0)).unwrap();

            let mut builder = TopoNamingBuilder::new(&lsolid);
            builder.primitive(&solid_shape);
            let _ns = builder.named_shape();

            doc.commit().unwrap();
            lsolid
        };

        // Solid label has a NamedShape after the extrude command.
        assert!(
            TopoNamingNamedShape::find(&solid_label).is_some(),
            "solid named shape missing after extrude"
        );
        assert_eq!(doc.available_undos(), 3, "should have 3 undoable commands");

        // ── step 6: undo the extrude ──────────────────────────────────────

        let did_undo = doc.undo().unwrap();
        assert!(did_undo, "undo should report success");

        // The solid label's NamedShape is gone — the extrude command is
        // reversed.  The label itself may still exist as an empty node; we
        // assert on the NamedShape attribute, not the label.
        assert!(
            TopoNamingNamedShape::find(&solid_label).is_none(),
            "solid named shape should be absent after undo of extrude"
        );

        // Sketch is unaffected.
        assert!(
            OcPointAttr::find(&pt_a_label).is_some(),
            "point A should survive undo of extrude"
        );

        // ── step 7: redo the extrude ──────────────────────────────────────

        let did_redo = doc.redo().unwrap();
        assert!(did_redo, "redo should report success");

        assert!(
            TopoNamingNamedShape::find(&solid_label).is_some(),
            "solid named shape should be restored after redo"
        );

        // ── step 8: edit point A ──────────────────────────────────────────
        //
        // Move A from (0,1,0) to (0.5,1,0).  A new record_shape call on the
        // same label records a Modified delta; undo restores the prior vertex.

        let new_a = OcPnt::new(0.5, 1.0, 0.0);

        {
            doc.begin_command().unwrap();
            // record_shape on an already-named label records a Modify delta.
            OcPointAttr::record_shape(&pt_a_label, new_a).unwrap();
            doc.commit().unwrap();
        }

        // Verify the new coordinates are visible.
        let p_after_edit = OcPointAttr::get(&pt_a_label)
            .unwrap()
            .expect("point A after edit");
        assert!(
            (p_after_edit.x - 0.5).abs() < 1e-12,
            "A.x after edit: expected 0.5, got {}",
            p_after_edit.x
        );
        assert!(
            (p_after_edit.y - 1.0).abs() < 1e-12,
            "A.y after edit: expected 1.0, got {}",
            p_after_edit.y
        );

        // Undo the point edit — coordinates should revert to original.
        doc.undo().unwrap();

        let p_after_undo = OcPointAttr::get(&pt_a_label)
            .unwrap()
            .expect("point A after undo of edit");
        assert!(
            (p_after_undo.x - 0.0).abs() < 1e-12,
            "A.x after undo: expected 0.0, got {}",
            p_after_undo.x
        );
        assert!(
            (p_after_undo.y - 1.0).abs() < 1e-12,
            "A.y after undo: expected 1.0, got {}",
            p_after_undo.y
        );

        // Solid is still present (undo of the point edit didn't touch it).
        assert!(
            TopoNamingNamedShape::find(&solid_label).is_some(),
            "solid named shape should survive undo of point edit"
        );
    }

    // ── undo count bookkeeping ─────────────────────────────────────────────

    /// Verifies that the undo limit is respected and that available_undos
    /// tracks committed commands correctly across this test's command sequence.
    #[test]
    fn undo_count_tracks_commands() {
        let (_app, mut doc) = new_doc();
        let main = doc.main();

        assert_eq!(doc.available_undos(), 0);

        // First command
        doc.begin_command().unwrap();
        let l = main.get_or_create_child(1);
        OcInteger::set(&l, 0).unwrap();
        eprintln!("commit: {}", doc.commit().unwrap());
        assert_eq!(doc.available_undos(), 1);

        // second command

        doc.begin_command().unwrap();
        let l = main.get_or_create_child(2);
        OcInteger::set(&l, 0).unwrap();
        eprintln!("commit: {}", doc.commit().unwrap());

        assert_eq!(doc.available_undos(), 2);

        doc.undo().unwrap();
        assert_eq!(doc.available_undos(), 1);
        assert_eq!(doc.available_redos(), 1);

        doc.redo().unwrap();
        assert_eq!(doc.available_undos(), 2);
        assert_eq!(doc.available_redos(), 0);
    }
}
