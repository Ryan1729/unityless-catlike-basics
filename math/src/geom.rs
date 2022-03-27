use core::ops::{Mul, MulAssign};

pub type Coord = f32;

pub const TAU: Coord = std::f32::consts::TAU;

#[derive(Clone, Copy, Default)]
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

pub struct Scale {
    pub x: Coord,
    pub y: Coord,
    pub z: Coord,
}

impl MulAssign<Scale> for Point {
    fn mul_assign(&mut self, scale: Scale) {
        self.x *= scale.x;
        self.y *= scale.y;
        self.z *= scale.z;
    }
}

impl Mul<Scale> for Point {
    type Output = Self;

    fn mul(mut self, coord: Scale) -> Self::Output {
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

const RING_POINT_COUNT: Index = 16;
const DISC_POINT_COUNT: Index = RING_POINT_COUNT + 1;

pub const CYLINDER_POINT_COUNT: Index = DISC_POINT_COUNT;
pub const CYLINDER_POINT_COUNT_USIZE: usize = CYLINDER_POINT_COUNT as usize;

pub const CYLINDER_INDEX_COUNT: Index = RING_POINT_COUNT * 3;
pub const CYLINDER_INDEX_COUNT_USIZE: usize = CYLINDER_INDEX_COUNT as usize;

const CYLINDER_INDICES: [Index; CYLINDER_INDEX_COUNT as usize] = [
    0, 1, 2,
    0, 2, 3,
    0, 3, 4,
    0, 4, 5,
    0, 5, 6,
    0, 6, 7,
    0, 7, 8,
    0, 8, 9,
    0, 9, 10,
    0, 10, 11,
    0, 11, 12,
    0, 12, 13,
    0, 13, 14,
    0, 14, 15,
    0, 15, 16,
    0, 16, 1,
];

pub fn gen_cylinder_mesh(scale: Scale)
-> IndexedMesh<CYLINDER_POINT_COUNT_USIZE, CYLINDER_INDEX_COUNT_USIZE> {
    let mut points = [Point::default(); DISC_POINT_COUNT as usize];

    const DISC_POINT_COUNT_COORD: Coord = DISC_POINT_COUNT as Coord;

    for i in 1..=RING_POINT_COUNT as usize {
        let theta = (i - 1) as Coord * TAU / DISC_POINT_COUNT_COORD;

        let (cos, sin) = theta.sin_cos();

        points[i] = Point {
            x: scale.x * cos,
            y: scale.y * sin,
            z: 0.,
        };
    }

    IndexedMesh{
        points,
        indices: CYLINDER_INDICES,
    }
}