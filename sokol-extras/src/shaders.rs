pub mod textured;
pub mod lit;
pub mod textured_lit;

/// From most significant to least significant. So in a hex literal that's
/// `0xAABBGGRR`, so `0xFFC08040` has full alpha, around 3/4 blue, around half green
/// and around 1/4 red.
pub type ABGR = u32;

/// ```
/// use sokol_extras::shaders::abgr_from_vec3;
/// use math::vec3::vec3;
/// # assert_eq!(
/// #    abgr_from_vec3(vec3!(0.0, 1.0, 2.0)),
/// #    0xFFFFFF00, "{:X}", abgr_from_vec3(vec3!(0.0, 1.0, 2.0))
/// # );
/// assert_eq!(
///     abgr_from_vec3(vec3!(0.0, 1.0, 2.0)),
///     0xFFFFFF00
/// );
/// # assert_eq!(
/// #    abgr_from_vec3(vec3!(0.25, 0.5, 0.75)),
/// #    0xFFC08040, "{:X}", abgr_from_vec3(vec3!(0.25, 0.5, 0.75))
/// # );
/// assert_eq!(
///     abgr_from_vec3(vec3!(0.25, 0.5, 0.75)),
///     0xFFC08040
/// );
/// # assert_eq!(
/// #    abgr_from_vec3(vec3!(1./255., 2./255., 254./255.)),
/// #    0xFFFE0201, "{:X}", abgr_from_vec3(vec3!(1./255., 2./255., 254./255.))
/// # );
/// assert_eq!(
///     abgr_from_vec3(vec3!(1./255., 2./255., 254./255.)),
///     0xFFFE0201
/// );
/// ```
pub fn abgr_from_vec3(v: math::vec3::Vec3) -> ABGR {
    0xFF000000 
    | (((v.z * 256.0) as u8) as ABGR) << 16
    | (((v.y * 256.0) as u8) as ABGR) << 8
    | (((v.x * 256.0) as u8) as ABGR)
}

pub type Index = u16;