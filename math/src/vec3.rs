use core::ops::{Neg, Sub, SubAssign};

#[macro_export]
macro_rules! _vec3 {
    () => {
        $crate::vec3::Vec3::default()
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

#[derive(Clone, Copy, Default, PartialEq, PartialOrd)]
pub struct Vec3 {
    pub x: Element,
    pub y: Element,
    pub z: Element,
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

impl Vec3 {
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
}

impl Vec3 {
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