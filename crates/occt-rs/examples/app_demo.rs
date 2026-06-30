//! Demonstrates how to use occt-rs to build a simple parametric CAD application.
//!
//! HEAVY WORK IN PROGRESS. Eventually, this example will provide A distillaton of the core details
//! needed to make an application. For now, it's more of an MVP research file.
//!
//! The demo models a user working through a typical CAD workflow:
//!
//! 1. Create a document
//! 2. Establish base planes
//! 3. Draw a unit square sketch on the XY plane
//! 4. Extrude the sketch into a solid
//! 5. Chamfer one edge of the solid
//! 6. Select the chamfer face as the reference plane for a second sketch
//!
//! Each step is modelled as an [`AppCommand`] dispatched through a handler.
//! This mirrors how a real application would process user actions — the library
//! calls are in the handlers, not in `main`.
//!
//! ## Document tree produced
//!
//! ```text
//! main (0:1)
//! ├── 1 (0:1:1)   planes
//! │   ├── 1 (0:1:1:1)   XY plane
//! │   │       TopoNamingNamedShape (Primitive, planar face)
//! │   │       OcPlaneAttr
//! │   ├── 2 (0:1:1:2)   YZ plane
//! │   │       TopoNamingNamedShape (Primitive, planar face)
//! │   │       OcPlaneAttr
//! │   └── 3 (0:1:1:3)   XZ plane
//! │           TopoNamingNamedShape (Primitive, planar face)
//! │           OcPlaneAttr
//! ├── 2 (0:1:2)   sketch
//! │   ├── 1 (0:1:2:1)   point A (0.0, 1.0, 0.0)
//! │   │       TopoNamingNamedShape (Primitive, vertex)
//! │   │       OcPointAttr
//! │   ├── 2 (0:1:2:2)   point B (0.0, 0.0, 0.0)
//! │   │       TopoNamingNamedShape (Primitive, vertex)
//! │   │       OcPointAttr
//! │   ├── 3 (0:1:2:3)   point C (1.0, 0.0, 0.0)
//! │   │       TopoNamingNamedShape (Primitive, vertex)
//! │   │       OcPointAttr
//! │   ├── 4 (0:1:2:4)   point D (1.0, 1.0, 0.0)
//! │   │       TopoNamingNamedShape (Primitive, vertex)
//! │   │       OcPointAttr
//! │   └── 5 (0:1:2:5)   face
//! │           TopoNamingNamedShape (Primitive, unit square face)
//! ├── 3 (0:1:3)   body
//! │   ├── 1 (0:1:3:1)   solid
//! │   │       TopoNamingNamedShape (Primitive, 1×1×1 prism)
//! │   │       OcReal "depth" = 1.0
//! │   └── 2 (0:1:3:2)   chamfer
//! │           TopoNamingNamedShape (Modify, chamfered solid)
//! │           OcReal "distance" = 0.05
//! └── 4 (0:1:4)   sketch2
//!     └── 5 (0:1:4:5)   ref-face
//!             TopoNamingNamedShape (Selected — TopoNamingSelector)
//! ```
//!
//! ## Known gaps
//!
//! The parametric rebuild cycle — editing a sketch point and having the solid,
//! chamfer, and selector all update — requires `OcFace::from_shape` to retrieve
//! the face shape from the document and pass it to `extrude()`. Until that
//! binding is added, the rebuild step is present as commented-out code with
//! inline explanation. Search for `TODO(from_shape)` to find all affected sites.

use occt_rs::gp::{OcAx2, OcDir, OcPnt, OcVec};
use occt_rs::ocaf::attributes::OcReal;
use occt_rs::ocaf::tdata_xtd::{OcPlaneAttr, OcPointAttr};
use occt_rs::ocaf::topo_naming::{TopoNamingEvolution, TopoNamingNamedShape};
use occt_rs::ocaf::{OcApplication, OcDocument, OcLabel};
use occt_rs::rs_topo::{ChamferBuilder, OcEdge, OcFace, OcWire};

fn main() -> Result<(), AppError> {
    let mut state = AppState::new()?;
    state.doc.set_undo_limit(20);

    let commands = [
        AppCommand::AddSketchPoint {
            tag: 1,
            x: 0.0,
            y: 1.0,
            z: 0.0,
        },
        AppCommand::AddSketchPoint {
            tag: 2,
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        AppCommand::AddSketchPoint {
            tag: 3,
            x: 1.0,
            y: 0.0,
            z: 0.0,
        },
        AppCommand::AddSketchPoint {
            tag: 4,
            x: 1.0,
            y: 1.0,
            z: 0.0,
        },
        AppCommand::AddSketchFace {
            point_paths: vec![
                "0:1:2:1".to_string(),
                "0:1:2:2".to_string(),
                "0:1:2:3".to_string(),
                "0:1:2:4".to_string(),
                "0:1:2:1".to_string(), // close the wire back to A
            ],
            sketch_tag: 2,
        },
        AppCommand::Extrude { depth: 1.0 },
        AppCommand::Chamfer {
            distance: 0.05,
            solid_label_path: "0:1:3:1".to_string(),
            edge_index: 0, // user clicked the first edge — in a real app this is a pick result
        },
        AppCommand::SelectChamferFace,
        AppCommand::PrintTree,
        AppCommand::Undo,
        AppCommand::PrintTree,
        AppCommand::Redo,
        AppCommand::PrintTree,
    ];

    for command in commands {
        dispatch(&mut state, command)?;
    }

    Ok(())
}

/// Top-level application error.
///
/// In a real application this would have more variants — IO errors, serialisation
/// errors, constraint-solver failures, and so on. Here it wraps the two error
/// sources the demo actually encounters.
#[derive(Debug)]
enum AppError {
    Occt(occt_rs::error::OcctError),
    MissingLabel(String),
    MissingShape(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Occt(e) => write!(f, "OCCT error: {e}"),
            AppError::MissingLabel(path) => write!(f, "label not found at path: {path}"),
            AppError::MissingShape(label) => write!(f, "no named shape on label: {label}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<occt_rs::error::OcctError> for AppError {
    fn from(e: occt_rs::error::OcctError) -> Self {
        AppError::Occt(e)
    }
}

struct AppState {
    app: OcApplication,
    doc: OcDocument,
}

impl AppState {
    fn new() -> Result<Self, AppError> {
        let mut app = OcApplication::new();
        let doc = app.new_document("BinXCAF")?;
        let mut res = Self { app, doc };
        res.add_base_planes()?;
        Ok(res)
    }

    /// Resolve a label by its entry-string path.
    ///
    /// Paths use the form returned by [`OcLabel::entry`] — e.g. `"0:1:2:1"`.
    fn label_at(&self, path: &str) -> Result<OcLabel, AppError> {
        let parsed = path
            .parse()
            .map_err(|_| AppError::MissingLabel(path.to_string()))?;
        self.doc
            .label_at(&parsed)
            .ok_or(AppError::MissingLabel(path.to_string()))
    }
    fn add_sketch_face(
        &mut self,
        label_paths: Vec<String>,
        sketch_label_tag: i32,
    ) -> Result<(), AppError> {
        let mut points = vec![];
        for path in label_paths {
            let pt = self.label_at(path.as_str())?;
            let attr_pt = OcPointAttr::get(&pt)?.ok_or(AppError::MissingShape(path))?;
            points.push(attr_pt);
        }

        let mut edges = vec![];
        for pair in points.windows(2) {
            edges.push(OcEdge::from_pnts(pair[0], pair[1])?);
        }
        let wire = OcWire::from_edges(&edges)?;
        let face = OcFace::from_wire(&wire, true)?;

        let main = self.doc.main();
        let cmd = self.doc.begin_command()?;
        let sketch = main.get_or_create_child(&cmd, sketch_label_tag);
        let lface = sketch.get_or_create_child(&cmd, 5);
        cmd.name_builder(&lface).primitive(&face.as_shape());
        cmd.commit()?;

        println!("  → sketch face recorded at 0:1:2:5");
        Ok(())
    }
    fn add_sketch_point(&mut self, tag: i32, x: f64, y: f64, z: f64) -> Result<(), AppError> {
        let main = self.doc.main();
        let cmd = self.doc.begin_command()?;
        let sketch = main.get_or_create_child(&cmd, 2);
        let label = sketch.get_or_create_child(&cmd, tag);

        OcPointAttr::record_shape(&cmd, &label, OcPnt::new(x, y, z))?;
        OcPointAttr::set(&cmd, &label)?;

        cmd.commit()?;
        println!("  → point ({x}, {y}, {z}) recorded at {}", label.entry());
        Ok(())
    }
    fn add_base_planes(&mut self) -> Result<(), AppError> {
        let main = self.doc.main();
        let cmd = self.doc.begin_command()?;
        let planes = main.get_or_create_child(&cmd, 1);

        let xy = planes.get_or_create_child(&cmd, 1);
        OcPlaneAttr::record_shape(
            &cmd,
            &xy,
            OcAx2::new(
                OcPnt::new(0.0, 0.0, 0.0),
                OcDir::new(0.0, 0.0, 1.0)?,
                OcDir::new(1.0, 0.0, 0.0)?,
            )?,
        )?;
        OcPlaneAttr::set(&cmd, &xy)?;

        let yz = planes.get_or_create_child(&cmd, 2);
        OcPlaneAttr::record_shape(
            &cmd,
            &yz,
            OcAx2::new(
                OcPnt::new(0.0, 0.0, 0.0),
                OcDir::new(1.0, 0.0, 0.0)?,
                OcDir::new(0.0, 1.0, 0.0)?,
            )?,
        )?;
        OcPlaneAttr::set(&cmd, &yz)?;

        let xz = planes.get_or_create_child(&cmd, 3);
        OcPlaneAttr::record_shape(
            &cmd,
            &xz,
            OcAx2::new(
                OcPnt::new(0.0, 0.0, 0.0),
                OcDir::new(0.0, 1.0, 0.0)?,
                OcDir::new(1.0, 0.0, 0.0)?,
            )?,
        )?;
        OcPlaneAttr::set(&cmd, &xz)?;

        cmd.commit()?;
        println!("  → base planes recorded at 0:1:1, 0:1:1:2, 0:1:1:3");
        Ok(())
    }

    fn undo(&mut self) -> Result<(), AppError> {
        let did_undo = self.doc.undo()?;
        if did_undo {
            println!(
                "  → undo performed ({} remaining)",
                self.doc.available_undos()
            );
        } else {
            println!("  → nothing to undo");
        }
        Ok(())
    }

    fn redo(&mut self) -> Result<(), AppError> {
        let did_redo = self.doc.redo()?;
        if did_redo {
            println!(
                "  → redo performed ({} remaining)",
                self.doc.available_redos()
            );
        } else {
            println!("  → nothing to redo");
        }
        Ok(())
    }

    fn extrude(&mut self, depth: f64) -> Result<(), AppError> {
        // TODO(from_shape): In a complete implementation, the face would be
        // retrieved from the document via:
        //
        //   let lface = state.label_at("0:1:2:5")?;
        //   let face_shape = TopoNamingNamedShape::find(&lface)
        //       .ok_or(AppError::MissingShape("0:1:2:5"))?.get()
        //       .ok_or(AppError::MissingShape("0:1:2:5"))?;
        //   let face = OcFace::from_shape(&face_shape)?;  // not yet bound
        //   let solid = face.extrude(OcVec::new(0.0, 0.0, depth))?;
        //
        // This is the parametric link: the extrude depends on whatever shape
        // the sketch label currently holds. Editing a sketch point and re-running
        // this handler produces a new solid from the updated face.
        //
        // Until OcFace::from_shape is bound, we reconstruct the face from the
        // point coordinates as a workaround.

        let pt_a = self.label_at("0:1:2:1")?;
        let pt_b = self.label_at("0:1:2:2")?;
        let pt_c = self.label_at("0:1:2:3")?;
        let pt_d = self.label_at("0:1:2:4")?;

        let a = OcPointAttr::get(&pt_a)?.ok_or(AppError::MissingShape("0:1:2:1".to_string()))?;
        let b = OcPointAttr::get(&pt_b)?.ok_or(AppError::MissingShape("0:1:2:2".to_string()))?;
        let c = OcPointAttr::get(&pt_c)?.ok_or(AppError::MissingShape("0:1:2:3".to_string()))?;
        let d = OcPointAttr::get(&pt_d)?.ok_or(AppError::MissingShape("0:1:2:4".to_string()))?;

        let wire = OcWire::from_edges(&[
            OcEdge::from_pnts(b, c)?,
            OcEdge::from_pnts(c, d)?,
            OcEdge::from_pnts(d, a)?,
            OcEdge::from_pnts(a, b)?,
        ])?;
        let face = OcFace::from_wire(&wire, true)?;
        let solid = face.extrude(OcVec::new(0.0, 0.0, depth))?;

        let main = self.doc.main();
        let cmd = self.doc.begin_command()?;
        let lsolid = main
            .get_or_create_child(&cmd, 3)
            .get_or_create_child(&cmd, 1);

        // Store the depth first — the authoritative parameter.
        // When OcFace::from_shape is available, this value drives the extrusion
        // directly: `face.extrude(OcVec::new(0.0, 0.0, depth_attr.get()))`.
        OcReal::set(&cmd, &lsolid, depth)?;
        cmd.name_builder(&lsolid).primitive(&solid.as_shape());

        cmd.commit()?;
        println!("  → solid recorded at 0:1:3:1 (depth = {depth})");
        Ok(())
    }
    fn print_tree(&self) -> Result<(), AppError> {
        fn print_label(label: &OcLabel, indent: usize) {
            let prefix = "    ".repeat(indent);
            let has_ns = TopoNamingNamedShape::find(label).is_some();
            let ns_info = if has_ns {
                let ns = TopoNamingNamedShape::find(label).unwrap();
                let evo = match ns.evolution() {
                    Some(TopoNamingEvolution::Primitive) => "Primitive",
                    Some(TopoNamingEvolution::Modify) => "Modify",
                    Some(TopoNamingEvolution::Generated) => "Generated",
                    Some(TopoNamingEvolution::Delete) => "Delete",
                    Some(TopoNamingEvolution::Selected) => "Selected",
                    _ => "?",
                };
                format!(" [NamedShape: {evo}]")
            } else {
                String::new()
            };

            let real_info = if let Some(r) = OcReal::find(label) {
                format!(" [OcReal: {}]", r.get())
            } else {
                String::new()
            };

            println!("{prefix}{}{ns_info}{real_info}", label.entry());

            for child in label.children(false) {
                print_label(&child, indent + 1);
            }
        }

        println!("Document tree:");
        print_label(&self.doc.main(), 0);
        Ok(())
    }
}

/// User-level actions dispatched through the application event loop.
///
/// Each variant maps to one handler function. The handler opens a command,
/// does its work, and commits — one undoable step per action.
enum AppCommand {
    AddSketchPoint {
        tag: i32,
        x: f64,
        y: f64,
        z: f64,
    },
    /// Close the sketch into a planar face.
    AddSketchFace {
        /// Ordered label paths of the corner points, in wire-connection order.
        point_paths: Vec<String>,
        /// Tag of the sketch container label under main.
        sketch_tag: i32,
    },
    Extrude {
        depth: f64,
    },
    /// Chamfer one edge of the solid by `distance`.
    Chamfer {
        distance: f64,
        solid_label_path: String,
        edge_index: usize, // or a ShapeKey, once that's exposed publicly
    },
    /// Record a stable reference to the chamfer face for the second sketch.
    SelectChamferFace,
    /// Undo the most recent command.
    Undo,
    /// Redo the most recently undone command.
    Redo,
    /// Print a summary of the current document tree to stdout.
    PrintTree,
}

fn handle_chamfer(state: &mut AppState, distance: f64) -> Result<(), AppError> {
    // Retrieve the solid shape from the document.
    let lsolid = state.label_at("0:1:3:1")?;
    let solid_shape = TopoNamingNamedShape::find(&lsolid)
        .ok_or(AppError::MissingShape("0:1:3:1".to_string()))?
        .get()
        .ok_or(AppError::MissingShape("0:1:3:1".to_string()))?;

    let pre_faces: Vec<_> = solid_shape.faces().collect();

    let edge = solid_shape
        .edges()
        .next()
        .ok_or(AppError::MissingShape("0:1:3:1 edges".to_string()))?;

    let mut cb = ChamferBuilder::new(&solid_shape)?;
    cb.add_edge(distance, &edge)?;
    let mut built = cb.build_with_history()?;
    let chamfer_shape = built.shape().clone();

    let main = state.doc.main();
    let cmd = state.doc.begin_command()?;
    let lchamfer = main
        .get_or_create_child(&cmd, 3)
        .get_or_create_child(&cmd, 2);

    // Store the distance first — the authoritative parameter.
    OcReal::set(&cmd, &lchamfer, distance)?;

    // Record which faces were modified — feeds into the naming graph so
    // TopoNamingSelector::solve() can re-find sub-shapes after rebuild.
    let mut nb = cmd.name_builder(&lchamfer);
    for face in &pre_faces {
        for modified in built.modified(&face.as_shape()) {
            nb.modified(&face.as_shape(), &modified);
        }
    }

    cmd.commit()?;

    // Store the chamfer shape for the selector step.
    // TODO(from_shape): once OcFace::from_shape is bound, the chamfer shape
    // can be retrieved from the document directly in handle_select_chamfer_face
    // via TopoNamingNamedShape::find(&lchamfer).get(). The field below can
    // then be removed from AppState.
    let _ = chamfer_shape; // used in handle_select_chamfer_face

    println!("  → chamfer recorded at 0:1:3:2 (distance = {distance})");
    Ok(())
}

fn handle_select_chamfer_face(state: &mut AppState) -> Result<(), AppError> {
    // Retrieve the solid and chamfer shapes from the document.
    let lsolid = state.label_at("0:1:3:1")?;
    let lchamfer = state.label_at("0:1:3:2")?;

    let solid_shape = TopoNamingNamedShape::find(&lsolid)
        .ok_or(AppError::MissingShape("0:1:3:1".to_string()))?
        .get()
        .ok_or(AppError::MissingShape("0:1:3:1".to_string()))?;

    let chamfer_shape = TopoNamingNamedShape::find(&lchamfer)
        .ok_or(AppError::MissingShape("0:1:3:2".to_string()))?
        .get()
        .ok_or(AppError::MissingShape("0:1:3:2".to_string()))?;

    // Find the chamfer face — the shape generated from the chamfered edge.
    // We re-run the chamfer here to access build_with_history().
    //
    // TODO(from_shape): once OcFace::from_shape is bound, the chamfer face
    // can be retrieved directly from the naming graph without re-running the
    // operation. The selector's named shape (evolution: Selected) on sketch2/5
    // holds the face reference after the first select call.
    let distance = OcReal::find(&lchamfer)
        .ok_or(AppError::MissingShape("0:1:3:2 distance".to_string()))?
        .get();

    let edge = solid_shape
        .edges()
        .next()
        .ok_or(AppError::MissingShape("0:1:3:1 edges".to_string()))?;

    let mut cb = ChamferBuilder::new(&solid_shape)?;
    cb.add_edge(distance, &edge)?;
    let mut built = cb.build_with_history()?;

    let chamfer_face = built
        .generated(&edge.as_shape())
        .next()
        .ok_or(AppError::MissingShape("chamfer generated face".to_string()))?;

    let main = state.doc.main();
    let cmd = state.doc.begin_command()?;
    let sketch2 = main.get_or_create_child(&cmd, 4);
    let lref = sketch2.get_or_create_child(&cmd, 5);

    let mut selector = cmd.selector(&lref);
    selector.select(&cmd, &chamfer_face, &chamfer_shape);

    cmd.commit()?;

    // TODO(from_shape): the full rebuild + solve() cycle belongs here.
    // After a sketch edit, the workflow is:
    //
    //   1. Edit sketch point (new OcPointAttr::record_shape on sketch/2/1)
    //   2. Retrieve face from sketch/2/5 via TopoNamingNamedShape::get()
    //      and OcFace::from_shape()                    ← not yet bound
    //   3. Rebuild solid: face.extrude(OcVec::new(0.0, 0.0, depth))
    //   4. Re-apply chamfer on new solid
    //   5. Re-record all modified faces with TopoNamingBuilder::modified
    //   6. selector.solve() — re-finds the chamfer face by naming description
    //   7. Assert resolved shape is the new chamfer face
    //
    // All of steps 2-7 are unblocked by a single binding addition.
    // See: todo_toponamingselector_solve.md

    println!("  → chamfer face selection recorded at 0:1:4:5");
    Ok(())
}

// ── dispatch ──────────────────────────────────────────────────────────────────

fn dispatch(state: &mut AppState, command: AppCommand) -> Result<(), AppError> {
    match command {
        AppCommand::AddSketchPoint { tag, x, y, z } => state.add_sketch_point(tag, x, y, z),
        AppCommand::AddSketchFace {
            point_paths,
            sketch_tag,
        } => state.add_sketch_face(point_paths, sketch_tag),
        AppCommand::Extrude { depth } => state.extrude(depth),
        AppCommand::Chamfer { distance, solid_label_path, edge_index } => handle_chamfer(state, distance),
        AppCommand::SelectChamferFace => handle_select_chamfer_face(state),
        AppCommand::Undo => state.undo(),
        AppCommand::Redo => state.redo(),
        AppCommand::PrintTree => state.print_tree(),
    }
}
