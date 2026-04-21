// occt_sys/exception.hxx — OCCT exception → std::runtime_error marshalling.
//
// Every fallible shim catches Standard_Failure and rethrows as
// std::runtime_error with what() = "OCCT:<DynamicTypeName>:<message>".
// occt_rs::error::OcctError parses this wire format.
//
// Sourced from OCCT 7.9 documentation.
// No derivation from any other binding crate.

#pragma once

#include <stdexcept>
#include <string>
#include <Standard_Failure.hxx>

[[noreturn]] inline void rethrow_occt_as_runtime_error() {
    try {
        throw;
    } catch (const Standard_Failure& e) {
        throw std::runtime_error(
            std::string("OCCT:") + e.DynamicType()->Name() + ":" + e.GetMessageString()
        );
    } catch (const std::exception& e) {
        throw std::runtime_error(std::string("OCCT:Other:") + e.what());
    } catch (...) {
        throw std::runtime_error("OCCT:Other:unknown C++ exception");
    }
}
