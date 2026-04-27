// TDF_Label is internally a Handle(TDF_LabelNode) — a non-owning reference
// into the TDF_Data tree.  Copying is cheap (ref-count increment on the
// underlying node).  The TdfLabel shim holds a TDF_Label by value and is
// wrapped in UniquePtr so it can cross the cxx FFI boundary.
//
// TDF_Label is NOT the owner of its data; the TDF_Data tree is.  Rust
// enforces this by binding OcLabel with a lifetime parameter tied to the
// OcDocument that owns the tree.
//
// Reference:
//   TDF_Label         — https://dev.opencascade.org/doc/refman/html/class_t_d_f___label.html
//   TDF_ChildIterator — https://dev.opencascade.org/doc/refman/html/class_t_d_f___child_iterator.html
//   TDF_Tool          — https://dev.opencascade.org/doc/refman/html/class_t_d_f___tool.html
//
// Sourced from OCCT 7.9 documentation.
// No derivation from any other binding crate.

#pragma once

#include <memory>
#include <string>

#include <TDF_ChildIterator.hxx>
#include <TDF_Label.hxx>
#include <TDF_Tool.hxx>
#include <TCollection_AsciiString.hxx>

#include "../exception.hxx"
#include "rust/cxx.h"

// ── TdfLabel shim ─────────────────────────────────────────────────────────────

struct TdfLabel {
    TDF_Label inner;
};

// Copy-constructs a TdfLabel.  Cheap: increments the internal Handle ref-count.
inline std::unique_ptr<TdfLabel> clone_tdf_label(const TdfLabel& l) {
    return std::make_unique<TdfLabel>(TdfLabel{l.inner});
}

// TDF_Label::IsNull() — true when the label has no associated node.
// A null label results from an unsuccessful FindChild(create=false) call.
inline bool tdf_label_is_null(const TdfLabel& l) {
    return l.inner.IsNull() == Standard_True;
}

// TDF_Label::IsRoot() — true when this label has no parent (is the framework root).
inline bool tdf_label_is_root(const TdfLabel& l) {
    return l.inner.IsRoot() == Standard_True;
}

// TDF_Label::Tag() — the integer tag that identifies this label among its siblings.
inline int tdf_label_tag(const TdfLabel& l) {
    return l.inner.Tag();
}

// TDF_Label::Father() — the parent label.
// Returns a null label if called on the root.
inline std::unique_ptr<TdfLabel> tdf_label_father(const TdfLabel& l) {
    return std::make_unique<TdfLabel>(TdfLabel{l.inner.Father()});
}

// TDF_Label::FindChild(tag, create):
//   create=true  — creates the child with this tag if it does not exist.
//                  Result is never null.
//   create=false — returns a null label if no child with this tag exists.
//
// FindChild is const on TDF_Label: the label value itself is unchanged even
// when create=true adds a child to the tree.
inline std::unique_ptr<TdfLabel> tdf_label_find_child(
    const TdfLabel& l, int tag, bool create)
{
    return std::make_unique<TdfLabel>(TdfLabel{
        l.inner.FindChild(tag, create ? Standard_True : Standard_False)
    });
}

// TDF_Label::HasAttribute() — true when at least one attribute is attached.
inline bool tdf_label_has_attribute(const TdfLabel& l) {
    return l.inner.HasAttribute() == Standard_True;
}

// TDF_Label::NbAttributes() — count of attributes attached to this label.
inline int tdf_label_nb_attributes(const TdfLabel& l) {
    return l.inner.NbAttributes();
}

// TDF_Tool::Entry — the label's path as a colon-delimited string, e.g. "0:1:2:3".
// Useful for debugging and stable identification within a session.
inline rust::String tdf_label_entry(const TdfLabel& l) {
    TCollection_AsciiString entry;
    TDF_Tool::Entry(l.inner, entry);
    return rust::String(entry.ToCString());
}

// ── TDF_ChildIterator shim ────────────────────────────────────────────────────
//
// Iterates over direct children (all_levels=false) or all descendants
// (all_levels=true) of a label.
//
// Pattern mirrors ShapeExplorer: more()/next()/value().
// value() is const (reads current, does not advance); next() is non-const.

struct TdfChildIteratorShim {
    TDF_ChildIterator inner;

    TdfChildIteratorShim(const TdfLabel& label, bool all_levels)
        : inner(label.inner, all_levels ? Standard_True : Standard_False) {}

    bool more() const { return inner.More() == Standard_True; }
    void next()       { inner.Next(); }

    std::unique_ptr<TdfLabel> value() const {
        return std::make_unique<TdfLabel>(TdfLabel{inner.Value()});
    }
};

inline std::unique_ptr<TdfChildIteratorShim> new_tdf_child_iterator(
    const TdfLabel& label, bool all_levels)
{
    return std::make_unique<TdfChildIteratorShim>(label, all_levels);
}
