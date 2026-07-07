// TFunction_Driver trampoline shim.
//
// Defines RustFunctionDriverShim, a single concrete TFunction_Driver subclass
// that forwards all virtual calls to Rust through extern "Rust" callbacks
// declared in the cxx bridge (sys_topo.rs). Also defines TFunctionLogbookHandle
// and TFunctionLabelListShim, which are the opaque bridge types used to
// carry logbook and label-list access across the FFI boundary.
//
// ODR constraint — IMPORTANT:
//   IMPLEMENT_STANDARD_RTTIEXT expands to two function definitions
//   (DynamicType and DownCast). This header must be included in exactly
//   one translation unit: the cxx-generated .cpp that results from
//   sys_topo.rs → topo.hxx (umbrella) → this file. Do not include this
//   header directly from any other file.
//
// Sourced from OCCT 7.9 documentation.
// No derivation from any other binding crate.
//
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_function___driver.html
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_function___driver_table.html
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_function___logbook.html
// Reference: https://dev.opencascade.org/doc/refman/html/class_standard___g_u_i_d.html

#pragma once

#include <cstdint>
#include <memory>
#include <string>

#include <Standard_GUID.hxx>
#include <TDF_LabelList.hxx>
#include <TFunction_IFunction.hxx>
#include <TFunction_Iterator.hxx>
#include <TFunction_Driver.hxx>
#include <TFunction_DriverTable.hxx>
#include <TFunction_Logbook.hxx>

#include "label.hxx"
#include "label_list.hxx"
#include "../exception.hxx"
#include "rust/cxx.h"

// ── Forward declarations for extern "Rust" callbacks ─────────────────────────
//
// Satisfied by the cxx-generated thunks compiled into the same translation
// unit. Signatures must match the extern "Rust" block in sys_topo.rs exactly.
// The pointer parameters are raw because cxx does not produce Pin wrappers for
// extern "Rust" — the callers (virtual method bodies below) guarantee validity
// for the duration of each call.

struct TFunctionLogbookHandle;

int32_t rust_driver_execute(uint64_t id, std::size_t log) noexcept;
bool    rust_driver_must_execute(uint64_t id, std::size_t log) noexcept;
void    rust_driver_validate(uint64_t id, std::size_t log) noexcept;
void    rust_driver_arguments(uint64_t id, std::size_t list) noexcept;
void    rust_driver_results(uint64_t id, std::size_t list) noexcept;

// ── TFunctionLogbookHandle ────────────────────────────────────────────────────
//
// Opaque cxx bridge type. Wraps Handle(TFunction_Logbook) by value.
//
// Instances are created on the C++ stack inside each virtual method body and
// passed to Rust by raw pointer for the duration of that call only. Copying
// the Handle bumps the refcount, ensuring the logbook remains alive regardless
// of what the C++ caller does with its own handle reference during the call.
//
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_function___logbook.html

struct TFunctionLogbookHandle {
    Handle(TFunction_Logbook) inner;
};

// IsModified — const. Returns true if L (or its children) is touched or
// impacted in this logbook. Called by must_execute implementations.
// Reference: TFunction_Logbook::IsModified
inline bool tfunction_logbook_is_modified(
    const TFunctionLogbookHandle& h,
    const TdfLabel& label,
    bool with_children)
{
    return h.inner->IsModified(
        label.inner,
        with_children ? Standard_True : Standard_False
    ) == Standard_True;
}

// SetImpacted — non-const. Marks L (and optionally its children) as impacted.
// Called by execute implementations to record output labels.
// Reference: TFunction_Logbook::SetImpacted
inline void tfunction_logbook_set_impacted(
    TFunctionLogbookHandle& h,
    const TdfLabel& label,
    bool with_children)
{
    h.inner->SetImpacted(
        label.inner,
        with_children ? Standard_True : Standard_False
    );
}

// SetValid — non-const. Marks L (and optionally its children) as valid.
// Called by validate implementations.
// Reference: TFunction_Logbook::SetValid
inline void tfunction_logbook_set_valid(
    TFunctionLogbookHandle& h,
    const TdfLabel& label,
    bool with_children)
{
    h.inner->SetValid(
        label.inner,
        with_children ? Standard_True : Standard_False
    );
}

// IsDone — const. Returns current execution status.
// Reference: TFunction_Logbook::IsDone
inline bool tfunction_logbook_is_done(const TFunctionLogbookHandle& h) {
    return h.inner->IsDone() == Standard_True;
}

// Done — non-const. Sets execution status.
// Reference: TFunction_Logbook::Done
inline void tfunction_logbook_done(TFunctionLogbookHandle& h, bool status) {
    h.inner->Done(status ? Standard_True : Standard_False);
}

// ── TFunctionLogbookHandle — new functions (type itself already declared) ──
//
// CORRECTION to the doc comment on the existing type: TFunction_Logbook is a
// TDF_Attribute with ordinary document-attribute lifetime (attached at the
// document root via TFunction_Logbook::Set) — NOT scoped to a single
// dispatch call. That was true only of how this bridge type was previously
// obtained (the raw pointer handed to driver callbacks), not of the
// underlying OCCT object. tfunction_logbook_set is the new owned path; safe
// alongside the existing dispatch-context pointer since
// Handle(TFunction_Logbook) is refcounted.
 
inline std::unique_ptr<TFunctionLogbookHandle> tfunction_logbook_set(const TdfLabel& access) {
    Handle(TFunction_Logbook) lb = TFunction_Logbook::Set(access.inner);
    return std::make_unique<TFunctionLogbookHandle>(TFunctionLogbookHandle{lb});
}
 
// SetTouched — non-const. NOTE: no with_children parameter, unlike
// SetImpacted/SetValid above. Confirmed from TFunction_Logbook.hxx.
inline void tfunction_logbook_set_touched(TFunctionLogbookHandle& h, const TdfLabel& label) {
    h.inner->SetTouched(label.inner);
}
 
inline void tfunction_logbook_get_touched(const TFunctionLogbookHandle& h, TdfLabelList& out) {
    const TDF_LabelMap& touched = h.inner->GetTouched();
    for (TDF_MapIteratorOfLabelMap it(touched); it.More(); it.Next()) {
        out.list->Append(it.Key());
    }
}
 
inline void tfunction_logbook_clear(TFunctionLogbookHandle& h) {
    h.inner->Clear();
}
 
// ── TFunction_IFunction — flat functions, no persistent bridge type ────────
//
// Not given an opaque type: TFunction_IFunction is a zero-cost value wrapper
// around one TDF_Label (confirmed from TFunction_IFunction.hxx — no Handle,
// no heap state). Each function below reconstructs it on the stack.
//
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_function___i_function.html
 
inline bool tfunction_ifunction_new_function(
    const TdfLabel& label,
    uint32_t a32b, uint16_t a16b1, uint16_t a16b2, uint16_t a16b3,
    uint8_t a8b1, uint8_t a8b2, uint8_t a8b3, uint8_t a8b4, uint8_t a8b5, uint8_t a8b6)
{
    // Cast convention copied exactly from tfunction_register_rust_driver
    // below, not reinvented.
    Standard_GUID guid(
        static_cast<int>(a32b),
        static_cast<char16_t>(a16b1),
        static_cast<char16_t>(a16b2),
        static_cast<char16_t>(a16b3),
        a8b1, a8b2, a8b3, a8b4, a8b5, a8b6
    );
    return TFunction_IFunction::NewFunction(label.inner, guid);
}
 
inline bool tfunction_ifunction_delete_function(const TdfLabel& label) {
    return TFunction_IFunction::DeleteFunction(label.inner);
}
 
inline bool tfunction_ifunction_update_dependencies_all(const TdfLabel& access) {
    return TFunction_IFunction::UpdateDependencies(access.inner);
}
 
inline bool tfunction_ifunction_update_dependencies_one(const TdfLabel& label) {
    TFunction_IFunction iFunc(label.inner);
    return iFunc.UpdateDependencies();
}
 
inline void tfunction_ifunction_arguments(const TdfLabel& label, TdfLabelList& out) {
    TFunction_IFunction iFunc(label.inner);
    iFunc.Arguments(*out.list);
}
 
inline void tfunction_ifunction_results(const TdfLabel& label, TdfLabelList& out) {
    TFunction_IFunction iFunc(label.inner);
    iFunc.Results(*out.list);
}
 
inline void tfunction_ifunction_get_previous(const TdfLabel& label, TdfLabelList& out) {
    TFunction_IFunction iFunc(label.inner);
    iFunc.GetPrevious(*out.list);
}
 
inline void tfunction_ifunction_get_next(const TdfLabel& label, TdfLabelList& out) {
    TFunction_IFunction iFunc(label.inner);
    iFunc.GetNext(*out.list);
}
 
inline int32_t tfunction_ifunction_get_status(const TdfLabel& label) {
    TFunction_IFunction iFunc(label.inner);
    return static_cast<int32_t>(iFunc.GetStatus());
}
 
inline void tfunction_ifunction_set_status(const TdfLabel& label, int32_t status) {
    TFunction_IFunction iFunc(label.inner);
    iFunc.SetStatus(static_cast<TFunction_ExecutionStatus>(status));
}
 
// ── TFunction_Iterator — stateful, gets a real opaque type ──────────────────
//
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_function___iterator.html
 
struct TFunctionIteratorShim {
    TFunction_Iterator it;
};
 
inline std::unique_ptr<TFunctionIteratorShim> new_tfunction_iterator(const TdfLabel& access) {
    auto shim = std::make_unique<TFunctionIteratorShim>();
    shim->it.Init(access.inner);
    return shim;
}
 
inline void tfunction_iterator_set_usage_of_execution_status(
    TFunctionIteratorShim& it, bool usage)
{
    it.it.SetUsageOfExecutionStatus(usage ? Standard_True : Standard_False);
}
 
inline bool tfunction_iterator_more(const TFunctionIteratorShim& it) {
    return it.it.More() == Standard_True;
}
 
inline void tfunction_iterator_next(TFunctionIteratorShim& it) {
    it.it.Next();
}
 
inline void tfunction_iterator_current(const TFunctionIteratorShim& it, TdfLabelList& out) {
    const TDF_LabelList& current = it.it.Current();
    for (TDF_ListIteratorOfLabelList lit(current); lit.More(); lit.Next()) {
        out.list->Append(lit.Value());
    }
}



// ── RustFunctionDriverShim ────────────────────────────────────────────────────
//
// A single concrete TFunction_Driver subclass used for all Rust-registered
// drivers. Each registration call creates one instance, differentiated by
// myRustId. The shim stores no geometry or document state; it is a pure
// dispatch trampoline.
//
// The instance is created via `new` and stored in a Handle(TFunction_Driver),
// which holds it alive for the session via the Standard_Transient refcount
// inherited through TFunction_Driver. OCCT's DriverTable singleton owns the
// Handle; the shim is never explicitly deleted.
//
// All five virtual methods are const on TFunction_Driver. The logbook and
// label-list parameters are bridged via stack-local shim structs passed by
// pointer — valid only for the duration of each call.
//
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_function___driver.html

class RustFunctionDriverShim : public TFunction_Driver {
public:
    explicit RustFunctionDriverShim(uint64_t rust_id) : myRustId(rust_id) {}

    // Execute — pure virtual in TFunction_Driver.
    // log: non-const Handle; Rust may call SetImpacted / Done.
    // Returns application-defined integer; 0 conventionally means success.
    // Reference: TFunction_Driver::Execute
    Standard_Integer Execute(Handle(TFunction_Logbook)& log) const override {
        TFunctionLogbookHandle h{ log };
        return static_cast<Standard_Integer>(
            rust_driver_execute(myRustId, reinterpret_cast<std::size_t>(&h)));
    }

    // MustExecute — virtual; base class has a default implementation.
    // log: const Handle (read-only); Rust may call IsModified.
    // We copy into a non-const TFunctionLogbookHandle — our copy, our rules.
    // Rust receives a pointer to it but the trait signature (&OcFunctionLogbook,
    // no &mut) prevents calling mutating methods.
    // Reference: TFunction_Driver::MustExecute
    Standard_Boolean MustExecute(const Handle(TFunction_Logbook)& log) const override {
        TFunctionLogbookHandle h{ log };
        return rust_driver_must_execute(myRustId, reinterpret_cast<std::size_t>(&h))
            ? Standard_True : Standard_False;
    }


    // Validate — virtual; base class has a default implementation.
    // log: non-const Handle; Rust may call SetValid.
    // Reference: TFunction_Driver::Validate
    void Validate(Handle(TFunction_Logbook)& log) const override {
        TFunctionLogbookHandle h{ log };
        rust_driver_validate(myRustId, reinterpret_cast<std::size_t>(&h));
    }

    // Arguments — virtual; base class has a default (empty) implementation.
    // Rust appends argument labels via OcFunctionLabelList::push.
    // Reference: TFunction_Driver::Arguments
   void Arguments(TDF_LabelList& args) const override {
      TdfLabelList shim(args);
      rust_driver_arguments(myRustId, reinterpret_cast<std::size_t>(&shim));
  }


    // Results — virtual; base class has a default (empty) implementation.
    // Rust appends result labels via OcFunctionLabelList::push.
    // Reference: TFunction_Driver::Results
  void Results(TDF_LabelList& res) const override {
      TdfLabelList shim(res);
      rust_driver_results(myRustId, reinterpret_cast<std::size_t>(&shim));
    }

    DEFINE_STANDARD_RTTIEXT(RustFunctionDriverShim, TFunction_Driver)

private:
    uint64_t myRustId;
};

// IMPLEMENT_STANDARD_RTTIEXT defines DynamicType() and DownCast() so that
// OCCT's RTTI machinery (IsKind, DownCast, type-name accessors) works for
// RustFunctionDriverShim. Placed in this header rather than a .cpp because
// the codebase is header-only and this header is included in exactly one
// translation unit (see ODR constraint at the top of this file).
//
// Reference: Standard_Type / DEFINE_STANDARD_RTTIEXT usage in OCCT source
IMPLEMENT_STANDARD_RTTIEXT(RustFunctionDriverShim, TFunction_Driver)

// ── Registration ──────────────────────────────────────────────────────────────
//
// Creates a RustFunctionDriverShim for rust_id and registers it under guid_str
// in the process-global TFunction_DriverTable.
//
// guid_str: UUID in "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx" format.
//   Standard_GUID(const char*) parses this format.
//   TODO: GUID API surface TBD — this may change to a typed wrapper.
//
// Returns true if added; false if a driver with this GUID already exists
// (TFunction_DriverTable::AddDriver semantics: no overwrite on false).
//
// Throws (via rethrow_occt_as_runtime_error) if guid_str is malformed or
// OCCT raises for another reason.
//
// Reference: TFunction_DriverTable::AddDriver
// Reference: Standard_GUID — https://dev.opencascade.org/doc/refman/html/class_standard___g_u_i_d.html
inline bool tfunction_register_rust_driver(
    uint32_t a32b,
    uint16_t a16b1, uint16_t a16b2, uint16_t a16b3,
    uint8_t a8b1, uint8_t a8b2, uint8_t a8b3,
    uint8_t a8b4, uint8_t a8b5, uint8_t a8b6,
    uint64_t rust_id)
{
    Standard_GUID guid(
        static_cast<int>(a32b),
        static_cast<char16_t>(a16b1),
        static_cast<char16_t>(a16b2),
        static_cast<char16_t>(a16b3),
        a8b1, a8b2, a8b3, a8b4, a8b5, a8b6
    );
    Handle(TFunction_Driver) driver = new RustFunctionDriverShim(rust_id);
    return TFunction_DriverTable::Get()->AddDriver(guid, driver) == Standard_True;
}
