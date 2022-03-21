use sokol_bindings::{
    *,
    cstr,
    sapp::{self, IconDesc, frame_duration, FPS},
    setup_default_context,
    sg::{self, begin_default_pass, end_pass, commit, query_backend, range, Action, Backend, Color, ColorAttachmentAction, PassAction, ShaderDesc},
};
use math::{
    angle::Degrees,
    mat4::Mat4,
    vec3::vec3,
};

const ATTR_VS_POSITION: u8 = 0;
const ATTR_VS_COLOR0: u8 = 1;
const ATTR_VS_TEXCOORD0: u8 = 2;
const SLOT_TEX: u8 = 0;
const SLOT_VS_PARAMS: u8 = 0;

fn cube_shader_desc(backend: Backend) -> ShaderDesc {
    // TODO support other backends besides `GlCore3_3`
    assert_eq!(backend, Backend::GlCore3_3);

    let mut desc = ShaderDesc::default();

    desc.attrs[ATTR_VS_POSITION as usize].name = cstr!("position");
    desc.attrs[ATTR_VS_COLOR0 as usize].name = cstr!("color0");
    desc.attrs[ATTR_VS_TEXCOORD0 as usize].name = cstr!("texcoord0");
    desc.vs.source = cstr!("#version 330

uniform vec4 vs_params[4];
layout(location = 0) in vec4 position;
out vec4 color;
layout(location = 1) in vec4 color0;
out vec2 uv;
layout(location = 2) in vec2 texcoord0;

void main()
{
    gl_Position = mat4(vs_params[0], vs_params[1], vs_params[2], vs_params[3]) * position;
    color = color0;
    uv = texcoord0;
}
");
    desc.vs.uniform_blocks[0].size = 64;
    desc.vs.uniform_blocks[0].layout = sg_uniform_layout_SG_UNIFORMLAYOUT_STD140;
    desc.vs.uniform_blocks[0].uniforms[0].name = cstr!("vs_params");
    desc.vs.uniform_blocks[0].uniforms[0].type_ = sg_uniform_type_SG_UNIFORMTYPE_FLOAT4;
    desc.vs.uniform_blocks[0].uniforms[0].array_count = 4;
    desc.vs.entry = cstr!("main");
    desc.fs.source = cstr!("#version 330

uniform sampler2D tex;

layout(location = 0) out vec4 frag_color;
in vec2 uv;
in vec4 color;

void main()
{
    frag_color = texture(tex, uv) * color;
}");
    desc.fs.entry = cstr!("main");
    desc.fs.images[0].name = cstr!("tex");
    desc.fs.images[0].image_type = sg_image_type_SG_IMAGETYPE_2D;
    desc.fs.images[0].sampler_type = sg_sampler_type_SG_SAMPLERTYPE_FLOAT;
    desc.label = cstr!("cube_shader");

    desc
}

#[derive(Default)]
struct State {
    bind: sg_bindings,
    pipe: sg_pipeline,
    rx: Degrees,
    ry: Degrees,
}

/// From most significant to least significant. So in a hex literal that's
/// `0xAABBGGRR`, so `0xFFC08040` has full alpha, around 3/4 blue, around half green
/// and adound 1/4 red.
type ABGR = u32;

#[derive(Default)]
struct Vertex {
    //We use these in the shader, after passing to them to `sg_make_buffer`.
    #[allow(dead_code)]
    pub x: f32,
    #[allow(dead_code)]
    pub y: f32,
    #[allow(dead_code)]
    pub z: f32,
    #[allow(dead_code)]
    pub color: ABGR,
    #[allow(dead_code)]
    pub u: i16,
    #[allow(dead_code)]
    pub v: i16,
}

macro_rules! vertex_array {
    (
        $(
            {$x: literal, $y: literal, $z: literal, $color: literal, $u: expr, $v: expr $(,)?}
        ),*

        $(,)?
    ) => {
        [
            $(
                Vertex {
                    x: $x,
                    y: $y,
                    z: $z,
                    color: $color,
                    u: $u,
                    v: $v,
                }
            ),*
        ]
    }
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
const VERTICIES: [Vertex; 24] = {
    macro_rules! m {
        (0/1) => {0};
        (1/1) => {32767};
        (1/4) => {32767/4};
        (1/3) => {32767/3};
        (1/2) => {32767/2};
        (2/3) => {m!(1/3) * 2};
        (3/4) => {m!(1/4) * 3};
    }
    vertex_array![
        /* pos                  color       uvs */
        { -1.0, -1.0, -1.0,  0xFFFFFFFF, m!(0/1), m!(1/3) },
        {  1.0, -1.0, -1.0,  0xFFFFFFFF, m!(1/4), m!(1/3) },
        {  1.0,  1.0, -1.0,  0xFFFFFFFF, m!(1/2), m!(1/3) },
        { -1.0,  1.0, -1.0,  0xFFFFFFFF, m!(3/4), m!(1/3) },

        { -1.0, -1.0,  1.0,  0xFFFFFFFF, m!(0/1), m!(2/3) },
        {  1.0, -1.0,  1.0,  0xFFFFFFFF, m!(1/4), m!(2/3) },
        {  1.0,  1.0,  1.0,  0xFFFFFFFF, m!(1/2), m!(2/3) },
        { -1.0,  1.0,  1.0,  0xFFFFFFFF, m!(3/4), m!(2/3) },

        { -1.0, -1.0, -1.0,  0xFFFFFFFF, m!(3/4), m!(1/3) },
        { -1.0,  1.0, -1.0,  0xFFFFFFFF, m!(3/4), m!(1/3) },
        { -1.0,  1.0,  1.0,  0xFFFFFFFF, m!(1/1), m!(2/3) },
        { -1.0, -1.0,  1.0,  0xFFFFFFFF, m!(1/1), m!(2/3) },

        {  1.0, -1.0, -1.0,  0xFFFFFFFF, m!(1/4), m!(1/3) },
        {  1.0,  1.0, -1.0,  0xFFFFFFFF, m!(1/2), m!(1/3) },
        {  1.0,  1.0,  1.0,  0xFFFFFFFF, m!(1/2), m!(2/3) },
        {  1.0, -1.0,  1.0,  0xFFFFFFFF, m!(1/4), m!(2/3) },

        { -1.0, -1.0, -1.0,  0xFFFFFFFF, m!(0/1), m!(1/3) },
        { -1.0, -1.0,  1.0,  0xFFFFFFFF, m!(0/1), m!(2/3) },
        {  1.0, -1.0,  1.0,  0xFFFFFFFF, m!(1/4), m!(2/3) },
        {  1.0, -1.0, -1.0,  0xFFFFFFFF, m!(1/4), m!(1/3) },

        { -1.0,  1.0, -1.0,  0xFFFFFFFF, m!(3/4), m!(1/3) },
        { -1.0,  1.0,  1.0,  0xFFFFFFFF, m!(3/4), m!(2/3) },
        {  1.0,  1.0,  1.0,  0xFFFFFFFF, m!(1/2), m!(2/3) },
        {  1.0,  1.0, -1.0,  0xFFFFFFFF, m!(1/2), m!(1/3) },
    ]
};

const INDEX_COUNT: Int = 36;

/* create an index buffer for the cube */
const INDICES: [u16; INDEX_COUNT as usize] = [
    0, 1, 2,  0, 2, 3,
    6, 5, 4,  7, 6, 4,
    8, 9, 10,  8, 10, 11,
    14, 13, 12,  15, 14, 12,
    16, 17, 18,  16, 18, 19,
    22, 21, 20,  23, 22, 20
];

/// The byte at index 0 is the red channel, index 1 green, index 2 blue, index 3
// alpha, index 4 red again, and so on.
type RGBAVec<'image> = std::borrow::Cow<'image, [u8]>;

struct DecodedPng<'image> {
    w: Int,
    h: Int,
    image_bytes: RGBAVec<'image>,
}

fn decode_png_with_checkerboard_fallback<'image>(png_bytes: &[u8]) -> DecodedPng<'image> {
    match png_decoder::decode(png_bytes) {
        Ok((header, image_data)) => DecodedPng {
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

            DecodedPng {
                w: W,
                h: H,
                image_bytes: RGBAVec::from(CHECKERBOARD.as_slice()),
            }
        }
    }
}

fn init(state: &mut State) {
    setup_default_context();

    let v_buffer_desc = sg_buffer_desc{
        data: range!(VERTICIES),
        label: cstr!("cube-vertices"),
        ..<_>::default()
    };

    state.bind.vertex_buffers[0] = unsafe{ sg_make_buffer(&v_buffer_desc) };

    let i_buffer_desc = sg_buffer_desc{
        type_: sg_buffer_type_SG_BUFFERTYPE_INDEXBUFFER,
        data: range!(INDICES),
        label: cstr!("cube-indices"),
        ..<_>::default()
    };

    state.bind.index_buffer = unsafe { sg_make_buffer(&i_buffer_desc) };

    let decoded = decode_png_with_checkerboard_fallback(
        include_bytes!("../../assets/skybox.png"),
    );

    let mut image_desc = sg_image_desc::default();
    image_desc.width = decoded.w;
    image_desc.height = decoded.h;
    image_desc.data.subimage[0][0] = range!(&decoded.image_bytes);
    image_desc.label = cstr!("cube-texture");

    state.bind.fs_images[SLOT_TEX as usize] = unsafe { sg_make_image(&image_desc) };

    let shader_desc = cube_shader_desc(query_backend());
    let shader = unsafe { sg_make_shader(&shader_desc) };

    let mut layout = sg_layout_desc::default();
    layout.attrs[ATTR_VS_POSITION as usize].format = sg_vertex_format_SG_VERTEXFORMAT_FLOAT3;
    layout.attrs[ATTR_VS_COLOR0 as usize].format = sg_vertex_format_SG_VERTEXFORMAT_UBYTE4N;
    layout.attrs[ATTR_VS_TEXCOORD0 as usize].format = sg_vertex_format_SG_VERTEXFORMAT_SHORT2N;

    let mut depth = sg_depth_state::default();
    depth.write_enabled = true;
    depth.compare = sg_compare_func_SG_COMPAREFUNC_LESS_EQUAL;

    let pipeline_desc = sg_pipeline_desc{
        layout,
        shader,
        index_type: sg_index_type_SG_INDEXTYPE_UINT16,
        cull_mode: sg_cull_mode_SG_CULLMODE_BACK,
        depth,
        label: cstr!("cube-pipeline"),
        ..sg_pipeline_desc::default()
    };
    /* create pipeline object */
    state.pipe = unsafe { sg_make_pipeline(&pipeline_desc) };
}

type VsParams = [f32; 4 * 4];

fn frame(state: &mut State) {
    let mut pass_action = PassAction::default();
    pass_action.colors[0] = ColorAttachmentAction {
        action: Action::Clear,
        value: Color{ r: 0.25, g: 0.5, b: 0.75, a: 1. },
    };

    let w = sapp::width();
    let h = sapp::height();

    /* compute model-view-projection matrix for vertex shader */
    let t = (frame_duration() * FPS as f64) as f32;
    let proj = Mat4::perspective(60., w as f32/h as f32, (0.01, 10.));
    let view = Mat4::look_at(vec3!(0., 1.5, 6.), vec3!(), vec3!(0., 1., 0.));
    let view_proj = proj * view;
    state.rx += Degrees(1. * t);
    state.ry += Degrees(2. * t);
    let rxm = Mat4::rotation(state.rx, vec3!(x));
    let rym = Mat4::rotation(state.ry, vec3!(y));
    let model = rxm * rym;
    let mvp = view_proj * model;

    let vs_params: VsParams = mvp.to_column_major();

    begin_default_pass(&pass_action, sapp::width(), sapp::height());
    unsafe {
        sg_apply_pipeline(state.pipe);
        sg_apply_bindings(&state.bind);
        sg_apply_uniforms(
            sg_shader_stage_SG_SHADERSTAGE_VS,
            SLOT_VS_PARAMS as _,
            &range!(vs_params)
        );
        sg_draw(0, INDEX_COUNT, 1);
    }
    end_pass();
    commit();
}

fn cleanup(_state: &mut State) {
    sg::shutdown()
}

fn event(_event: &sapp::Event, _state: &mut State) {

}

fn fail(_msg: &std::ffi::CStr, _state: &mut State) {

}

fn main() {
    const WINDOW_TITLE: &str = concat!(env!("CARGO_CRATE_NAME"), "\0");

    sapp::run_with_userdata!(
        cbs: {
            type: State,
            init: init,
            frame: frame,
            cleanup: cleanup,
            event: event,
            fail: fail,
        },
        sapp::Desc{
            width: 800,
            height: 600,
            sample_count: 4,
            window_title: WINDOW_TITLE.as_ptr() as _,
            icon: IconDesc {
                sokol_default: true,
                ..<_>::default()
            },
            ..<_>::default()
        }
    );
}
