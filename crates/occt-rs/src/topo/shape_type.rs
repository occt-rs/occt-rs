// Reference: https://dev.opencascade.org/doc/refman/html/_top_abs___shape_enum_8hxx.html
// Sourced from OCCT 7.9 documentation. No derivation from any other binding crate.

/// Corresponds to `TopAbs_ShapeEnum`. Integer values match OCCT constants exactly;
/// `From<i32>` is infallible — unknown values map to `Shape` (the wildcard).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShapeType {
    Compound = 0,
    CompSolid = 1,
    Solid = 2,
    Shell = 3,
    Face = 4,
    Wire = 5,
    Edge = 6,
    Vertex = 7,
    /// `TopAbs_SHAPE` — wildcard; any shape type. Also the fallback for unknown values.
    Shape = 8,
}

impl From<i32> for ShapeType {
    fn from(v: i32) -> Self {
        match v {
            0 => Self::Compound,
            1 => Self::CompSolid,
            2 => Self::Solid,
            3 => Self::Shell,
            4 => Self::Face,
            5 => Self::Wire,
            6 => Self::Edge,
            7 => Self::Vertex,
            _ => Self::Shape,
        }
    }
}
