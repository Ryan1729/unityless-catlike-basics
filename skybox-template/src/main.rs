use sokol_bindings::{
    *,
    cstr,
    sapp::{self, IconDesc},
    setup_default_context,
    sg::{self, begin_default_pass, end_pass, commit, query_backend, range, Action, Backend, Color, ColorAttachmentAction, PassAction, ShaderDesc},
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
    uv = texcoord0 * 5.0;
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
}

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
            {$x: literal, $y: literal, $z: literal, $color: literal, $u: literal, $v: literal $(,)?}
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
const VERTICIES: [Vertex; 24] = vertex_array![
    /* pos                  color       uvs */
    { -1.0, -1.0, -1.0,  0xFF0000FF,     0,     0 },
    {  1.0, -1.0, -1.0,  0xFF0000FF, 32767,     0 },
    {  1.0,  1.0, -1.0,  0xFF0000FF, 32767, 32767 },
    { -1.0,  1.0, -1.0,  0xFF0000FF,     0, 32767 },

    { -1.0, -1.0,  1.0,  0xFF00FF00,     0,     0 },
    {  1.0, -1.0,  1.0,  0xFF00FF00, 32767,     0 },
    {  1.0,  1.0,  1.0,  0xFF00FF00, 32767, 32767 },
    { -1.0,  1.0,  1.0,  0xFF00FF00,     0, 32767 },

    { -1.0, -1.0, -1.0,  0xFFFF0000,     0,     0 },
    { -1.0,  1.0, -1.0,  0xFFFF0000, 32767,     0 },
    { -1.0,  1.0,  1.0,  0xFFFF0000, 32767, 32767 },
    { -1.0, -1.0,  1.0,  0xFFFF0000,     0, 32767 },

    {  1.0, -1.0, -1.0,  0xFFFF007F,     0,     0 },
    {  1.0,  1.0, -1.0,  0xFFFF007F, 32767,     0 },
    {  1.0,  1.0,  1.0,  0xFFFF007F, 32767, 32767 },
    {  1.0, -1.0,  1.0,  0xFFFF007F,     0, 32767 },

    { -1.0, -1.0, -1.0,  0xFFFF7F00,     0,     0 },
    { -1.0, -1.0,  1.0,  0xFFFF7F00, 32767,     0 },
    {  1.0, -1.0,  1.0,  0xFFFF7F00, 32767, 32767 },
    {  1.0, -1.0, -1.0,  0xFFFF7F00,     0, 32767 },

    { -1.0,  1.0, -1.0,  0xFF007FFF,     0,     0 },
    { -1.0,  1.0,  1.0,  0xFF007FFF, 32767,     0 },
    {  1.0,  1.0,  1.0,  0xFF007FFF, 32767, 32767 },
    {  1.0,  1.0, -1.0,  0xFF007FFF,     0, 32767 },
];

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

    let image_bytes = include_bytes!("../../assets/skybox.png");
    let (header, image_data) = png_decoder::decode(image_bytes).unwrap();

    let mut image_desc = sg_image_desc::default();
    image_desc.width = header.width as _;
    image_desc.height = header.height as _;
    image_desc.data.subimage[0][0] = range!(&image_data);
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

    let vs_params: VsParams = [
        0.12511493265628815, 0.10831947781532758, 0.18738500347083775, 0.0,
        -0.21643994748592377, 0.06261498549435007, 0.10831947781532758, 0.0,
        0.0, -0.21643994748592377, 0.12511493265628815, 0.0,
        0.0, 0.0, 0.0, 1.0
    ];

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
