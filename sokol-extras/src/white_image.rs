pub const TEXTURE: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];

use sokol_bindings::{sg, cstr};

pub fn make() -> sg::Image {
    let mut image_desc = sg::ImageDesc::default();
    image_desc.width = 1;
    image_desc.height = 1;
    image_desc.data.subimage[0][0] = sg::range!(TEXTURE);
    image_desc.label = cstr!("white-texture");

    unsafe { sg::make_image(&image_desc) }
}
