#[derive(Copy, Clone)]
pub enum Angle {
    Radians(f64),
    Degrees(f64),
}

impl Angle {
    /// Create a new [`Angle`] from a [`f64`] value representing an angle in radians.
    pub fn from_radians(radians: f64) -> Angle {
        Angle::Radians(radians)
    }

    /// Create a new [`Angle`] from a [`f64`] value representing an angle in degrees.
    pub fn from_degrees(degrees: f64) -> Angle {
        Angle::Degrees(degrees)
    }

    #[inline]
    pub fn as_radians_f64(&self) -> f64 {
        match self {
            Angle::Radians(radians) => *radians,
            Angle::Degrees(degrees) => degrees.to_radians(),
        }
    }

    #[inline]
    pub fn as_degrees_f64(&self) -> f64 {
        match self {
            Angle::Radians(radians) => radians.to_degrees(),
            Angle::Degrees(degrees) => *degrees,
        }
    }

    #[inline]
    pub fn sin(&self) -> f64 {
        match self {
            Angle::Radians(radians) => radians.sin(),
            Angle::Degrees(degrees) => degrees.to_radians().sin(),
        }
    }

    #[inline]
    pub fn cos(&self) -> f64 {
        match self {
            Angle::Radians(radians) => radians.cos(),
            Angle::Degrees(degrees) => degrees.to_radians().cos(),
        }
    }
}
