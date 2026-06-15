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
#include <TDataStd_RealArray.hxx>
#include <TDataStd_IntegerArray.hxx>
#include <TDataStd_BooleanArray.hxx>
#include <TDataStd_ByteArray.hxx>
#include <TDataStd_ExtStringArray.hxx>
#include <TDataStd_IntegerList.hxx>
#include <TDataStd_RealList.hxx>
#include <TDataStd_ExtStringList.hxx>
#include <TDataStd_BooleanList.hxx>
#include <TColStd_ListOfInteger.hxx>
#include <TColStd_ListOfReal.hxx>
#include <TDataStd_ListOfExtendedString.hxx>
#include <TDataStd_ListOfByte.hxx>
#include <TDataStd_UAttribute.hxx>
#include <Standard_GUID.hxx>
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

// ── TDataStd_RealArray ────────────────────────────────────────────────────────

struct TDataStdRealArrayHandle {
    Handle(TDataStd_RealArray) inner;
};

// TDataStd_RealArray::Set(L, lower, upper) — static.
// Finds, or creates, a real array attribute on L with 0-based bounds
// [0, len-1]. isDelta omitted: OCCT's compiled-in default (Standard_False,
// DefaultDeltaOnModification) applies. Elements are zero-initialized until
// set_value is called. Must be called inside an open command scope.
inline std::unique_ptr<TDataStdRealArrayHandle> tdatastd_realarray_set(
    const TdfLabel& label, Standard_Integer len)
{
    try {
        auto result = std::make_unique<TDataStdRealArrayHandle>();
        result->inner = TDataStd_RealArray::Set(label.inner, 0, len - 1);
        return result;
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}

// Find TDataStd_RealArray on a label. Returns nullptr if not present.
inline std::unique_ptr<TDataStdRealArrayHandle> tdatastd_realarray_find(const TdfLabel& label) {
    Handle(TDataStd_RealArray) attr;
    if (label.inner.FindAttribute(TDataStd_RealArray::GetID(), attr)) {
        auto result = std::make_unique<TDataStdRealArrayHandle>();
        result->inner = attr;
        return result;
    }
    return nullptr;
}

// TDF_Label::ForgetAttribute(GUID) const — removes the RealArray attribute
// if present. Returns false if it was not present. No exception path.
inline bool tdatastd_realarray_forget(const TdfLabel& label) {
    return label.inner.ForgetAttribute(TDataStd_RealArray::GetID()) == Standard_True;
}

// TDataStd_RealArray::Length() const — number of elements (== len passed to set).
inline Standard_Integer tdatastd_realarray_length(const TDataStdRealArrayHandle& h) {
    return h.inner->Length();
}

// TDataStd_RealArray::Value(index) const — 0-based (Set always called with
// lower=0). Raises OutOfRange if index is outside [0, Length()-1].
inline Standard_Real tdatastd_realarray_value(
    const TDataStdRealArrayHandle& h, Standard_Integer index)
{
    try {
        return h.inner->Value(index);
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}

// TDataStd_RealArray::SetValue(index, value) — 0-based. Raises OutOfRange if
// index is outside [0, Length()-1]. Non-const on the attribute, but callable
// through a const handle reference (see Handle::operator-> note in
// bound_api_reference.md). Must be called inside an open command scope.
inline void tdatastd_realarray_set_value(
    const TDataStdRealArrayHandle& h, Standard_Integer index, Standard_Real value)
{
    try {
        h.inner->SetValue(index, value);
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}

// ── TDataStd_IntegerArray ─────────────────────────────────────────────────────

struct TDataStdIntegerArrayHandle {
    Handle(TDataStd_IntegerArray) inner;
};

// TDataStd_IntegerArray::Set(L, lower, upper) — static.
// Finds, or creates, an integer array attribute on L with 0-based bounds
// [0, len-1]. isDelta omitted: OCCT's compiled-in default (Standard_False,
// DefaultDeltaOnModification) applies. Elements are zero-initialized until
// set_value is called. Must be called inside an open command scope.
inline std::unique_ptr<TDataStdIntegerArrayHandle> tdatastd_integerarray_set(
    const TdfLabel& label, Standard_Integer len)
{
    try {
        auto result = std::make_unique<TDataStdIntegerArrayHandle>();
        result->inner = TDataStd_IntegerArray::Set(label.inner, 0, len - 1);
        return result;
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}

// Find TDataStd_IntegerArray on a label. Returns nullptr if not present.
inline std::unique_ptr<TDataStdIntegerArrayHandle> tdatastd_integerarray_find(const TdfLabel& label) {
    Handle(TDataStd_IntegerArray) attr;
    if (label.inner.FindAttribute(TDataStd_IntegerArray::GetID(), attr)) {
        auto result = std::make_unique<TDataStdIntegerArrayHandle>();
        result->inner = attr;
        return result;
    }
    return nullptr;
}

// TDF_Label::ForgetAttribute(GUID) const — removes the IntegerArray attribute
// if present. Returns false if it was not present. No exception path.
inline bool tdatastd_integerarray_forget(const TdfLabel& label) {
    return label.inner.ForgetAttribute(TDataStd_IntegerArray::GetID()) == Standard_True;
}

// TDataStd_IntegerArray::Length() const — number of elements (== len passed to set).
inline Standard_Integer tdatastd_integerarray_length(const TDataStdIntegerArrayHandle& h) {
    return h.inner->Length();
}

// TDataStd_IntegerArray::Value(index) const — 0-based (Set always called with
// lower=0). Raises OutOfRange if index is outside [0, Length()-1].
inline Standard_Integer tdatastd_integerarray_value(
    const TDataStdIntegerArrayHandle& h, Standard_Integer index)
{
    try {
        return h.inner->Value(index);
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}

// TDataStd_IntegerArray::SetValue(index, value) — 0-based. Raises OutOfRange
// if index is outside [0, Length()-1]. Non-const on the attribute, but
// callable through a const handle reference (see Handle::operator-> note in
// bound_api_reference.md). Must be called inside an open command scope.
inline void tdatastd_integerarray_set_value(
    const TDataStdIntegerArrayHandle& h, Standard_Integer index, Standard_Integer value)
{
    try {
        h.inner->SetValue(index, value);
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}

// ── TDataStd_BooleanArray ─────────────────────────────────────────────────────

struct TDataStdBooleanArrayHandle {
    Handle(TDataStd_BooleanArray) inner;
};

// TDataStd_BooleanArray::Set(L, lower, upper) — static. No isDelta parameter
// (unlike RealArray/IntegerArray/ByteArray/ExtStringArray).
// Finds, or creates, a boolean array attribute on L with 0-based bounds
// [0, len-1]. Elements are false-initialized until set_value is called.
// Must be called inside an open command scope.
inline std::unique_ptr<TDataStdBooleanArrayHandle> tdatastd_booleanarray_set(
    const TdfLabel& label, Standard_Integer len)
{
    try {
        auto result = std::make_unique<TDataStdBooleanArrayHandle>();
        result->inner = TDataStd_BooleanArray::Set(label.inner, 0, len - 1);
        return result;
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}

// Find TDataStd_BooleanArray on a label. Returns nullptr if not present.
inline std::unique_ptr<TDataStdBooleanArrayHandle> tdatastd_booleanarray_find(const TdfLabel& label) {
    Handle(TDataStd_BooleanArray) attr;
    if (label.inner.FindAttribute(TDataStd_BooleanArray::GetID(), attr)) {
        auto result = std::make_unique<TDataStdBooleanArrayHandle>();
        result->inner = attr;
        return result;
    }
    return nullptr;
}

// TDF_Label::ForgetAttribute(GUID) const — removes the BooleanArray attribute
// if present. Returns false if it was not present. No exception path.
inline bool tdatastd_booleanarray_forget(const TdfLabel& label) {
    return label.inner.ForgetAttribute(TDataStd_BooleanArray::GetID()) == Standard_True;
}

// TDataStd_BooleanArray::Length() const — number of elements (== len passed to set).
inline Standard_Integer tdatastd_booleanarray_length(const TDataStdBooleanArrayHandle& h) {
    return h.inner->Length();
}

// TDataStd_BooleanArray::Value(index) const — 0-based. Standard_Boolean is
// `bool`; crosses cxx directly. Raises OutOfRange if index is outside
// [0, Length()-1].
inline bool tdatastd_booleanarray_value(
    const TDataStdBooleanArrayHandle& h, Standard_Integer index)
{
    try {
        return h.inner->Value(index);
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}

// TDataStd_BooleanArray::SetValue(index, value) — 0-based. Raises OutOfRange
// if index is outside [0, Length()-1]. Non-const on the attribute, but
// callable through a const handle reference (see Handle::operator-> note in
// bound_api_reference.md). Must be called inside an open command scope.
inline void tdatastd_booleanarray_set_value(
    const TDataStdBooleanArrayHandle& h, Standard_Integer index, bool value)
{
    try {
        h.inner->SetValue(index, value);
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}

// ── TDataStd_ByteArray ────────────────────────────────────────────────────────

struct TDataStdByteArrayHandle {
    Handle(TDataStd_ByteArray) inner;
};

// TDataStd_ByteArray::Set(L, lower, upper) — static. isDelta omitted: OCCT's
// compiled-in default (Standard_False, DefaultDeltaOnModification) applies.
// Finds, or creates, a byte array attribute on L with 0-based bounds
// [0, len-1]. Elements are zero-initialized until set_value is called.
// Must be called inside an open command scope.
inline std::unique_ptr<TDataStdByteArrayHandle> tdatastd_bytearray_set(
    const TdfLabel& label, Standard_Integer len)
{
    try {
        auto result = std::make_unique<TDataStdByteArrayHandle>();
        result->inner = TDataStd_ByteArray::Set(label.inner, 0, len - 1);
        return result;
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}

// Find TDataStd_ByteArray on a label. Returns nullptr if not present.
inline std::unique_ptr<TDataStdByteArrayHandle> tdatastd_bytearray_find(const TdfLabel& label) {
    Handle(TDataStd_ByteArray) attr;
    if (label.inner.FindAttribute(TDataStd_ByteArray::GetID(), attr)) {
        auto result = std::make_unique<TDataStdByteArrayHandle>();
        result->inner = attr;
        return result;
    }
    return nullptr;
}

// TDF_Label::ForgetAttribute(GUID) const — removes the ByteArray attribute
// if present. Returns false if it was not present. No exception path.
inline bool tdatastd_bytearray_forget(const TdfLabel& label) {
    return label.inner.ForgetAttribute(TDataStd_ByteArray::GetID()) == Standard_True;
}

// TDataStd_ByteArray::Length() const — number of elements (== len passed to set).
inline Standard_Integer tdatastd_bytearray_length(const TDataStdByteArrayHandle& h) {
    return h.inner->Length();
}

// TDataStd_ByteArray::Value(index) const — 0-based. Standard_Byte is
// `unsigned char`, same width/representation as cxx's u8 (uint8_t).
// Raises OutOfRange if index is outside [0, Length()-1].
inline Standard_Byte tdatastd_bytearray_value(
    const TDataStdByteArrayHandle& h, Standard_Integer index)
{
    try {
        return h.inner->Value(index);
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}

// TDataStd_ByteArray::SetValue(index, value) — 0-based. Raises OutOfRange if
// index is outside [0, Length()-1]. Non-const on the attribute, but callable
// through a const handle reference (see Handle::operator-> note in
// bound_api_reference.md). Must be called inside an open command scope.
inline void tdatastd_bytearray_set_value(
    const TDataStdByteArrayHandle& h, Standard_Integer index, Standard_Byte value)
{
    try {
        h.inner->SetValue(index, value);
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}

// ── TDataStd_ExtStringArray ───────────────────────────────────────────────────

struct TDataStdExtStringArrayHandle {
    Handle(TDataStd_ExtStringArray) inner;
};

// TDataStd_ExtStringArray::Set(L, lower, upper) — static. isDelta omitted:
// OCCT's compiled-in default (Standard_False, DefaultDeltaOnModification)
// applies. Finds, or creates, an ExtStringArray attribute on L with 0-based
// bounds [0, len-1]. Elements are empty-string-initialized until set_value
// is called. Must be called inside an open command scope.
inline std::unique_ptr<TDataStdExtStringArrayHandle> tdatastd_extstringarray_set(
    const TdfLabel& label, Standard_Integer len)
{
    try {
        auto result = std::make_unique<TDataStdExtStringArrayHandle>();
        result->inner = TDataStd_ExtStringArray::Set(label.inner, 0, len - 1);
        return result;
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}

// Find TDataStd_ExtStringArray on a label. Returns nullptr if not present.
inline std::unique_ptr<TDataStdExtStringArrayHandle> tdatastd_extstringarray_find(const TdfLabel& label) {
    Handle(TDataStd_ExtStringArray) attr;
    if (label.inner.FindAttribute(TDataStd_ExtStringArray::GetID(), attr)) {
        auto result = std::make_unique<TDataStdExtStringArrayHandle>();
        result->inner = attr;
        return result;
    }
    return nullptr;
}

// TDF_Label::ForgetAttribute(GUID) const — removes the ExtStringArray
// attribute if present. Returns false if it was not present. No exception path.
inline bool tdatastd_extstringarray_forget(const TdfLabel& label) {
    return label.inner.ForgetAttribute(TDataStd_ExtStringArray::GetID()) == Standard_True;
}

// TDataStd_ExtStringArray::Length() const — number of elements (== len passed to set).
inline Standard_Integer tdatastd_extstringarray_length(const TDataStdExtStringArrayHandle& h) {
    return h.inner->Length();
}

// TDataStd_ExtStringArray::Value(index) const — 0-based. Same UTF-8
// conversion as tdatastd_name_get/tdatastd_comment_get, applied per element.
// Raises OutOfRange if index is outside [0, Length()-1].
inline rust::String tdatastd_extstringarray_value(
    const TDataStdExtStringArrayHandle& h, Standard_Integer index)
{
    try {
        const TCollection_ExtendedString& ext = h.inner->Value(index);
        Standard_Integer len = ext.LengthOfCString();
        std::string buf(static_cast<size_t>(len) + 1, '\0');
        char* ptr = buf.data();
        ext.ToUTF8CString(ptr);
        return rust::String(buf.c_str());
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}

// TDataStd_ExtStringArray::SetValue(index, value) — 0-based. isMultiByte=true:
// see tdatastd_name_set. Raises OutOfRange if index is outside
// [0, Length()-1]. Must be called inside an open command scope.
inline void tdatastd_extstringarray_set_value(
    const TDataStdExtStringArrayHandle& h, Standard_Integer index, rust::Str value)
{
    try {
        std::string s(value.data(), value.size());
        TCollection_ExtendedString ext(s.c_str(), Standard_True);
        h.inner->SetValue(index, ext);
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}

// ── TDataStd_IntegerList ──────────────────────────────────────────────────────

struct TDataStdIntegerListHandle {
    Handle(TDataStd_IntegerList) inner;
};

// TDataStd_IntegerList::Set(L) — static.
// Finds, or creates, an empty list-of-integers attribute on L.
// Must be called inside an open command scope.
inline std::unique_ptr<TDataStdIntegerListHandle> tdatastd_integerlist_set(const TdfLabel& label)
{
    try {
        auto result = std::make_unique<TDataStdIntegerListHandle>();
        result->inner = TDataStd_IntegerList::Set(label.inner);
        return result;
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}

// Find TDataStd_IntegerList on a label. Returns nullptr if not present.
inline std::unique_ptr<TDataStdIntegerListHandle> tdatastd_integerlist_find(const TdfLabel& label) {
    Handle(TDataStd_IntegerList) attr;
    if (label.inner.FindAttribute(TDataStd_IntegerList::GetID(), attr)) {
        auto result = std::make_unique<TDataStdIntegerListHandle>();
        result->inner = attr;
        return result;
    }
    return nullptr;
}

// TDF_Label::ForgetAttribute(GUID) const — removes the IntegerList attribute
// if present. Returns false if it was not present. No exception path.
inline bool tdatastd_integerlist_forget(const TdfLabel& label) {
    return label.inner.ForgetAttribute(TDataStd_IntegerList::GetID()) == Standard_True;
}

// TDataStd_IntegerList::Extent() const — number of elements.
inline Standard_Integer tdatastd_integerlist_extent(const TDataStdIntegerListHandle& h) {
    return h.inner->Extent();
}

// TDataStd_IntegerList::IsEmpty() const.
inline bool tdatastd_integerlist_is_empty(const TDataStdIntegerListHandle& h) {
    return h.inner->IsEmpty() == Standard_True;
}

// TDataStd_IntegerList::List() const, 0-based walk-and-advance — same pattern
// as TDataStd_ReferenceList::List(). Caller must ensure 0 <= index < Extent().
inline Standard_Integer tdatastd_integerlist_at(
    const TDataStdIntegerListHandle& h, Standard_Integer index)
{
    const TColStd_ListOfInteger& lst = h.inner->List();
    auto it = lst.begin();
    std::advance(it, static_cast<std::ptrdiff_t>(index));
    return *it;
}

// TDataStd_IntegerList::Append(value) — non-const on the attribute, but
// callable through a const handle reference (see Handle::operator-> note in
// bound_api_reference.md). Must be called inside an open command scope.
inline void tdatastd_integerlist_append(const TDataStdIntegerListHandle& h, Standard_Integer value) {
    h.inner->Append(value);
}

// ── TDataStd_RealList ─────────────────────────────────────────────────────────

struct TDataStdRealListHandle {
    Handle(TDataStd_RealList) inner;
};

// TDataStd_RealList::Set(L) — static.
// Finds, or creates, an empty list-of-reals attribute on L.
// Must be called inside an open command scope.
inline std::unique_ptr<TDataStdRealListHandle> tdatastd_reallist_set(const TdfLabel& label)
{
    try {
        auto result = std::make_unique<TDataStdRealListHandle>();
        result->inner = TDataStd_RealList::Set(label.inner);
        return result;
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}

// Find TDataStd_RealList on a label. Returns nullptr if not present.
inline std::unique_ptr<TDataStdRealListHandle> tdatastd_reallist_find(const TdfLabel& label) {
    Handle(TDataStd_RealList) attr;
    if (label.inner.FindAttribute(TDataStd_RealList::GetID(), attr)) {
        auto result = std::make_unique<TDataStdRealListHandle>();
        result->inner = attr;
        return result;
    }
    return nullptr;
}

// TDF_Label::ForgetAttribute(GUID) const — removes the RealList attribute
// if present. Returns false if it was not present. No exception path.
inline bool tdatastd_reallist_forget(const TdfLabel& label) {
    return label.inner.ForgetAttribute(TDataStd_RealList::GetID()) == Standard_True;
}

// TDataStd_RealList::Extent() const — number of elements.
inline Standard_Integer tdatastd_reallist_extent(const TDataStdRealListHandle& h) {
    return h.inner->Extent();
}

// TDataStd_RealList::IsEmpty() const.
inline bool tdatastd_reallist_is_empty(const TDataStdRealListHandle& h) {
    return h.inner->IsEmpty() == Standard_True;
}

// TDataStd_RealList::List() const, 0-based walk-and-advance. Caller must
// ensure 0 <= index < Extent().
inline Standard_Real tdatastd_reallist_at(
    const TDataStdRealListHandle& h, Standard_Integer index)
{
    const TColStd_ListOfReal& lst = h.inner->List();
    auto it = lst.begin();
    std::advance(it, static_cast<std::ptrdiff_t>(index));
    return *it;
}

// TDataStd_RealList::Append(value) — non-const on the attribute, but callable
// through a const handle reference (see Handle::operator-> note in
// bound_api_reference.md). Must be called inside an open command scope.
inline void tdatastd_reallist_append(const TDataStdRealListHandle& h, Standard_Real value) {
    h.inner->Append(value);
}

// ── TDataStd_ExtStringList ────────────────────────────────────────────────────

struct TDataStdExtStringListHandle {
    Handle(TDataStd_ExtStringList) inner;
};

// TDataStd_ExtStringList::Set(L) — static.
// Finds, or creates, an empty list-of-strings attribute on L.
// Must be called inside an open command scope.
inline std::unique_ptr<TDataStdExtStringListHandle> tdatastd_extstringlist_set(const TdfLabel& label)
{
    try {
        auto result = std::make_unique<TDataStdExtStringListHandle>();
        result->inner = TDataStd_ExtStringList::Set(label.inner);
        return result;
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}

// Find TDataStd_ExtStringList on a label. Returns nullptr if not present.
inline std::unique_ptr<TDataStdExtStringListHandle> tdatastd_extstringlist_find(const TdfLabel& label) {
    Handle(TDataStd_ExtStringList) attr;
    if (label.inner.FindAttribute(TDataStd_ExtStringList::GetID(), attr)) {
        auto result = std::make_unique<TDataStdExtStringListHandle>();
        result->inner = attr;
        return result;
    }
    return nullptr;
}

// TDF_Label::ForgetAttribute(GUID) const — removes the ExtStringList
// attribute if present. Returns false if it was not present. No exception path.
inline bool tdatastd_extstringlist_forget(const TdfLabel& label) {
    return label.inner.ForgetAttribute(TDataStd_ExtStringList::GetID()) == Standard_True;
}

// TDataStd_ExtStringList::Extent() const — number of elements.
inline Standard_Integer tdatastd_extstringlist_extent(const TDataStdExtStringListHandle& h) {
    return h.inner->Extent();
}

// TDataStd_ExtStringList::IsEmpty() const.
inline bool tdatastd_extstringlist_is_empty(const TDataStdExtStringListHandle& h) {
    return h.inner->IsEmpty() == Standard_True;
}

// TDataStd_ExtStringList::List() const, 0-based walk-and-advance. Same UTF-8
// conversion as tdatastd_name_get/tdatastd_extstringarray_value, applied per
// element. Caller must ensure 0 <= index < Extent().
inline rust::String tdatastd_extstringlist_at(
    const TDataStdExtStringListHandle& h, Standard_Integer index)
{
    const TDataStd_ListOfExtendedString& lst = h.inner->List();
    auto it = lst.begin();
    std::advance(it, static_cast<std::ptrdiff_t>(index));
    const TCollection_ExtendedString& ext = *it;
    Standard_Integer len = ext.LengthOfCString();
    std::string buf(static_cast<size_t>(len) + 1, '\0');
    char* ptr = buf.data();
    ext.ToUTF8CString(ptr);
    return rust::String(buf.c_str());
}

// TDataStd_ExtStringList::Append(value) — isMultiByte=true: see
// tdatastd_name_set. Non-const on the attribute, but callable through a
// const handle reference (see Handle::operator-> note in
// bound_api_reference.md). Must be called inside an open command scope.
inline void tdatastd_extstringlist_append(const TDataStdExtStringListHandle& h, rust::Str value) {
    std::string s(value.data(), value.size());
    TCollection_ExtendedString ext(s.c_str(), Standard_True);
    h.inner->Append(ext);
}

// ── TDataStd_BooleanList ──────────────────────────────────────────────────────

struct TDataStdBooleanListHandle {
    Handle(TDataStd_BooleanList) inner;
};

// TDataStd_BooleanList::Set(L) — static.
// Finds, or creates, an empty list-of-booleans attribute on L.
// Must be called inside an open command scope.
inline std::unique_ptr<TDataStdBooleanListHandle> tdatastd_booleanlist_set(const TdfLabel& label)
{
    try {
        auto result = std::make_unique<TDataStdBooleanListHandle>();
        result->inner = TDataStd_BooleanList::Set(label.inner);
        return result;
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}

// Find TDataStd_BooleanList on a label. Returns nullptr if not present.
inline std::unique_ptr<TDataStdBooleanListHandle> tdatastd_booleanlist_find(const TdfLabel& label) {
    Handle(TDataStd_BooleanList) attr;
    if (label.inner.FindAttribute(TDataStd_BooleanList::GetID(), attr)) {
        auto result = std::make_unique<TDataStdBooleanListHandle>();
        result->inner = attr;
        return result;
    }
    return nullptr;
}

// TDF_Label::ForgetAttribute(GUID) const — removes the BooleanList attribute
// if present. Returns false if it was not present. No exception path.
inline bool tdatastd_booleanlist_forget(const TdfLabel& label) {
    return label.inner.ForgetAttribute(TDataStd_BooleanList::GetID()) == Standard_True;
}

// TDataStd_BooleanList::Extent() const — number of elements.
inline Standard_Integer tdatastd_booleanlist_extent(const TDataStdBooleanListHandle& h) {
    return h.inner->Extent();
}

// TDataStd_BooleanList::IsEmpty() const.
inline bool tdatastd_booleanlist_is_empty(const TDataStdBooleanListHandle& h) {
    return h.inner->IsEmpty() == Standard_True;
}

// TDataStd_BooleanList::List() const, 0-based walk-and-advance. Underlying
// storage is TDataStd_ListOfByte (NCollection_List<Standard_Byte>), 1=TRUE
// / 0=FALSE per the OCCT header's documented convention — converted to bool
// here. Caller must ensure 0 <= index < Extent().
inline bool tdatastd_booleanlist_at(
    const TDataStdBooleanListHandle& h, Standard_Integer index)
{
    const TDataStd_ListOfByte& lst = h.inner->List();
    auto it = lst.begin();
    std::advance(it, static_cast<std::ptrdiff_t>(index));
    return *it != 0;
}

// TDataStd_BooleanList::Append(value) — Standard_Boolean is bool, no
// conversion needed for write. Non-const on the attribute, but callable
// through a const handle reference (see Handle::operator-> note in
// bound_api_reference.md). Must be called inside an open command scope.
inline void tdatastd_booleanlist_append(const TDataStdBooleanListHandle& h, bool value) {
    h.inner->Append(value);
}

// ── TDataStd_UAttribute ───────────────────────────────────────────────────────

// Constructs a Standard_GUID from its 10 canonical fields (4-2-2-2-6 UUID
// grouping, matching Standard_GUID's scalar constructor). Pure value
// construction, no OCCT state — called immediately before
// Set/FindAttribute/ForgetAttribute, never stored. Generic infrastructure;
// UAttribute is its first consumer but not its only plausible one — could be
// relocated to a shared header if a second consumer appears.
inline Standard_GUID make_guid(
    uint32_t a32b, uint16_t a16b1, uint16_t a16b2, uint16_t a16b3,
    uint8_t a8b1, uint8_t a8b2, uint8_t a8b3, uint8_t a8b4, uint8_t a8b5, uint8_t a8b6)
{
    return Standard_GUID(
        static_cast<Standard_Integer>(a32b),
        static_cast<Standard_ExtCharacter>(a16b1),
        static_cast<Standard_ExtCharacter>(a16b2),
        static_cast<Standard_ExtCharacter>(a16b3),
        static_cast<Standard_Byte>(a8b1), static_cast<Standard_Byte>(a8b2),
        static_cast<Standard_Byte>(a8b3), static_cast<Standard_Byte>(a8b4),
        static_cast<Standard_Byte>(a8b5), static_cast<Standard_Byte>(a8b6));
}

// TDataStd_UAttribute::Set(L, guid) — static. Finds, or creates, a
// presence-only marker attribute identified by guid on L. No value to
// retrieve — presence is the entire state. Must be called inside an open
// command scope.
inline void tdatastd_uattribute_set(
    const TdfLabel& label,
    uint32_t a32b, uint16_t a16b1, uint16_t a16b2, uint16_t a16b3,
    uint8_t a8b1, uint8_t a8b2, uint8_t a8b3, uint8_t a8b4, uint8_t a8b5, uint8_t a8b6)
{
    try {
        TDataStd_UAttribute::Set(label.inner, make_guid(a32b, a16b1, a16b2, a16b3, a8b1, a8b2, a8b3, a8b4, a8b5, a8b6));
    } catch (const std::runtime_error&) { throw; }
    catch (...) { rethrow_occt_as_runtime_error(); }
}

// TDF_Label::FindAttribute(guid, attr) const — true if a UAttribute marker
// with this guid is present. The found handle is discarded; only presence
// matters. No exception path, no command scope required.
inline bool tdatastd_uattribute_is_present(
    const TdfLabel& label,
    uint32_t a32b, uint16_t a16b1, uint16_t a16b2, uint16_t a16b3,
    uint8_t a8b1, uint8_t a8b2, uint8_t a8b3, uint8_t a8b4, uint8_t a8b5, uint8_t a8b6)
{
    Handle(TDataStd_UAttribute) attr;
    return label.inner.FindAttribute(make_guid(a32b, a16b1, a16b2, a16b3, a8b1, a8b2, a8b3, a8b4, a8b5, a8b6), attr) == Standard_True;
}

// TDF_Label::ForgetAttribute(guid) const — removes the UAttribute marker with
// this guid if present. Returns false if it was not present. No exception
// path. Must be called inside an open command scope.
inline bool tdatastd_uattribute_forget(
    const TdfLabel& label,
    uint32_t a32b, uint16_t a16b1, uint16_t a16b2, uint16_t a16b3,
    uint8_t a8b1, uint8_t a8b2, uint8_t a8b3, uint8_t a8b4, uint8_t a8b5, uint8_t a8b6)
{
    return label.inner.ForgetAttribute(make_guid(a32b, a16b1, a16b2, a16b3, a8b1, a8b2, a8b3, a8b4, a8b5, a8b6)) == Standard_True;
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
