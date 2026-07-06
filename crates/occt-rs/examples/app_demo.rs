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
//! │   └── (0:1:1:[1, 2, 3])   [XY, YZ, XZ ] plane
//! │         TopoNamingNamedShape (Primitive, planar face)
//! │         OcPlaneAttr
//! ├── (0:1:2)   sketch
//! │   ├── (0:1:2:[1, 2, 3, 4])   point [A, B, C, D]
//! │   │     TopoNamingNamedShape (Primitive, vertex)
//! │   │     OcPointAttr
//! │   └── (0:1:2:5)   face
//! │           TopoNamingNamedShape (Primitive, unit square face)
//! ├── (0:1:3)   body
//! │   ├── (0:1:3:1)   solid
//! │   │     TopoNamingNamedShape (Generated, 1×1×1 prism)
//! │   │     OcReal "depth" = 1.0
//! │   └── (0:1:3:2)   chamfer
//! │           TopoNamingNamedShape (Modify, chamfered solid)
//! │           OcReal "distance" = 0.05
//! └── (0:1:4)   sketch2
//!     └── (0:1:4:5)   ref-face
//!           TopoNamingNamedShape (Selected — TopoNamingSelector)
//! ```
//!
//! ## Known gaps
//!
//! `extrude()` now takes its input face's label path as data (not a
//! hardcoded constant) and retrieves the shape from the document via
//! `OcFace::try_from`, so calling it again after a sketch edit *should*
//! produce a new solid from the updated face — but nothing in this file's
//! `main()` command sequence actually exercises that yet; there's no
//! second `AddSketchPoint` + `Extrude` pair to prove it. The chamfer and
//! selector side of the rebuild cycle is further behind:
//! `handle_select_chamfer_face` still re-runs the chamfer operation to
//! derive the chamfer face rather than re-finding it via
//! `TopoNamingSelector::solve()`. Search for `TODO(try_from)` to find the
//! remaining affected sites.

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
            sketch_tag: 2,
            x: 0.0,
            y: 1.0,
            z: 0.0,
        },
        AppCommand::AddSketchPoint {
            tag: 2,
            sketch_tag: 2,
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        AppCommand::AddSketchPoint {
            tag: 3,
            sketch_tag: 2,
            x: 1.0,
            y: 0.0,
            z: 0.0,
        },
        AppCommand::AddSketchPoint {
            tag: 4,
            sketch_tag: 2,
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
        AppCommand::Extrude {
            face_label_path: "0:1:2:5".to_string(),
            depth: 1.0,
        },
        AppCommand::Chamfer {
            distance: 0.05,
            solid_label_path: "0:1:3:1".to_string(),
            edge_index: 0, // user clicked the first edge — in a real app this is a pick result
        },
        AppCommand::SelectChamferFace {
            solid_label_path: "0:1:3:1".to_string(),
            chamfer_label_path: "0:1:3:2".to_string(),
            edge_index: 0, // must match the Chamfer command above
        },
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
    _app: OcApplication,
    doc: OcDocument,
}

impl AppState {
    fn new() -> Result<Self, AppError> {
        let mut app = OcApplication::new();
        let doc = app.new_document("BinXCAF")?;
        let mut res = Self { _app: app, doc };
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
        self.doc.begin_command()?;
        let sketch = main.get_or_create_child(sketch_label_tag);
        let lface = sketch.get_or_create_child(5);
        self.doc.name_builder(&lface).primitive(&face.as_shape());
        self.doc.commit()?;

        println!("  → sketch face recorded at 0:1:2:5");
        Ok(())
    }
    fn add_sketch_point(
        &mut self,
        tag: i32,
        sketch_tag: i32,
        x: f64,
        y: f64,
        z: f64,
    ) -> Result<(), AppError> {
        let main = self.doc.main();
        self.doc.begin_command()?;
        let sketch = main.get_or_create_child(sketch_tag);
        let label = sketch.get_or_create_child(tag);

        OcPointAttr::record_shape(&label, OcPnt::new(x, y, z))?;
        OcPointAttr::set(&label)?;

        self.doc.commit()?;
        println!("  → point ({x}, {y}, {z}) recorded at {}", label.entry());
        Ok(())
    }
    fn add_base_planes(&mut self) -> Result<(), AppError> {
        let main = self.doc.main();
        self.doc.begin_command()?;
        let planes = main.get_or_create_child(1);

        let xy = planes.get_or_create_child(1);
        OcPlaneAttr::record_shape(
            &xy,
            OcAx2::new(
                OcPnt::new(0.0, 0.0, 0.0),
                OcDir::new(0.0, 0.0, 1.0)?,
                OcDir::new(1.0, 0.0, 0.0)?,
            )?,
        )?;
        OcPlaneAttr::set(&xy)?;

        let yz = planes.get_or_create_child(2);
        OcPlaneAttr::record_shape(
            &yz,
            OcAx2::new(
                OcPnt::new(0.0, 0.0, 0.0),
                OcDir::new(1.0, 0.0, 0.0)?,
                OcDir::new(0.0, 1.0, 0.0)?,
            )?,
        )?;
        OcPlaneAttr::set(&yz)?;

        let xz = planes.get_or_create_child(3);
        OcPlaneAttr::record_shape(
            &xz,
            OcAx2::new(
                OcPnt::new(0.0, 0.0, 0.0),
                OcDir::new(0.0, 1.0, 0.0)?,
                OcDir::new(1.0, 0.0, 0.0)?,
            )?,
        )?;
        OcPlaneAttr::set(&xz)?;

        self.doc.commit()?;
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

    fn extrude(&mut self, face_label_path: &str, depth: f64) -> Result<(), AppError> {
        let lface = self.label_at(face_label_path)?;
        let face_shape = TopoNamingNamedShape::find(&lface)
            .ok_or(AppError::MissingShape(face_label_path.to_string()))?
            .get()
            .ok_or(AppError::MissingShape(face_label_path.to_string()))?;
        let face = OcFace::try_from(&face_shape)?;
        let solid_shape = face.extrude(OcVec::new(0.0, 0.0, depth))?;

        let main = self.doc.main();
        self.doc.begin_command()?;
        let lsolid = main.get_or_create_child(3).get_or_create_child(1);

        // Store the depth first — the authoritative parameter.
        OcReal::set(&lsolid, depth)?;
        self.doc
            .name_builder(&lsolid)
            .generated(&face_shape, &solid_shape);

        self.doc.commit()?;
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
        /// Tag of the sketch container label under main.
        sketch_tag: i32,
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
        /// Label path of the face to extrude — e.g. the path the user
        /// selected in the feature tree or clicked in the viewport.
        face_label_path: String,
        depth: f64,
    },
    /// Chamfer one edge of the solid by `distance`.
    Chamfer {
        distance: f64,
        solid_label_path: String,
        /// Index into the solid's edge iterator — stands in for a real pick
        /// result (e.g. a `ShapeKey`) in a full application.
        edge_index: usize,
    },
    /// Record a stable reference to the chamfer face for the second sketch.
    SelectChamferFace {
        solid_label_path: String,
        chamfer_label_path: String,
        /// Must match the `edge_index` used in the `Chamfer` command being
        /// re-derived — see `TODO(try_from)` in `handle_select_chamfer_face`
        /// for why this is still a re-derivation rather than a lookup.
        edge_index: usize,
    },
    /// Undo the most recent command.
    Undo,
    /// Redo the most recently undone command.
    Redo,
    /// Print a summary of the current document tree to stdout.
    PrintTree,
}

fn handle_chamfer(
    state: &mut AppState,
    solid_label_path: &str,
    edge_index: usize,
    distance: f64,
) -> Result<(), AppError> {
    // Retrieve the solid shape from the document.
    let lsolid = state.label_at(solid_label_path)?;
    let solid_shape = TopoNamingNamedShape::find(&lsolid)
        .ok_or(AppError::MissingShape(solid_label_path.to_string()))?
        .get()
        .ok_or(AppError::MissingShape(solid_label_path.to_string()))?;

    let pre_faces: Vec<_> = solid_shape.faces().collect();

    let edge = solid_shape
        .edges()
        .nth(edge_index)
        .ok_or(AppError::MissingShape(format!(
            "{solid_label_path} edge {edge_index}"
        )))?;

    let mut cb = ChamferBuilder::new(&solid_shape)?;
    cb.add_edge(distance, &edge)?;
    let mut built = cb.build_with_history()?;

    let main = state.doc.main();
    state.doc.begin_command()?;
    let lchamfer = main.get_or_create_child(3).get_or_create_child(2);

    // Store the distance first — the authoritative parameter.
    OcReal::set(&lchamfer, distance)?;

    // Record which faces were modified — feeds into the naming graph so
    // TopoNamingSelector::solve() can re-find sub-shapes after rebuild.
    let mut nb = state.doc.name_builder(&lchamfer);
    for face in &pre_faces {
        for modified in built.modified(&face.as_shape()) {
            nb.modified(&face.as_shape(), &modified);
        }
    }

    state.doc.commit()?;

    println!("  → chamfer recorded at 0:1:3:2 (distance = {distance})");
    Ok(())
}

fn handle_select_chamfer_face(
    state: &mut AppState,
    solid_label_path: &str,
    chamfer_label_path: &str,
    edge_index: usize,
) -> Result<(), AppError> {
    // Retrieve the solid and chamfer shapes from the document.
    let lsolid = state.label_at(solid_label_path)?;
    let lchamfer = state.label_at(chamfer_label_path)?;

    let solid_shape = TopoNamingNamedShape::find(&lsolid)
        .ok_or(AppError::MissingShape(solid_label_path.to_string()))?
        .get()
        .ok_or(AppError::MissingShape(solid_label_path.to_string()))?;

    let chamfer_shape = TopoNamingNamedShape::find(&lchamfer)
        .ok_or(AppError::MissingShape(chamfer_label_path.to_string()))?
        .get()
        .ok_or(AppError::MissingShape(chamfer_label_path.to_string()))?;

    // Find the chamfer face — the shape generated from the chamfered edge.
    // We re-run the chamfer here to access build_with_history().
    //
    // TODO(try_from): handle_chamfer never records the chamfer face's own
    // provenance (only the modified pre-existing faces), so there's nothing
    // in the naming graph yet to look this up by. Re-deriving it here by
    // re-running the operation with the same edge_index is a stand-in until
    // the solve() cycle replaces both this and the edge_index duplication
    // with a real lookup. See: todo_toponamingselector_solve.md
    let distance = OcReal::find(&lchamfer)
        .ok_or(AppError::MissingShape(format!(
            "{chamfer_label_path} distance"
        )))?
        .get();

    let edge = solid_shape
        .edges()
        .nth(edge_index)
        .ok_or(AppError::MissingShape(format!(
            "{solid_label_path} edge {edge_index}"
        )))?;

    let mut cb = ChamferBuilder::new(&solid_shape)?;
    cb.add_edge(distance, &edge)?;
    let mut built = cb.build_with_history()?;

    let chamfer_face = built
        .generated(&edge.as_shape())
        .next()
        .ok_or(AppError::MissingShape("chamfer generated face".to_string()))?;

    let main = state.doc.main();
    state.doc.begin_command()?;
    let sketch2 = main.get_or_create_child(4);
    let lref = sketch2.get_or_create_child(5);

    let mut selector = state.doc.selector(&lref);
    selector.select(&chamfer_face, &chamfer_shape);

    state.doc.commit()?;

    // TODO(try_from): the full rebuild + solve() cycle belongs here.
    // After a sketch edit, the workflow is:
    //
    //   1. Edit sketch point (new OcPointAttr::record_shape on sketch/2/1)
    //   2. Retrieve face from sketch/2/5 via TopoNamingNamedShape::get()
    //      and OcFace::try_from()                    ← ~~not yet bound~~ not yet put to use
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
        AppCommand::AddSketchPoint {
            tag,
            sketch_tag,
            x,
            y,
            z,
        } => state.add_sketch_point(tag, sketch_tag, x, y, z),
        AppCommand::AddSketchFace {
            point_paths,
            sketch_tag,
        } => state.add_sketch_face(point_paths, sketch_tag),
        AppCommand::Extrude {
            face_label_path,
            depth,
        } => state.extrude(&face_label_path, depth),
        AppCommand::Chamfer {
            distance,
            solid_label_path,
            edge_index,
        } => handle_chamfer(state, &solid_label_path, edge_index, distance),
        AppCommand::SelectChamferFace {
            solid_label_path,
            chamfer_label_path,
            edge_index,
        } => handle_select_chamfer_face(state, &solid_label_path, &chamfer_label_path, edge_index),
        AppCommand::Undo => state.undo(),
        AppCommand::Redo => state.redo(),
        AppCommand::PrintTree => state.print_tree(),
    }
}
