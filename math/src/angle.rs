use core::ops::{Add, AddAssign, Sub, SubAssign};

type RawRadians = f32;

pub const TAU: RawRadians = std::f32::consts::TAU;
pub const PI: RawRadians = std::f32::consts::PI;

pub trait Angle {
    fn raw_radians(self) -> RawRadians;
}

#[derive(Clone, Copy, Default, Debug, PartialEq, PartialOrd)]
pub struct Radians(pub RawRadians);

impl Angle for Radians {
    fn raw_radians(self) -> RawRadians {
        self.0
    }
}

impl AddAssign for Radians {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl Add for Radians {
    type Output = Self;

    fn add(mut self, other: Self) -> Self::Output {
        self += other;
        self
    }
}

impl SubAssign for Radians {
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0;
    }
}

impl Sub for Radians {
    type Output = Self;

    fn sub(mut self, other: Self) -> Self::Output {
        self -= other;
        self
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq, PartialOrd)]
pub struct Degrees(pub f32);

impl Angle for Degrees {
    fn raw_radians(self) -> RawRadians {
        self.0 * (PI / 180.)
    }
}

impl AddAssign for Degrees {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl Add for Degrees {
    type Output = Self;

    fn add(mut self, other: Self) -> Self::Output {
        self += other;
        self
    }
}
