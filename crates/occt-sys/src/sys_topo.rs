//! cxx bridge for OCCT topological shape builders and inspectors.
//!
//! The topo types form a dependency chain (Vertex → Edge → Wire → Face →
//! Solid), so all are declared in a single bridge to avoid cross-bridge
//! `ExternType` forwarding boilerplate.
//!
//! # Builder lifetime
//!
//! Builder types that expose `Modified()`, `Generated()`, or `IsDeleted()`
//! (currently `MakePrismBuilder`) must not be dropped before shape history
//! queries are complete.
//!
//! Generated using LLMs from information in:
//!   - OCCT 7.9 reference: <https://dev.opencascade.org/doc/refman/html/>
//!   - cxx docs: <https://cxx.rs/>
//!
//! No derivation from any other binding crate.

#[allow(clippy::too_many_arguments)]
#[cxx::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("occt_sys/topo.hxx");
        // ---------------------------------------------------------------------------
        // TNaming — topological naming attribute (DOC-1)
        // Reference: https://dev.opencascade.org/doc/refman/html/class_t_naming___builder.html
        // Reference: https://dev.opencascade.org/doc/refman/html/class_t_naming___named_shape.html
        // ---------------------------------------------------------------------------

        type TnamingNamedShapeHandle;
        fn is_null(self: &TnamingNamedShapeHandle) -> bool;

        // Builder — must be used inside an open Command
        type TnamingBuilderShim;
        fn new_tnaming_builder(label: &TdfLabel) -> UniquePtr<TnamingBuilderShim>;
        fn generated_fresh(self: Pin<&mut TnamingBuilderShim>, s: &TopodsShape);
        fn generated_from(
            self: Pin<&mut TnamingBuilderShim>,
            old_s: &TopodsShape,
            new_s: &TopodsShape,
        );
        fn modify(self: Pin<&mut TnamingBuilderShim>, old_s: &TopodsShape, new_s: &TopodsShape);
        fn delete_shape(self: Pin<&mut TnamingBuilderShim>, old_s: &TopodsShape);
        fn select(self: Pin<&mut TnamingBuilderShim>, s: &TopodsShape, in_s: &TopodsShape);
        fn named_shape(self: &TnamingBuilderShim) -> UniquePtr<TnamingNamedShapeHandle>;

        // NamedShape read-side
        fn find_tnaming_named_shape(lw: &TdfLabel, out: Pin<&mut TnamingNamedShapeHandle>) -> bool;
        fn tnaming_named_shape_get(h: &TnamingNamedShapeHandle) -> UniquePtr<TopodsShape>;
        fn tnaming_named_shape_evolution(h: &TnamingNamedShapeHandle) -> i32;
        fn tnaming_tool_original_shape(h: &TnamingNamedShapeHandle) -> UniquePtr<TopodsShape>;
        fn new_tnaming_named_shape_handle() -> UniquePtr<TnamingNamedShapeHandle>;

        type TnamingSelectorShim;

        fn new_tnaming_selector(label: &TdfLabel) -> UniquePtr<TnamingSelectorShim>;
        // sel is Pin<&mut> because Select/Solve are non-const
        fn tnaming_selector_select(
            sel: Pin<&mut TnamingSelectorShim>,
            shape: &TopodsShape,
            context: &TopodsShape,
        ) -> bool;
        fn tnaming_selector_solve(sel: Pin<&mut TnamingSelectorShim>) -> bool;
        // NamedShape is const — plain &
        fn tnaming_selector_named_shape(
            sel: &TnamingSelectorShim,
            out: Pin<&mut TnamingNamedShapeHandle>,
        ) -> bool;
        // ── TDataStdNameHandle ────────────────────────────────────────────────────
        // Shim holding Handle(TDataStd_Name) by value.
        //
        // Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_std___name.html
        type TDataStdNameHandle;

        // Set: static on TDataStd_Name; attaches or updates the attribute on label.
        // Must be called inside an open command scope.
        fn tdatastd_name_set(
            label: &TdfLabel,
            value: &str,
        ) -> Result<UniquePtr<TDataStdNameHandle>>;
        // Get: const — reads the string value as UTF-8.
        fn tdatastd_name_get(h: &TDataStdNameHandle) -> String;
        // Find: returns nullptr (None on Rust side) when attribute is absent.
        fn tdatastd_name_find(label: &TdfLabel) -> UniquePtr<TDataStdNameHandle>;
        // ForgetAttribute(GUID) const — true if present and removed.
        fn tdatastd_name_forget(label: &TdfLabel) -> bool;

        // ── TDataStdCommentHandle ──────────────────────────────────────────────────
        // Shim holding Handle(TDataStd_Comment) by value.
        //
        // Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_std___comment.html
        type TDataStdCommentHandle;

        // Set: static on TDataStd_Comment; attaches or updates the attribute on label.
        // Must be called inside an open command scope.
        fn tdatastd_comment_set(
            label: &TdfLabel,
            value: &str,
        ) -> Result<UniquePtr<TDataStdCommentHandle>>;
        // Get: const — reads the string value as UTF-8.
        fn tdatastd_comment_get(h: &TDataStdCommentHandle) -> String;
        // Find: returns nullptr (None on Rust side) when attribute is absent.
        fn tdatastd_comment_find(label: &TdfLabel) -> UniquePtr<TDataStdCommentHandle>;
        // ForgetAttribute(GUID) const — true if present and removed.
        fn tdatastd_comment_forget(label: &TdfLabel) -> bool;
        // ── TDataStdIntegerHandle ─────────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_std___integer.html
        type TDataStdIntegerHandle;

        fn tdatastd_integer_set(
            label: &TdfLabel,
            value: i32,
        ) -> Result<UniquePtr<TDataStdIntegerHandle>>;
        fn tdatastd_integer_get(h: &TDataStdIntegerHandle) -> i32;
        fn tdatastd_integer_find(label: &TdfLabel) -> UniquePtr<TDataStdIntegerHandle>;
        fn tdatastd_integer_forget(label: &TdfLabel) -> bool;

        // ── TDataStdRealHandle ────────────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_std___real.html
        type TDataStdRealHandle;

        fn tdatastd_real_set(label: &TdfLabel, value: f64)
            -> Result<UniquePtr<TDataStdRealHandle>>;
        fn tdatastd_real_get(h: &TDataStdRealHandle) -> f64;
        fn tdatastd_real_find(label: &TdfLabel) -> UniquePtr<TDataStdRealHandle>;
        fn tdatastd_real_forget(label: &TdfLabel) -> bool;
        // ── TDataStdAsciiStringHandle ────────────────────────────────────────────────
        // Shim holding Handle(TDataStd_AsciiString) by value.
        //
        // Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_std___ascii_string.html
        type TDataStdAsciiStringHandle;

        // Set: static on TDataStd_AsciiString; attaches or updates the attribute on
        // label. TCollection_AsciiString is an 8-bit char buffer with no ASCII
        // validation, so any valid-UTF-8 &str round-trips unchanged. Must be called
        // inside an open command scope.
        fn tdatastd_asciistring_set(
            label: &TdfLabel,
            value: &str,
        ) -> Result<UniquePtr<TDataStdAsciiStringHandle>>;
        // Get: const — reads the ASCII string value (pure ASCII, valid UTF-8 unchanged).
        fn tdatastd_asciistring_get(h: &TDataStdAsciiStringHandle) -> String;
        // Find: returns nullptr (None on Rust side) when attribute is absent.
        fn tdatastd_asciistring_find(label: &TdfLabel) -> UniquePtr<TDataStdAsciiStringHandle>;
        // ForgetAttribute(GUID) const — true if present and removed.
        fn tdatastd_asciistring_forget(label: &TdfLabel) -> bool;

        // ── TDataStdReferenceListHandle ──────────────────────────────────────────────
        // Shim holding Handle(TDataStd_ReferenceList) by value.
        //
        // Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_std___reference_list.html
        type TDataStdReferenceListHandle;

        // Set: static on TDataStd_ReferenceList; finds or creates an empty
        // list-of-references attribute on label. Must be called inside an open
        // command scope.
        fn tdatastd_referencelist_set(
            label: &TdfLabel,
        ) -> Result<UniquePtr<TDataStdReferenceListHandle>>;
        // Find: returns nullptr (None on Rust side) when attribute is absent.
        fn tdatastd_referencelist_find(label: &TdfLabel) -> UniquePtr<TDataStdReferenceListHandle>;
        // ForgetAttribute(GUID) const — true if present and removed.
        fn tdatastd_referencelist_forget(label: &TdfLabel) -> bool;
        // Extent: const — number of label references.
        fn tdatastd_referencelist_extent(h: &TDataStdReferenceListHandle) -> i32;
        // IsEmpty: const.
        fn tdatastd_referencelist_is_empty(h: &TDataStdReferenceListHandle) -> bool;
        // At: 0-based walk-and-advance indexed access. Caller must ensure
        // 0 <= index < extent.
        fn tdatastd_referencelist_at(
            h: &TDataStdReferenceListHandle,
            index: i32,
        ) -> UniquePtr<TdfLabel>;
        // Append: non-const on the attribute, but callable through a const handle
        // reference (see shim comment). Must be called inside an open command scope.
        fn tdatastd_referencelist_append(h: &TDataStdReferenceListHandle, value: &TdfLabel);

        // ── TDataStdIntegerListHandle ─────────────────────────────────────────────────
        // Shim holding Handle(TDataStd_IntegerList) by value.
        //
        // Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_std___integer_list.html
        type TDataStdIntegerListHandle;

        fn tdatastd_integerlist_set(
            label: &TdfLabel,
        ) -> Result<UniquePtr<TDataStdIntegerListHandle>>;
        fn tdatastd_integerlist_find(label: &TdfLabel) -> UniquePtr<TDataStdIntegerListHandle>;
        fn tdatastd_integerlist_forget(label: &TdfLabel) -> bool;
        fn tdatastd_integerlist_extent(h: &TDataStdIntegerListHandle) -> i32;
        fn tdatastd_integerlist_is_empty(h: &TDataStdIntegerListHandle) -> bool;
        // At: 0-based walk-and-advance. Caller must ensure 0 <= index < extent.
        fn tdatastd_integerlist_at(h: &TDataStdIntegerListHandle, index: i32) -> i32;
        fn tdatastd_integerlist_append(h: &TDataStdIntegerListHandle, value: i32);

        // ── TDataStdRealListHandle ────────────────────────────────────────────────────
        // Shim holding Handle(TDataStd_RealList) by value.
        //
        // Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_std___real_list.html
        type TDataStdRealListHandle;

        fn tdatastd_reallist_set(label: &TdfLabel) -> Result<UniquePtr<TDataStdRealListHandle>>;
        fn tdatastd_reallist_find(label: &TdfLabel) -> UniquePtr<TDataStdRealListHandle>;
        fn tdatastd_reallist_forget(label: &TdfLabel) -> bool;
        fn tdatastd_reallist_extent(h: &TDataStdRealListHandle) -> i32;
        fn tdatastd_reallist_is_empty(h: &TDataStdRealListHandle) -> bool;
        // At: 0-based walk-and-advance. Caller must ensure 0 <= index < extent.
        fn tdatastd_reallist_at(h: &TDataStdRealListHandle, index: i32) -> f64;
        fn tdatastd_reallist_append(h: &TDataStdRealListHandle, value: f64);

        // ── TDataStdExtStringListHandle ───────────────────────────────────────────────
        // Shim holding Handle(TDataStd_ExtStringList) by value.
        //
        // Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_std___ext_string_list.html
        type TDataStdExtStringListHandle;

        fn tdatastd_extstringlist_set(
            label: &TdfLabel,
        ) -> Result<UniquePtr<TDataStdExtStringListHandle>>;
        fn tdatastd_extstringlist_find(label: &TdfLabel) -> UniquePtr<TDataStdExtStringListHandle>;
        fn tdatastd_extstringlist_forget(label: &TdfLabel) -> bool;
        fn tdatastd_extstringlist_extent(h: &TDataStdExtStringListHandle) -> i32;
        fn tdatastd_extstringlist_is_empty(h: &TDataStdExtStringListHandle) -> bool;
        // At: 0-based walk-and-advance, UTF-8 per element (same conversion as
        // tdatastd_name_get). Caller must ensure 0 <= index < extent.
        fn tdatastd_extstringlist_at(h: &TDataStdExtStringListHandle, index: i32) -> String;
        // Append: isMultiByte=true UTF-8 decode, same as tdatastd_name_set, per element.
        fn tdatastd_extstringlist_append(h: &TDataStdExtStringListHandle, value: &str);

        // ── TDataStdBooleanListHandle ─────────────────────────────────────────────────
        // Shim holding Handle(TDataStd_BooleanList) by value.
        //
        // Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_std___boolean_list.html
        type TDataStdBooleanListHandle;

        fn tdatastd_booleanlist_set(
            label: &TdfLabel,
        ) -> Result<UniquePtr<TDataStdBooleanListHandle>>;
        fn tdatastd_booleanlist_find(label: &TdfLabel) -> UniquePtr<TDataStdBooleanListHandle>;
        fn tdatastd_booleanlist_forget(label: &TdfLabel) -> bool;
        fn tdatastd_booleanlist_extent(h: &TDataStdBooleanListHandle) -> i32;
        fn tdatastd_booleanlist_is_empty(h: &TDataStdBooleanListHandle) -> bool;
        // At: 0-based walk-and-advance over underlying ListOfByte (1=true/0=false).
        // Caller must ensure 0 <= index < extent.
        fn tdatastd_booleanlist_at(h: &TDataStdBooleanListHandle, index: i32) -> bool;
        fn tdatastd_booleanlist_append(h: &TDataStdBooleanListHandle, value: bool);

        // ── List attribute cursor iterators ───────────────────────────────────
        // Cursor-based O(n) iteration over NCollection_List<T> attribute storage.
        // Each iter type owns a begin/end iterator pair initialised from List().
        // more() = cursor != end; next() = ++cursor; value() = *cursor.
        //
        // Reference: https://dev.opencascade.org/doc/refman/html/class_n_collection___list.html

        // OcIntegerListIter
        type OcIntegerListIter;
        fn tdatastd_integerlist_iter_new(
            h: &TDataStdIntegerListHandle,
        ) -> UniquePtr<OcIntegerListIter>;
        fn tdatastd_integerlist_iter_more(it: &OcIntegerListIter) -> bool;
        fn tdatastd_integerlist_iter_next(it: Pin<&mut OcIntegerListIter>);
        fn tdatastd_integerlist_iter_value(it: &OcIntegerListIter) -> i32;

        // OcRealListIter
        type OcRealListIter;
        fn tdatastd_reallist_iter_new(h: &TDataStdRealListHandle) -> UniquePtr<OcRealListIter>;
        fn tdatastd_reallist_iter_more(it: &OcRealListIter) -> bool;
        fn tdatastd_reallist_iter_next(it: Pin<&mut OcRealListIter>);
        fn tdatastd_reallist_iter_value(it: &OcRealListIter) -> f64;

        // OcExtStringListIter
        type OcExtStringListIter;
        fn tdatastd_extstringlist_iter_new(
            h: &TDataStdExtStringListHandle,
        ) -> UniquePtr<OcExtStringListIter>;
        fn tdatastd_extstringlist_iter_more(it: &OcExtStringListIter) -> bool;
        fn tdatastd_extstringlist_iter_next(it: Pin<&mut OcExtStringListIter>);
        fn tdatastd_extstringlist_iter_value(it: &OcExtStringListIter) -> String;

        // OcBooleanListIter
        type OcBooleanListIter;
        fn tdatastd_booleanlist_iter_new(
            h: &TDataStdBooleanListHandle,
        ) -> UniquePtr<OcBooleanListIter>;
        fn tdatastd_booleanlist_iter_more(it: &OcBooleanListIter) -> bool;
        fn tdatastd_booleanlist_iter_next(it: Pin<&mut OcBooleanListIter>);
        fn tdatastd_booleanlist_iter_value(it: &OcBooleanListIter) -> bool;

        // OcReferenceListIter
        type OcReferenceListIter;
        fn tdatastd_referencelist_iter_new(
            h: &TDataStdReferenceListHandle,
        ) -> UniquePtr<OcReferenceListIter>;
        fn tdatastd_referencelist_iter_more(it: &OcReferenceListIter) -> bool;
        fn tdatastd_referencelist_iter_next(it: Pin<&mut OcReferenceListIter>);
        fn tdatastd_referencelist_iter_value(it: &OcReferenceListIter) -> UniquePtr<TdfLabel>;
        // ── ShapeListIter ─────────────────────────────────────────────────────────
        // Cursor over a TopTools_ListOfShape snapshot from Modified()/Generated().
        // Snapshots the list at construction; subsequent calls do not touch the builder.
        //
        // Reference: https://dev.opencascade.org/doc/refman/html/class_top_tools___list_of_shape.html
        type ShapeListIter;

        fn shape_list_iter_more(it: &ShapeListIter) -> bool;
        fn shape_list_iter_next(it: Pin<&mut ShapeListIter>);
        fn shape_list_iter_value(it: &ShapeListIter) -> UniquePtr<TopodsShape>;

        // modified_iter / generated_iter — one pair per builder
        fn fillet_modified_iter(
            b: Pin<&mut MakeFilletBuilder>,
            s: &TopodsShape,
        ) -> UniquePtr<ShapeListIter>;
        fn fillet_generated_iter(
            b: Pin<&mut MakeFilletBuilder>,
            s: &TopodsShape,
        ) -> UniquePtr<ShapeListIter>;

        fn chamfer_modified_iter(
            b: Pin<&mut MakeChamferBuilder>,
            s: &TopodsShape,
        ) -> UniquePtr<ShapeListIter>;
        fn chamfer_generated_iter(
            b: Pin<&mut MakeChamferBuilder>,
            s: &TopodsShape,
        ) -> UniquePtr<ShapeListIter>;

        fn offset_shape_modified_iter(
            b: Pin<&mut MakeOffsetShapeBuilder>,
            s: &TopodsShape,
        ) -> UniquePtr<ShapeListIter>;
        fn offset_shape_generated_iter(
            b: Pin<&mut MakeOffsetShapeBuilder>,
            s: &TopodsShape,
        ) -> UniquePtr<ShapeListIter>;

        fn thick_solid_modified_iter(
            b: Pin<&mut MakeThickSolidBuilder>,
            s: &TopodsShape,
        ) -> UniquePtr<ShapeListIter>;
        fn thick_solid_generated_iter(
            b: Pin<&mut MakeThickSolidBuilder>,
            s: &TopodsShape,
        ) -> UniquePtr<ShapeListIter>;

        // ── TDataStd_UAttribute ────────────────────────────────────────────────────────
        // Presence-only marker attribute, keyed by a caller-supplied GUID rather than
        // a fixed per-type GetID(). No value to retrieve.
        //
        // GUID passed as its 10 canonical fields (4-2-2-2-6 UUID grouping);
        // materialized via Standard_GUID's scalar constructor inside the shim.
        //
        // Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_std___u_attribute.html

        // Set: finds or creates a UAttribute marker with the given GUID. Must be
        // called inside an open command scope.
        fn tdatastd_uattribute_set(
            label: &TdfLabel,
            a32b: u32,
            a16b1: u16,
            a16b2: u16,
            a16b3: u16,
            a8b1: u8,
            a8b2: u8,
            a8b3: u8,
            a8b4: u8,
            a8b5: u8,
            a8b6: u8,
        ) -> Result<()>;
        // IsPresent: true if a UAttribute with this GUID is attached to label.
        // No command scope required.
        fn tdatastd_uattribute_is_present(
            label: &TdfLabel,
            a32b: u32,
            a16b1: u16,
            a16b2: u16,
            a16b3: u16,
            a8b1: u8,
            a8b2: u8,
            a8b3: u8,
            a8b4: u8,
            a8b5: u8,
            a8b6: u8,
        ) -> bool;
        // ForgetAttribute(guid) const — true if present and removed.
        fn tdatastd_uattribute_forget(
            label: &TdfLabel,
            a32b: u32,
            a16b1: u16,
            a16b2: u16,
            a16b3: u16,
            a8b1: u8,
            a8b2: u8,
            a8b3: u8,
            a8b4: u8,
            a8b5: u8,
            a8b6: u8,
        ) -> bool;

        // ── TDataStdNamedDataHandle ───────────────────────────────────────────────────
        // Shim holding Handle(TDataStd_NamedData) by value. Scalar-valued groups only
        // (Integer/Real/String/Byte) — see attributes.hxx for what's deferred.
        //
        // Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_std___named_data.html
        type TDataStdNamedDataHandle;

        fn tdatastd_nameddata_set(label: &TdfLabel) -> Result<UniquePtr<TDataStdNamedDataHandle>>;
        fn tdatastd_nameddata_find(label: &TdfLabel) -> UniquePtr<TDataStdNamedDataHandle>;
        fn tdatastd_nameddata_forget(label: &TdfLabel) -> bool;

        // Integers — get returns 0 if name absent; use has_integer to check first.
        fn tdatastd_nameddata_has_integers(h: &TDataStdNamedDataHandle) -> bool;
        fn tdatastd_nameddata_has_integer(h: &TDataStdNamedDataHandle, name: &str) -> bool;
        fn tdatastd_nameddata_get_integer(h: &TDataStdNamedDataHandle, name: &str) -> i32;
        fn tdatastd_nameddata_set_integer(h: &TDataStdNamedDataHandle, name: &str, value: i32);

        // Reals — get returns 0.0 if name absent; use has_real to check first.
        fn tdatastd_nameddata_has_reals(h: &TDataStdNamedDataHandle) -> bool;
        fn tdatastd_nameddata_has_real(h: &TDataStdNamedDataHandle, name: &str) -> bool;
        fn tdatastd_nameddata_get_real(h: &TDataStdNamedDataHandle, name: &str) -> f64;
        fn tdatastd_nameddata_set_real(h: &TDataStdNamedDataHandle, name: &str, value: f64);

        // Strings — get returns "" if name absent; use has_string to check first.
        // isMultiByte=true UTF-8 conversion on both key and value.
        fn tdatastd_nameddata_has_strings(h: &TDataStdNamedDataHandle) -> bool;
        fn tdatastd_nameddata_has_string(h: &TDataStdNamedDataHandle, name: &str) -> bool;
        fn tdatastd_nameddata_get_string(h: &TDataStdNamedDataHandle, name: &str) -> String;
        fn tdatastd_nameddata_set_string(h: &TDataStdNamedDataHandle, name: &str, value: &str);

        // Bytes — get returns 0 if name absent; use has_byte to check first.
        fn tdatastd_nameddata_has_bytes(h: &TDataStdNamedDataHandle) -> bool;
        fn tdatastd_nameddata_has_byte(h: &TDataStdNamedDataHandle, name: &str) -> bool;
        fn tdatastd_nameddata_get_byte(h: &TDataStdNamedDataHandle, name: &str) -> u8;
        fn tdatastd_nameddata_set_byte(h: &TDataStdNamedDataHandle, name: &str, value: u8);

        // ── TDataStdReferenceArrayHandle ─────────────────────────────────────────────
        // Shim holding Handle(TDataStd_ReferenceArray) by value.
        //
        // Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_std___reference_array.html
        type TDataStdReferenceArrayHandle;

        // Set: static on TDataStd_ReferenceArray; finds or creates a reference array
        // attribute on label with 0-based bounds [0, len-1]. Elements are
        // default-initialized (null labels) until set_value is called. Must be
        // called inside an open command scope.
        fn tdatastd_referencearray_set(
            label: &TdfLabel,
            len: i32,
        ) -> Result<UniquePtr<TDataStdReferenceArrayHandle>>;
        // Find: returns nullptr (None on Rust side) when attribute is absent.
        fn tdatastd_referencearray_find(
            label: &TdfLabel,
        ) -> UniquePtr<TDataStdReferenceArrayHandle>;
        // ForgetAttribute(GUID) const — true if present and removed.
        fn tdatastd_referencearray_forget(label: &TdfLabel) -> bool;
        // Length: const — number of elements (== len passed to set).
        fn tdatastd_referencearray_length(h: &TDataStdReferenceArrayHandle) -> i32;
        // Value: 0-based. Raises OutOfRange (-> Err) if index is outside [0, length-1].
        fn tdatastd_referencearray_value(
            h: &TDataStdReferenceArrayHandle,
            index: i32,
        ) -> Result<UniquePtr<TdfLabel>>;
        // SetValue: 0-based. Raises OutOfRange (-> Err) if index is outside [0, length-1].
        // Must be called inside an open command scope.
        fn tdatastd_referencearray_set_value(
            h: &TDataStdReferenceArrayHandle,
            index: i32,
            value: &TdfLabel,
        ) -> Result<()>;

        // ── TDataStdRealArrayHandle ───────────────────────────────────────────────────
        // Shim holding Handle(TDataStd_RealArray) by value.
        //
        // Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_std___real_array.html
        type TDataStdRealArrayHandle;

        // Set: static on TDataStd_RealArray; finds or creates a real array attribute
        // on label with 0-based bounds [0, len-1]. isDelta omitted (OCCT default
        // Standard_False applies). Must be called inside an open command scope.
        fn tdatastd_realarray_set(
            label: &TdfLabel,
            len: i32,
        ) -> Result<UniquePtr<TDataStdRealArrayHandle>>;
        // Find: returns nullptr (None on Rust side) when attribute is absent.
        fn tdatastd_realarray_find(label: &TdfLabel) -> UniquePtr<TDataStdRealArrayHandle>;
        // ForgetAttribute(GUID) const — true if present and removed.
        fn tdatastd_realarray_forget(label: &TdfLabel) -> bool;
        // Length: const — number of elements (== len passed to set).
        fn tdatastd_realarray_length(h: &TDataStdRealArrayHandle) -> i32;
        // Value: 0-based. Raises OutOfRange (-> Err) if index is outside [0, length-1].
        fn tdatastd_realarray_value(h: &TDataStdRealArrayHandle, index: i32) -> Result<f64>;
        // SetValue: 0-based. Raises OutOfRange (-> Err) if index is outside [0, length-1].
        // Must be called inside an open command scope.
        fn tdatastd_realarray_set_value(
            h: &TDataStdRealArrayHandle,
            index: i32,
            value: f64,
        ) -> Result<()>;

        // ── TDataStdIntegerArrayHandle ────────────────────────────────────────────────
        // Shim holding Handle(TDataStd_IntegerArray) by value.
        //
        // Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_std___integer_array.html
        type TDataStdIntegerArrayHandle;

        // Set: static on TDataStd_IntegerArray; finds or creates an integer array
        // attribute on label with 0-based bounds [0, len-1]. isDelta omitted (OCCT
        // default Standard_False applies). Must be called inside an open command scope.
        fn tdatastd_integerarray_set(
            label: &TdfLabel,
            len: i32,
        ) -> Result<UniquePtr<TDataStdIntegerArrayHandle>>;
        // Find: returns nullptr (None on Rust side) when attribute is absent.
        fn tdatastd_integerarray_find(label: &TdfLabel) -> UniquePtr<TDataStdIntegerArrayHandle>;
        // ForgetAttribute(GUID) const — true if present and removed.
        fn tdatastd_integerarray_forget(label: &TdfLabel) -> bool;
        // Length: const — number of elements (== len passed to set).
        fn tdatastd_integerarray_length(h: &TDataStdIntegerArrayHandle) -> i32;
        // Value: 0-based. Raises OutOfRange (-> Err) if index is outside [0, length-1].
        fn tdatastd_integerarray_value(h: &TDataStdIntegerArrayHandle, index: i32) -> Result<i32>;
        // SetValue: 0-based. Raises OutOfRange (-> Err) if index is outside [0, length-1].
        // Must be called inside an open command scope.
        fn tdatastd_integerarray_set_value(
            h: &TDataStdIntegerArrayHandle,
            index: i32,
            value: i32,
        ) -> Result<()>;

        // ── TDataStdBooleanArrayHandle ────────────────────────────────────────────────
        // Shim holding Handle(TDataStd_BooleanArray) by value.
        //
        // Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_std___boolean_array.html
        type TDataStdBooleanArrayHandle;

        // Set: static on TDataStd_BooleanArray; finds or creates a boolean array
        // attribute on label with 0-based bounds [0, len-1]. No isDelta parameter
        // (unlike RealArray/IntegerArray/ByteArray/ExtStringArray). Must be called
        // inside an open command scope.
        fn tdatastd_booleanarray_set(
            label: &TdfLabel,
            len: i32,
        ) -> Result<UniquePtr<TDataStdBooleanArrayHandle>>;
        fn tdatastd_booleanarray_find(label: &TdfLabel) -> UniquePtr<TDataStdBooleanArrayHandle>;
        fn tdatastd_booleanarray_forget(label: &TdfLabel) -> bool;
        fn tdatastd_booleanarray_length(h: &TDataStdBooleanArrayHandle) -> i32;
        fn tdatastd_booleanarray_value(h: &TDataStdBooleanArrayHandle, index: i32) -> Result<bool>;
        fn tdatastd_booleanarray_set_value(
            h: &TDataStdBooleanArrayHandle,
            index: i32,
            value: bool,
        ) -> Result<()>;

        // ── TDataStdByteArrayHandle ───────────────────────────────────────────────────
        // Shim holding Handle(TDataStd_ByteArray) by value.
        //
        // Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_std___byte_array.html
        type TDataStdByteArrayHandle;

        // Set: static on TDataStd_ByteArray; finds or creates a byte array attribute
        // on label with 0-based bounds [0, len-1]. isDelta omitted (OCCT default
        // Standard_False applies). Must be called inside an open command scope.
        fn tdatastd_bytearray_set(
            label: &TdfLabel,
            len: i32,
        ) -> Result<UniquePtr<TDataStdByteArrayHandle>>;
        fn tdatastd_bytearray_find(label: &TdfLabel) -> UniquePtr<TDataStdByteArrayHandle>;
        fn tdatastd_bytearray_forget(label: &TdfLabel) -> bool;
        fn tdatastd_bytearray_length(h: &TDataStdByteArrayHandle) -> i32;
        fn tdatastd_bytearray_value(h: &TDataStdByteArrayHandle, index: i32) -> Result<u8>;
        fn tdatastd_bytearray_set_value(
            h: &TDataStdByteArrayHandle,
            index: i32,
            value: u8,
        ) -> Result<()>;

        // ── TDataStdExtStringArrayHandle ──────────────────────────────────────────────
        // Shim holding Handle(TDataStd_ExtStringArray) by value.
        //
        // Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_std___ext_string_array.html
        type TDataStdExtStringArrayHandle;

        // Set: static on TDataStd_ExtStringArray; finds or creates an ExtStringArray
        // attribute on label with 0-based bounds [0, len-1]. isDelta omitted (OCCT
        // default Standard_False applies). Must be called inside an open command scope.
        fn tdatastd_extstringarray_set(
            label: &TdfLabel,
            len: i32,
        ) -> Result<UniquePtr<TDataStdExtStringArrayHandle>>;
        fn tdatastd_extstringarray_find(
            label: &TdfLabel,
        ) -> UniquePtr<TDataStdExtStringArrayHandle>;
        fn tdatastd_extstringarray_forget(label: &TdfLabel) -> bool;
        fn tdatastd_extstringarray_length(h: &TDataStdExtStringArrayHandle) -> i32;
        // Value: UTF-8, same conversion as tdatastd_name_get, per element.
        fn tdatastd_extstringarray_value(
            h: &TDataStdExtStringArrayHandle,
            index: i32,
        ) -> Result<String>;
        // SetValue: isMultiByte=true UTF-8 decode, same as tdatastd_name_set, per element.
        fn tdatastd_extstringarray_set_value(
            h: &TDataStdExtStringArrayHandle,
            index: i32,
            value: &str,
        ) -> Result<()>;

        // ── TDataXtd_Geometry ─────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___geometry.html
        type TDataXtdGeometryHandle;

        // Set(label, geom_type) — finds or creates the attribute with the given
        // type set atomically before AddAttribute, so undo cleanly removes it.
        // Must be inside an open command scope.
        fn tdataxtd_geometry_set(
            label: &TdfLabel,
            geom_type: i32,
        ) -> Result<UniquePtr<TDataXtdGeometryHandle>>;

        // SetType(T) — non-const; updates the geometry kind.
        // Must be inside an open command scope.
        fn tdataxtd_geometry_set_type(h: Pin<&mut TDataXtdGeometryHandle>, geom_type: i32);

        // GetType() const — reads the TDataXtd_GeometryEnum ordinal.
        fn tdataxtd_geometry_get_type(h: &TDataXtdGeometryHandle) -> i32;

        // FindAttribute — returns null UniquePtr when attribute is absent.
        fn tdataxtd_geometry_find(label: &TdfLabel) -> UniquePtr<TDataXtdGeometryHandle>;

        fn tdataxtd_geometry_forget(label: &TdfLabel) -> bool;

        // Type(label) — static; infers GeometryKind from TNaming_NamedShape
        // topology on label.  Does not read the TDataXtd_Geometry attribute.
        // Returns Err when no TNaming_NamedShape is present on the label.
        // Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___geometry.html
        fn tdataxtd_geometry_type_on_label(label: &TdfLabel) -> Result<i32>;

        // ── TDataXtd_Constraint ───────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___constraint.html
        //
        // Geometry participants are Handle(TNaming_NamedShape).
        // C++ overload set (1–4 geometries) is split into four named shims;
        // the safe Rust API collapses them into a single set(&[&TnamingNamedShape]).
        type TDataXtdConstraintHandle;

        fn tdataxtd_constraint_set1(
            label: &TdfLabel,
            constraint_type: i32,
            g1: &TnamingNamedShapeHandle,
        ) -> Result<UniquePtr<TDataXtdConstraintHandle>>;

        fn tdataxtd_constraint_set2(
            label: &TdfLabel,
            constraint_type: i32,
            g1: &TnamingNamedShapeHandle,
            g2: &TnamingNamedShapeHandle,
        ) -> Result<UniquePtr<TDataXtdConstraintHandle>>;

        fn tdataxtd_constraint_set3(
            label: &TdfLabel,
            constraint_type: i32,
            g1: &TnamingNamedShapeHandle,
            g2: &TnamingNamedShapeHandle,
            g3: &TnamingNamedShapeHandle,
        ) -> Result<UniquePtr<TDataXtdConstraintHandle>>;

        fn tdataxtd_constraint_set4(
            label: &TdfLabel,
            constraint_type: i32,
            g1: &TnamingNamedShapeHandle,
            g2: &TnamingNamedShapeHandle,
            g3: &TnamingNamedShapeHandle,
            g4: &TnamingNamedShapeHandle,
        ) -> Result<UniquePtr<TDataXtdConstraintHandle>>;

        // SetGeometry(index, ns) — 1-based; non-const.
        fn tdataxtd_constraint_set_geometry(
            c: Pin<&mut TDataXtdConstraintHandle>,
            index: i32,
            ns: &TnamingNamedShapeHandle,
        );

        // SetValue — associates a TDataStd_Real as dimension value; non-const.
        fn tdataxtd_constraint_set_value(
            c: Pin<&mut TDataXtdConstraintHandle>,
            val: &TDataStdRealHandle,
        );

        // SetType — non-const.
        fn tdataxtd_constraint_set_type(
            c: Pin<&mut TDataXtdConstraintHandle>,
            constraint_type: i32,
        );

        fn tdataxtd_constraint_get_type(c: &TDataXtdConstraintHandle) -> i32;
        fn tdataxtd_constraint_nb_geometries(c: &TDataXtdConstraintHandle) -> i32;

        // GetGeometry — 1-based; returns null UniquePtr when OOB.
        fn tdataxtd_constraint_get_geometry(
            c: &TDataXtdConstraintHandle,
            index: i32,
        ) -> UniquePtr<TnamingNamedShapeHandle>;

        fn tdataxtd_constraint_is_dimension(c: &TDataXtdConstraintHandle) -> bool;

        // GetValue — returns null UniquePtr when IsDimension() is false.
        fn tdataxtd_constraint_get_value(
            c: &TDataXtdConstraintHandle,
        ) -> UniquePtr<TDataStdRealHandle>;

        fn tdataxtd_constraint_verified(c: &TDataXtdConstraintHandle) -> bool;

        // Verified(bool) — non-const.
        fn tdataxtd_constraint_set_verified(c: Pin<&mut TDataXtdConstraintHandle>, status: bool);

        fn tdataxtd_constraint_is_planar(c: &TDataXtdConstraintHandle) -> bool;

        // GetPlane — returns null UniquePtr when IsPlanar() is false.
        fn tdataxtd_constraint_get_plane(
            c: &TDataXtdConstraintHandle,
        ) -> UniquePtr<TnamingNamedShapeHandle>;

        // SetPlane — non-const.
        fn tdataxtd_constraint_set_plane(
            c: Pin<&mut TDataXtdConstraintHandle>,
            plane: &TnamingNamedShapeHandle,
        );

        fn tdataxtd_constraint_find(label: &TdfLabel) -> UniquePtr<TDataXtdConstraintHandle>;

        fn tdataxtd_constraint_forget(label: &TdfLabel) -> bool;

        // ── TDataXtd_Position ─────────────────────────────────────────────
        // TDataXtd_Position stores a gp_Pnt directly in the attribute (unlike
        // TDataXtd_Point/Axis/Plane which are GenericEmpty tags).
        // gp_Pnt is decomposed to f64 scalars at the bridge boundary.
        //
        // Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___position.html
        type TDataXtdPositionHandle;

        // Set(label, x, y, z) — finds or creates attribute with position set
        // atomically before AddAttribute (undo ordering rule).
        // Must be inside an open command scope.
        fn tdataxtd_position_set(
            label: &TdfLabel,
            x: f64,
            y: f64,
            z: f64,
        ) -> Result<UniquePtr<TDataXtdPositionHandle>>;

        // SetPosition — non-const; updates position on a committed attribute.
        // Must be inside an open command scope.
        fn tdataxtd_position_set_position(
            h: Pin<&mut TDataXtdPositionHandle>,
            x: f64,
            y: f64,
            z: f64,
        );

        // GetPosition — const; decomposes gp_Pnt to scalar out-params.
        fn tdataxtd_position_get_position(
            h: &TDataXtdPositionHandle,
            x: &mut f64,
            y: &mut f64,
            z: &mut f64,
        );

        // FindAttribute — returns null UniquePtr when attribute is absent.
        fn tdataxtd_position_find(label: &TdfLabel) -> UniquePtr<TDataXtdPositionHandle>;

        fn tdataxtd_position_forget(label: &TdfLabel) -> bool;

        // ── Shape constructors for TDataXtd tag attributes ────────────────
        // Free functions that cross the gp boundary and return TopoDS shapes
        // for use with TnamingBuilderShim::generated_fresh.  Used by the
        // Option B safe API for OcPointAttr / OcAxisAttr / OcPlaneAttr.

        // BRepBuilderAPI_MakeVertex from point coordinates.
        // Reference: https://dev.opencascade.org/doc/refman/html/class_b_rep_builder_a_p_i___make_vertex.html
        fn tdataxtd_make_vertex_shape(x: f64, y: f64, z: f64) -> Result<UniquePtr<TopodsVertex>>;

        // BRepBuilderAPI_MakeEdge(gp_Lin) from Ax1 scalars (origin + direction).
        // Produces an infinite linear edge.
        // Reference: https://dev.opencascade.org/doc/refman/html/class_b_rep_builder_a_p_i___make_edge.html
        fn tdataxtd_make_infinite_edge_from_ax1(
            ox: f64,
            oy: f64,
            oz: f64,
            dx: f64,
            dy: f64,
            dz: f64,
        ) -> Result<UniquePtr<TopodsEdge>>;

        // BRepBuilderAPI_MakeFace(gp_Pln) from Ax2 scalars
        // (origin + normal + x_direction).  Produces an infinite planar face.
        // Reference: https://dev.opencascade.org/doc/refman/html/class_b_rep_builder_a_p_i___make_face.html
        fn tdataxtd_make_face_from_ax2(
            ox: f64,
            oy: f64,
            oz: f64,
            nx: f64,
            ny: f64,
            nz: f64,
            xx: f64,
            xy: f64,
            xz: f64,
        ) -> Result<UniquePtr<TopdsFace>>;

        // ── TDataXtd_Point ────────────────────────────────────────────────
        // Tag attribute: label whose NamedShape contains a vertex.
        // Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___point.html
        type TDataXtdPointHandle;

        // Set(label) — finds or creates the tag; no geometry here.
        // Shape must be placed on the label via TnamingBuilder in the same command.
        fn tdataxtd_point_set(label: &TdfLabel) -> Result<UniquePtr<TDataXtdPointHandle>>;

        // FindAttribute — returns null UniquePtr when attribute is absent.
        fn tdataxtd_point_find(label: &TdfLabel) -> UniquePtr<TDataXtdPointHandle>;

        fn tdataxtd_point_forget(label: &TdfLabel) -> bool;

        // ── TDataXtd_Axis ─────────────────────────────────────────────────
        // Tag attribute: label whose NamedShape contains a linear edge.
        // Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___axis.html
        type TDataXtdAxisHandle;

        // Set(label) — finds or creates the tag; no geometry here.
        // Shape must be placed on the label via TnamingBuilder in the same command.
        fn tdataxtd_axis_set(label: &TdfLabel) -> Result<UniquePtr<TDataXtdAxisHandle>>;

        // FindAttribute — returns null UniquePtr when attribute is absent.
        fn tdataxtd_axis_find(label: &TdfLabel) -> UniquePtr<TDataXtdAxisHandle>;

        fn tdataxtd_axis_forget(label: &TdfLabel) -> bool;

        // ── TDataXtd_Plane ────────────────────────────────────────────────
        // Tag attribute: label whose NamedShape contains a planar face.
        // Reference: https://dev.opencascade.org/doc/refman/html/class_t_data_xtd___plane.html
        type TDataXtdPlaneHandle;

        // Set(label) — finds or creates the tag; no geometry here.
        // Shape must be placed on the label via TnamingBuilder in the same command.
        fn tdataxtd_plane_set(label: &TdfLabel) -> Result<UniquePtr<TDataXtdPlaneHandle>>;

        // FindAttribute — returns null UniquePtr when attribute is absent.
        fn tdataxtd_plane_find(label: &TdfLabel) -> UniquePtr<TDataXtdPlaneHandle>;

        fn tdataxtd_plane_forget(label: &TdfLabel) -> bool;

        // ── TdfLabel ──────────────────────────────────────────────────────────────
        // Shim holding TDF_Label by value.  TDF_Label is a non-owning reference
        // into a TDF_Data tree; the Rust wrapper carries a lifetime parameter
        // to enforce that labels cannot outlive the document that owns the tree.
        //
        // Reference: https://dev.opencascade.org/doc/refman/html/class_t_d_f___label.html
        type TdfLabel;

        fn clone_tdf_label(l: &TdfLabel) -> UniquePtr<TdfLabel>;

        // Const queries — TDF_Label const methods.
        fn tdf_label_is_null(l: &TdfLabel) -> bool;
        fn tdf_label_is_root(l: &TdfLabel) -> bool;
        fn tdf_label_tag(l: &TdfLabel) -> i32;
        fn tdf_label_father(l: &TdfLabel) -> UniquePtr<TdfLabel>;
        fn tdf_label_root(l: &TdfLabel) -> UniquePtr<TdfLabel>;
        // FindChild is const on TDF_Label even when create=true; the label
        // value itself is unchanged — only the external tree is mutated.
        fn tdf_label_find_child(l: &TdfLabel, tag: i32, create: bool) -> UniquePtr<TdfLabel>;
        fn tdf_label_has_attribute(l: &TdfLabel) -> bool;
        fn tdf_label_nb_attributes(l: &TdfLabel) -> i32;
        // Entry string, e.g. "0:1:2:3".
        fn tdf_label_entry(l: &TdfLabel) -> String;
        // ForgetAllAttributes — const on TDF_Label, compatible with
        // Transaction & Delta. Rust side requires &Command<'_> (see
        // OcLabel::forget_all_attributes).
        fn tdf_label_forget_all_attributes(l: &TdfLabel, clear_children: bool);

        // ── TdfChildIteratorShim ──────────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/class_t_d_f___child_iterator.html
        type TdfChildIteratorShim;

        fn new_tdf_child_iterator(
            label: &TdfLabel,
            all_levels: bool,
        ) -> UniquePtr<TdfChildIteratorShim>;
        fn more(self: &TdfChildIteratorShim) -> bool;
        fn next(self: Pin<&mut TdfChildIteratorShim>);
        // value() is const — reads current label without advancing.
        fn value(self: &TdfChildIteratorShim) -> UniquePtr<TdfLabel>;

        // ── DocumentHandle ────────────────────────────────────────────────────────
        // Shim holding Handle(TDocStd_Document) by value.
        //
        // Reference: https://dev.opencascade.org/doc/refman/html/class_t_doc_std___document.html
        type DocumentHandle;

        // document_main: const — returns the root label of the user data section.
        fn document_main(doc: &DocumentHandle) -> UniquePtr<TdfLabel>;

        // Const queries.
        fn document_get_available_undos(doc: &DocumentHandle) -> i32;
        fn document_get_available_redos(doc: &DocumentHandle) -> i32;

        // Non-const command / transaction operations.
        fn document_has_open_command(doc: &DocumentHandle) -> bool;
        fn document_new_command(doc: Pin<&mut DocumentHandle>) -> Result<()>;
        fn document_commit_command(doc: Pin<&mut DocumentHandle>) -> Result<bool>;
        fn document_abort_command(doc: Pin<&mut DocumentHandle>) -> Result<()>;
        fn document_undo(doc: Pin<&mut DocumentHandle>) -> Result<bool>;
        fn document_redo(doc: Pin<&mut DocumentHandle>) -> Result<bool>;
        fn document_set_undo_limit(doc: Pin<&mut DocumentHandle>, n: i32);
        // Non-const: severs both ownership edges. Guarded by IsOpened(); safe to call
        // on a never-opened or already-closed document.
        fn document_close(doc: Pin<&mut DocumentHandle>) -> Result<()>;
        // Const: IsOpened() == !myApplication.IsNull().
        fn document_is_opened(doc: &DocumentHandle) -> bool;

        // ── ApplicationHandle ─────────────────────────────────────────────────────
        // Shim holding Handle(TDocStd_Application) by value.
        //
        // Reference: https://dev.opencascade.org/doc/refman/html/class_t_doc_std___application.html
        type ApplicationHandle;

        fn new_application() -> UniquePtr<ApplicationHandle>;
        // Non-const: registers the new document with the application.
        fn application_new_document(
            app: Pin<&mut ApplicationHandle>,
            format: &str,
        ) -> Result<UniquePtr<DocumentHandle>>;
        // Const: NbDocuments() — number of documents registered with this application.
        fn application_nb_documents(app: &ApplicationHandle) -> i32;

        // ── MakeOffsetShapeBuilder ────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/class_b_rep_offset_a_p_i___make_offset_shape.html
        type MakeOffsetShapeBuilder;

        fn new_make_offset_shape_builder() -> UniquePtr<MakeOffsetShapeBuilder>;
        fn perform(
            self: Pin<&mut MakeOffsetShapeBuilder>,
            shape: &TopodsShape,
            offset: f64,
        ) -> Result<()>;
        fn is_done(self: &MakeOffsetShapeBuilder) -> bool;
        fn shape(self: Pin<&mut MakeOffsetShapeBuilder>) -> UniquePtr<TopodsShape>;
        fn modified_count(self: Pin<&mut MakeOffsetShapeBuilder>, s: &TopodsShape) -> i32;
        fn modified_at(
            self: Pin<&mut MakeOffsetShapeBuilder>,
            s: &TopodsShape,
            i: i32,
        ) -> UniquePtr<TopodsShape>;
        fn generated_count(self: Pin<&mut MakeOffsetShapeBuilder>, s: &TopodsShape) -> i32;
        fn generated_at(
            self: Pin<&mut MakeOffsetShapeBuilder>,
            s: &TopodsShape,
            i: i32,
        ) -> UniquePtr<TopodsShape>;
        fn is_deleted(self: Pin<&mut MakeOffsetShapeBuilder>, s: &TopodsShape) -> bool;

        // ── MakeThickSolidBuilder ─────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/class_b_rep_offset_a_p_i___make_thick_solid.html
        type MakeThickSolidBuilder;

        fn new_make_thick_solid_builder() -> UniquePtr<MakeThickSolidBuilder>;
        fn add_closing_face(self: Pin<&mut MakeThickSolidBuilder>, face: &TopdsFace);
        fn build(
            self: Pin<&mut MakeThickSolidBuilder>,
            shape: &TopodsShape,
            offset: f64,
            tol: f64,
        ) -> Result<()>;
        fn is_done(self: &MakeThickSolidBuilder) -> bool;
        fn shape(self: Pin<&mut MakeThickSolidBuilder>) -> UniquePtr<TopodsShape>;
        fn modified_count(self: Pin<&mut MakeThickSolidBuilder>, s: &TopodsShape) -> i32;
        fn modified_at(
            self: Pin<&mut MakeThickSolidBuilder>,
            s: &TopodsShape,
            i: i32,
        ) -> UniquePtr<TopodsShape>;
        fn generated_count(self: Pin<&mut MakeThickSolidBuilder>, s: &TopodsShape) -> i32;
        fn generated_at(
            self: Pin<&mut MakeThickSolidBuilder>,
            s: &TopodsShape,
            i: i32,
        ) -> UniquePtr<TopodsShape>;
        fn is_deleted(self: Pin<&mut MakeThickSolidBuilder>, s: &TopodsShape) -> bool;
        // ── MakeChamferBuilder ────────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/class_b_rep_fillet_a_p_i___make_chamfer.html
        type MakeChamferBuilder;

        fn new_make_chamfer_builder(shape: &TopodsShape) -> Result<UniquePtr<MakeChamferBuilder>>;
        fn add_edge(self: Pin<&mut MakeChamferBuilder>, dis: f64, edge: &TopodsEdge) -> Result<()>;
        fn add_edge_asymmetric(
            self: Pin<&mut MakeChamferBuilder>,
            dis1: f64,
            dis2: f64,
            edge: &TopodsEdge,
            face: &TopdsFace,
        ) -> Result<()>;
        fn add_edge_dist_angle(
            self: Pin<&mut MakeChamferBuilder>,
            dis: f64,
            angle: f64,
            edge: &TopodsEdge,
            face: &TopdsFace,
        ) -> Result<()>;
        fn build(self: Pin<&mut MakeChamferBuilder>) -> Result<()>;
        fn is_done(self: &MakeChamferBuilder) -> bool;
        fn shape(self: Pin<&mut MakeChamferBuilder>) -> UniquePtr<TopodsShape>;
        fn modified_count(self: Pin<&mut MakeChamferBuilder>, s: &TopodsShape) -> i32;
        fn modified_at(
            self: Pin<&mut MakeChamferBuilder>,
            s: &TopodsShape,
            i: i32,
        ) -> UniquePtr<TopodsShape>;
        fn generated_count(self: Pin<&mut MakeChamferBuilder>, s: &TopodsShape) -> i32;
        fn generated_at(
            self: Pin<&mut MakeChamferBuilder>,
            s: &TopodsShape,
            i: i32,
        ) -> UniquePtr<TopodsShape>;
        fn is_deleted(self: Pin<&mut MakeChamferBuilder>, s: &TopodsShape) -> bool;
        // ── MakeFilletBuilder ─────────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/class_b_rep_fillet_a_p_i___make_fillet.html
        type MakeFilletBuilder;

        fn new_make_fillet_builder(shape: &TopodsShape) -> Result<UniquePtr<MakeFilletBuilder>>;
        fn add_edge(
            self: Pin<&mut MakeFilletBuilder>,
            radius: f64,
            edge: &TopodsEdge,
        ) -> Result<()>;
        fn build(self: Pin<&mut MakeFilletBuilder>) -> Result<()>;
        fn is_done(self: &MakeFilletBuilder) -> bool;
        fn shape(self: Pin<&mut MakeFilletBuilder>) -> UniquePtr<TopodsShape>;
        fn modified_count(self: Pin<&mut MakeFilletBuilder>, s: &TopodsShape) -> i32;
        fn modified_at(
            self: Pin<&mut MakeFilletBuilder>,
            s: &TopodsShape,
            i: i32,
        ) -> UniquePtr<TopodsShape>;
        fn generated_count(self: Pin<&mut MakeFilletBuilder>, s: &TopodsShape) -> i32;
        fn generated_at(
            self: Pin<&mut MakeFilletBuilder>,
            s: &TopodsShape,
            i: i32,
        ) -> UniquePtr<TopodsShape>;
        fn is_deleted(self: Pin<&mut MakeFilletBuilder>, s: &TopodsShape) -> bool;
        // ── Boolean operations ────────────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/class_b_rep_algo_a_p_i___fuse.html
        fn fuse_shapes(s1: &TopodsShape, s2: &TopodsShape) -> Result<UniquePtr<TopodsShape>>;
        // Reference: https://dev.opencascade.org/doc/refman/html/class_b_rep_algo_a_p_i___cut.html
        fn cut_shapes(s1: &TopodsShape, s2: &TopodsShape) -> Result<UniquePtr<TopodsShape>>;
        // Reference: https://dev.opencascade.org/doc/refman/html/class_b_rep_algo_a_p_i___common.html
        fn common_shapes(s1: &TopodsShape, s2: &TopodsShape) -> Result<UniquePtr<TopodsShape>>;
        // Reference: https://dev.opencascade.org/doc/refman/html/class_b_rep_builder_a_p_i___transform.html
        fn transform_shape(
            shape: &TopodsShape,
            r11: f64,
            r12: f64,
            r13: f64,
            t1: f64,
            r21: f64,
            r22: f64,
            r23: f64,
            t2: f64,
            r31: f64,
            r32: f64,
            r33: f64,
            t3: f64,
        ) -> Result<UniquePtr<TopodsShape>>;

        // ── TopoDS_Vertex ─────────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/class_topo_d_s___vertex.html
        #[cxx_name = "TopoDS_Vertex"]
        type TopodsVertex;

        fn make_vertex(x: f64, y: f64, z: f64) -> UniquePtr<TopodsVertex>;
        fn clone_vertex(v: &TopodsVertex) -> UniquePtr<TopodsVertex>;
        fn vertex_pnt_x(v: &TopodsVertex) -> f64;
        fn vertex_pnt_y(v: &TopodsVertex) -> f64;
        fn vertex_pnt_z(v: &TopodsVertex) -> f64;

        // ── TopoDS_Edge ───────────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/class_topo_d_s___edge.html
        #[cxx_name = "TopoDS_Edge"]
        type TopodsEdge;

        fn clone_edge(e: &TopodsEdge) -> UniquePtr<TopodsEdge>;
        fn edge_start_vertex(e: &TopodsEdge) -> UniquePtr<TopodsVertex>;
        fn edge_end_vertex(e: &TopodsEdge) -> UniquePtr<TopodsVertex>;

        // ── MakeEdgeBuilder ───────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/class_b_rep_builder_a_p_i___make_edge.html
        type MakeEdgeBuilder;

        fn new_make_edge_builder(
            v1: &TopodsVertex,
            v2: &TopodsVertex,
        ) -> UniquePtr<MakeEdgeBuilder>;
        fn is_done(self: &MakeEdgeBuilder) -> bool;
        fn error(self: &MakeEdgeBuilder) -> i32;
        fn edge(self: Pin<&mut MakeEdgeBuilder>) -> UniquePtr<TopodsEdge>;

        // ── TopoDS_Wire ───────────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/class_topo_d_s___wire.html
        #[cxx_name = "TopoDS_Wire"]
        type TopodsWire;

        fn clone_wire(w: &TopodsWire) -> UniquePtr<TopodsWire>;

        // ── MakeWireBuilder ───────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/class_b_rep_builder_a_p_i___make_wire.html
        type MakeWireBuilder;

        fn new_make_wire_builder() -> UniquePtr<MakeWireBuilder>;
        fn add_edge(self: Pin<&mut MakeWireBuilder>, e: &TopodsEdge);
        fn is_done(self: &MakeWireBuilder) -> bool;
        fn error(self: &MakeWireBuilder) -> i32;
        fn wire(self: Pin<&mut MakeWireBuilder>) -> UniquePtr<TopodsWire>;

        // ── WireEdgeExplorer ──────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/class_b_rep_tools___wire_explorer.html
        type WireEdgeExplorer;

        fn new_wire_edge_explorer(w: &TopodsWire) -> UniquePtr<WireEdgeExplorer>;
        fn more(self: &WireEdgeExplorer) -> bool;
        fn next(self: Pin<&mut WireEdgeExplorer>);
        fn current_edge(self: &WireEdgeExplorer) -> UniquePtr<TopodsEdge>;

        // ── TopoDS_Face ───────────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/class_topo_d_s___face.html
        #[cxx_name = "TopoDS_Face"]
        type TopdsFace;

        fn clone_face(f: &TopdsFace) -> UniquePtr<TopdsFace>;
        fn face_outer_wire(f: &TopdsFace) -> UniquePtr<TopodsWire>;
        fn face_is_reversed(f: &TopdsFace) -> bool;

        // ── MakeFaceBuilder ───────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/class_b_rep_builder_a_p_i___make_face.html
        type MakeFaceBuilder;

        fn new_make_face_from_wire(w: &TopodsWire, only_plane: bool) -> UniquePtr<MakeFaceBuilder>;
        // Reference: https://dev.opencascade.org/doc/refman/html/class_b_rep_builder_a_p_i___make_face.html
        fn new_make_face_from_plane_and_wire(
            px: f64,
            py: f64,
            pz: f64,
            nx: f64,
            ny: f64,
            nz: f64,
            w: &TopodsWire,
        ) -> Result<UniquePtr<MakeFaceBuilder>>;
        fn is_done(self: &MakeFaceBuilder) -> bool;
        fn error(self: &MakeFaceBuilder) -> i32;
        fn face(self: Pin<&mut MakeFaceBuilder>) -> UniquePtr<TopdsFace>;

        // ── TopoDS_Solid ──────────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/class_topo_d_s___solid.html
        #[cxx_name = "TopoDS_Solid"]
        type TopdsSolid;

        fn clone_solid(s: &TopdsSolid) -> UniquePtr<TopdsSolid>;

        // ── MakePrismBuilder ──────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/class_b_rep_prim_a_p_i___make_prism.html
        //
        // Returns Result because MakePrism computes immediately in its
        // constructor and throws on failure rather than deferring to IsDone().
        type MakePrismBuilder;

        fn new_make_prism_from_face(
            face: &TopdsFace,
            vx: f64,
            vy: f64,
            vz: f64,
        ) -> Result<UniquePtr<MakePrismBuilder>>;
        fn is_done(self: &MakePrismBuilder) -> bool;
        fn solid(self: Pin<&mut MakePrismBuilder>) -> UniquePtr<TopdsSolid>;

        // ── TopoDS_Shape ──────────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/class_topo_d_s___shape.html
        #[cxx_name = "TopoDS_Shape"]
        type TopodsShape;

        // Reference: https://dev.opencascade.org/doc/refman/html/class_topo_d_s___shape.html
        fn topods_shape_type(shape: &TopodsShape) -> i32;
        fn topods_compound_child_count(shape: &TopodsShape) -> i32;
        // Clone (ref-count bump only — no geometry copy).
        fn clone_shape(s: &TopodsShape) -> UniquePtr<TopodsShape>;

        // Placed-instance identity key: hashes TShape ptr + Location + Orientation.
        // Use this for ShapeKey derivation; distinct faces of a MakePrism solid
        // that share a TShape (top/bottom caps) are distinguished by their Location.
        fn shape_key(s: &TopodsShape) -> usize;

        // Geometry-only identity: raw TShape pointer.
        // Useful for deduplication of instances sharing the same underlying surface.
        // Not suitable as a unique per-face key — use shape_key() for that.
        fn shape_tshape_ptr(s: &TopodsShape) -> usize;

        // Null predicates
        // TopoDS_Shape::IsNull() and its per-subtype shims.
        // Reference: https://dev.opencascade.org/doc/refman/html/class_topo_d_s___shape.html
        fn topods_shape_is_null(s: &TopodsShape) -> bool;
        fn topods_face_is_null(s: &TopdsFace) -> bool;
        fn topods_edge_is_null(s: &TopodsEdge) -> bool;
        fn topods_wire_is_null(s: &TopodsWire) -> bool;
        fn topods_vertex_is_null(s: &TopodsVertex) -> bool;
        fn topods_solid_is_null(s: &TopdsSolid) -> bool;
        // Up-casts — zero-cost reference casts; lifetime tied to input.
        fn face_as_shape(f: &TopdsFace) -> &TopodsShape;
        fn solid_as_shape(s: &TopdsSolid) -> &TopodsShape;
        fn edge_as_shape(e: &TopodsEdge) -> &TopodsShape;
        fn wire_as_shape(w: &TopodsWire) -> &TopodsShape;
        fn vertex_as_shape(v: &TopodsVertex) -> &TopodsShape;

        // Down-casts — caller guarantees the shape type matches.
        // On type mismatch Standard_TypeMismatch is thrown; cxx terminates.
        fn shape_as_face(s: &TopodsShape) -> UniquePtr<TopdsFace>;
        fn shape_as_vertex(s: &TopodsShape) -> UniquePtr<TopodsVertex>;
        fn shape_as_edge(s: &TopodsShape) -> UniquePtr<TopodsEdge>;
        fn shape_as_wire(s: &TopodsShape) -> UniquePtr<TopodsWire>;
        fn shape_as_solid(s: &TopodsShape) -> UniquePtr<TopdsSolid>;

        // ── ShapeExplorer ─────────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/class_top_exp___explorer.html
        //
        // shape_enum values (TopAbs_ShapeEnum):
        //   TopAbs_FACE = 4, TopAbs_EDGE = 6, TopAbs_VERTEX = 7
        type ShapeExplorer;

        fn new_shape_explorer(shape: &TopodsShape, shape_enum: i32) -> UniquePtr<ShapeExplorer>;
        fn more(self: &ShapeExplorer) -> bool;
        fn next(self: Pin<&mut ShapeExplorer>);
        // Returned reference borrows self; valid until next() or drop.
        fn current(self: &ShapeExplorer) -> &TopodsShape;
        fn current_owned(self: &ShapeExplorer) -> UniquePtr<TopodsShape>;

        // ── IncrementalMeshBuilder ────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/class_b_rep_mesh___incremental_mesh.html
        //
        // Returns Result because construction computes immediately and
        // throws on failure (no IsDone()/Error() deferred pattern).
        type IncrementalMeshBuilder;

        fn new_incremental_mesh(
            shape: &TopodsShape,
            lin_def: f64,
            is_relative: bool,
            ang_def: f64,
            is_in_parallel: bool,
        ) -> Result<UniquePtr<IncrementalMeshBuilder>>;
        fn is_done(self: &IncrementalMeshBuilder) -> bool;

        // ── PolyTriangulationHandle ───────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/class_poly___triangulation.html
        //
        // Wraps Handle(Poly_Triangulation).  Check is_null() before any
        // other method call.  Node and triangle indices are 1-based (OCCT
        // convention); convert to 0-based on the Rust side.
        type PolyTriangulationHandle;

        fn face_triangulation(f: &TopdsFace) -> UniquePtr<PolyTriangulationHandle>;
        fn is_null(self: &PolyTriangulationHandle) -> bool;
        fn nb_nodes(self: &PolyTriangulationHandle) -> i32;
        fn nb_triangles(self: &PolyTriangulationHandle) -> i32;
        fn node_x(self: &PolyTriangulationHandle, i: i32) -> f64;
        fn node_y(self: &PolyTriangulationHandle, i: i32) -> f64;
        fn node_z(self: &PolyTriangulationHandle, i: i32) -> f64;
        // Poly_Triangle::Get returns all three at once; three separate methods
        // keep the shim surface minimal while remaining cxx-compatible.
        fn triangle_n1(self: &PolyTriangulationHandle, i: i32) -> i32;
        fn triangle_n2(self: &PolyTriangulationHandle, i: i32) -> i32;
        fn triangle_n3(self: &PolyTriangulationHandle, i: i32) -> i32;
        fn placement_is_identity(self: &PolyTriangulationHandle) -> bool;
        fn placement_value(self: &PolyTriangulationHandle, i: i32) -> f64;
        // Reference: https://dev.opencascade.org/doc/refman/html/class_poly___triangulation.html
        fn has_normals(self: &PolyTriangulationHandle) -> bool;
        fn normal_x(self: &PolyTriangulationHandle, i: i32) -> f64;
        fn normal_y(self: &PolyTriangulationHandle, i: i32) -> f64;
        fn normal_z(self: &PolyTriangulationHandle, i: i32) -> f64;
        // Reference: https://dev.opencascade.org/doc/refman/html/class_b_rep_lib___tool_triangulated_shape.html
        fn compute_face_normals(f: &TopdsFace) -> bool;
        // ── Poly_Polygon3D ────────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/class_poly___polygon3_d.html
        type PolyPolygon3D;

        fn edge_polygon3d(e: &TopodsEdge) -> UniquePtr<PolyPolygon3D>;
        fn edge_polygon3d_location(e: &TopodsEdge) -> UniquePtr<TopLocLocation>;
        fn polygon3d_nb_nodes(p: &PolyPolygon3D) -> i32;
        fn polygon3d_node_x(p: &PolyPolygon3D, i: i32) -> f64;
        fn polygon3d_node_y(p: &PolyPolygon3D, i: i32) -> f64;
        fn polygon3d_node_z(p: &PolyPolygon3D, i: i32) -> f64;

        // ── TopLoc_Location ───────────────────────────────────────────────
        // Reference: https://dev.opencascade.org/doc/refman/html/class_top_loc___location.html
        type TopLocLocation;

        fn apply_location_to_point(
            loc: &TopLocLocation,
            x: f64,
            y: f64,
            z: f64,
            out_x: &mut f64,
            out_y: &mut f64,
            out_z: &mut f64,
        );
        fn location_is_identity(loc: &TopLocLocation) -> bool;
    }
}
