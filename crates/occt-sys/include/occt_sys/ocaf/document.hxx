// DocumentHandle holds Handle(TDocStd_Document) by value.
// TDocStd_Document inherits CDF_Document → Standard_Transient; it is
// ref-counted.  The Handle keeps the document alive independently of the
// application that created it.
//
// Command API (NewCommand/CommitCommand/AbortCommand) brackets a set of
// TDF modifications as an undoable unit.  All three are non-const;
// GetAvailableUndos/Redos are const.
//
// Reference:
//   TDocStd_Document — https://dev.opencascade.org/doc/refman/html/class_t_doc_std___document.html
//
// Sourced from OCCT 7.9 documentation.
// No derivation from any other binding crate.

#pragma once

#include <memory>

#include <TDocStd_Document.hxx>

#include "label.hxx"
#include "../exception.hxx"

// ── DocumentHandle shim ───────────────────────────────────────────────────────

struct DocumentHandle {
    Handle(TDocStd_Document) inner;
};

// TDocStd_Document::Main() — the root label of the user data section.
// Const: does not modify document state.  The returned label is valid for
// the lifetime of the document.
inline std::unique_ptr<TdfLabel> document_main(const DocumentHandle& doc) {
    return std::make_unique<TdfLabel>(TdfLabel{doc.inner->Main()});
}

// ── Command / transaction API ─────────────────────────────────────────────────

// NewCommand() opens a new modification scope.  Any TDF changes made after
// this call are grouped into one undoable delta on CommitCommand().
// Non-const.
inline void document_new_command(DocumentHandle& doc) {
    try {
        doc.inner->NewCommand();
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}

// CommitCommand() closes the current scope and records the delta.
// Returns true when the commit succeeded (at least one change was recorded).
// Non-const.
inline bool document_commit_command(DocumentHandle& doc) {
    try {
        return doc.inner->CommitCommand() == Standard_True;
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}

// AbortCommand() discards all changes since the last NewCommand().
// Non-const.
inline void document_abort_command(DocumentHandle& doc) {
    try {
        doc.inner->AbortCommand();
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}

// GetAvailableUndos() — number of commands available for undo.
// Const.
inline int document_get_available_undos(const DocumentHandle& doc) {
    return doc.inner->GetAvailableUndos();
}

// GetAvailableRedos() — number of commands available for redo.
// Const.
inline int document_get_available_redos(const DocumentHandle& doc) {
    return doc.inner->GetAvailableRedos();
}

// Undo() — reverts the most recent committed command.
// Returns true when an undo was actually performed.
// Non-const.
inline bool document_undo(DocumentHandle& doc) {
    try {
        return doc.inner->Undo() == Standard_True;
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}

// Redo() — re-applies the most recently undone command.
// Returns true when a redo was actually performed.
// Non-const.
inline bool document_redo(DocumentHandle& doc) {
    try {
        return doc.inner->Redo() == Standard_True;
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}

// SetUndoLimit(n) — caps the undo stack depth.  Older entries are discarded
// when the limit is reached.
// Non-const.
inline void document_set_undo_limit(DocumentHandle& doc, int n) {
    doc.inner->SetUndoLimit(n);
}
