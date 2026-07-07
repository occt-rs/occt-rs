//! Reference: https://dev.opencascade.org/doc/refman/html/class_t_function___iterator.html

use occt_sys::ffi;

use crate::ocaf::func_iface::{OcExecutionStatus, OcIFunction};
use crate::ocaf::OcLabel;

pub struct OcFunctionIterator {
    inner: cxx::UniquePtr<ffi::TFunctionIteratorShim>,
}

impl OcFunctionIterator {
    /// Initializes from the scope reachable from `access` (typically the
    /// document's main/root label). Finds all root functions (no previous
    /// dependencies) as the first `current()` set.
    pub fn new(access: &OcLabel) -> Self {
        OcFunctionIterator {
            inner: ffi::new_tfunction_iterator(&access.inner),
        }
    }

    /// If `true`, traversal respects `TFunction_ExecutionStatus` — only
    /// `NotExecuted` functions are considered, and advancement checks that
    /// all previous functions in the graph succeeded before admitting a
    /// function to the next `current()` set. If `false` (the default after
    /// `new`), the iterator just walks the raw graph once, ignoring status.
    ///
    /// Set this to `true` before a real rebuild; leave `false` for the kind
    /// of one-shot structural walk `TFunction_Iterator::Dump` does.
    pub fn set_usage_of_execution_status(&mut self, usage: bool) {
        ffi::tfunction_iterator_set_usage_of_execution_status(self.inner.pin_mut(), usage);
    }

    pub fn more(&self) -> bool {
        ffi::tfunction_iterator_more(&self.inner)
    }

    pub fn next(&mut self) {
        ffi::tfunction_iterator_next(self.inner.pin_mut());
    }

    /// The current "wave" of functions — all mutually independent, safe to
    /// execute in any order (or in parallel; see `GetMaxNbThreads` on the
    /// OCCT side, not yet bound).
    pub fn current(&self) -> Vec<OcLabel> {
        let mut shim = ffi::new_tdf_label_list();
        ffi::tfunction_iterator_current(&self.inner, shim.pin_mut());
        let len = ffi::tdf_labellist_len(&shim);
        (0..len)
            .map(|i| unsafe { OcLabel::from_ffi_unchecked(ffi::tdf_labellist_get(&shim, i)) })
            .collect()
    }

    /// Drives a full rebuild: for each wave in `current()`, calls `execute`
    /// on every label in the wave, then advances.
    ///
    /// GAP NOT CLOSED HERE: `execute` is the caller's responsibility to
    /// actually invoke the registered FunctionDriver for that label. The
    /// dispatch *callee* side (register_driver / rust_driver_execute) is
    /// already in function_driver.rs, but the *caller* side — something
    /// like `TFunction_IFunction(label).GetDriver()->Execute(logbook)`,
    /// exposed as a shim — is not bound anywhere in this work order's three
    /// files. Without it, `execute` has no way to actually run the driver
    /// from a label; this method is scaffolding for that call, not a
    /// complete rebuild loop. Needs its own follow-up.
    pub fn run(&mut self, mut execute: impl FnMut(&OcLabel)) {
        self.set_usage_of_execution_status(true);
        while self.more() {
            for label in self.current() {
                execute(&label);
                OcIFunction::set_status(&label, OcExecutionStatus::Succeeded);
            }
            self.next();
        }
    }
}
// ============================================================================
// Addition to crates/occt-rs/src/ocaf/func_iter.rs — new #[cfg(test)] module,
// append at the end of the file.
//
// All three tests exercise OcFunctionIterator::run's LOOPING LOGIC (wave
// ordering, no double-visits, termination) using no-op drivers. That's
// deliberately distinct from the still-open gap in run's doc comment
// (invoking a label's real driver Execute) — the loop itself doesn't touch
// that at all, only the `execute` closure argument would need it, and these
// tests pass a closure that just records visit order instead.
// ============================================================================

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use uuid::Uuid;

    use super::*;
    use crate::function_driver::{register_driver, FunctionDriver, OcFunctionLogbook};
    use crate::ocaf::OcApplication;

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

    fn guid(n: u8) -> Uuid {
        // Distinct high bytes from func_iface.rs's test GUIDs so both test
        // modules can run in the same process without colliding on the
        // process-global TFunction_DriverTable.
        Uuid::from_bytes([0xA0 + n; 16])
    }

    /// Sets up a function label: NewFunction + explicit NotExecuted status.
    /// The NotExecuted step is required — NewFunction always leaves a
    /// function at WrongDefinition (confirmed in func_iface.rs's own test),
    /// and TFunction_Iterator with usage_of_execution_status(true) only
    /// admits NotExecuted functions as iteration roots. Mirrors the setup
    /// sequence confirmed from mainwindow.cpp
    /// (NewFunction, then iFuncX.SetStatus(TFunction_ES_NotExecuted)).
    fn wire_function(label: &OcLabel, guid: Uuid) {
        OcIFunction::new_function(label, guid);
        OcIFunction::set_status(label, OcExecutionStatus::NotExecuted);
    }

    #[test]
    fn single_function_no_dependencies_runs_once() {
        let mut app = OcApplication::new();
        let mut doc = app.new_document("BinXCAF").unwrap();
        let main = doc.main();

        doc.begin_command().unwrap();
        let func = main.get_or_create_child(1);
        doc.commit().unwrap();

        let g = guid(1);
        register_driver(
            g,
            FixedIoDriver {
                args: vec![],
                results: vec![],
            },
        );

        doc.begin_command().unwrap();
        wire_function(&func, g);
        OcIFunction::update_dependencies_all(&main);
        doc.commit().unwrap();

        let visited = RefCell::new(Vec::<String>::new());
        doc.begin_command().unwrap();
        let mut iter = OcFunctionIterator::new(&main);
        iter.run(|label| visited.borrow_mut().push(label.entry()));
        doc.commit().unwrap();

        assert_eq!(visited.into_inner(), vec![func.entry()]);
    }

    #[test]
    fn producer_consumer_chain_runs_in_dependency_order() {
        let mut app = OcApplication::new();
        let mut doc = app.new_document("BinXCAF").unwrap();
        let main = doc.main();

        doc.begin_command().unwrap();
        let shared = main.get_or_create_child(1);
        let producer = main.get_or_create_child(2);
        let consumer = main.get_or_create_child(3);
        doc.commit().unwrap();

        let producer_guid = guid(2);
        let consumer_guid = guid(3);
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
        wire_function(&producer, producer_guid);
        wire_function(&consumer, consumer_guid);
        OcIFunction::update_dependencies_all(&main);
        doc.commit().unwrap();

        let visited = RefCell::new(Vec::<String>::new());
        doc.begin_command().unwrap();
        let mut iter = OcFunctionIterator::new(&main);
        iter.run(|label| visited.borrow_mut().push(label.entry()));
        doc.commit().unwrap();

        // Single-element waves throughout this graph shape, so call order
        // IS wave order: producer's wave must fully complete (there is only
        // one function in it) before consumer's wave starts.
        assert_eq!(
            visited.into_inner(),
            vec![producer.entry(), consumer.entry()]
        );
    }

    #[test]
    fn diamond_dependency_visits_sink_once_after_both_branches() {
        // A -> B, A -> C, B -> D, C -> D. D must be visited exactly once,
        // strictly after both B and C — this is the case
        // TFunction_Iterator::Next's "all previous must be Succeeded before
        // admitting a next function" check (confirmed from
        // TFunction_Iterator.cxx) exists specifically to get right; a naive
        // "admit when any previous succeeds" traversal would visit D twice
        // or admit it after only one of B/C.
        let mut app = OcApplication::new();
        let mut doc = app.new_document("BinXCAF").unwrap();
        let main = doc.main();

        doc.begin_command().unwrap();
        let x = main.get_or_create_child(1); // A's result, B's and C's argument
        let y = main.get_or_create_child(2); // B's result, D's argument
        let z = main.get_or_create_child(3); // C's result, D's argument
        let a = main.get_or_create_child(4);
        let b = main.get_or_create_child(5);
        let c = main.get_or_create_child(6);
        let d = main.get_or_create_child(7);
        doc.commit().unwrap();

        let ga = guid(4);
        let gb = guid(5);
        let gc = guid(6);
        let gd = guid(7);
        register_driver(
            ga,
            FixedIoDriver {
                args: vec![],
                results: vec![x.clone()],
            },
        );
        register_driver(
            gb,
            FixedIoDriver {
                args: vec![x.clone()],
                results: vec![y.clone()],
            },
        );
        register_driver(
            gc,
            FixedIoDriver {
                args: vec![x.clone()],
                results: vec![z.clone()],
            },
        );
        register_driver(
            gd,
            FixedIoDriver {
                args: vec![y.clone(), z.clone()],
                results: vec![],
            },
        );

        doc.begin_command().unwrap();
        wire_function(&a, ga);
        wire_function(&b, gb);
        wire_function(&c, gc);
        wire_function(&d, gd);
        OcIFunction::update_dependencies_all(&main);
        doc.commit().unwrap();

        let visited = RefCell::new(Vec::<String>::new());
        doc.begin_command().unwrap();
        let mut iter = OcFunctionIterator::new(&main);
        iter.run(|label| visited.borrow_mut().push(label.entry()));
        doc.commit().unwrap();

        let order = visited.into_inner();

        // Exactly one visit each, four total — the double-visit failure
        // mode would show up as len() > 4 or d.entry() appearing twice.
        assert_eq!(order.len(), 4);
        let d_pos = order.iter().position(|e| e == &d.entry()).unwrap();
        let b_pos = order.iter().position(|e| e == &b.entry()).unwrap();
        let c_pos = order.iter().position(|e| e == &c.entry()).unwrap();
        assert!(d_pos > b_pos && d_pos > c_pos);
    }
}
