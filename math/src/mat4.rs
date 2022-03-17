use core::ops::{Index, IndexMut};

pub type Element = f32;

pub const PI: Element = std::f32::consts::PI;

// Use this type alias in case we want to switch to `[[f32; 4]; 4]` later.
pub type Elements = [Element; 16];

/// We have this wrapper struct so we can implement `Mul`, etc.
#[repr(transparent)]
#[derive(Default)]
pub struct Mat4(pub Elements);

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

// If we want to exose these, consider using an enum instead.
const _0_0: usize = 0;
const _1_1: usize = WIDTH as usize + 1;
const _2_2: usize = WIDTH as usize * 2 + 2;
const _2_3: usize = WIDTH as usize * 2 + 3;
const _3_2: usize = WIDTH as usize * 3 + 2;
const _3_3: usize = WIDTH as usize * 3 + 3;

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
        output[_2_3] = -1.;
        output[_2_2] = (near + far) / (near - far);
        output[_3_2] = (2. * near * far) / (near - far);
        output[_3_3] = 0.;

        output
    }
}

