// occt_sys/topo.hxx — Umbrella include for all topo shims.
//
// Included by the cxx bridge in src/topo.rs.
// The sub-headers form a dependency chain (each includes its predecessor),
// so including solid.hxx would transitively pull in everything.  The explicit
// list here documents the chain and is more readable to both humans and tools.
//
// Sourced from OCCT 7.9 documentation.
// No derivation from any other binding crate.

#pragma once

#include "topo/edge_polygon.hxx"
#include "topo/vertex.hxx"
#include "topo/edge.hxx"
#include "topo/wire.hxx"
#include "topo/face.hxx"
#include "topo/solid.hxx"
#include "topo/shape.hxx"
#include "topo/explorer.hxx"
#include "topo/mesh.hxx"
#include "topo/bool_op.hxx"
#include "topo/transform.hxx"
#include "topo/fillet.hxx"
#include "topo/chamfer.hxx"
#include "topo/offset.hxx"
