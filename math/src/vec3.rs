use core::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

#[macro_export]
macro_rules! _vec3 {
    () => {
        $crate::vec3::DEFAULT
    };
    (x) => {
        $crate::vec3::Vec3 { x: 1., y: 0., z: 0. }
    };
    (y) => {
        $crate::vec3::Vec3 { x: 0., y: 1., z: 0. }
    };
    (z) => {
        $crate::vec3::Vec3 { x: 0., y: 0., z: 1. }
    };
    (-x) => {
        $crate::vec3::Vec3 { x: -1., y: 0., z: 0. }
    };
    (-y) => {
        $crate::vec3::Vec3 { x: 0., y: -1., z: 0. }
    };
    (-z) => {
        $crate::vec3::Vec3 { x: 0., y: 0., z: -1. }
    };
    ($x: literal $y: literal $z: literal) => {
        $crate::vec3::Vec3 { x: $x, y: $y, z: $z }
    };
    ($x: expr, $y: expr, $z: expr $(,)?) => {
        $crate::vec3::Vec3 { x: $x, y: $y, z: $z }
    }
}
pub use _vec3 as vec3;

pub type Element = f32;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Vec3 {
    pub x: Element,
    pub y: Element,
    pub z: Element,
}

pub const DEFAULT: Vec3 = Vec3 {
    x: 0.,
    y: 0.,
    z: 0.,
};

impl Default for Vec3 {
    fn default() -> Self {
        DEFAULT
    }
}

impl core::fmt::Display for Vec3 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

impl Add for Vec3 {
    type Output = Self;

    fn add(mut self, other: Self) -> Self::Output {
        self += other;
        self
    }
}

impl SubAssign for Vec3 {
    fn sub_assign(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
        self.z -= other.z;
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(mut self, other: Self) -> Self::Output {
        self -= other;
        self
    }
}

impl Neg for Vec3 {
    type Output = Self;

    fn neg(mut self) -> Self::Output {
        self.x = -self.x;
        self.y = -self.y;
        self.z = -self.z;
        self
    }
}

impl MulAssign<Element> for Vec3 {
    fn mul_assign(&mut self, scale: Element) {
        self.x *= scale;
        self.y *= scale;
        self.z *= scale;
    }
}

impl Mul<Element> for Vec3 {
    type Output = Self;

    fn mul(mut self, scale: Element) -> Self::Output {
        self *= scale;
        self
    }
}

impl Vec3 {
    /// Returns a new `Vec3` that has a length of `1.0`, unless the passed in `Vec3`
    /// is the all zeroes `Vec3`. In that case, the same `Vec3` will be returned.
    // TODO would a method that returns a `Result` be better? Should this be called
    // `normalized_or_zero` instead?
    pub fn normalize(mut self) -> Self {
        let length = self.length();

        // Avoid divide-by-zero
        if length != 0. {
            let one_over_length = 1. / length;
            self.x *= one_over_length;
            self.y *= one_over_length;
            self.z *= one_over_length;
        }

        self
    }

    pub fn length(self) -> Element {
        self.length_squared().sqrt()
    }

    pub fn length_squared(self) -> Element {
        self.dot(self)
    }

    pub fn dot(self, other: Self) -> Element {
        self.x * other.x
        + self.y * other.y
        + self.z * other.z
    }

    pub fn cross(self, other: Self) -> Self {
        Self {
            x: (self.y * other.z) - (self.z * other.y),
            y: (self.z * other.x) - (self.x * other.z),
            z: (self.x * other.y) - (self.y * other.x),
        }
    }
}

/// A wrapper around `Vec3` that guarentees that it is if the contained `Vec3` has
/// had `Vec3::normalize` called on it. This guarentee means the inner `Vec3` either
/// has length one, or zero. Defaults to containing the all zero `Vec3`.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Normal(Vec3);

impl From<Vec3> for Normal {
    fn from(v: Vec3) -> Self {
        Self(v.normalize())
    }
}

impl From<Normal> for Vec3 {
    fn from(normal: Normal) -> Self {
        normal.0
    }
}

pub const DEFAULT_NORMAL: Normal = Normal(vec3!());

pub const X_AXIS: Normal = Normal(vec3!(x));
pub const NEG_X_AXIS: Normal = Normal(vec3!(-x));

pub const Y_AXIS: Normal = Normal(vec3!(y));
pub const NEG_Y_AXIS: Normal = Normal(vec3!(-y));

pub const Z_AXIS: Normal = Normal(vec3!(z));
pub const NEG_Z_AXIS: Normal = Normal(vec3!(-z));

#[macro_export]
macro_rules! _normal {
    () => {
        $crate::vec3::DEFAULT_NORMAL
    };
    (x) => {
        $crate::vec3::X_AXIS
    };
    (y) => {
        $crate::vec3::Y_AXIS
    };
    (z) => {
        $crate::vec3::Z_AXIS
    };
    (-x) => {
        $crate::vec3::NEG_X_AXIS
    };
    (-y) => {
        $crate::vec3::NEG_Y_AXIS
    };
    (-z) => {
        $crate::vec3::NEG_Z_AXIS
    };
    ($($tokens: tt)*) => {
        $crate::vec3::Normal::from($crate::vec3::vec3!($($tokens)*))
    }
}
pub use _normal as normal;