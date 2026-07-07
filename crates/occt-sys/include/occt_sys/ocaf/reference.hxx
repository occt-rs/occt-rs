// Reference: https://dev.opencascade.org/doc/refman/html/class_t_d_f___reference.html
//
// Sourced from OCCT 7.9 documentation.
// No derivation from any other binding crate.

#pragma once

#include <memory>

#include <TDF_Reference.hxx>

#include "label.hxx"
#include "rust/cxx.h"

// ── TdfReferenceHandle shim ──────────────────────────────────────────────────
//
// Field name `.inner` matches the convention already established by
// TdfLabel and TFunctionLogbookHandle in this codebase (label.hxx,
// function.hxx) — not reinvented here.

struct TdfReferenceHandle {
    Handle(TDF_Reference) inner;
};

// TDF_Reference::Set — static. Attaches or updates the reference attribute
// on `at`, pointing it at `target`. Must be called inside an open command,
// same requirement as every other `_set` shim in this bridge (TDataStd_Name
// etc.) — exceptions on that precondition are handled the same way, via
// the Result<> return on the Rust side.
inline std::unique_ptr<TdfReferenceHandle> tdf_reference_set(
    const TdfLabel& at, const TdfLabel& target)
{
    Handle(TDF_Reference) ref = TDF_Reference::Set(at.inner, target.inner);
    return std::make_unique<TdfReferenceHandle>(TdfReferenceHandle{ref});
}

// Find — returns nullptr (None on the Rust side) if `at` has no
// TDF_Reference attribute. Mirrors the tdatastd_x_find convention.
inline std::unique_ptr<TdfReferenceHandle> tdf_reference_find(const TdfLabel& at) {
    Handle(TDF_Reference) ref;
    if (!at.inner.FindAttribute(TDF_Reference::GetID(), ref)) {
        return nullptr;
    }
    return std::make_unique<TdfReferenceHandle>(TdfReferenceHandle{ref});
}

// Get — const. TDF_Reference::Get() returns TDF_Label by value.
inline std::unique_ptr<TdfLabel> tdf_reference_get(const TdfReferenceHandle& h) {
    return std::make_unique<TdfLabel>(TdfLabel{h.inner->Get()});
}
