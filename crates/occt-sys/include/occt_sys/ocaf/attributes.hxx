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
#include <TDataStd_Real.hxx>
#include <TDF_Label.hxx>

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
        TCollection_ExtendedString ext(s.c_str());
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
