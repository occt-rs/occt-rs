// Reference: https://dev.opencascade.org/doc/refman/html/class_n_collection___list.html
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_d_f___label.html
//
// Sourced from OCCT 7.9 documentation.
// No derivation from any other binding crate.

#pragma once

#include <cstddef>
#include <memory>

#include <TDF_LabelList.hxx>
#include <TDF_ListIteratorOfLabelList.hxx>

#include "label.hxx"
#include "rust/cxx.h"

// ── TdfLabelList shim ────────────────────────────────────────────────────────
//
// Two constructions:
//   - new_tdf_label_list(): standalone. `list` points at the struct's own
//     `owned` member.
//   - TdfLabelList(TDF_LabelList&): wraps an existing list by reference,
//     zero-copy — `list` aliases the caller's list directly. Used by
//     TFunction_Driver::Arguments/Results, where OCCT already hands us a
//     live out-param and there is nothing to copy in or out.
//
// append/len/get always go through `list`, so Rust-side code is identical
// regardless of which constructor built the instance.
//
// Safety of the self-reference: `list` pointing at a sibling member (`&owned`)
// is sound only if the struct's address never changes after construction —
// a self-referential struct that gets moved leaves `list` dangling. cxx only
// ever exposes opaque C++ types to Rust via UniquePtr<T> / Pin<&mut T> / &T,
// none of which relocate the underlying object; `std::make_unique<TdfLabelList>()`
// allocates once and that address is stable for the object's lifetime. The
// pattern is safe because of cxx's ownership model, not despite it.

struct TdfLabelList {
    TDF_LabelList owned;
    TDF_LabelList* list;

    TdfLabelList() : list(&owned) {}
    explicit TdfLabelList(TDF_LabelList& external) : list(&external) {}
    TdfLabelList(const TdfLabelList&) = delete;
    TdfLabelList& operator=(const TdfLabelList&) = delete;
};

inline std::unique_ptr<TdfLabelList> new_tdf_label_list() {
    return std::make_unique<TdfLabelList>();
}

// Reference: NCollection_List::Append
inline void tdf_labellist_append(TdfLabelList& shim, const TdfLabel& label) {
    shim.list->Append(label.inner);
}

inline size_t tdf_labellist_len(const TdfLabelList& shim) {
    return static_cast<size_t>(shim.list->Extent());
}

// TDF_LabelList has no random-access index — walk it each call. Fine for the
// list sizes involved (argument/result counts per function), not bulk data.
inline std::unique_ptr<TdfLabel> tdf_labellist_get(const TdfLabelList& shim, size_t index) {
    TDF_ListIteratorOfLabelList it(*shim.list);
    for (size_t i = 0; i < index && it.More(); ++i) it.Next();
    return std::make_unique<TdfLabel>(TdfLabel{it.Value()});
}
