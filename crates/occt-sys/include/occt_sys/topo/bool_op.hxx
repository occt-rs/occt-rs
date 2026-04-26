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
