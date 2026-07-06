// Sourced from OCCT 7.9 documentation.
// No derivation from any other binding crate.
//
// Reference: https://dev.opencascade.org/doc/refman/html/class_b_rep_algo_a_p_i___fuse.html
// Reference: https://dev.opencascade.org/doc/refman/html/class_b_rep_algo_a_p_i___boolean_operation.html
// Reference: https://dev.opencascade.org/doc/refman/html/class_top_tools___list_of_shape.html

#pragma once

#include "solid.hxx"

#include <BRepAlgoAPI_Fuse.hxx>
#include <Message_ProgressRange.hxx>
#include <TopTools_ListOfShape.hxx>
#include <BRepAlgoAPI_Cut.hxx>
#include <BRepAlgoAPI_Common.hxx>

// Reference: https://dev.opencascade.org/doc/refman/html/class_b_rep_algo_a_p_i___cut.html
// Reference: https://dev.opencascade.org/doc/refman/html/class_b_rep_algo_a_p_i___common.html
//
// MakeFuseBuilder / MakeCutBuilder / MakeCommonBuilder: empty-ctor +
// SetArguments/SetTools/Build, with the builder instance kept alive so
// Modified/Generated/IsDeleted can be read afterward.
//
// Three distinct structs, not one generic type: BRepAlgoAPI_Fuse, _Cut, and
// _Common share the SetArguments/SetTools/Build shape but have no common
// instantiation point above the abstract BRepAlgoAPI_BuilderAlgo.

struct MakeFuseBuilder {
    BRepAlgoAPI_Fuse inner;
    MakeFuseBuilder() = default;

    void build(const TopoDS_Shape& s1, const TopoDS_Shape& s2) {
        try {
            TopTools_ListOfShape args, tools;
            args.Append(s1);
            tools.Append(s2);
            inner.SetArguments(args);
            inner.SetTools(tools);
            inner.Build(Message_ProgressRange());
        } catch (const std::runtime_error&) { throw; }
        catch (...) { rethrow_occt_as_runtime_error(); }
    }
    bool is_done() const { return inner.IsDone(); }
    bool has_errors() const { return inner.HasErrors(); }
    std::unique_ptr<TopoDS_Shape> shape() {
        return std::make_unique<TopoDS_Shape>(inner.Shape());
    }
    bool is_deleted(const TopoDS_Shape& s) {
        return inner.IsDeleted(s) == Standard_True;
    }
};
inline std::unique_ptr<ShapeListIter>
fuse_modified_iter(MakeFuseBuilder& b, const TopoDS_Shape& s) {
    return shape_list_iter_new(b.inner.Modified(s));
}
inline std::unique_ptr<ShapeListIter>
fuse_generated_iter(MakeFuseBuilder& b, const TopoDS_Shape& s) {
    return shape_list_iter_new(b.inner.Generated(s));
}
inline std::unique_ptr<MakeFuseBuilder> new_make_fuse_builder() {
    return std::make_unique<MakeFuseBuilder>();
}

struct MakeCutBuilder {
    BRepAlgoAPI_Cut inner;
    MakeCutBuilder() = default;

    void build(const TopoDS_Shape& s1, const TopoDS_Shape& s2) {
        try {
            TopTools_ListOfShape args, tools;
            args.Append(s1);
            tools.Append(s2);
            inner.SetArguments(args);
            inner.SetTools(tools);
            inner.Build(Message_ProgressRange());
        } catch (const std::runtime_error&) { throw; }
        catch (...) { rethrow_occt_as_runtime_error(); }
    }
    bool is_done() const { return inner.IsDone(); }
    bool has_errors() const { return inner.HasErrors(); }
    std::unique_ptr<TopoDS_Shape> shape() {
        return std::make_unique<TopoDS_Shape>(inner.Shape());
    }
    bool is_deleted(const TopoDS_Shape& s) {
        return inner.IsDeleted(s) == Standard_True;
    }
};
inline std::unique_ptr<ShapeListIter>
cut_modified_iter(MakeCutBuilder& b, const TopoDS_Shape& s) {
    return shape_list_iter_new(b.inner.Modified(s));
}
inline std::unique_ptr<ShapeListIter>
cut_generated_iter(MakeCutBuilder& b, const TopoDS_Shape& s) {
    return shape_list_iter_new(b.inner.Generated(s));
}
inline std::unique_ptr<MakeCutBuilder> new_make_cut_builder() {
    return std::make_unique<MakeCutBuilder>();
}

struct MakeCommonBuilder {
    BRepAlgoAPI_Common inner;
    MakeCommonBuilder() = default;

    void build(const TopoDS_Shape& s1, const TopoDS_Shape& s2) {
        try {
            TopTools_ListOfShape args, tools;
            args.Append(s1);
            tools.Append(s2);
            inner.SetArguments(args);
            inner.SetTools(tools);
            inner.Build(Message_ProgressRange());
        } catch (const std::runtime_error&) { throw; }
        catch (...) { rethrow_occt_as_runtime_error(); }
    }
    bool is_done() const { return inner.IsDone(); }
    bool has_errors() const { return inner.HasErrors(); }
    std::unique_ptr<TopoDS_Shape> shape() {
        return std::make_unique<TopoDS_Shape>(inner.Shape());
    }
    bool is_deleted(const TopoDS_Shape& s) {
        return inner.IsDeleted(s) == Standard_True;
    }
};
inline std::unique_ptr<ShapeListIter>
common_modified_iter(MakeCommonBuilder& b, const TopoDS_Shape& s) {
    return shape_list_iter_new(b.inner.Modified(s));
}
inline std::unique_ptr<ShapeListIter>
common_generated_iter(MakeCommonBuilder& b, const TopoDS_Shape& s) {
    return shape_list_iter_new(b.inner.Generated(s));
}
inline std::unique_ptr<MakeCommonBuilder> new_make_common_builder() {
    return std::make_unique<MakeCommonBuilder>();
}
