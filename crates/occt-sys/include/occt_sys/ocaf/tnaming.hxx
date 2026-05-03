// Sourced from OCCT 7.9 documentation.
// No derivation from any other binding crate.
//
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_naming___builder.html
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_naming___named_shape.html
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_naming___tool.html
// Toolkit: TKBRep (verify on refman class page before final build.rs commit)

#pragma once

#include <TNaming_Builder.hxx>
#include <TNaming_NamedShape.hxx>
#include <TNaming_Tool.hxx>
#include <TopoDS_Shape.hxx>

// ---------------------------------------------------------------------------
// Handle wrapper for TNaming_NamedShape
// ---------------------------------------------------------------------------
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_naming___named_shape.html

struct TnamingNamedShapeHandle {
    Handle(TNaming_NamedShape) inner;
    bool is_null() const { return inner.IsNull(); }
};

// ---------------------------------------------------------------------------
// TNaming_Builder shim
//
// TNaming_Builder is a stack-allocated C++ value type. It is owned by value
// inside this wrapper struct so that cxx can treat it as a heap-allocated
// opaque type via UniquePtr.
//
// Constructor takes const TDF_Label& — the label on which the NamedShape
// attribute will be recorded. Must be called inside an open Command.
// ---------------------------------------------------------------------------

struct TnamingBuilderShim {
    TNaming_Builder inner;

    // Reference: constructor — https://dev.opencascade.org/doc/refman/html/class_t_naming___builder.html
    TnamingBuilderShim(const TDF_Label& label) : inner(label) {}

    // Generated(const TopoDS_Shape& S) — shape with no topological ancestor
    // Reference: https://dev.opencascade.org/doc/refman/html/class_t_naming___builder.html#a_generated
    void generated_fresh(const TopoDS_Shape& s) {
        inner.Generated(s);
    }

    // Generated(const TopoDS_Shape& old, const TopoDS_Shape& gen)
    void generated_from(const TopoDS_Shape& old_s, const TopoDS_Shape& new_s) {
        inner.Generated(old_s, new_s);
    }

    // Modify(const TopoDS_Shape& old, const TopoDS_Shape& mod)
    void modify(const TopoDS_Shape& old_s, const TopoDS_Shape& new_s) {
        inner.Modify(old_s, new_s);
    }

    // Delete(const TopoDS_Shape& old)
    void delete_shape(const TopoDS_Shape& old_s) {
        inner.Delete(old_s);
    }

    // Select(const TopoDS_Shape& S, const TopoDS_Shape& InS)
    // Records a sub-shape selection in context — used by DOC-4
    void select(const TopoDS_Shape& s, const TopoDS_Shape& in_s) {
        inner.Select(s, in_s);
    }

    // NamedShape() — returns the Handle to the attribute written on the label.
    // Verify const-qualification on the refman before finalising.
    std::unique_ptr<TnamingNamedShapeHandle> named_shape() const {
        return std::unique_ptr<TnamingNamedShapeHandle>(
            new TnamingNamedShapeHandle{ inner.NamedShape() }
        );
    }
};
inline std::unique_ptr<TnamingBuilderShim> new_tnaming_builder(const TdfLabel& label) {
    return std::unique_ptr<TnamingBuilderShim>(new TnamingBuilderShim(label.inner));
}

// ---------------------------------------------------------------------------
// TNaming_NamedShape free-function shims
// ---------------------------------------------------------------------------

// find_tnaming_named_shape — template-attribute shim pattern.
// Returns false if no TNaming_NamedShape attribute is present on the label.
// Reference: TDF_Label::FindAttribute template — cannot be bound directly in cxx.

inline bool find_tnaming_named_shape(
    const TdfLabel& label,
    TnamingNamedShapeHandle& out
) {
    Handle(TNaming_NamedShape) h;
    bool found = label.inner.FindAttribute(TNaming_NamedShape::GetID(), h);
    if (found) out.inner = h;
    return found;
}

// TNaming_NamedShape::Get() — current shape on the label.
// After undo, reflects the pre-operation shape.
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_naming___named_shape.html
inline std::unique_ptr<TopoDS_Shape> tnaming_named_shape_get(
    const TnamingNamedShapeHandle& h
) {
    return std::unique_ptr<TopoDS_Shape>(new TopoDS_Shape(h.inner->Get()));
}

// TNaming_NamedShape::Evolution() — provenance enum.
// TNaming_Evolution: PRIMITIVE, GENERATED, MODIFY, DELETE, SELECTED
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_naming___named_shape.html
inline int tnaming_named_shape_evolution(const TnamingNamedShapeHandle& h) {
    return static_cast<int>(h.inner->Evolution());
}

// TNaming_Tool::OriginalShape — the shape before any evolution on this label.
// Reference: https://dev.opencascade.org/doc/refman/html/class_t_naming___tool.html
inline std::unique_ptr<TopoDS_Shape> tnaming_tool_original_shape(
    const TnamingNamedShapeHandle& h
) {
    return std::unique_ptr<TopoDS_Shape>(
        new TopoDS_Shape(TNaming_Tool::OriginalShape(h.inner))
    );
}
inline std::unique_ptr<TnamingNamedShapeHandle> new_tnaming_named_shape_handle() {
    return std::unique_ptr<TnamingNamedShapeHandle>(new TnamingNamedShapeHandle{});
}
