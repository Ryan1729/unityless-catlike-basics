use sokol_bindings::{sg, cstr, Int};

const W: Int = 4 * 5;
const H: Int = 4 * 5;

/// Color Channel Count.
const C: usize = 4;

const LEN: usize = W as usize * H as usize * C;

// R, G, B, A, R, ...
pub const TEXTURE: [u8; LEN] = {
    let mut texture = [0; LEN];

    const BASE_LEN: usize = 2 * C;

    const BASE_TEXTURE: [u8; BASE_LEN] = [
        0xFF, 0xFF, 0xFF, 0xFF,
        0x00, 0x00, 0x00, 0xFF,
    ];

    const BASE_TEXTURE_OFFSET: [u8; BASE_LEN] = [
        0x00, 0x00, 0x00, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF,
    ];

    let mut i = 0;
    while i < LEN {
        texture[i] = if (i / (W as usize * C)) % 2 == 0 {
            BASE_TEXTURE[i % BASE_LEN]
        } else {
            BASE_TEXTURE_OFFSET[i % BASE_LEN]
        };

        i += 1;
    }

    texture
};

pub fn make() -> sg::Image {
    let mut image_desc = sg::ImageDesc::default();
    image_desc.width = W;
    image_desc.height = H;
    image_desc.data.subimage[0][0] = sg::range!(TEXTURE);
    image_desc.label = cstr!("checkerboard-5x-texture");

    unsafe { sg::make_image(&image_desc) }
}