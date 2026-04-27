use crate::OcctError;

/// Error returned by [`OcShape::fuse`].
#[derive(Debug)]
pub enum FuseError {
    /// The two input shapes are disjoint — OCCT performed the operation successfully
    /// but the result is a `TopoDS_Compound` containing both inputs unchanged.
    /// The compound is returned so the caller can opt in to using it.
    DisjointInputs(crate::topo::OcShape),
    /// An OCCT exception was raised during the fuse operation.
    Occt(OcctError),
}

impl std::fmt::Display for FuseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DisjointInputs(_) => write!(f, "fuse: inputs are disjoint; result is a compound"),
            Self::Occt(e) => write!(f, "fuse: OCCT error: {e}"),
        }
    }
}

impl std::error::Error for FuseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Occt(e) => Some(e),
            _ => None,
        }
    }
}
