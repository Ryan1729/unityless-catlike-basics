use core::ops::{Index, IndexMut, Mul, MulAssign};
use crate::{
    angle::Angle,
    vec3::{vec3, Vec3},
};

pub type Element = f32;

pub const PI: Element = std::f32::consts::PI;

// Use this type alias in case we want to switch to `[[f32; 4]; 4]` later.
pub type Elements = [Element; 16];

/// We have this wrapper struct so we can implement `Mul`, etc.
/// We keep the interior `Elements` private so we have the option to change
/// the internal representation between column-major to row-major  without external
/// code needing to care.
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct Mat4(Elements);

impl Mat4 {
    pub fn from_row_major(elements: Elements) -> Self {
        Mat4(elements)
    }

    pub fn from_column_major(elements: Elements) -> Self {
        Mat4(elements).transpose()
    }

    pub fn to_row_major(self) -> Elements {
        self.0
    }

    pub fn to_column_major(self) -> Elements {
        self.transpose().0
    }

    // Having these be contiguous in memory instead of spread out like this is an
    // argument for changing to column-major representation at some point.
    pub fn x_axis(self) -> Vec3 {
        vec3!(self[_0_0], self[_1_0], self[_2_0])
    }

    pub fn y_axis(self) -> Vec3 {
        vec3!(self[_0_1], self[_1_1], self[_2_1])
    }

    pub fn z_axis(self) -> Vec3 {
        vec3!(self[_0_2], self[_1_2], self[_2_2])
    }
}

impl core::fmt::Display for Mat4 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "[row_col!(row, column)\n")?;

        for row in 0..HEIGHT {
            write!(f, "    ")?;
            for column in 0..WIDTH {
                let val = self[row_col!(row, column)];
                if val.is_sign_negative() {
                    write!(f, "{val:.8} ({row} {column}), ")?;
                } else {
                    write!(f, " {val:.8} ({row} {column}), ")?;
                }
            }
            write!(f, "\n")?;
        }

        write!(f, "]\n")
    }
}

impl Index<u8> for Mat4 {
    type Output = Element;

    fn index(&self, index: u8) -> &Self::Output {
        self.0.index(index as usize)
    }
}

impl IndexMut<u8> for Mat4 {
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        self.0.index_mut(index as usize)
    }
}

impl Index<i32> for Mat4 {
    type Output = Element;

    fn index(&self, index: i32) -> &Self::Output {
        self.0.index(index as usize)
    }
}

impl IndexMut<i32> for Mat4 {
    fn index_mut(&mut self, index: i32) -> &mut Self::Output {
        self.0.index_mut(index as usize)
    }
}

impl Index<usize> for Mat4 {
    type Output = Element;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

impl IndexMut<usize> for Mat4 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

pub const WIDTH: u8 = 4;
pub const HEIGHT: u8 = 4;
pub const LENGTH: u8 = WIDTH * HEIGHT;

#[macro_export]
macro_rules! _row_col {
    ($row: literal $col: literal) => {
        row_col!($row, $col)
    };
    ($row: expr, $col: expr $(,)?) => {
        WIDTH * $row + $col
    }
}
pub use _row_col as row_col;

// If we want to expose these, consider using an enum instead.
const _0_0: u8 = row_col!(0 0);
const _0_1: u8 = row_col!(0 1);
const _0_2: u8 = row_col!(0 2);
const _0_3: u8 = row_col!(0 3);
const _1_0: u8 = row_col!(1 0);
const _1_1: u8 = row_col!(1 1);
const _1_2: u8 = row_col!(1 2);
const _1_3: u8 = row_col!(1 3);
const _2_0: u8 = row_col!(2 0);
const _2_1: u8 = row_col!(2 1);
const _2_2: u8 = row_col!(2 2);
const _2_3: u8 = row_col!(2 3);
const _3_0: u8 = row_col!(3 0);
const _3_1: u8 = row_col!(3 1);
const _3_2: u8 = row_col!(3 2);
const _3_3: u8 = row_col!(3 3);

impl Mul for Mat4 {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
       let mut output = Self::default();

        // TODO add SSE version

        for column in 0..WIDTH {
            for row in 0..HEIGHT {
                let mut sum = 0.;

                // This assumes `WIDTH == HEIGHT == 4`, which is unlikely to change.
                for index in 0..4 {
                    sum += self[row_col!(row, index)]
                        * other[row_col!(index, column)];
                }

                output[(WIDTH * row + column) as usize] = sum;
            }
        }

        output
    }
}

impl MulAssign for Mat4 {
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}

#[test]
fn mat4_mul_works_on_this_prime_number_example() {
    let actual = Mat4([
        2.,3.,5.,7.,
        11.,13.,17.,19.,
        23.,29.,31.,37.,
        41.,43.,47.,53.,
    ]) * Mat4([
        59.,61.,67.,71.,
        73.,79.,83.,89.,
        97.,101.,103.,107.,
        109.,113.,127.,131.,
    ]);

    let expected = Mat4([
        1585.,1655.,1787.,1861.,
        5318.,5562.,5980.,6246.,
        10514.,11006.,11840.,12378.,
        15894.,16634.,17888.,18710.,
    ]);

    assert_eq!(actual, expected);
}

#[test]
fn mat4_mul_works_on_this_asymmetrical_example() {
    let initial = Mat4([
        0.,1.,2.,3.,
        0.,0.,4.,5.,
        0.,0.,0.,6.,
        0.,0.,0.,0.,
    ]);

    assert_eq!(initial * Mat4::identity(), initial);
}

impl Mat4 {
    pub fn diagonal(value: f32) -> Self {
        Self([
            value, 0., 0., 0.,
            0., value, 0., 0.,
            0., 0., value, 0.,
            0., 0., 0., value,
        ])
    }

    pub fn identity() -> Self {
        Self::diagonal(1.)
    }
}

/// Expressed in distance from camera. `(Near, Far)`
pub type ClipPlanes = (Element, Element);

/// Width / Height
pub type AspectRatio = Element;

impl Mat4 {
    pub fn perspective(
        field_of_view: Element,
        aspect_ratio: AspectRatio,
        (near, far): ClipPlanes
    ) -> Self {
        let mut output = Self::default();

        let tan_theta_over_2 = Element::tan(
            field_of_view * (PI / 360.)
        );

        output[_0_0] = 1. / tan_theta_over_2;
        output[_1_1] = aspect_ratio / tan_theta_over_2;
        output[_3_2] = -1.;
        output[_2_2] = (near + far) / (near - far);
        output[_2_3] = (2. * near * far) / (near - far);
        output[_3_3] = 0.;

        output
    }

    pub fn look_at(
        eye: Vec3,
        center: Vec3,
        up: Vec3,
    ) -> Self {
        let mut output = Self::default();

        let f = (center - eye).normalize();
        let s = f.cross(up).normalize();
        let u = s.cross(f);

        output[_0_0] = s.x;
        output[_1_0] = u.x;
        output[_2_0] = -f.x;

        output[_0_1] = s.y;
        output[_1_1] = u.y;
        output[_2_1] = -f.y;

        output[_0_2] = s.z;
        output[_1_2] = u.z;
        output[_2_2] = -f.z;

        output[_0_3] = -(s.dot(eye));
        output[_1_3] = -(u.dot(eye));
        output[_2_3] = f.dot(eye);
        output[_3_3] = 1.;

        output
    }

    pub fn rotation(angle: impl Angle, mut axis: Vec3) -> Self {
        let radians = angle.raw_radians();

        let mut output = Self::default();

        axis = axis.normalize();

        let (sin_theta, cos_theta) = radians.sin_cos();
        let cos_value = 1. - cos_theta;

        output[_0_0] = (axis.x * axis.x * cos_value) + cos_theta;
        output[_1_0] = (axis.x * axis.y * cos_value) + (axis.z * sin_theta);
        output[_2_0] = (axis.x * axis.z * cos_value) - (axis.y * sin_theta);

        output[_0_1] = (axis.y * axis.x * cos_value) - (axis.z * sin_theta);
        output[_1_1] = (axis.y * axis.y * cos_value) + cos_theta;
        output[_2_1] = (axis.y * axis.z * cos_value) + (axis.x * sin_theta);

        output[_0_2] = (axis.z * axis.x * cos_value) + (axis.y * sin_theta);
        output[_1_2] = (axis.z * axis.y * cos_value) - (axis.x * sin_theta);
        output[_2_2] = (axis.z * axis.z * cos_value) + cos_theta;

        output[_3_3] = 1.;

        output
    }

    pub fn transpose(self) -> Self {
        Self([
            self[_0_0], self[_1_0], self[_2_0], self[_3_0],
            self[_0_1], self[_1_1], self[_2_1], self[_3_1],
            self[_0_2], self[_1_2], self[_2_2], self[_3_2],
            self[_0_3], self[_1_3], self[_2_3], self[_3_3],
        ])
    }
}

