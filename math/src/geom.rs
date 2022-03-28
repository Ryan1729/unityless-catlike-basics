use core::ops::{Mul, MulAssign};

pub type Coord = f32;

pub const TAU: Coord = std::f32::consts::TAU;

#[derive(Clone, Copy, Debug, Default)]
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

pub const CYLINDER_POINT_COUNT: Index = DISC_POINT_COUNT * 2;
pub const CYLINDER_POINT_COUNT_USIZE: usize = CYLINDER_POINT_COUNT as usize;

pub const CYLINDER_INDEX_COUNT: Index = RING_POINT_COUNT * (
    // Top disc
    3
    // Downward pointing edge triangles
    + 3
    // Upward pointing edge triangles
    + 3
    // Bottom disc
    + 3
);
pub const CYLINDER_INDEX_COUNT_USIZE: usize = CYLINDER_INDEX_COUNT as usize;

pub fn gen_cylinder_mesh(scale: Scale)
-> IndexedMesh<CYLINDER_POINT_COUNT_USIZE, CYLINDER_INDEX_COUNT_USIZE> {
    let mut points = [Point::default(); CYLINDER_POINT_COUNT_USIZE];

    const TOP_RING_START: Index = 1;
    const BOTTOM_RING_START: Index = TOP_RING_START + RING_POINT_COUNT;
    const BOTTOM_DISC_CENTER: Index = BOTTOM_RING_START + RING_POINT_COUNT;

    const RING_POINT_COUNT_COORD: Coord = RING_POINT_COUNT as Coord;

    // Leave the first point, (at index 0,)  as the default.
    for i in 1..=RING_POINT_COUNT as usize {
        let theta = (i - 1) as Coord * TAU / RING_POINT_COUNT_COORD;

        let (cos, sin) = theta.sin_cos();

        let p = Point {
            x: scale.x * cos,
            y: scale.y * sin,
            z: 0.,
        };
        points[i] = p;

        points[i + RING_POINT_COUNT as usize] = Point {
            z: scale.z,
            ..p
        };
    }

    points[BOTTOM_DISC_CENTER as usize] = Point {
        x: 0.,
        y: 0.,
        z: scale.z,
    };

    let mut indices = [0; CYLINDER_INDEX_COUNT as usize];

    // Top disc
    for index in 0..RING_POINT_COUNT {
        let i = (index * 3) as usize;
        indices[i] = ((index + TOP_RING_START) % RING_POINT_COUNT) + 1;
        indices[i + 1] = index + TOP_RING_START;
        indices[i + 2] = 0;
    }

    // Downward pointing edge triangles
    for index in 0..RING_POINT_COUNT {
        let i = ((index + RING_POINT_COUNT) * 3) as usize;
        indices[i] = index + TOP_RING_START;
        indices[i + 1] = ((index + 1) % RING_POINT_COUNT) + TOP_RING_START;
        // Jump up to next disc
        indices[i + 2] = index + BOTTOM_RING_START;
    }

    // Upward pointing edge triangles
    for index in 0..RING_POINT_COUNT {
        let i = ((index + RING_POINT_COUNT * 2) * 3) as usize;
        // Jump back to previous disc
        indices[i] = ((index + 1) % RING_POINT_COUNT) + TOP_RING_START;
        indices[i + 1] = ((index + 1) % RING_POINT_COUNT) + BOTTOM_RING_START;
        indices[i + 2] = index + BOTTOM_RING_START;
    }

    // Bottom disc
    for index in 0..RING_POINT_COUNT {
        let i = ((index + RING_POINT_COUNT * 3) * 3) as usize;
        indices[i] = BOTTOM_DISC_CENTER;
        indices[i + 1] = index + BOTTOM_RING_START;
        indices[i + 2] = ((index + 1) % RING_POINT_COUNT) + BOTTOM_RING_START;
    }

    IndexedMesh{
        points,
        indices,
    }
}