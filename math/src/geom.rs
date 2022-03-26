use core::ops::{Mul, MulAssign};

pub type Coord = f32;

pub struct Point {
    pub x: Coord,
    pub y: Coord,
    pub z: Coord,
}

impl MulAssign<Coord> for Point {
    fn mul_assign(&mut self, coord: Coord) {
        self.x *= coord;
        self.y *= coord;
        self.z *= coord;
    }
}

impl Mul<Coord> for Point {
    type Output = Self;

    fn mul(mut self, coord: Coord) -> Self::Output {
        self *= coord;
        self
    }
}

#[macro_export]
macro_rules! _point {
    ($x: literal $(,)? $y: literal $(,)? $z: literal $(,)?) => {
        $crate::geom::Point {
            x: $x,
            y: $y,
            z: $z,
        }
    };
    ($x: expr, $y: expr, $z: expr $(,)?) => {
        $crate::geom::Point {
            x: $x,
            y: $y,
            z: $z,
        }
    };
}
pub use _point as point;

pub type Index = u16;

pub struct IndexedMesh<const POINT_COUNT: usize, const INDEX_COUNT: usize> {
    pub points: [Point; POINT_COUNT],
    pub indices: [Index; INDEX_COUNT],
}

pub const CUBE_INDEX_COUNT: Index = 36;
pub const CUBE_INDEX_COUNT_USIZE: usize = CUBE_INDEX_COUNT as usize;

const CUBE_INDICES: [Index; CUBE_INDEX_COUNT_USIZE] = [
    0, 1, 2,  0, 2, 3,
    6, 5, 4,  7, 6, 4,
    8, 9, 10,  8, 10, 11,
    14, 13, 12,  15, 14, 12,
    16, 17, 18,  16, 18, 19,
    22, 21, 20,  23, 22, 20
];

pub const CUBE_POINT_COUNT: Index = 24;
pub const CUBE_POINT_COUNT_USIZE: usize = CUBE_POINT_COUNT as usize;

const UNSCALED_CUBE_POINTS: [Point; CUBE_POINT_COUNT_USIZE] = [
    point!(-1. -1. -1.),
    point!( 1. -1. -1.),
    point!( 1.  1. -1.),
    point!(-1.  1. -1.),

    point!(-1. -1.  1.),
    point!( 1. -1.  1.),
    point!( 1.  1.  1.),
    point!(-1.  1.  1.),

    point!(-1. -1. -1.),
    point!(-1.  1. -1.),
    point!(-1.  1.  1.),
    point!(-1. -1.  1.),

    point!( 1. -1. -1.),
    point!( 1.  1. -1.),
    point!( 1.  1.  1.),
    point!( 1. -1.  1.),

    point!(-1. -1. -1.),
    point!(-1. -1.  1.),
    point!( 1. -1.  1.),
    point!( 1. -1. -1.),

    point!(-1.  1. -1.),
    point!(-1.  1.  1.),
    point!( 1.  1.  1.),
    point!( 1.  1. -1.),
];

pub fn gen_cube_mesh(scale: Coord)
-> IndexedMesh<CUBE_POINT_COUNT_USIZE, CUBE_INDEX_COUNT_USIZE> {
    let mut points = UNSCALED_CUBE_POINTS;

    for i in 0..CUBE_POINT_COUNT {
        points[i as usize] *= scale;
    }

    IndexedMesh{
        points,
        indices: CUBE_INDICES,
    }
}