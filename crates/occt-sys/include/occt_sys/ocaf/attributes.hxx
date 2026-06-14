// occt_sys/ocaf/attributes.hxx — TDataStd attribute shims.
//
// Three standard scalar attributes: Name (string), Integer (i32), Real (f64).
// Each attribute type is a TDF_Attribute subclass accessed via Handle(T).
// The shim struct owns the Handle by value (UniquePtr<ShimT> pattern).
//
// GUIDs stay entirely on the C++ side: find_on_label helpers call
// FindAttribute(GetID(), ...) internally, so no Standard_GUID type
// crosses the cxx bridge.
//
// Set() is a static method on each attribute class — it attaches or
// updates the attribute on the given label and returns a Handle to it.
// It must be called inside an open command scope.
//
// Reference:
//   TDataStd_Name    — https://dev.opencascade.org/doc/refman/html/class_t_data_std___name.html
//   TDataStd_Integer — https://dev.opencascade.org/doc/refman/html/class_t_data_std___integer.html
//   TDataStd_Real    — https://dev.opencascade.org/doc/refman/html/class_t_data_std___real.html
//   TDF_Label::FindAttribute — https://dev.opencascade.org/doc/refman/html/class_t_d_f___label.html
//
// Sourced from OCCT 7.9 documentation.
// No derivation from any other binding crate.

#pragma once

#include <memory>

#include <TCollection_ExtendedString.hxx>
#include <TDataStd_Integer.hxx>
#include <TDataStd_Name.hxx>
#include <TDataStd_AsciiString.hxx>
#include <TDataStd_Comment.hxx>
#include <TDataStd_Real.hxx>
#include <TDataStd_ReferenceList.hxx>
#include <TDataStd_ReferenceArray.hxx>
#include <TDF_Label.hxx>
#include <TDF_LabelList.hxx>

#include "label.hxx"
#include "../exception.hxx"
#include "rust/cxx.h"

// ── TDataStd_Name ─────────────────────────────────────────────────────────────

struct TDataStdNameHandle {
    Handle(TDataStd_Name) inner;
};

// TDataStd_Name::Set(L, string) — static.
// Attaches or updates the Name attribute on L.
// Must be called inside an open command scope.
inline std::unique_ptr<TDataStdNameHandle> tdatastd_name_set(
    const TdfLabel& label, rust::Str value)
{
    try {
        std::string s(value.data(), value.size());
        // isMultiByte=true: decode input bytes as UTF-8, not Latin-1.
        // Without this, each byte >= 0x80 becomes its own UCS-2 code
        // unit, which ToUTF8CString then re-encodes as a *separate*
        // 2-byte sequence — corrupting non-ASCII input (e.g.
        // "café" -> "cafÃ©").
        TCollection_ExtendedString ext(s.c_str(), Standard_True);
        auto result = std::make_unique<TDataStdNameHandle>();
        result->inner = TDataStd_Name::Set(label.inner, ext);
        return result;
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}

// TDataStd_Name::Get() const — reads the string value.
// Returns a UTF-8 encoded rust::String.
inline rust::String tdatastd_name_get(const TDataStdNameHandle& h) {
    // TCollection_ExtendedString stores UCS-2; ToUTF8CString writes UTF-8.
    // We go via std::string using the ASCII-safe path: if the name is
    // pure ASCII, ToCString() is sufficient.  For full Unicode, allocate
    // a buffer via ToUTF8CString.
    const TCollection_ExtendedString& ext = h.inner->Get();
    // Allocate a buffer large enough for UTF-8 (worst case 3× code units).
    Standard_Integer len = ext.LengthOfCString();
    std::string buf(static_cast<size_t>(len) + 1, '\0');
    char* ptr = buf.data();
    ext.ToUTF8CString(ptr);
    buf.resize(std::strlen(ptr));
    return rust::String(buf);
}

// Find TDataStd_Name on a label.  Returns nullptr if not present.
inline std::unique_ptr<TDataStdNameHandle> tdatastd_name_find(const TdfLabel& label) {
    Handle(TDataStd_Name) attr;
    if (label.inner.FindAttribute(TDataStd_Name::GetID(), attr)) {
        auto result = std::make_unique<TDataStdNameHandle>();
        result->inner = attr;
        return result;
    }
    return nullptr;
}

// TDF_Label::ForgetAttribute(GUID) const — removes the Name attribute if
// present. Returns false if it was not present. No exception path.
inline bool tdatastd_name_forget(const TdfLabel& label) {
    return label.inner.ForgetAttribute(TDataStd_Name::GetID()) == Standard_True;
}
// ── TDataStd_Comment ──────────────────────────────────────────────────────────

struct TDataStdCommentHandle {
    Handle(TDataStd_Comment) inner;
};

// TDataStd_Comment::Set(L, string) — static.
// Attaches or updates the Comment attribute on L.
// Must be called inside an open command scope.
inline std::unique_ptr<TDataStdCommentHandle> tdatastd_comment_set(
    const TdfLabel& label, rust::Str value)
{
    try {
        std::string s(value.data(), value.size());
        // isMultiByte=true: see tdatastd_name_set.
        TCollection_ExtendedString ext(s.c_str(), Standard_True);
        auto result = std::make_unique<TDataStdCommentHandle>();
        result->inner = TDataStd_Comment::Set(label.inner, ext);
        return result;
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}

// TDataStd_Comment::Get() const — reads the string value (inherited from
// TDataStd_GenericExtString, same conversion as TDataStd_Name::Get()).
// Returns a UTF-8 encoded rust::String.
inline rust::String tdatastd_comment_get(const TDataStdCommentHandle& h) {
    const TCollection_ExtendedString& ext = h.inner->Get();
    Standard_Integer len = ext.LengthOfCString();
    std::string buf(static_cast<size_t>(len) + 1, '\0');
    char* ptr = buf.data();
    ext.ToUTF8CString(ptr);
    return rust::String(buf.c_str());
}

// Find TDataStd_Comment on a label.  Returns nullptr if not present.
inline std::unique_ptr<TDataStdCommentHandle> tdatastd_comment_find(const TdfLabel& label) {
    Handle(TDataStd_Comment) attr;
    if (label.inner.FindAttribute(TDataStd_Comment::GetID(), attr)) {
        auto result = std::make_unique<TDataStdCommentHandle>();
        result->inner = attr;
        return result;
    }
    return nullptr;
}

// TDF_Label::ForgetAttribute(GUID) const — removes the Comment attribute if
// present. Returns false if it was not present. No exception path.
inline bool tdatastd_comment_forget(const TdfLabel& label) {
    return label.inner.ForgetAttribute(TDataStd_Comment::GetID()) == Standard_True;
}

// ── TDataStd_AsciiString ──────────────────────────────────────────────────────

struct TDataStdAsciiStringHandle {
    Handle(TDataStd_AsciiString) inner;
};

// TDataStd_AsciiString::Set(L, string) — static.
// TCollection_AsciiString is an 8-bit char buffer with no ASCII validation;
// constructing directly from the input bytes is a faithful byte copy, so any
// valid-UTF-8 &str round-trips unchanged through Set/Get.
// Must be called inside an open command scope.
inline std::unique_ptr<TDataStdAsciiStringHandle> tdatastd_asciistring_set(
    const TdfLabel& label, rust::Str value)
{
    try {
        std::string s(value.data(), value.size());
        TCollection_AsciiString ascii(s.c_str());
        auto result = std::make_unique<TDataStdAsciiStringHandle>();
        result->inner = TDataStd_AsciiString::Set(label.inner, ascii);
        return result;
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}

// TDataStd_AsciiString::Get() const — returns the stored bytes verbatim.
inline rust::String tdatastd_asciistring_get(const TDataStdAsciiStringHandle& h) {
    return rust::String(h.inner->Get().ToCString());
}

// Find TDataStd_AsciiString on a label.  Returns nullptr if not present.
inline std::unique_ptr<TDataStdAsciiStringHandle> tdatastd_asciistring_find(const TdfLabel& label) {
    Handle(TDataStd_AsciiString) attr;
    if (label.inner.FindAttribute(TDataStd_AsciiString::GetID(), attr)) {
        auto result = std::make_unique<TDataStdAsciiStringHandle>();
        result->inner = attr;
        return result;
    }
    return nullptr;
}

// TDF_Label::ForgetAttribute(GUID) const — removes the AsciiString attribute if
// present. Returns false if it was not present. No exception path.
inline bool tdatastd_asciistring_forget(const TdfLabel& label) {
    return label.inner.ForgetAttribute(TDataStd_AsciiString::GetID()) == Standard_True;
}

// ── TDataStd_ReferenceList ────────────────────────────────────────────────────

struct TDataStdReferenceListHandle {
    Handle(TDataStd_ReferenceList) inner;
};

// TDataStd_ReferenceList::Set(L) — static.
// Finds, or creates, an empty list-of-references attribute on L.
// Must be called inside an open command scope.
inline std::unique_ptr<TDataStdReferenceListHandle> tdatastd_referencelist_set(
    const TdfLabel& label)
{
    try {
        auto result = std::make_unique<TDataStdReferenceListHandle>();
        result->inner = TDataStd_ReferenceList::Set(label.inner);
        return result;
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}

// Find TDataStd_ReferenceList on a label. Returns nullptr if not present.
inline std::unique_ptr<TDataStdReferenceListHandle> tdatastd_referencelist_find(const TdfLabel& label) {
    Handle(TDataStd_ReferenceList) attr;
    if (label.inner.FindAttribute(TDataStd_ReferenceList::GetID(), attr)) {
        auto result = std::make_unique<TDataStdReferenceListHandle>();
        result->inner = attr;
        return result;
    }
    return nullptr;
}

// TDF_Label::ForgetAttribute(GUID) const — removes the ReferenceList attribute
// if present. Returns false if it was not present. No exception path.
inline bool tdatastd_referencelist_forget(const TdfLabel& label) {
    return label.inner.ForgetAttribute(TDataStd_ReferenceList::GetID()) == Standard_True;
}

// TDataStd_ReferenceList::Extent() const — number of label references.
inline Standard_Integer tdatastd_referencelist_extent(const TDataStdReferenceListHandle& h) {
    return h.inner->Extent();
}

// TDataStd_ReferenceList::IsEmpty() const.
inline bool tdatastd_referencelist_is_empty(const TDataStdReferenceListHandle& h) {
    return h.inner->IsEmpty() == Standard_True;
}

// TDataStd_ReferenceList::List() const, 0-based walk-and-advance — same
// pattern as MakeFilletBuilder::modified_at/generated_at for
// TopTools_ListOfShape, applied to TDF_LabelList. Caller must ensure
// 0 <= index < Extent().
inline std::unique_ptr<TdfLabel> tdatastd_referencelist_at(
    const TDataStdReferenceListHandle& h, Standard_Integer index)
{
    const TDF_LabelList& lst = h.inner->List();
    auto it = lst.begin();
    std::advance(it, static_cast<std::ptrdiff_t>(index));
    return std::make_unique<TdfLabel>(TdfLabel{*it});
}

// TDataStd_ReferenceList::Append(value) — non-const on the attribute, but
// Handle::operator-> returns a non-const pointer regardless of handle
// constness, so a const handle reference suffices.
// Must be called inside an open command scope.
inline void tdatastd_referencelist_append(
    const TDataStdReferenceListHandle& h, const TdfLabel& value)
{
    h.inner->Append(value.inner);
}

// ── TDataStd_ReferenceArray ───────────────────────────────────────────────────

struct TDataStdReferenceArrayHandle {
    Handle(TDataStd_ReferenceArray) inner;
};

// TDataStd_ReferenceArray::Set(L, lower, upper) — static.
// Finds, or creates, a reference array attribute on L with 0-based bounds
// [0, len-1]. Elements are default-initialized (null labels) until
// set_value is called. Must be called inside an open command scope.
inline std::unique_ptr<TDataStdReferenceArrayHandle> tdatastd_referencearray_set(
    const TdfLabel& label, Standard_Integer len)
{
    try {
        auto result = std::make_unique<TDataStdReferenceArrayHandle>();
        result->inner = TDataStd_ReferenceArray::Set(label.inner, 0, len - 1);
        return result;
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}

// Find TDataStd_ReferenceArray on a label. Returns nullptr if not present.
inline std::unique_ptr<TDataStdReferenceArrayHandle> tdatastd_referencearray_find(const TdfLabel& label) {
    Handle(TDataStd_ReferenceArray) attr;
    if (label.inner.FindAttribute(TDataStd_ReferenceArray::GetID(), attr)) {
        auto result = std::make_unique<TDataStdReferenceArrayHandle>();
        result->inner = attr;
        return result;
    }
    return nullptr;
}

// TDF_Label::ForgetAttribute(GUID) const — removes the ReferenceArray
// attribute if present. Returns false if it was not present. No exception path.
inline bool tdatastd_referencearray_forget(const TdfLabel& label) {
    return label.inner.ForgetAttribute(TDataStd_ReferenceArray::GetID()) == Standard_True;
}

// TDataStd_ReferenceArray::Length() const — number of elements (== len passed to set).
inline Standard_Integer tdatastd_referencearray_length(const TDataStdReferenceArrayHandle& h) {
    return h.inner->Length();
}

// TDataStd_ReferenceArray::Value(index) const — 0-based (Set always called
// with lower=0). Raises OutOfRange if index is outside [0, Length()-1].
inline std::unique_ptr<TdfLabel> tdatastd_referencearray_value(
    const TDataStdReferenceArrayHandle& h, Standard_Integer index)
{
    try {
        return std::make_unique<TdfLabel>(TdfLabel{h.inner->Value(index)});
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}

// TDataStd_ReferenceArray::SetValue(index, value) — 0-based. Raises
// OutOfRange if index is outside [0, Length()-1]. Non-const on the
// attribute, but callable through a const handle reference (see
// Handle::operator-> note in bound_api_reference.md).
// Must be called inside an open command scope.
inline void tdatastd_referencearray_set_value(
    const TDataStdReferenceArrayHandle& h, Standard_Integer index, const TdfLabel& value)
{
    try {
        h.inner->SetValue(index, value.inner);
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}

// ── TDataStd_Integer ──────────────────────────────────────────────────────────

struct TDataStdIntegerHandle {
    Handle(TDataStd_Integer) inner;
};

// TDataStd_Integer::Set(L, value) — static.
inline std::unique_ptr<TDataStdIntegerHandle> tdatastd_integer_set(
    const TdfLabel& label, int value)
{
    try {
        auto result = std::make_unique<TDataStdIntegerHandle>();
        result->inner = TDataStd_Integer::Set(label.inner, value);
        return result;
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}

// TDataStd_Integer::Get() const — reads the integer value.
inline int tdatastd_integer_get(const TDataStdIntegerHandle& h) {
    return h.inner->Get();
}

// Find TDataStd_Integer on a label.  Returns nullptr if not present.
inline std::unique_ptr<TDataStdIntegerHandle> tdatastd_integer_find(
    const TdfLabel& label)
{
    Handle(TDataStd_Integer) attr;
    if (label.inner.FindAttribute(TDataStd_Integer::GetID(), attr)) {
        auto result = std::make_unique<TDataStdIntegerHandle>();
        result->inner = attr;
        return result;
    }
    return nullptr;
}
inline bool tdatastd_integer_forget(const TdfLabel& label) {
    return label.inner.ForgetAttribute(TDataStd_Integer::GetID()) == Standard_True;
}

// ── TDataStd_Real ─────────────────────────────────────────────────────────────

struct TDataStdRealHandle {
    Handle(TDataStd_Real) inner;
};

// TDataStd_Real::Set(L, value) — static.
inline std::unique_ptr<TDataStdRealHandle> tdatastd_real_set(
    const TdfLabel& label, double value)
{
    try {
        auto result = std::make_unique<TDataStdRealHandle>();
        result->inner = TDataStd_Real::Set(label.inner, value);
        return result;
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}

// TDataStd_Real::Get() const — reads the real value.
inline double tdatastd_real_get(const TDataStdRealHandle& h) {
    return h.inner->Get();
}

// Find TDataStd_Real on a label.  Returns nullptr if not present.
inline std::unique_ptr<TDataStdRealHandle> tdatastd_real_find(
    const TdfLabel& label)
{
    Handle(TDataStd_Real) attr;
    if (label.inner.FindAttribute(TDataStd_Real::GetID(), attr)) {
        auto result = std::make_unique<TDataStdRealHandle>();
        result->inner = attr;
        return result;
    }
    return nullptr;
}
inline bool tdatastd_real_forget(const TdfLabel& label) {
    return label.inner.ForgetAttribute(TDataStd_Real::GetID()) == Standard_True;
}
