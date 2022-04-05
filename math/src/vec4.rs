use core::ops::{Index, IndexMut};

use crate::vec3::{Vec3, vec3};

#[macro_export]
macro_rules! _vec4 {
    () => {
        $crate::vec4::Vec4::default()
    };
    (x) => {
        $crate::vec4::Vec4 { x: 1., y: 0., z: 0., w: 0. }
    };
    (y) => {
        $crate::vec4::Vec4 { x: 0., y: 1., z: 0., w: 0. }
    };
    (z) => {
        $crate::vec4::Vec4 { x: 0., y: 0., z: 1., w: 0. }
    };
    (w) => {
        $crate::vec4::Vec4 { x: 0., y: 0., z: 0., w: 1. }
    };
    ($x: literal $y: literal $z: literal $w: literal) => {
        $crate::vec4::Vec4 { x: $x, y: $y, z: $z, w: $w }
    };
    ($x: expr, $y: expr, $z: expr, $w: expr $(,)?) => {
        $crate::vec4::Vec4 { x: $x, y: $y, z: $z, w: $w }
    }
}
pub use _vec4 as vec4;

pub type Element = f32;

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Vec4 {
    pub x: Element,
    pub y: Element,
    pub z: Element,
    pub w: Element,
}

impl core::fmt::Display for Vec4 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "({}, {}, {}, {})", self.x, self.y, self.z, self.w)
    }
}

impl Index<u8> for Vec4 {
    type Output = Element;

    fn index(&self, index: u8) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            3 => &self.w,
            _ => unreachable!(),
        }
    }
}

impl IndexMut<u8> for Vec4 {
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            3 => &mut self.w,
            _ => unreachable!(),
        }
    }
}

impl Vec4 {
    pub fn xyz(self) -> Vec3 {
        vec3!(self.x, self.y, self.z)
    }

    pub fn normalize(mut self) -> Self {
        let length = self.length();

        // Avoid divide-by-zero
        if length != 0. {
            let one_over_length = 1. / length;
            self.x *= one_over_length;
            self.y *= one_over_length;
            self.z *= one_over_length;
            self.w *= one_over_length;
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
        + self.w * other.w
    }
}