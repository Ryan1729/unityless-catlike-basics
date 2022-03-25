use sokol_bindings::{
    cstr,
    sg::{
        self,
        range,
        Bindings,
        Pipeline,
        PipelineDesc,
    },
    Int,
};
use math::mat4::Mat4;
use sokol_extras::textured;

#[derive(Default)]
pub struct State {
    pub bind: Bindings,
    pub pipe: Pipeline,
}

/*
    Cube vertex buffer with packed vertex formats for color and texture coords.
    Note that a vertex format which must be portable across all
    backends must only use the normalized integer formats
    (BYTE4N, UBYTE4N, SHORT2N, SHORT4N), which can be converted
    to floating point formats in the vertex shader inputs.

    The reason is that D3D11 cannot convert from non-normalized
    formats to floating point inputs (only to integer inputs),
    and WebGL2 / GLES2 don't support integer vertex shader inputs.
*/
const SKYBOX_VERTICIES: [textured::Vertex; 24] = {
    // Short for Cube Scale.
    const C_S: f32
        // We want something that we're sure won't ever clip.
        = crate::FAR * 0.5;
    macro_rules! m {
        (0/1) => {0};
        (1/1) => {32767};
        (1/4) => {32767/4};
        (1/3) => {32767/3};
        (1/2) => {32767/2};
        (2/3) => {m!(1/3) * 2};
        (3/4) => {m!(1/4) * 3};
    }
    textured::vertex_array![
        /* pos                  color       uvs */
        { -C_S, -C_S, -C_S,  0xFFFFFFFF, m!(0/1), m!(1/3) },
        {  C_S, -C_S, -C_S,  0xFFFFFFFF, m!(1/4), m!(1/3) },
        {  C_S,  C_S, -C_S,  0xFFFFFFFF, m!(1/2), m!(1/3) },
        { -C_S,  C_S, -C_S,  0xFFFFFFFF, m!(3/4), m!(1/3) },

        { -C_S, -C_S,  C_S,  0xFFFFFFFF, m!(0/1), m!(2/3) },
        {  C_S, -C_S,  C_S,  0xFFFFFFFF, m!(1/4), m!(2/3) },
        {  C_S,  C_S,  C_S,  0xFFFFFFFF, m!(1/2), m!(2/3) },
        { -C_S,  C_S,  C_S,  0xFFFFFFFF, m!(3/4), m!(2/3) },

        { -C_S, -C_S, -C_S,  0xFFFFFFFF, m!(3/4), m!(1/3) },
        { -C_S,  C_S, -C_S,  0xFFFFFFFF, m!(3/4), m!(1/3) },
        { -C_S,  C_S,  C_S,  0xFFFFFFFF, m!(1/1), m!(2/3) },
        { -C_S, -C_S,  C_S,  0xFFFFFFFF, m!(1/1), m!(2/3) },

        {  C_S, -C_S, -C_S,  0xFFFFFFFF, m!(1/4), m!(1/3) },
        {  C_S,  C_S, -C_S,  0xFFFFFFFF, m!(1/2), m!(1/3) },
        {  C_S,  C_S,  C_S,  0xFFFFFFFF, m!(1/2), m!(2/3) },
        {  C_S, -C_S,  C_S,  0xFFFFFFFF, m!(1/4), m!(2/3) },

        { -C_S, -C_S, -C_S,  0xFFFFFFFF, m!(0/1), m!(1/3) },
        { -C_S, -C_S,  C_S,  0xFFFFFFFF, m!(0/1), m!(2/3) },
        {  C_S, -C_S,  C_S,  0xFFFFFFFF, m!(1/4), m!(2/3) },
        {  C_S, -C_S, -C_S,  0xFFFFFFFF, m!(1/4), m!(1/3) },

        { -C_S,  C_S, -C_S,  0xFFFFFFFF, m!(3/4), m!(1/3) },
        { -C_S,  C_S,  C_S,  0xFFFFFFFF, m!(3/4), m!(2/3) },
        {  C_S,  C_S,  C_S,  0xFFFFFFFF, m!(1/2), m!(2/3) },
        {  C_S,  C_S, -C_S,  0xFFFFFFFF, m!(1/2), m!(1/3) },
    ]
};

const CUBE_INDEX_COUNT: Int = 36;

const CUBE_INDICES: [u16; CUBE_INDEX_COUNT as usize] = [
    0, 1, 2,  0, 2, 3,
    6, 5, 4,  7, 6, 4,
    8, 9, 10,  8, 10, 11,
    14, 13, 12,  15, 14, 12,
    16, 17, 18,  16, 18, 19,
    22, 21, 20,  23, 22, 20
];

pub fn init(skybox: &mut State) {
    skybox.bind.vertex_buffers[0] = sg::make_immutable_vertex_buffer!(
        SKYBOX_VERTICIES
        "skybox-vertices"
    );

    skybox.bind.index_buffer = sg::make_immutable_index_buffer!(
        CUBE_INDICES
        "cube-indices"
    );

    let decoded = crate::decode_png_with_checkerboard_fallback(
        include_bytes!("../../assets/skybox.png"),
    );

    let mut image_desc = sg::ImageDesc::default();
    image_desc.width = decoded.w;
    image_desc.height = decoded.h;
    image_desc.data.subimage[0][0] = range!(&decoded.image_bytes);
    image_desc.label = cstr!("skybox-texture");

    skybox.bind.fs_images[textured::SLOT_TEX as usize]
        = unsafe { sg::make_image(&image_desc) };

    let (shader, layout, depth) = textured::make_shader_etc(sg::query_backend());

    let pipeline_desc = PipelineDesc{
        shader,
        layout,
        depth,
        index_type: sg::IndexType::UInt16 as _,
        cull_mode: sg::CullMode::Front as _,
        label: cstr!("skybox-pipeline"),
        ..PipelineDesc::default()
    };
    /* create pipeline objects */
    skybox.pipe = unsafe { sg::make_pipeline(&pipeline_desc) };
}

pub fn draw(skybox: &State, view_proj: Mat4) {
    unsafe {
        sg::apply_pipeline(skybox.pipe);
        sg::apply_bindings(&skybox.bind);
    }

    textured::apply_uniforms(view_proj.to_column_major());

    unsafe { sg::draw(0, crate::CUBE_INDEX_COUNT, 1); }
}