use crate::OcctError;

/// Error returned by [`OcShape::oc_common`].
#[derive(Debug)]
pub enum CommonError {
    /// The two input shapes do not intersect — OCCT performed the operation
    /// successfully but the result is an empty `TopoDS_Compound`.
    ///
    /// Unlike [`FuseError::DisjointInputs`], the empty compound carries no
    /// useful content and is not returned.
    NoIntersection,
    /// An OCCT exception was raised during the common operation.
    Occt(OcctError),
}

impl std::fmt::Display for CommonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoIntersection => write!(f, "common: inputs do not intersect; result is empty"),
            Self::Occt(e) => write!(f, "common: OCCT error: {e}"),
        }
    }
}

impl std::error::Error for CommonError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Occt(e) => Some(e),
            _ => None,
        }
    }
}
