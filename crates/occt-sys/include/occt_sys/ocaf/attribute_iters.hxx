// ── List iterators ────────────────────────────────────────────────────────────
// Cursor-based O(n) iteration over NCollection_List<T> attributes.
// Each type gets an opaque iterator struct + new/more/next/value shims.
// Reference: https://dev.opencascade.org/doc/refman/html/class_n_collection___list.html
// No derivation from any other binding crate.

#include <TDataStd_IntegerList.hxx>
#include <TDataStd_RealList.hxx>
#include <TDataStd_ExtStringList.hxx>
#include <TDataStd_BooleanList.hxx>
#include <TDataStd_ReferenceList.hxx>

// ── List attribute cursor iterators ──────────────────────────────────────────
//
// Cursor-based O(n) iteration over NCollection_List<T> attribute storage.
// Each struct owns a copy of the list iterator, initialised from List() at
// construction. The cursor field is named distinctly from the handle shims'
// `inner` (which holds a Handle(T)) to avoid shadowing confusion.
//
// Reference: https://dev.opencascade.org/doc/refman/html/class_n_collection___list.html
// Sourced from OCCT 7.9 documentation.
// No derivation from any other binding crate.

// ── OcIntegerListIter ─────────────────────────────────────────────────────────

struct OcIntegerListIter {
    TColStd_ListOfInteger::iterator cursor;
    TColStd_ListOfInteger::iterator end;
    OcIntegerListIter(const TDataStdIntegerListHandle& h)
        : cursor(h.inner->List().begin()), end(h.inner->List().end()) {}
};

inline std::unique_ptr<OcIntegerListIter>
tdatastd_integerlist_iter_new(const TDataStdIntegerListHandle& h) {
    return std::make_unique<OcIntegerListIter>(h);
}
inline bool tdatastd_integerlist_iter_more(const OcIntegerListIter& it) {
    return it.cursor != it.end;
}
inline void tdatastd_integerlist_iter_next(OcIntegerListIter& it) {
    ++it.cursor;
}
inline Standard_Integer tdatastd_integerlist_iter_value(const OcIntegerListIter& it) {
    return *it.cursor;
}

// ── OcRealListIter ────────────────────────────────────────────────────────────

struct OcRealListIter {
    TColStd_ListOfReal::iterator cursor;
    TColStd_ListOfReal::iterator end;
    OcRealListIter(const TDataStdRealListHandle& h)
        : cursor(h.inner->List().begin()), end(h.inner->List().end()) {}
};

inline std::unique_ptr<OcRealListIter>
tdatastd_reallist_iter_new(const TDataStdRealListHandle& h) {
    return std::make_unique<OcRealListIter>(h);
}
inline bool tdatastd_reallist_iter_more(const OcRealListIter& it) {
    return it.cursor != it.end;
}
inline void tdatastd_reallist_iter_next(OcRealListIter& it) {
    ++it.cursor;
}
inline Standard_Real tdatastd_reallist_iter_value(const OcRealListIter& it) {
    return *it.cursor;
}

// ── OcExtStringListIter ───────────────────────────────────────────────────────

struct OcExtStringListIter {
    TDataStd_ListOfExtendedString::iterator cursor;
    TDataStd_ListOfExtendedString::iterator end;
    OcExtStringListIter(const TDataStdExtStringListHandle& h)
        : cursor(h.inner->List().begin()), end(h.inner->List().end()) {}
};

inline std::unique_ptr<OcExtStringListIter>
tdatastd_extstringlist_iter_new(const TDataStdExtStringListHandle& h) {
    return std::make_unique<OcExtStringListIter>(h);
}
inline bool tdatastd_extstringlist_iter_more(const OcExtStringListIter& it) {
    return it.cursor != it.end;
}
inline void tdatastd_extstringlist_iter_next(OcExtStringListIter& it) {
    ++it.cursor;
}
inline rust::String tdatastd_extstringlist_iter_value(const OcExtStringListIter& it) {
    return ext_string_to_rust(*it.cursor);
}

// ── OcBooleanListIter ─────────────────────────────────────────────────────────
// TDataStd_ListOfByte is NCollection_List<Standard_Byte>; 1=true/0=false.

struct OcBooleanListIter {
    TDataStd_ListOfByte::iterator cursor;
    TDataStd_ListOfByte::iterator end;
    OcBooleanListIter(const TDataStdBooleanListHandle& h)
        : cursor(h.inner->List().begin()), end(h.inner->List().end()) {}
};

inline std::unique_ptr<OcBooleanListIter>
tdatastd_booleanlist_iter_new(const TDataStdBooleanListHandle& h) {
    return std::make_unique<OcBooleanListIter>(h);
}
inline bool tdatastd_booleanlist_iter_more(const OcBooleanListIter& it) {
    return it.cursor != it.end;
}
inline void tdatastd_booleanlist_iter_next(OcBooleanListIter& it) {
    ++it.cursor;
}
inline bool tdatastd_booleanlist_iter_value(const OcBooleanListIter& it) {
    return *it.cursor != 0;
}

// ── OcReferenceListIter ───────────────────────────────────────────────────────
// TDF_LabelList is NCollection_List<TDF_Label>.

struct OcReferenceListIter {
    TDF_LabelList::iterator cursor;
    TDF_LabelList::iterator end;
    OcReferenceListIter(const TDataStdReferenceListHandle& h)
        : cursor(h.inner->List().begin()), end(h.inner->List().end()) {}
};

inline std::unique_ptr<OcReferenceListIter>
tdatastd_referencelist_iter_new(const TDataStdReferenceListHandle& h) {
    return std::make_unique<OcReferenceListIter>(h);
}
inline bool tdatastd_referencelist_iter_more(const OcReferenceListIter& it) {
    return it.cursor != it.end;
}
inline void tdatastd_referencelist_iter_next(OcReferenceListIter& it) {
    ++it.cursor;
}
inline std::unique_ptr<TdfLabel> tdatastd_referencelist_iter_value(const OcReferenceListIter& it) {
    return std::make_unique<TdfLabel>(TdfLabel{*it.cursor});
}
