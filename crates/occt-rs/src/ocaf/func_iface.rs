//! Reference: https://dev.opencascade.org/doc/refman/html/class_t_function___i_function.html

use occt_sys::ffi;

use crate::ocaf::OcLabel;

/// Execution status of a function, mirroring `TFunction_ExecutionStatus`.
/// Variant order confirmed from TFunction_ExecutionStatus.hxx.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum OcExecutionStatus {
    WrongDefinition = 0,
    NotExecuted = 1,
    Executing = 2,
    Succeeded = 3,
    Failed = 4,
}

impl OcExecutionStatus {
    fn from_raw(v: i32) -> Self {
        match v {
            0 => Self::WrongDefinition,
            1 => Self::NotExecuted,
            2 => Self::Executing,
            3 => Self::Succeeded,
            4 => Self::Failed,
            other => panic!("unknown TFunction_ExecutionStatus value {other}"),
        }
    }
}

/// Reads a `TdfLabelList` fully into a `Vec<OcLabel>`.
///
/// Uses `from_ffi_unchecked`: every label placed in one of these lists comes
/// from OCCT's own graph traversal / driver Arguments()/Results() output —
/// always a real, non-null label, same precondition `get_or_create_child`
/// already relies on elsewhere in this crate.
fn drain_label_list(shim: &cxx::UniquePtr<ffi::TdfLabelList>) -> Vec<OcLabel> {
    let len = ffi::tdf_labellist_len(shim);
    (0..len)
        .map(|i| unsafe { OcLabel::from_ffi_unchecked(ffi::tdf_labellist_get(shim, i)) })
        .collect()
}

pub struct OcIFunction;

impl OcIFunction {
    /// Creates a new function at `label` driven by `guid`. Registers the
    /// label in the document's function scope and attaches a fresh
    /// `TFunction_GraphNode` with status `WrongDefinition`.
    ///
    /// Returns `true` if a driver is already registered for `guid`
    /// (`TFunction_DriverTable::HasDriver`) — `false` does not mean this
    /// call failed, it means no driver is registered *yet*.
    ///
    /// GUID decomposition verified against `register_raw`'s existing
    /// implementation in `sys_topo.rs` (`let (d0, d1, d2, d3) = uuid.as_fields();
    /// let a16b3 = (d3[0] as u16) << 8 | d3[1] as u16; ...`), not guessed.
    pub fn new_function(label: &OcLabel, guid: uuid::Uuid) -> bool {
        let (a32b, a16b1, a16b2, a8bytes) = guid.as_fields();
        let a16b3 = (a8bytes[0] as u16) << 8 | a8bytes[1] as u16;
        ffi::tfunction_ifunction_new_function(
            &label.inner,
            a32b,
            a16b1,
            a16b2,
            a16b3,
            a8bytes[2],
            a8bytes[3],
            a8bytes[4],
            a8bytes[5],
            a8bytes[6],
            a8bytes[7],
        )
    }

    pub fn delete_function(label: &OcLabel) -> bool {
        ffi::tfunction_ifunction_delete_function(&label.inner)
    }

    /// Rebuilds dependencies for every function in the scope reachable from
    /// `access`. Call once after wiring all functions with `OcReference`.
    pub fn update_dependencies_all(access: &OcLabel) -> bool {
        ffi::tfunction_ifunction_update_dependencies_all(&access.inner)
    }

    /// Rebuilds dependencies for this function only (incremental).
    pub fn update_dependencies(label: &OcLabel) -> bool {
        ffi::tfunction_ifunction_update_dependencies_one(&label.inner)
    }

    pub fn arguments(label: &OcLabel) -> Vec<OcLabel> {
        let mut shim = ffi::new_tdf_label_list();
        ffi::tfunction_ifunction_arguments(&label.inner, shim.pin_mut());
        drain_label_list(&shim)
    }

    pub fn results(label: &OcLabel) -> Vec<OcLabel> {
        let mut shim = ffi::new_tdf_label_list();
        ffi::tfunction_ifunction_results(&label.inner, shim.pin_mut());
        drain_label_list(&shim)
    }

    pub fn get_status(label: &OcLabel) -> OcExecutionStatus {
        OcExecutionStatus::from_raw(ffi::tfunction_ifunction_get_status(&label.inner))
    }

    pub fn set_status(label: &OcLabel, status: OcExecutionStatus) {
        ffi::tfunction_ifunction_set_status(&label.inner, status as i32);
    }

    /// Marks `input` as touched in the scope's logbook. Uses the owned
    /// `TFunction_Logbook::Set` constructor path, not the dispatch-context
    /// raw pointer — safe to call from setup code, before any rebuild.
    pub fn set_touched(scope_access: &OcLabel, input: &OcLabel) {
        let mut logbook = ffi::tfunction_logbook_set(&scope_access.inner);
        ffi::tfunction_logbook_set_touched(logbook.pin_mut(), &input.inner);
    }

    /// Labels of functions that produce one of this function's arguments.
    pub fn get_previous(label: &OcLabel) -> Vec<OcLabel> {
        let mut shim = ffi::new_tdf_label_list();
        ffi::tfunction_ifunction_get_previous(&label.inner, shim.pin_mut());
        drain_label_list(&shim)
    }

    /// Labels of functions that consume one of this function's results.
    pub fn get_next(label: &OcLabel) -> Vec<OcLabel> {
        let mut shim = ffi::new_tdf_label_list();
        ffi::tfunction_ifunction_get_next(&label.inner, shim.pin_mut());
        drain_label_list(&shim)
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;
    use crate::function_driver::{register_driver, FunctionDriver, OcFunctionLogbook};
    use crate::ocaf::OcApplication;

    /// A driver whose arguments()/results() are fixed at construction —
    /// captures label clones, same pattern as LabelCreatingDriver in
    /// function_driver.rs's own tests. `execute` is a no-op: these tests
    /// exercise TFunction_IFunction's graph-structure surface (Arguments/
    /// Results delegation, status, dependency wiring), not real dispatch —
    /// invoking a label's driver via Execute is still the open gap noted in
    /// OcFunctionIterator::run's doc comment.
    struct FixedIoDriver {
        args: Vec<OcLabel>,
        results: Vec<OcLabel>,
    }

    impl FunctionDriver for FixedIoDriver {
        fn execute(&self, log: &mut OcFunctionLogbook<'_>) -> i32 {
            log.done(true);
            0
        }
        fn arguments(&self) -> Vec<OcLabel> {
            self.args.clone()
        }
        fn results(&self) -> Vec<OcLabel> {
            self.results.clone()
        }
    }

    /// OcLabel has no PartialEq — compare by entry string throughout, same
    /// as reference.rs's own round-trip test does.
    fn entries(labels: &[OcLabel]) -> Vec<String> {
        labels.iter().map(|l| l.entry()).collect()
    }

    fn guid(n: u8) -> Uuid {
        Uuid::from_bytes([n; 16])
    }

    #[test]
    fn new_function_sets_wrong_definition_status() {
        let mut app = OcApplication::new();
        let mut doc = app.new_document("BinXCAF").unwrap();
        let main = doc.main();

        doc.begin_command().unwrap();
        let func = main.get_or_create_child(1);
        OcIFunction::new_function(&func, guid(1));
        doc.commit().unwrap();

        // NewFunction always sets WrongDefinition initially, regardless of
        // whether a driver is registered for the GUID — confirmed from
        // TFunction_IFunction.cxx (graphNode->SetStatus(WrongDefinition)
        // happens unconditionally, HasDriver() is only the return value).
        assert_eq!(
            OcIFunction::get_status(&func),
            OcExecutionStatus::WrongDefinition
        );
    }

    #[test]
    fn set_status_round_trips() {
        let mut app = OcApplication::new();
        let mut doc = app.new_document("BinXCAF").unwrap();
        let main = doc.main();

        doc.begin_command().unwrap();
        let func = main.get_or_create_child(1);
        OcIFunction::new_function(&func, guid(2));
        OcIFunction::set_status(&func, OcExecutionStatus::NotExecuted);
        doc.commit().unwrap();

        assert_eq!(
            OcIFunction::get_status(&func),
            OcExecutionStatus::NotExecuted
        );
    }

    #[test]
    fn arguments_and_results_delegate_to_registered_driver() {
        let mut app = OcApplication::new();
        let mut doc = app.new_document("BinXCAF").unwrap();
        let main = doc.main();

        doc.begin_command().unwrap();
        let arg_label = main.get_or_create_child(1);
        let result_label = main.get_or_create_child(2);
        let func = main.get_or_create_child(3);
        doc.commit().unwrap();

        let g = guid(3);
        register_driver(
            g,
            FixedIoDriver {
                args: vec![arg_label.clone()],
                results: vec![result_label.clone()],
            },
        );

        doc.begin_command().unwrap();
        OcIFunction::new_function(&func, g);
        doc.commit().unwrap();

        // The actual thing under test: TFunction_IFunction::Arguments/Results
        // delegate to the driver's own Arguments()/Results() — NOT a read of
        // label/1/n, label/2/n sublabels (confirmed from
        // TFunction_IFunction.cxx). No TDF_Reference wiring happens in this
        // test at all, deliberately, to isolate that delegation path.
        assert_eq!(
            entries(&OcIFunction::arguments(&func)),
            entries(&[arg_label])
        );
        assert_eq!(
            entries(&OcIFunction::results(&func)),
            entries(&[result_label])
        );
    }

    #[test]
    fn update_dependencies_links_producer_and_consumer() {
        let mut app = OcApplication::new();
        let mut doc = app.new_document("BinXCAF").unwrap();
        let main = doc.main();

        doc.begin_command().unwrap();
        let shared = main.get_or_create_child(1); // producer's result, consumer's argument
        let producer = main.get_or_create_child(2);
        let consumer = main.get_or_create_child(3);
        doc.commit().unwrap();

        let producer_guid = guid(4);
        let consumer_guid = guid(5);
        register_driver(
            producer_guid,
            FixedIoDriver {
                args: vec![],
                results: vec![shared.clone()],
            },
        );
        register_driver(
            consumer_guid,
            FixedIoDriver {
                args: vec![shared.clone()],
                results: vec![],
            },
        );

        doc.begin_command().unwrap();
        OcIFunction::new_function(&producer, producer_guid);
        OcIFunction::new_function(&consumer, consumer_guid);
        doc.commit().unwrap();

        doc.begin_command().unwrap();
        // ASSUMPTION, not confirmed this session: TFunction_Scope::Set(Access)
        // scopes to the whole document regardless of which label Access is —
        // inferred from TFunction_Logbook::Set explicitly using Access.Root(),
        // and from mainwindow.cpp always passing mainLabel here. Using `main`
        // consistently rather than `producer`/`consumer` on that assumption;
        // if TFunction_Scope.hxx says otherwise this test's premise is wrong,
        // not just its wiring.
        OcIFunction::update_dependencies_all(&main);
        doc.commit().unwrap();

        assert!(OcIFunction::get_previous(&producer).is_empty());
        assert_eq!(
            entries(&OcIFunction::get_next(&producer)),
            entries(&[consumer.clone()])
        );
        assert_eq!(
            entries(&OcIFunction::get_previous(&consumer)),
            entries(&[producer.clone()])
        );
        assert!(OcIFunction::get_next(&consumer).is_empty());
    }
}
