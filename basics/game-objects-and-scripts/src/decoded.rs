use crate::Int;

/// The byte at index 0 is the red channel, index 1 green, index 2 blue, index 3
// alpha, index 4 red again, and so on.
pub type RGBAVec<'image> = std::borrow::Cow<'image, [u8]>;

pub struct Png<'image> {
    pub w: Int,
    pub h: Int,
    pub image_bytes: RGBAVec<'image>,
}

pub fn png_with_checkerboard_fallback<'image>(png_bytes: &[u8]) -> Png<'image> {
    match png_decoder::decode(png_bytes) {
        Ok((header, image_data)) => Png {
            w: header.width as _,
            h: header.height as _,
            image_bytes: RGBAVec::from(image_data),
        },
        Err(err) => {
            eprintln!("{}:{}:{} {:?}\nfalling back to checkerboard", file!(), line!(), column!(), err);

            const W: Int = 4;
            const H: Int = 4;

            // R, G, B, A, R, ...
            const CHECKERBOARD: [u8; W as usize * H as usize * 4 as usize ] = [
                0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0xFF,
                0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0xFF,
                0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            ];

            Png {
                w: W,
                h: H,
                image_bytes: RGBAVec::from(CHECKERBOARD.as_slice()),
            }
        }
    }
}