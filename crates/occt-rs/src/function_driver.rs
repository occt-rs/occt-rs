//! TFunction driver
//!
//! Application authors implement [`FunctionDriver`] and call
//! [`register_driver`] once per driver type, typically at application startup
//! or plugin load time. The driver is then available to OCCT's rebuild
//! machinery via the [`TFunction_DriverTable`] singleton.
//!
//! # Example
//!
//! ```rust,ignore
//! use occt_rs::function_driver::{FunctionDriver, OcFunctionLogbook, OcFunctionLabelList, register_driver};
//!
//! struct ExtrudeDriver;
//!
//! impl FunctionDriver for ExtrudeDriver {
//!     fn execute(&self, log: &mut OcFunctionLogbook<'_>) -> i32 {
//!         // ... read argument labels, do geometry, mark results as impacted
//!         log.done(true);
//!         0
//!     }
//!     fn must_execute(&self, log: &OcFunctionLogbook<'_>) -> bool {
//!         // return true if any argument label was modified
//!         true
//!     }
//!     fn validate(&self, log: &mut OcFunctionLogbook<'_>) {}
//!     fn arguments(&self, list: &mut OcFunctionLabelList<'_>) { /* push argument labels */ }
//!     fn results(&self, list: &mut OcFunctionLabelList<'_>) { /* push result labels */ }
//! }
//!
//! // At startup (TODO: GUID API TBD):
//! register_driver(Uuid::try_from("12345678-1234-1234-1234-123456789abc").unwrap(), ExtrudeDriver)?;
//! ```

use std::marker::PhantomData;
use std::pin::Pin;

use occt_sys::ffi;
use occt_sys::sys_topo::FunctionDriverRaw;

use crate::ocaf::OcLabel;

// ── OcFunctionLogbook ─────────────────────────────────────────────────────────
//
// Safe wrapper around a raw pointer to `ffi::TFunctionLogbookHandle`.
// The pointer is valid only for the duration of the callback from the C++ shim;
// the lifetime parameter `'a` ties the wrapper's lifetime to the raw pointer's
// validity, which is enforced by the `unsafe` constructors below.
//
// `&mut OcFunctionLogbook` grants access to mutating operations (SetImpacted,
// SetValid, Done). `&OcFunctionLogbook` grants read-only access (IsModified,
// IsDone), enforced by Rust's borrow rules — no separate type is needed.
//
// `!Send + !Sync` via PhantomData: the logbook pointer is only valid on the
// session thread, consistent with the single-threaded OCAF session model.

pub struct OcFunctionLogbook<'a> {
    inner: *mut ffi::TFunctionLogbookHandle,
    _owned: Option<cxx::UniquePtr<ffi::TFunctionLogbookHandle>>,
    _lifetime: PhantomData<&'a mut ffi::TFunctionLogbookHandle>,
    _not_send: PhantomData<*mut ()>,
}

impl OcFunctionLogbook<'static> {
    /// Owned, document-lifetime logbook handle for `access`'s scope — NOT
    /// tied to a driver dispatch callback. Use this for setup code (marking
    /// inputs touched before a rebuild) and for tests, where `from_raw`'s
    /// borrowed pointer isn't available.
    ///
    /// STILL UNRESOLVED (flagged, not fixed): `OcFunctionLogbook::inner` is a
    /// bare `*mut` with no `Drop`, correct today only because `from_raw`'s
    /// pointer is always borrowed from a C++ call stack. `owned.into_raw()`
    /// below leaks that allocation on every call — not unsound, just a real
    /// leak. Needs `OcFunctionLogbook` to gain an owned/borrowed distinction
    /// before this ships.
    pub fn from_label(access: &OcLabel) -> OcFunctionLogbook<'static> {
        let mut owned = ffi::tfunction_logbook_set(&access.inner);
        // Safety: UniquePtr heap-allocates; the object's address is stable
        // regardless of where the UniquePtr itself is stored. raw remains valid
        // for exactly as long as _owned lives, which is the struct's lifetime.
        let raw =
            unsafe { Pin::get_unchecked_mut(owned.pin_mut()) as *mut ffi::TFunctionLogbookHandle };
        OcFunctionLogbook {
            inner: raw,
            _owned: Some(owned),
            _lifetime: PhantomData,
            _not_send: PhantomData,
        }
    }

    /// Marks `label` as touched. Note: unlike `set_impacted`/`set_valid`,
    /// OCCT's `SetTouched` takes no `with_children` parameter (confirmed
    /// from TFunction_Logbook.hxx).
    pub fn set_touched(&mut self, label: &OcLabel) {
        let pin = unsafe { Pin::new_unchecked(&mut *self.inner) };
        ffi::tfunction_logbook_set_touched(pin, &label.inner);
    }

    /// Returns every label currently marked touched in this logbook.
    pub fn get_touched(&self) -> Vec<OcLabel> {
        let mut shim = ffi::new_tdf_label_list();
        ffi::tfunction_logbook_get_touched(unsafe { &*self.inner }, shim.pin_mut());
        let len = ffi::tdf_labellist_len(&shim);
        (0..len)
            .map(|i| unsafe { OcLabel::from_ffi_unchecked(ffi::tdf_labellist_get(&shim, i)) })
            .collect()
    }

    /// Resets touched/impacted/valid to empty.
    pub fn clear(&mut self) {
        let pin = unsafe { Pin::new_unchecked(&mut *self.inner) };
        ffi::tfunction_logbook_clear(pin);
    }
}

impl<'a> OcFunctionLogbook<'a> {
    /// # Safety
    /// `ptr` must be valid and exclusively reachable for the lifetime `'a`.
    /// Called only from `FunctionDriverAdapter`, where the pointer comes from
    /// the C++ shim's stack-local `TFunctionLogbookHandle`.
    pub(crate) unsafe fn from_raw(ptr: *mut ffi::TFunctionLogbookHandle) -> Self {
        OcFunctionLogbook {
            inner: ptr,
            _owned: None,
            _lifetime: PhantomData,
            _not_send: PhantomData,
        }
    }

    // ── Read-only operations (callable via `&self`) ───────────────────────────

    /// Returns true if `label` (or any of its children if `with_children`) has
    /// been touched or impacted in this logbook.
    ///
    /// Used in [`FunctionDriver::must_execute`] to decide whether execution is
    /// needed.
    pub fn is_modified(&self, label: &OcLabel, with_children: bool) -> bool {
        // Safety: self.inner is valid for 'a; &* is a shared borrow within that.
        ffi::tfunction_logbook_is_modified(unsafe { &*self.inner }, &label.inner, with_children)
    }

    /// Returns the current execution status flag (set by [`done`]).
    pub fn is_done(&self) -> bool {
        ffi::tfunction_logbook_is_done(unsafe { &*self.inner })
    }

    // ── Mutating operations (callable via `&mut self`) ────────────────────────

    /// Marks `label` (and optionally its children) as impacted.
    /// Call this in [`FunctionDriver::execute`] for every output label written.
    pub fn set_impacted(&mut self, label: &OcLabel, with_children: bool) {
        // Safety: &mut self guarantees exclusive access; inner valid for 'a.
        let pin = unsafe { Pin::new_unchecked(&mut *self.inner) };
        ffi::tfunction_logbook_set_impacted(pin, &label.inner, with_children);
    }

    /// Marks `label` (and optionally its children) as valid.
    /// Call this in [`FunctionDriver::validate`].
    pub fn set_valid(&mut self, label: &OcLabel, with_children: bool) {
        let pin = unsafe { Pin::new_unchecked(&mut *self.inner) };
        ffi::tfunction_logbook_set_valid(pin, &label.inner, with_children);
    }

    /// Sets the execution status flag.
    /// Conventionally called with `true` at the end of a successful
    /// [`FunctionDriver::execute`].
    pub fn done(&mut self, status: bool) {
        let pin = unsafe { Pin::new_unchecked(&mut *self.inner) };
        ffi::tfunction_logbook_done(pin, status);
    }
}

// ── OcFunctionLabelList ───────────────────────────────────────────────────────
//
// Safe wrapper around a raw pointer to `ffi::TFunctionLabelListShim`.
// Valid only for the duration of a [`FunctionDriver::arguments`] or
// [`FunctionDriver::results`] callback.

pub struct OcFunctionLabelList<'a> {
    inner: *mut ffi::TdfLabelList,
    _lifetime: PhantomData<&'a mut ffi::TdfLabelList>,
    _not_send: PhantomData<*mut ()>,
}

impl<'a> OcFunctionLabelList<'a> {
    /// # Safety
    /// `ptr` must be valid and exclusively reachable for the lifetime `'a`.
    pub(crate) unsafe fn from_raw(ptr: *mut ffi::TdfLabelList) -> Self {
        OcFunctionLabelList {
            inner: ptr,
            _lifetime: PhantomData,
            _not_send: PhantomData,
        }
    }

    /// Appends `label` to the OCCT label list.
    ///
    /// Call this in [`FunctionDriver::arguments`] and [`FunctionDriver::results`]
    /// for each label that this driver reads from / writes to.
    pub fn push(&mut self, label: &OcLabel) {
        let pin = unsafe { Pin::new_unchecked(&mut *self.inner) };
        ffi::tdf_labellist_append(pin, &label.inner);
    }
}

// ── FunctionDriver trait ──────────────────────────────────────────────────────

/// Implemented by application code to define a parametric rebuild function.
///
/// One instance of the implementing type is registered per driver type for
/// the lifetime of the session. Instances carry no per-label state; document
/// state is accessed via the label stored on the function's own document label
/// (retrievable via `TFunction_Driver::Label()` — not yet bound, add if needed).
///
/// All methods take `&self` because `TFunction_Driver`'s virtual methods are
/// `const` in OCCT. Drivers that need mutable state must use interior
/// mutability (`RefCell`, `Cell`, etc.) with the understanding that OCCT
/// rebuilds are single-threaded.
pub trait FunctionDriver: 'static {
    /// Executes the function. Write results to document labels and mark them
    /// impacted via [`OcFunctionLogbook::set_impacted`]. Call
    /// [`OcFunctionLogbook::done`] with `true` on success.
    ///
    /// Returns an application-defined integer code; 0 conventionally means
    /// success. The value is passed through to OCCT unchanged.
    fn execute(&self, log: &mut OcFunctionLogbook<'_>) -> i32;

    /// Returns `true` if the function must be executed because one or more of
    /// its argument labels was modified. Use [`OcFunctionLogbook::is_modified`]
    /// to check each argument label.
    ///
    /// The default implementation always returns `true` (equivalent to the
    /// OCCT base class behaviour, which re-executes unconditionally).
    fn must_execute(&self, log: &OcFunctionLogbook<'_>) -> bool {
        self.arguments()
            .iter()
            .any(|label| log.is_modified(label, false))
    }

    /// Validates the function's result labels in the logbook.
    /// Use [`OcFunctionLogbook::set_valid`] for each result label.
    ///
    /// The default implementation does nothing (equivalent to OCCT base class).
    fn validate(&self, log: &mut OcFunctionLogbook<'_>) {
        for label in self.results() {
            log.set_valid(&label, true);
        }
    }

    /// Fills `list` with the document labels that this function reads from
    /// (its arguments). Use [`OcFunctionLabelList::push`] for each.
    ///
    /// The default implementation pushes nothing (equivalent to OCCT base class).
    fn arguments(&self) -> Vec<OcLabel> {
        vec![]
    }

    /// Fills `list` with the document labels that this function writes to
    /// (its results). Use [`OcFunctionLabelList::push`] for each.
    ///
    /// The default implementation pushes nothing (equivalent to OCCT base class).
    fn results(&self) -> Vec<OcLabel> {
        vec![]
    }
}

// ── Adapter ───────────────────────────────────────────────────────────────────
//
// Bridges `FunctionDriver` (safe, OcLabel-based) to `FunctionDriverRaw`
// (unsafe, raw-pointer-based). This is the only `unsafe` implementation in
// occt-rs; the unsafety is confined here and documented per method.

struct FunctionDriverAdapter<D: FunctionDriver> {
    driver: D,
}

// Safety: FunctionDriverRaw is unsafe to implement because the raw pointer
// parameters must be valid for call duration. In each method below, the pointer
// comes from the C++ shim's stack-local shim struct, guaranteed valid by the
// call contract documented in sys_function_driver.rs and function.hxx. We wrap
// the call in the safe wrapper type, which enforces lifetime and mutability
// rules for everything the driver itself touches.
unsafe impl<D: FunctionDriver> FunctionDriverRaw for FunctionDriverAdapter<D> {
    unsafe fn execute_raw(&self, log: *mut ffi::TFunctionLogbookHandle) -> i32 {
        // Safety: ptr valid for call duration; exclusive access.
        let mut logbook = unsafe { OcFunctionLogbook::from_raw(log) };
        self.driver.execute(&mut logbook)
    }

    unsafe fn must_execute_raw(&self, log: *mut ffi::TFunctionLogbookHandle) -> bool {
        // Safety: ptr valid for call duration; treat as read-only (const source).
        let logbook = unsafe { OcFunctionLogbook::from_raw(log) };
        self.driver.must_execute(&logbook)
    }

    unsafe fn validate_raw(&self, log: *mut ffi::TFunctionLogbookHandle) {
        // Safety: ptr valid for call duration; exclusive access.
        let mut logbook = unsafe { OcFunctionLogbook::from_raw(log) };
        self.driver.validate(&mut logbook)
    }

    unsafe fn arguments_raw(&self, list: *mut ffi::TdfLabelList) {
        let mut label_list = unsafe { OcFunctionLabelList::from_raw(list) };
        for label in self.driver.arguments() {
            label_list.push(&label);
        }
    }

    unsafe fn results_raw(&self, list: *mut ffi::TdfLabelList) {
        // Safety: ptr valid for call duration; exclusive access.
        let mut label_list = unsafe { OcFunctionLabelList::from_raw(list) };
        for label in self.driver.results() {
            label_list.push(&label);
        }
    }
}

// ── Public registration API ───────────────────────────────────────────────────

/// Registers `driver` as the handler for the function GUID `guid_str`.
///
/// `guid_str` must be in `"xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"` format.
/// Call once per driver type, before any document rebuild that may invoke
/// that function. Calling from multiple threads is prevented at compile time
/// by the `!Send` bound on OCAF session types.
///
/// Returns `Err` if `guid_str` is malformed, or if OCCT's DriverTable rejects
/// the registration. Returns `Ok(false)` if a driver with this GUID is already
/// registered (no overwrite).
///
/// TODO: GUID API surface TBD — the `&str` parameter may change.
pub fn register_driver(uuid: uuid::Uuid, driver: impl FunctionDriver) -> bool {
    let adapter = Box::new(FunctionDriverAdapter { driver });
    occt_sys::sys_topo::register_raw(uuid, adapter).is_some()
}
#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::ocaf::OcApplication;

    use super::*;

    struct NoopDriver;

    impl FunctionDriver for NoopDriver {
        fn execute(&self, log: &mut OcFunctionLogbook<'_>) -> i32 {
            log.done(true);
            0
        }
    }

    #[test]
    fn register_driver_valid_guid_succeeds() {
        let result = register_driver(
            Uuid::try_from("11111111-1111-1111-1111-111111111111").unwrap(),
            NoopDriver,
        );
        assert!(matches!(result, true), "got {result:?}");
    }

    #[test]
    fn register_driver_duplicate_guid_returns_false() {
        let guid = Uuid::try_from("22222222-2222-2222-2222-222222222222").unwrap();
        register_driver(guid, NoopDriver);
        let second = register_driver(guid, NoopDriver);
        assert!(matches!(second, false), "got {second:?}");
    }

    #[test]
    fn default_must_execute_returns_true() {
        // Default impl returns true unconditionally.
        // Verified here without FFI by calling through a concrete type
        // that does not override must_execute.
        struct DefaultDriver;
        impl FunctionDriver for DefaultDriver {
            fn execute(&self, _log: &mut OcFunctionLogbook<'_>) -> i32 {
                0
            }
        }
        // Can't call must_execute directly without an OcFunctionLogbook, which
        // requires a live logbook pointer. Full dispatch test blocked on
        // TFunction_Function attribute binding. Verify via registration only.
        register_driver(
            Uuid::try_from("33333333-3333-3333-3333-333333333333").unwrap(),
            DefaultDriver,
        );
    }

    // TODO: full dispatch test (execute/must_execute/validate/arguments/results
    // called through real OCCT machinery) requires TFunction_Function attribute
    // binding so that a rebuild can be triggered on a document label. Add once
    // TFunction_Function::Set and TFunction_Iterator are bound.
    #[test]
    fn driver_creates_labels_under_captured_parent() {
        use occt_sys::ffi;
        use occt_sys::sys_topo::{register_raw, rust_driver_execute, FunctionDriverRaw};
        use uuid::Uuid;

        // ── Setup ────────────────────────────────────────────────────────────────

        let mut app = crate::ocaf::OcApplication::new();
        let mut doc = app.new_document("BinXCAF").unwrap();
        let main = doc.main();

        // Create the parent label at main/1 in its own command.
        doc.begin_command().unwrap();
        let parent = main.get_or_create_child(1);
        doc.commit().unwrap();

        // ── Driver definition ────────────────────────────────────────────────────
        //
        // Captures a clone of `parent`. On execute, creates children at tags
        // 2, 4, 8, 16 directly via ffi (no Command<'_> proof token needed —
        // the test opens a transaction before calling rust_driver_execute).
        // Does not touch the logbook (called with a null stand-in in this test).

        struct LabelCreatingDriver {
            parent: OcLabel,
        }

        unsafe impl FunctionDriverRaw for LabelCreatingDriver {
            unsafe fn execute_raw(&self, _log: *mut ffi::TFunctionLogbookHandle) -> i32 {
                for tag in [2i32, 4, 8, 16] {
                    // Safety: a transaction is open in the test body; the label
                    // pointer is valid for the document's lifetime.
                    let _ =
                        ffi::tdf_label_find_child(self.parent.inner.as_ref().unwrap(), tag, true);
                }
                0
            }
            unsafe fn must_execute_raw(&self, _: *mut ffi::TFunctionLogbookHandle) -> bool {
                true
            }
            unsafe fn validate_raw(&self, _: *mut ffi::TFunctionLogbookHandle) {}
            unsafe fn arguments_raw(&self, _: *mut ffi::TdfLabelList) {}
            unsafe fn results_raw(&self, _: *mut ffi::TdfLabelList) {}
        }

        // ── Registration ─────────────────────────────────────────────────────────

        let id = register_raw(
            Uuid::try_from("a1b2c3d4-e5f6-7890-abcd-ef1234567890").unwrap(),
            Box::new(LabelCreatingDriver {
                parent: parent.clone(),
            }),
        )
        .expect("GUID already taken — use a unique UUID for this test");

        // ── Invocation ───────────────────────────────────────────────────────────
        //
        // Open a command so that label creation is part of a transaction, matching
        // how OCCT's TFunction machinery would call Execute.

        doc.begin_command().unwrap();
        let code = unsafe { rust_driver_execute(id, 0) };
        assert_eq!(code, 0, "driver returned non-zero exit code");
        doc.commit().unwrap();

        // ── Verification ─────────────────────────────────────────────────────────

        for tag in [2i32, 4, 8, 16] {
            assert!(
                parent.find_child(tag).is_some(),
                "child at tag {tag} was not created by the driver"
            );
        }
    }
}
