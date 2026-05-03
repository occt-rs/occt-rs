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

// fuse_shapes: binary union via BRepAlgoAPI_Fuse.
//
// Uses the preferred empty-ctor + SetArguments/SetTools/Build pattern rather
// than the legacy two-arg constructor (per OCCT 7.4+ recommended usage).
// TopTools_ListOfShape is constructed on the C++ stack and never crosses the
// FFI boundary.
//
// Throws std::runtime_error (OCCT: wire format) on construction failure or
// if the operation reports errors.
inline std::unique_ptr<TopoDS_Shape> fuse_shapes(
    const TopoDS_Shape& s1,
    const TopoDS_Shape& s2)
{
    try {
        TopTools_ListOfShape args, tools;
        args.Append(s1);
        tools.Append(s2);

        BRepAlgoAPI_Fuse fuse;
        fuse.SetArguments(args);
        fuse.SetTools(tools);
        fuse.Build(Message_ProgressRange());

        if (!fuse.IsDone() || fuse.HasErrors()) {
            throw std::runtime_error("OCCT:BRepAlgoAPI_Fuse:Boolean operation failed");
        }
        return std::make_unique<TopoDS_Shape>(fuse.Shape());
    } catch (const std::runtime_error&) {
        throw;
    } catch (...) {
        rethrow_occt_as_runtime_error();
    }
}

// Reference: https://dev.opencascade.org/doc/refman/html/class_b_rep_algo_a_p_i___cut.html
// Reference: https://dev.opencascade.org/doc/refman/html/class_b_rep_algo_a_p_i___common.html

// cut_shapes: binary subtraction via BRepAlgoAPI_Cut.
//
// SetArguments receives the "object" (left operand, the shape being cut into);
// SetTools receives the "tool" (right operand, the shape being subtracted).
// Semantics: result = s1 − s2.
//
// For disjoint inputs, OCCT returns s1 unchanged (IsDone()==true, no error).
// The Rust layer receives a valid shape and does not need special-casing.
inline std::unique_ptr<TopoDS_Shape> cut_shapes(
    const TopoDS_Shape& s1,
    const TopoDS_Shape& s2)
{
    try {
        TopTools_ListOfShape args, tools;
        args.Append(s1);
        tools.Append(s2);

        BRepAlgoAPI_Cut cut;
        cut.SetArguments(args);
        cut.SetTools(tools);
        cut.Build(Message_ProgressRange());

        if (!cut.IsDone() || cut.HasErrors()) {
            throw std::runtime_error("OCCT:BRepAlgoAPI_Cut:Boolean operation failed");
        }
        return std::make_unique<TopoDS_Shape>(cut.Shape());
    } catch (const std::runtime_error&) {
        throw;
    } catch (...) {
        rethrow_occt_as_runtime_error();
    }
}

// common_shapes: binary intersection via BRepAlgoAPI_Common.
//
// For disjoint inputs, OCCT returns an empty TopoDS_Compound (IsDone()==true).
// The Rust layer detects this via ShapeType == Compound and maps to
// CommonError::NoIntersection.
inline std::unique_ptr<TopoDS_Shape> common_shapes(
    const TopoDS_Shape& s1,
    const TopoDS_Shape& s2)
{
    try {
        TopTools_ListOfShape args, tools;
        args.Append(s1);
        tools.Append(s2);

        BRepAlgoAPI_Common common;
        common.SetArguments(args);
        common.SetTools(tools);
        common.Build(Message_ProgressRange());

        if (!common.IsDone() || common.HasErrors()) {
            throw std::runtime_error("OCCT:BRepAlgoAPI_Common:Boolean operation failed");
        }
        return std::make_unique<TopoDS_Shape>(common.Shape());
    } catch (const std::runtime_error&) {
        throw;
    } catch (...) {
        rethrow_occt_as_runtime_error();
    }
}
