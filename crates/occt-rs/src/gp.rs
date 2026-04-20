//! Pure-Rust geometric primitive types corresponding to the OCCT `gp` package.
//!
//! `OcPnt`, `OcVec`, and `OcDir` store their coordinates natively in Rust.
//! All arithmetic and geometric predicates are implemented here without
//! crossing the FFI boundary.  The only FFI interaction is `to_ffi()`, called
//! exclusively when materialising a value for consumption by a higher-level
//! OCCT API (BRep construction, surface queries, etc.).
//!
//! # Naming
//!
//! The `Oc` prefix marks unqualified usage as explicitly serving
//! OpenCascade, avoiding ambiguity with `std` types (`Vec`, etc.) and
//! distinguishing from any future Rust-native geometry layer.

use crate::error::{OcctError, OcctErrorKind};
use occt_sys::ffi;

// ── Constants ─────────────────────────────────────────────────────────────
// gp::Resolution() in OCCT is defined as 1e-15 in gp.hxx.
// Used as the null-magnitude threshold for OcDir construction.
const GP_RESOLUTION: f64 = 1e-15;

// ── OcPnt ─────────────────────────────────────────────────────────────────

/// A 3-D Cartesian point.
///
/// Corresponds to `gp_Pnt` in OCCT.
/// Reference: <https://dev.opencascade.org/doc/refman/html/classgp___pnt.html>
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OcPnt {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl OcPnt {
    #[inline]
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub fn origin() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    /// Euclidean distance to `other`.
    #[inline]
    pub fn distance(&self, other: &OcPnt) -> f64 {
        self.square_distance(other).sqrt()
    }

    /// Squared Euclidean distance to `other`.  Avoids the `sqrt` when only
    /// relative comparison is needed.
    #[inline]
    pub fn square_distance(&self, other: &OcPnt) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        dx * dx + dy * dy + dz * dz
    }

    /// Vector from `self` to `other` (`other - self`).
    #[inline]
    pub fn vector_to(&self, other: &OcPnt) -> OcVec {
        OcVec {
            x: other.x - self.x,
            y: other.y - self.y,
            z: other.z - self.z,
        }
    }

    /// Materialises a `gp_Pnt` for passing to an OCCT API.
    /// This is the only point at which an `OcPnt` crosses the FFI boundary.
    #[inline]
    pub(crate) fn to_ffi(&self) -> cxx::UniquePtr<ffi::GpPnt> {
        ffi::new_gp_pnt_xyz(self.x, self.y, self.z)
    }
}

/// `point + vector → point`
impl std::ops::Add<OcVec> for OcPnt {
    type Output = OcPnt;
    #[inline]
    fn add(self, v: OcVec) -> OcPnt {
        OcPnt {
            x: self.x + v.x,
            y: self.y + v.y,
            z: self.z + v.z,
        }
    }
}

/// `point - vector → point`
impl std::ops::Sub<OcVec> for OcPnt {
    type Output = OcPnt;
    #[inline]
    fn sub(self, v: OcVec) -> OcPnt {
        OcPnt {
            x: self.x - v.x,
            y: self.y - v.y,
            z: self.z - v.z,
        }
    }
}

/// `point - point → vector` (the displacement from rhs to lhs)
impl std::ops::Sub for OcPnt {
    type Output = OcVec;
    #[inline]
    fn sub(self, other: OcPnt) -> OcVec {
        OcVec {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

// ── OcVec ─────────────────────────────────────────────────────────────────

/// A 3-D vector with arbitrary magnitude.
///
/// Corresponds to `gp_Vec` in OCCT.
/// Reference: <https://dev.opencascade.org/doc/refman/html/classgp___vec.html>
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OcVec {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl OcVec {
    #[inline]
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    #[inline]
    pub fn magnitude(&self) -> f64 {
        self.square_magnitude().sqrt()
    }

    #[inline]
    pub fn square_magnitude(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// Scalar (dot) product.
    #[inline]
    pub fn dot(&self, other: &OcVec) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Cross product.  The result has zero magnitude when `self` and `other`
    /// are parallel; use [`OcVec::normalize`] on the result if a direction is
    /// needed and check for `NullMagnitude` accordingly.
    #[inline]
    pub fn cross(&self, other: &OcVec) -> OcVec {
        OcVec {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    /// Triple scalar product: `self · (v1 × v2)`.
    #[inline]
    pub fn dot_cross(&self, v1: &OcVec, v2: &OcVec) -> f64 {
        self.dot(&v1.cross(v2))
    }

    /// Magnitude of the cross product `‖self × other‖`.
    #[inline]
    pub fn cross_magnitude(&self, other: &OcVec) -> f64 {
        self.cross(other).magnitude()
    }

    /// Normalises `self` into a unit-direction vector.
    ///
    /// Returns `Err(NullMagnitude)` when the magnitude is ≤ `gp::Resolution`.
    pub fn normalize(&self) -> Result<OcDir, OcctError> {
        OcDir::new(self.x, self.y, self.z)
    }

    /// Angle between `self` and `other` in [0, π] radians.
    ///
    /// Returns `Err(NullMagnitude)` if either vector has zero magnitude.
    pub fn angle(&self, other: &OcVec) -> Result<f64, OcctError> {
        let mag_self = self.magnitude();
        let mag_other = other.magnitude();
        if mag_self <= GP_RESOLUTION || mag_other <= GP_RESOLUTION {
            return Err(OcctError {
                kind: OcctErrorKind::NullMagnitude,
                message: "angle requires non-zero magnitude on both vectors".to_owned(),
            });
        }
        let cos_a = (self.dot(other) / (mag_self * mag_other)).clamp(-1.0, 1.0);
        Ok(cos_a.acos())
    }

    /// Signed angle in (−π, π] radians.  `vref` defines the positive rotation
    /// sense: the result is positive when `self × other` has the same
    /// orientation as `vref`.
    ///
    /// Returns `Err(NullMagnitude)` on zero magnitude, `Err(DomainError)` when
    /// `self` and `other` are parallel (cross product is zero, sign undefined).
    pub fn angle_with_ref(&self, other: &OcVec, vref: &OcVec) -> Result<f64, OcctError> {
        let unsigned = self.angle(other)?;
        if unsigned < GP_RESOLUTION || (std::f64::consts::PI - unsigned) < GP_RESOLUTION {
            // Parallel or anti-parallel: cross product is ~zero, sign undefined.
            return Err(OcctError {
                kind: OcctErrorKind::DomainError,
                message: "angle_with_ref: vectors are parallel, sign is undefined".to_owned(),
            });
        }
        let cross = self.cross(other);
        if cross.dot(vref) >= 0.0 {
            Ok(unsigned)
        } else {
            Ok(-unsigned)
        }
    }

    /// Materialises a `gp_Vec` for passing to an OCCT API.
    #[inline]
    pub(crate) fn to_ffi(&self) -> cxx::UniquePtr<ffi::GpVec> {
        ffi::new_gp_vec_xyz(self.x, self.y, self.z)
    }
}

impl std::ops::Add for OcVec {
    type Output = OcVec;
    #[inline]
    fn add(self, other: OcVec) -> OcVec {
        OcVec {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl std::ops::Sub for OcVec {
    type Output = OcVec;
    #[inline]
    fn sub(self, other: OcVec) -> OcVec {
        OcVec {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl std::ops::Mul<f64> for OcVec {
    type Output = OcVec;
    #[inline]
    fn mul(self, s: f64) -> OcVec {
        OcVec {
            x: self.x * s,
            y: self.y * s,
            z: self.z * s,
        }
    }
}

impl std::ops::Mul<OcVec> for f64 {
    type Output = OcVec;
    #[inline]
    fn mul(self, v: OcVec) -> OcVec {
        OcVec {
            x: v.x * self,
            y: v.y * self,
            z: v.z * self,
        }
    }
}

impl std::ops::Div<f64> for OcVec {
    type Output = OcVec;
    #[inline]
    fn div(self, s: f64) -> OcVec {
        OcVec {
            x: self.x / s,
            y: self.y / s,
            z: self.z / s,
        }
    }
}

impl std::ops::Neg for OcVec {
    type Output = OcVec;
    #[inline]
    fn neg(self) -> OcVec {
        OcVec {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl std::ops::AddAssign for OcVec {
    #[inline]
    fn add_assign(&mut self, other: OcVec) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

impl std::ops::SubAssign for OcVec {
    #[inline]
    fn sub_assign(&mut self, other: OcVec) {
        self.x -= other.x;
        self.y -= other.y;
        self.z -= other.z;
    }
}

impl std::ops::MulAssign<f64> for OcVec {
    #[inline]
    fn mul_assign(&mut self, s: f64) {
        self.x *= s;
        self.y *= s;
        self.z *= s;
    }
}

impl std::ops::DivAssign<f64> for OcVec {
    #[inline]
    fn div_assign(&mut self, s: f64) {
        self.x /= s;
        self.y /= s;
        self.z /= s;
    }
}

// ── OcDir ─────────────────────────────────────────────────────────────────

/// A unit-length direction in 3-D space.
///
/// Corresponds to `gp_Dir` in OCCT.
/// Reference: <https://dev.opencascade.org/doc/refman/html/classgp___dir.html>
///
/// Fields are private to maintain the unit-magnitude invariant.
/// Construction validates and normalises; all methods can assume `‖self‖ = 1`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OcDir {
    x: f64,
    y: f64,
    z: f64,
}

impl OcDir {
    /// Constructs a unit direction from raw coordinates.
    ///
    /// Returns `Err(ConstructionError)` when
    /// `sqrt(x² + y² + z²) ≤ gp::Resolution` (1e-15).
    pub fn new(x: f64, y: f64, z: f64) -> Result<Self, OcctError> {
        let mag = (x * x + y * y + z * z).sqrt();
        if mag <= GP_RESOLUTION {
            return Err(OcctError {
                kind: OcctErrorKind::ConstructionError,
                message: "cannot construct OcDir from a zero-magnitude vector".to_owned(),
            });
        }
        Ok(Self {
            x: x / mag,
            y: y / mag,
            z: z / mag,
        })
    }

    /// Constructs a unit direction from an `OcVec`.
    ///
    /// Returns `Err(ConstructionError)` when the vector has zero magnitude.
    #[inline]
    pub fn from_vec(v: &OcVec) -> Result<Self, OcctError> {
        Self::new(v.x, v.y, v.z)
    }

    #[inline]
    pub fn x(&self) -> f64 {
        self.x
    }
    #[inline]
    pub fn y(&self) -> f64 {
        self.y
    }
    #[inline]
    pub fn z(&self) -> f64 {
        self.z
    }

    /// Returns the direction as a unit `OcVec`.
    #[inline]
    pub fn to_vec(&self) -> OcVec {
        OcVec {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }

    /// Reverses the direction.
    #[inline]
    pub fn reversed(&self) -> OcDir {
        OcDir {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }

    /// Scalar (dot) product of two unit directions.
    /// Result is in [−1, 1] and equals cos(angle).
    #[inline]
    pub fn dot(&self, other: &OcDir) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Triple scalar product: `self · (v1 × v2)`.
    #[inline]
    pub fn dot_cross(&self, v1: &OcDir, v2: &OcDir) -> f64 {
        self.to_vec().dot_cross(&v1.to_vec(), &v2.to_vec())
    }

    /// Angle between two unit directions in [0, π] radians.
    ///
    /// Both operands are unit vectors so this is always in domain;
    /// the result is infallible.
    #[inline]
    pub fn angle(&self, other: &OcDir) -> f64 {
        self.dot(other).clamp(-1.0, 1.0).acos()
    }

    /// Signed angle in (−π, π] radians.
    ///
    /// Returns `Err(DomainError)` when `self` and `other` are parallel or
    /// anti-parallel (cross product is zero; sign is undefined).
    pub fn angle_with_ref(&self, other: &OcDir, vref: &OcDir) -> Result<f64, OcctError> {
        let unsigned = self.angle(other);
        if unsigned < GP_RESOLUTION || (std::f64::consts::PI - unsigned) < GP_RESOLUTION {
            return Err(OcctError {
                kind: OcctErrorKind::DomainError,
                message: "angle_with_ref: directions are parallel, sign is undefined".to_owned(),
            });
        }
        let cross = self.to_vec().cross(&other.to_vec());
        if cross.dot(&vref.to_vec()) >= 0.0 {
            Ok(unsigned)
        } else {
            Ok(-unsigned)
        }
    }

    /// Cross product of two unit directions.
    ///
    /// Returns `Err(DomainError)` when the directions are parallel
    /// (cross product would be zero magnitude).
    pub fn cross(&self, other: &OcDir) -> Result<OcDir, OcctError> {
        let c = self.to_vec().cross(&other.to_vec());
        OcDir::from_vec(&c).map_err(|_| OcctError {
            kind: OcctErrorKind::DomainError,
            message: "cross product of parallel directions has zero magnitude".to_owned(),
        })
    }

    /// Tolerance-based equality: `angle(self, other) ≤ ang_tol`.
    #[inline]
    pub fn is_equal(&self, other: &OcDir, ang_tol: f64) -> bool {
        self.angle(other) <= ang_tol
    }

    /// Returns `true` when the directions are approximately perpendicular:
    /// `|angle − π/2| ≤ ang_tol`.
    #[inline]
    pub fn is_normal(&self, other: &OcDir, ang_tol: f64) -> bool {
        (self.angle(other) - std::f64::consts::FRAC_PI_2).abs() <= ang_tol
    }

    /// Returns `true` when the directions are approximately opposite:
    /// `π − angle ≤ ang_tol`.
    #[inline]
    pub fn is_opposite(&self, other: &OcDir, ang_tol: f64) -> bool {
        std::f64::consts::PI - self.angle(other) <= ang_tol
    }

    /// Returns `true` when the directions are approximately parallel
    /// (including anti-parallel): `angle ≤ ang_tol || π − angle ≤ ang_tol`.
    #[inline]
    pub fn is_parallel(&self, other: &OcDir, ang_tol: f64) -> bool {
        let a = self.angle(other);
        a <= ang_tol || (std::f64::consts::PI - a) <= ang_tol
    }

    /// Materialises a `gp_Dir` for passing to an OCCT API.
    ///
    /// `self` is already validated and normalised, so this should never
    /// return `Err`.  The `expect` here guards against invariant violations;
    /// a panic indicates a bug in `OcDir`'s construction logic.
    #[inline]
    pub(crate) fn to_ffi(&self) -> cxx::UniquePtr<ffi::GpDir> {
        ffi::new_gp_dir_xyz(self.x, self.y, self.z)
            .expect("pre-normalised OcDir failed to materialise — invariant violated")
    }
}

impl std::ops::Neg for OcDir {
    type Output = OcDir;
    #[inline]
    fn neg(self) -> OcDir {
        self.reversed()
    }
}
// ── OcAx1 ─────────────────────────────────────────────────────────────────

/// An axis in 3-D space: an origin point and a unit direction.
///
/// Corresponds to `gp_Ax1` in OCCT.
/// Reference: <https://dev.opencascade.org/doc/refman/html/classgp___ax1.html>
///
/// Construction is infallible: any `OcPnt` and `OcDir` pair is a valid axis.
/// The direction invariant (unit magnitude) is enforced by `OcDir`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OcAx1 {
    location: OcPnt,
    direction: OcDir,
}

impl OcAx1 {
    /// Creates an axis from an origin point and a unit direction.
    #[inline]
    pub fn new(location: OcPnt, direction: OcDir) -> Self {
        Self {
            location,
            direction,
        }
    }

    /// The Z axis of the reference coordinate system: origin (0,0,0),
    /// direction (0,0,1).  Corresponds to `gp_Ax1()` default constructor.
    #[inline]
    pub fn z_axis() -> Self {
        Self {
            location: OcPnt::origin(),
            // SAFETY: (0,0,1) has unit magnitude; unwrap cannot panic.
            direction: OcDir::new(0.0, 0.0, 1.0).expect("unit Z direction"),
        }
    }

    /// Returns the origin (location point) of this axis.
    #[inline]
    pub fn location(&self) -> OcPnt {
        self.location
    }

    /// Returns the unit direction of this axis.
    #[inline]
    pub fn direction(&self) -> OcDir {
        self.direction
    }

    /// Returns a new axis with the direction reversed.
    ///
    /// Corresponds to `gp_Ax1::Reversed()`.
    #[inline]
    pub fn reversed(&self) -> Self {
        Self {
            location: self.location,
            direction: self.direction.reversed(),
        }
    }

    /// Translates this axis by vector `v` (shifts the origin; direction unchanged).
    #[inline]
    pub fn translated(&self, v: OcVec) -> Self {
        Self {
            location: self.location + v,
            direction: self.direction,
        }
    }

    /// Angle between the directions of `self` and `other`, in [0, π] radians.
    ///
    /// Corresponds to `gp_Ax1::Angle()`.
    /// Note: the OCCT reference documentation states the range as [0, 2π] but
    /// the underlying `gp_Dir::Angle` uses `acos`, giving [0, π].  This
    /// implementation matches the actual OCCT behaviour.
    #[inline]
    pub fn angle(&self, other: &OcAx1) -> f64 {
        self.direction.angle(&other.direction)
    }

    /// Returns `true` when `self` and `other` share the same infinite line,
    /// within tolerances.
    ///
    /// Conditions (from `gp_Ax1::IsCoaxial` documentation):
    /// - `self.angle(other) ≤ ang_tol`
    /// - distance from `self.location()` to the line through `other` ≤ `lin_tol`
    /// - distance from `other.location()` to the line through `self` ≤ `lin_tol`
    pub fn is_coaxial(&self, other: &OcAx1, ang_tol: f64, lin_tol: f64) -> bool {
        if self.angle(other) > ang_tol {
            return false;
        }
        point_to_axis_distance(&self.location, &other.location, &other.direction) <= lin_tol
            && point_to_axis_distance(&other.location, &self.location, &self.direction) <= lin_tol
    }

    /// Returns `true` when the directions are approximately perpendicular:
    /// `|angle − π/2| ≤ ang_tol`.
    ///
    /// Corresponds to `gp_Ax1::IsNormal()`.
    #[inline]
    pub fn is_normal(&self, other: &OcAx1, ang_tol: f64) -> bool {
        self.direction.is_normal(&other.direction, ang_tol)
    }

    /// Returns `true` when the directions are approximately opposite (anti-parallel):
    /// `π − angle ≤ ang_tol`.
    ///
    /// Corresponds to `gp_Ax1::IsOpposite()`.
    #[inline]
    pub fn is_opposite(&self, other: &OcAx1, ang_tol: f64) -> bool {
        self.direction.is_opposite(&other.direction, ang_tol)
    }

    /// Returns `true` when the directions are approximately parallel (same or
    /// opposite orientation): `angle ≤ ang_tol || π − angle ≤ ang_tol`.
    ///
    /// Corresponds to `gp_Ax1::IsParallel()`.
    #[inline]
    pub fn is_parallel(&self, other: &OcAx1, ang_tol: f64) -> bool {
        self.direction.is_parallel(&other.direction, ang_tol)
    }

    /// Materialises a `gp_Ax1` for passing to an OCCT API.
    /// This is the only point at which an `OcAx1` crosses the FFI boundary.
    #[inline]
    pub(crate) fn to_ffi(&self) -> cxx::UniquePtr<ffi::GpAx1> {
        ffi::new_gp_ax1(
            self.location.x,
            self.location.y,
            self.location.z,
            self.direction.x(),
            self.direction.y(),
            self.direction.z(),
        )
        .expect("pre-validated OcAx1 failed to materialise — invariant violated")
    }
}

impl std::ops::Neg for OcAx1 {
    type Output = OcAx1;
    #[inline]
    fn neg(self) -> OcAx1 {
        self.reversed()
    }
}

// ── OcAx2 ─────────────────────────────────────────────────────────────────

/// A right-handed coordinate system in 3-D space.
///
/// Defined by an origin point and three mutually orthogonal unit vectors:
/// `direction` (the "main" or "Z" direction), `x_dir`, and `y_dir`.
/// The right-handedness invariant is: `direction = x_dir × y_dir`.
///
/// Corresponds to `gp_Ax2` in OCCT.
/// Reference: <https://dev.opencascade.org/doc/refman/html/classgp___ax2.html>
///
/// Fields are private to maintain the right-handedness and unit-magnitude
/// invariants.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OcAx2 {
    location: OcPnt,
    /// Main/"Z" direction.
    direction: OcDir,
    x_dir: OcDir,
    /// Derived: `direction × x_dir`.  Stored to make `y_direction()` O(1).
    y_dir: OcDir,
}

impl OcAx2 {
    /// Creates a right-handed coordinate system from an origin, a main
    /// direction `n`, and a hint vector `vx` for the X direction.
    ///
    /// The actual X direction is computed from the OCCT-documented formula:
    /// `actual_x = (n × vx) × n`
    /// which is the projection of `vx` onto the plane perpendicular to `n`,
    /// normalised.  Y is then `n × actual_x`.
    ///
    /// Returns `Err(ConstructionError)` when `n` and `vx` are parallel (same
    /// or opposite orientation), because the cross product would be zero.
    ///
    /// Corresponds to `gp_Ax2(P, N, Vx)`.
    /// Reference: <https://dev.opencascade.org/doc/refman/html/classgp___ax2.html>
    pub fn new(location: OcPnt, direction: OcDir, x_dir: OcDir) -> Result<Self, OcctError> {
        let n = direction.to_vec();
        let vx = x_dir.to_vec();

        // (n × vx) × n  — projects vx onto the plane perpendicular to n.
        // Zero if n ∥ vx.
        let cross1 = n.cross(&vx); // n × vx
        let actual_x_vec = cross1.cross(&n); // (n × vx) × n

        let actual_x = OcDir::from_vec(&actual_x_vec).map_err(|_| OcctError {
            kind: OcctErrorKind::ConstructionError,
            message: "OcAx2: direction and x_dir are parallel".to_owned(),
        })?;

        // Right-hand rule: y = n × actual_x.  Since n and actual_x are unit
        // and mutually perpendicular, the cross product is also unit.
        let y_vec = n.cross(&actual_x.to_vec());
        let y_dir = OcDir::from_vec(&y_vec)
            .expect("y direction non-zero: n and actual_x are perpendicular — invariant");

        Ok(Self {
            location,
            direction,
            x_dir: actual_x,
            y_dir,
        })
    }

    /// Creates a right-handed coordinate system from an origin and a main
    /// direction, computing the X and Y directions automatically.
    ///
    /// The X direction is chosen perpendicular to `direction` using a
    /// standard axis-independent algorithm: the least-component axis of
    /// `direction` is used as the reference vector, then projected.
    /// This algorithm is independent of OCCT's `gp_Ax2(P, V)` implementation;
    /// its result may differ from OCCT's for the same inputs.
    ///
    /// Use [`OcAx2::new`] when an explicit X direction is required.
    pub fn with_direction(location: OcPnt, direction: OcDir) -> Self {
        // Pick the standard basis vector whose component along `direction`
        // has the smallest absolute value — guaranteeing non-parallelism.
        let (ax, ay, az) = (
            direction.x().abs(),
            direction.y().abs(),
            direction.z().abs(),
        );
        let reference = if ax <= ay && ax <= az {
            OcVec::new(1.0, 0.0, 0.0)
        } else if ay <= az {
            OcVec::new(0.0, 1.0, 0.0)
        } else {
            OcVec::new(0.0, 0.0, 1.0)
        };

        // x_raw = direction × reference.  Non-zero because reference is not
        // parallel to direction (we chose the least-aligned axis).
        let d = direction.to_vec();
        let x_raw = d.cross(&reference);
        let x_dir =
            OcDir::from_vec(&x_raw).expect("reference not parallel to direction — invariant");

        // y = direction × x_dir.  Unit because direction ⊥ x_dir.
        let y_vec = d.cross(&x_dir.to_vec());
        let y_dir =
            OcDir::from_vec(&y_vec).expect("y direction non-zero by right-hand rule — invariant");

        Self {
            location,
            direction,
            x_dir,
            y_dir,
        }
    }

    /// The reference coordinate system OXYZ: origin (0,0,0), Z axis (0,0,1),
    /// X axis (1,0,0), Y axis (0,1,0).
    ///
    /// Corresponds to `gp_Ax2()` default constructor.
    pub fn oxyz() -> Self {
        // Constructed with explicit unit vectors — unwraps cannot panic.
        let direction = OcDir::new(0.0, 0.0, 1.0).expect("unit Z");
        let x_dir = OcDir::new(1.0, 0.0, 0.0).expect("unit X");
        let y_dir = OcDir::new(0.0, 1.0, 0.0).expect("unit Y");
        Self {
            location: OcPnt::origin(),
            direction,
            x_dir,
            y_dir,
        }
    }

    /// Returns the origin (location point) of this coordinate system.
    #[inline]
    pub fn location(&self) -> OcPnt {
        self.location
    }

    /// Returns the main ("Z") direction of this coordinate system.
    ///
    /// Corresponds to `gp_Ax2::Direction()`.
    #[inline]
    pub fn direction(&self) -> OcDir {
        self.direction
    }

    /// Returns the X direction of this coordinate system.
    ///
    /// Corresponds to `gp_Ax2::XDirection()`.
    #[inline]
    pub fn x_direction(&self) -> OcDir {
        self.x_dir
    }

    /// Returns the Y direction of this coordinate system.
    ///
    /// Computed as `direction × x_direction` and stored at construction.
    /// Corresponds to `gp_Ax2::YDirection()`.
    #[inline]
    pub fn y_direction(&self) -> OcDir {
        self.y_dir
    }

    /// Returns the main axis (origin + main direction) as an `OcAx1`.
    ///
    /// Corresponds to `gp_Ax2::Axis()`.
    #[inline]
    pub fn axis(&self) -> OcAx1 {
        OcAx1::new(self.location, self.direction)
    }

    /// Angle between the main directions of `self` and `other`, in [0, π] radians.
    ///
    /// Corresponds to `gp_Ax2::Angle()`.
    #[inline]
    pub fn angle(&self, other: &OcAx2) -> f64 {
        self.direction.angle(&other.direction)
    }

    /// Translates this coordinate system by vector `v` (shifts the origin;
    /// directions unchanged).
    #[inline]
    pub fn translated(&self, v: OcVec) -> Self {
        Self {
            location: self.location + v,
            ..*self
        }
    }

    /// Materialises a `gp_Ax2` for passing to an OCCT API.
    /// This is the only point at which an `OcAx2` crosses the FFI boundary.
    #[inline]
    pub(crate) fn to_ffi(&self) -> cxx::UniquePtr<ffi::GpAx2> {
        ffi::new_gp_ax2(
            self.location.x,
            self.location.y,
            self.location.z,
            self.direction.x(),
            self.direction.y(),
            self.direction.z(),
            self.x_dir.x(),
            self.x_dir.y(),
            self.x_dir.z(),
        )
        .expect("pre-validated OcAx2 failed to materialise — invariant violated")
    }
}

// ── Internal geometry helpers ─────────────────────────────────────────────

/// Distance from `point` to the infinite line defined by `origin` and `direction`.
///
/// Formula: ‖(point − origin) × direction‖
/// Valid because `direction` is a unit vector.
#[inline]
fn point_to_axis_distance(point: &OcPnt, origin: &OcPnt, direction: &OcDir) -> f64 {
    let op = OcVec::new(point.x - origin.x, point.y - origin.y, point.z - origin.z);
    op.cross(&direction.to_vec()).magnitude()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::{FRAC_PI_2, PI};

    // ── OcPnt ────────────────────────────────────────────────────────────

    #[test]
    fn pnt_distance() {
        let a = OcPnt::new(0.0, 0.0, 0.0);
        let b = OcPnt::new(3.0, 4.0, 0.0);
        assert!((a.distance(&b) - 5.0).abs() < 1e-12);
    }

    #[test]
    fn pnt_vector_to() {
        let a = OcPnt::new(1.0, 0.0, 0.0);
        let b = OcPnt::new(4.0, 0.0, 0.0);
        let v = a.vector_to(&b);
        assert_eq!(v, OcVec::new(3.0, 0.0, 0.0));
    }

    #[test]
    fn pnt_sub_gives_vec() {
        let a = OcPnt::new(3.0, 0.0, 0.0);
        let b = OcPnt::new(1.0, 0.0, 0.0);
        assert_eq!(a - b, OcVec::new(2.0, 0.0, 0.0));
    }

    // ── OcVec ────────────────────────────────────────────────────────────

    #[test]
    fn vec_dot() {
        let a = OcVec::new(1.0, 0.0, 0.0);
        let b = OcVec::new(0.0, 1.0, 0.0);
        assert_eq!(a.dot(&b), 0.0);
        assert_eq!(a.dot(&a), 1.0);
    }

    #[test]
    fn vec_cross() {
        let x = OcVec::new(1.0, 0.0, 0.0);
        let y = OcVec::new(0.0, 1.0, 0.0);
        assert_eq!(x.cross(&y), OcVec::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn vec_normalize_zero_fails() {
        assert!(OcVec::zero().normalize().is_err());
    }

    #[test]
    fn vec_angle_right_angle() {
        let x = OcVec::new(1.0, 0.0, 0.0);
        let y = OcVec::new(0.0, 1.0, 0.0);
        let a = x.angle(&y).unwrap();
        assert!((a - FRAC_PI_2).abs() < 1e-12);
    }

    #[test]
    fn vec_angle_zero_vector_fails() {
        let x = OcVec::new(1.0, 0.0, 0.0);
        assert!(x.angle(&OcVec::zero()).is_err());
    }

    #[test]
    fn vec_ops_no_alloc() {
        // All ops are pure arithmetic — verified by inspection that no heap
        // allocation occurs (no UniquePtr created, no to_ffi call).
        let a = OcVec::new(1.0, 2.0, 3.0);
        let b = OcVec::new(4.0, 5.0, 6.0);
        let _ = a + b;
        let _ = a - b;
        let _ = a * 2.0;
        let _ = 2.0 * a;
        let _ = a / 2.0;
        let _ = -a;
    }

    // ── OcDir ────────────────────────────────────────────────────────────

    #[test]
    fn dir_construction_normalises() {
        let d = OcDir::new(3.0, 0.0, 0.0).unwrap();
        assert!((d.x() - 1.0).abs() < 1e-15);
    }

    #[test]
    fn dir_zero_vector_fails() {
        assert!(OcDir::new(0.0, 0.0, 0.0).is_err());
    }

    #[test]
    fn dir_angle() {
        let x = OcDir::new(1.0, 0.0, 0.0).unwrap();
        let y = OcDir::new(0.0, 1.0, 0.0).unwrap();
        assert!((x.angle(&y) - FRAC_PI_2).abs() < 1e-12);
    }

    #[test]
    fn dir_cross_perpendicular() {
        let x = OcDir::new(1.0, 0.0, 0.0).unwrap();
        let y = OcDir::new(0.0, 1.0, 0.0).unwrap();
        let z = x.cross(&y).unwrap();
        assert!((z.z() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn dir_cross_parallel_fails() {
        let x = OcDir::new(1.0, 0.0, 0.0).unwrap();
        assert!(x.cross(&x).is_err());
    }

    #[test]
    fn dir_is_parallel() {
        let x = OcDir::new(1.0, 0.0, 0.0).unwrap();
        let mx = OcDir::new(-1.0, 0.0, 0.0).unwrap();
        let y = OcDir::new(0.0, 1.0, 0.0).unwrap();
        assert!(x.is_parallel(&mx, 1e-10));
        assert!(!x.is_parallel(&y, 1e-10));
    }

    #[test]
    fn dir_angle_with_ref_sign() {
        let x = OcDir::new(1.0, 0.0, 0.0).unwrap();
        let y = OcDir::new(0.0, 1.0, 0.0).unwrap();
        let z = OcDir::new(0.0, 0.0, 1.0).unwrap();
        let mz = OcDir::new(0.0, 0.0, -1.0).unwrap();
        // x→y measured around +z should be positive (CCW)
        let pos = x.angle_with_ref(&y, &z).unwrap();
        assert!(pos > 0.0);
        // same rotation measured around −z should be negative
        let neg = x.angle_with_ref(&y, &mz).unwrap();
        assert!(neg < 0.0);
        assert!((pos.abs() - neg.abs()).abs() < 1e-12);
    }

    #[test]
    fn dir_reversed_is_neg() {
        let x = OcDir::new(1.0, 0.0, 0.0).unwrap();
        assert_eq!(-x, OcDir::new(-1.0, 0.0, 0.0).unwrap());
    }
    #[cfg(test)]
    mod ax_tests {
        use super::*;
        use std::f64::consts::{FRAC_PI_2, PI};

        // ── OcAx1 ────────────────────────────────────────────────────────────

        #[test]
        fn ax1_default_is_z_axis() {
            let a = OcAx1::z_axis();
            assert_eq!(a.location(), OcPnt::origin());
            assert!((a.direction().z() - 1.0).abs() < 1e-15);
        }

        #[test]
        fn ax1_accessors_roundtrip() {
            let p = OcPnt::new(1.0, 2.0, 3.0);
            let d = OcDir::new(0.0, 0.0, 1.0).unwrap();
            let a = OcAx1::new(p, d);
            assert_eq!(a.location(), p);
            assert_eq!(a.direction(), d);
        }

        #[test]
        fn ax1_reversed() {
            let a = OcAx1::new(OcPnt::origin(), OcDir::new(0.0, 0.0, 1.0).unwrap());
            let r = a.reversed();
            assert!((r.direction().z() + 1.0).abs() < 1e-15);
            assert_eq!(r.location(), OcPnt::origin());
        }

        #[test]
        fn ax1_neg_operator() {
            let a = OcAx1::new(OcPnt::origin(), OcDir::new(1.0, 0.0, 0.0).unwrap());
            assert_eq!(-a, a.reversed());
        }

        #[test]
        fn ax1_translated() {
            let a = OcAx1::new(OcPnt::origin(), OcDir::new(0.0, 0.0, 1.0).unwrap());
            let t = a.translated(OcVec::new(1.0, 2.0, 3.0));
            assert_eq!(t.location(), OcPnt::new(1.0, 2.0, 3.0));
            assert_eq!(t.direction(), a.direction());
        }

        #[test]
        fn ax1_angle_right_angle() {
            let x = OcAx1::new(OcPnt::origin(), OcDir::new(1.0, 0.0, 0.0).unwrap());
            let y = OcAx1::new(OcPnt::origin(), OcDir::new(0.0, 1.0, 0.0).unwrap());
            assert!((x.angle(&y) - FRAC_PI_2).abs() < 1e-12);
        }

        #[test]
        fn ax1_is_parallel() {
            let a = OcAx1::new(OcPnt::origin(), OcDir::new(1.0, 0.0, 0.0).unwrap());
            let ma = OcAx1::new(
                OcPnt::new(5.0, 0.0, 0.0),
                OcDir::new(-1.0, 0.0, 0.0).unwrap(),
            );
            let b = OcAx1::new(OcPnt::origin(), OcDir::new(0.0, 1.0, 0.0).unwrap());
            assert!(a.is_parallel(&ma, 1e-10));
            assert!(!a.is_parallel(&b, 1e-10));
        }

        #[test]
        fn ax1_is_normal() {
            let x = OcAx1::new(OcPnt::origin(), OcDir::new(1.0, 0.0, 0.0).unwrap());
            let y = OcAx1::new(OcPnt::origin(), OcDir::new(0.0, 1.0, 0.0).unwrap());
            assert!(x.is_normal(&y, 1e-10));
            assert!(!x.is_normal(&x, 1e-10));
        }

        #[test]
        fn ax1_is_opposite() {
            let a = OcAx1::new(OcPnt::origin(), OcDir::new(1.0, 0.0, 0.0).unwrap());
            let ma = OcAx1::new(OcPnt::origin(), OcDir::new(-1.0, 0.0, 0.0).unwrap());
            assert!(a.is_opposite(&ma, 1e-10));
            assert!(!a.is_opposite(&a, 1e-10));
        }

        #[test]
        fn ax1_is_coaxial_same_line() {
            // Two axes on the same line, different locations.
            let a = OcAx1::new(OcPnt::origin(), OcDir::new(1.0, 0.0, 0.0).unwrap());
            let b = OcAx1::new(
                OcPnt::new(5.0, 0.0, 0.0),
                OcDir::new(1.0, 0.0, 0.0).unwrap(),
            );
            assert!(a.is_coaxial(&b, 1e-10, 1e-10));
        }

        #[test]
        fn ax1_is_coaxial_offset_line_fails() {
            // Parallel axes but offset — not coaxial.
            let a = OcAx1::new(OcPnt::origin(), OcDir::new(1.0, 0.0, 0.0).unwrap());
            let b = OcAx1::new(
                OcPnt::new(0.0, 1.0, 0.0),
                OcDir::new(1.0, 0.0, 0.0).unwrap(),
            );
            assert!(!a.is_coaxial(&b, 1e-10, 0.5));
        }

        // ── OcAx2 ────────────────────────────────────────────────────────────

        #[test]
        fn ax2_oxyz_default() {
            let cs = OcAx2::oxyz();
            assert_eq!(cs.location(), OcPnt::origin());
            assert!((cs.direction().z() - 1.0).abs() < 1e-15);
            assert!((cs.x_direction().x() - 1.0).abs() < 1e-15);
            assert!((cs.y_direction().y() - 1.0).abs() < 1e-15);
        }

        #[test]
        fn ax2_right_handed_invariant() {
            // direction must equal x_direction × y_direction for any OcAx2.
            let cs = OcAx2::new(
                OcPnt::origin(),
                OcDir::new(0.0, 0.0, 1.0).unwrap(),
                OcDir::new(1.0, 1.0, 0.0).unwrap(), // oblique hint, not yet normalised
            )
            .unwrap();
            let x = cs.x_direction().to_vec();
            let y = cs.y_direction().to_vec();
            let n = cs.direction().to_vec();
            let computed_n = x.cross(&y);
            assert!((computed_n.x - n.x).abs() < 1e-12);
            assert!((computed_n.y - n.y).abs() < 1e-12);
            assert!((computed_n.z - n.z).abs() < 1e-12);
        }

        #[test]
        fn ax2_new_x_dir_projected() {
            // When an oblique Vx hint is given, the stored x_dir should be
            // perpendicular to direction.
            let n = OcDir::new(0.0, 0.0, 1.0).unwrap();
            let vx = OcDir::new(1.0, 1.0, 0.0).unwrap(); // 45° in XY plane
            let cs = OcAx2::new(OcPnt::origin(), n, vx).unwrap();
            // actual_x must be perpendicular to n (dot = 0)
            assert!(cs.x_direction().dot(&n).abs() < 1e-12);
            // and unit
            let mag = cs.x_direction().to_vec().magnitude();
            assert!((mag - 1.0).abs() < 1e-12);
        }

        #[test]
        fn ax2_parallel_fails() {
            let n = OcDir::new(0.0, 0.0, 1.0).unwrap();
            let vx = OcDir::new(0.0, 0.0, 1.0).unwrap(); // parallel to n
            assert!(OcAx2::new(OcPnt::origin(), n, vx).is_err());
        }

        #[test]
        fn ax2_anti_parallel_fails() {
            let n = OcDir::new(0.0, 0.0, 1.0).unwrap();
            let vx = OcDir::new(0.0, 0.0, -1.0).unwrap(); // anti-parallel to n
            assert!(OcAx2::new(OcPnt::origin(), n, vx).is_err());
        }

        #[test]
        fn ax2_with_direction_right_handed() {
            for (dx, dy, dz) in [
                (1.0, 0.0, 0.0),
                (0.0, 1.0, 0.0),
                (0.0, 0.0, 1.0),
                (1.0, 1.0, 0.0),
                (1.0, 1.0, 1.0),
            ] {
                let d = OcDir::new(dx, dy, dz).unwrap();
                let cs = OcAx2::with_direction(OcPnt::origin(), d);
                // Right-handedness: direction = x × y
                let x = cs.x_direction().to_vec();
                let y = cs.y_direction().to_vec();
                let n = cs.direction().to_vec();
                let computed = x.cross(&y);
                assert!(
                    (computed.x - n.x).abs() < 1e-12,
                    "failed for ({dx},{dy},{dz})"
                );
                assert!(
                    (computed.y - n.y).abs() < 1e-12,
                    "failed for ({dx},{dy},{dz})"
                );
                assert!(
                    (computed.z - n.z).abs() < 1e-12,
                    "failed for ({dx},{dy},{dz})"
                );
            }
        }

        #[test]
        fn ax2_axis_returns_correct_ax1() {
            let cs = OcAx2::oxyz();
            let ax = cs.axis();
            assert_eq!(ax.location(), cs.location());
            assert_eq!(ax.direction(), cs.direction());
        }

        #[test]
        fn ax2_angle_between_systems() {
            let oxyz = OcAx2::oxyz();
            // A second CS whose Z is the X axis.
            let rotated = OcAx2::new(
                OcPnt::origin(),
                OcDir::new(1.0, 0.0, 0.0).unwrap(),
                OcDir::new(0.0, 1.0, 0.0).unwrap(),
            )
            .unwrap();
            assert!((oxyz.angle(&rotated) - FRAC_PI_2).abs() < 1e-12);
        }

        #[test]
        fn ax2_translated() {
            let cs = OcAx2::oxyz();
            let t = cs.translated(OcVec::new(3.0, 0.0, 0.0));
            assert_eq!(t.location(), OcPnt::new(3.0, 0.0, 0.0));
            assert_eq!(t.direction(), cs.direction());
            assert_eq!(t.x_direction(), cs.x_direction());
        }

        #[test]
        fn point_to_axis_distance_known() {
            // Point (0,1,0) to the X axis (origin, direction X).
            let p = OcPnt::new(0.0, 1.0, 0.0);
            let o = OcPnt::origin();
            let d = OcDir::new(1.0, 0.0, 0.0).unwrap();
            let dist = point_to_axis_distance(&p, &o, &d);
            assert!((dist - 1.0).abs() < 1e-12);
        }
    }
}
