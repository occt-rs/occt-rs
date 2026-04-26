//! Error types for the `occt-rs` safe API.
//!
//! # Wire format
//!
//! OCCT C++ exceptions are caught in `occt-sys`'s shim layer and rethrown as
//! `std::runtime_error` with a `what()` string of the form:
//!
//! ```text
//! OCCT:<DynamicTypeName>:<GetMessageString>
//! ```
//!
//! `DynamicTypeName` is the string returned by OCCT's own RTTI
//! (`Standard_Failure::DynamicType()->Name()`), e.g.
//! `"Standard_ConstructionError"` or `"gp_VectorWithNullMagnitude"`.
//! The `From<cxx::Exception>` implementation parses this format to produce
//! a fully typed [`OcctError`].

mod fuse_error;
pub use fuse_error::*;
mod common_error;
pub use common_error::*;

use std::fmt;

/// The typed error returned by all fallible `occt-rs` operations.
#[derive(Debug, Clone)]
pub struct OcctError {
    /// Structured kind, derived from the OCCT exception class name.
    pub kind: OcctErrorKind,
    /// The message string from `Standard_Failure::GetMessageString()`.
    pub message: String,
}

/// Coarse-grained classification of OCCT exceptions.
///
/// `#[non_exhaustive]` so that adding variants as new modules are bound is
/// not a breaking change.  Match arms should always include a `_` fallback.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum OcctErrorKind {
    /// `Standard_ConstructionError` — geometry that cannot be constructed
    /// (e.g. `OcDir` from a zero vector).
    ConstructionError,
    /// `gp_VectorWithNullMagnitude` — operation requiring non-zero magnitude
    /// called on a zero vector.
    NullMagnitude,
    /// `Standard_DomainError` — valid input outside the operation's domain.
    DomainError,
    /// `Standard_OutOfRange` — index out of permitted range.
    OutOfRange,
    /// Any other `Standard_Failure` subclass; the inner string is the OCCT
    /// dynamic type name.
    Other(String),
}

impl OcctErrorKind {
    fn from_class_name(name: &str) -> Self {
        match name {
            "Standard_ConstructionError" => Self::ConstructionError,
            "gp_VectorWithNullMagnitude" => Self::NullMagnitude,
            "Standard_DomainError" => Self::DomainError,
            "Standard_OutOfRange" => Self::OutOfRange,
            other => Self::Other(other.to_owned()),
        }
    }
}

impl fmt::Display for OcctError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.message)
    }
}

impl std::error::Error for OcctError {}

impl From<cxx::Exception> for OcctError {
    fn from(e: cxx::Exception) -> Self {
        parse_occt_exception(e.what())
    }
}

pub(crate) fn parse_occt_exception(what: &str) -> OcctError {
    if let Some(rest) = what.strip_prefix("OCCT:") {
        // Split on first ':' only; the message itself may contain colons.
        if let Some((kind_str, message)) = rest.split_once(':') {
            return OcctError {
                kind: OcctErrorKind::from_class_name(kind_str),
                message: message.to_owned(),
            };
        }
    }
    OcctError {
        kind: OcctErrorKind::Other(String::new()),
        message: what.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_construction_error() {
        let e = parse_occt_exception("OCCT:Standard_ConstructionError:null magnitude in gp_Dir");
        assert_eq!(e.kind, OcctErrorKind::ConstructionError);
        assert_eq!(e.message, "null magnitude in gp_Dir");
    }

    #[test]
    fn parse_null_magnitude() {
        let e =
            parse_occt_exception("OCCT:gp_VectorWithNullMagnitude:gp_Vec : cannot be normalized");
        assert_eq!(e.kind, OcctErrorKind::NullMagnitude);
    }

    #[test]
    fn parse_message_with_internal_colons() {
        // Splitting on first ':' only must not truncate the message.
        let e = parse_occt_exception(
            "OCCT:Standard_DomainError:gp_Vec : coplanar: cannot compute AngleWithRef",
        );
        assert_eq!(e.kind, OcctErrorKind::DomainError);
        assert_eq!(e.message, "gp_Vec : coplanar: cannot compute AngleWithRef");
    }

    #[test]
    fn parse_unknown_prefix_falls_back() {
        let e = parse_occt_exception("something unexpected");
        assert_eq!(e.kind, OcctErrorKind::Other(String::new()));
    }
}
