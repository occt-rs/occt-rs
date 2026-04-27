// ApplicationHandle holds Handle(TDocStd_Application) by value.
// TDocStd_Application inherits Standard_Transient and is ref-counted.
//
// new_application() creates an independent application instance.
// application_new_document() creates an in-memory document.  The format
// string (e.g. "BinXCAF") is stored on the document but is not validated
// against registered persistence drivers at creation time — drivers are
// only required when saving/loading.
//
// Reference:
//   TDocStd_Application — https://dev.opencascade.org/doc/refman/html/class_t_doc_std___application.html
//
// Sourced from OCCT 7.9 documentation.
// No derivation from any other binding crate.

#pragma once

#include <memory>
#include <string>

#include <TCollection_ExtendedString.hxx>
#include <TDocStd_Application.hxx>

#include "document.hxx"
#include "../exception.hxx"
#include "rust/cxx.h"

// ── ApplicationHandle shim ────────────────────────────────────────────────────

struct ApplicationHandle {
    Handle(TDocStd_Application) inner;
};

// Creates a fresh TDocStd_Application.
// The Handle keeps it alive independently; Rust owns this via UniquePtr.
inline std::unique_ptr<ApplicationHandle> new_application() {
    auto result = std::make_unique<ApplicationHandle>();
    result->inner = new TDocStd_Application();
    return result;
}

// application_new_document(app, format):
//   Creates a new in-memory document under this application.
//   NewDocument is non-const: registers the new document with the application.
//   Throws if OCCT returns a null document handle (should not happen for
//   in-memory creation, but guarded defensively).
inline std::unique_ptr<DocumentHandle> application_new_document(
    ApplicationHandle& app, rust::Str format)
{
    try {
        auto result = std::make_unique<DocumentHandle>();
        // construct from rust::Str directly
        std::string fmt_str(format.data(), format.size());
        TCollection_ExtendedString fmt(fmt_str.c_str());
        app.inner->NewDocument(fmt, result->inner);
        if (result->inner.IsNull()) {
            throw std::runtime_error(
                "OCCT:TDocStd_Application:NewDocument returned a null handle");
        }
        return result;
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}
