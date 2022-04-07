pub mod textured;
pub mod lit;
pub mod textured_lit;

/// From most significant to least significant. So in a hex literal that's
/// `0xAABBGGRR`, so `0xFFC08040` has full alpha, around 3/4 blue, around half green
/// and around 1/4 red.
pub type ABGR = u32;

pub type Index = u16;