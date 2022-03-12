use sokol_bindings::{
    *,
    sapp::{self, IconDesc},
    setup_default_context,
    sg::{self, begin_default_pass, end_pass, commit, query_backend, range, Action, Backend, Color, ColorAttachmentAction, PassAction, ShaderDesc},
};

const ATTR_VS_POSITION: u8 = 0;
const ATTR_VS_COLOR0: u8 = 1;

fn cube_shader_desc(backend: Backend) -> ShaderDesc {
    // TODO support other backends besides `GlCore3_3`
    assert_eq!(backend, Backend::GlCore3_3);

    let mut desc = ShaderDesc::default();

    desc.attrs[ATTR_VS_POSITION as usize].name = b"position\0".as_ptr() as _;
    desc.attrs[ATTR_VS_COLOR0 as usize].name = b"color0\0".as_ptr() as _;
    desc.vs.source = br#"
#version 330
layout(location = 0) in vec4 position;
out vec4 color;
layout(location = 1) in vec4 color0;
void main()
{
    gl_Position = ((mat4(vec4(0.25, 0.0, 0.0, 0.0), vec4(0.0, 0.25, 0.0, 0.0), vec4(0.0, 0.0, 0.25, 0.0), vec4(0.0, 0.0, 0.0, 1.0)) * mat4(vec4(1.0, 0.0, 0.0, 0.0), vec4(0.0, 0.500459730625152587890625, -0.865759789943695068359375, 0.0), vec4(0.0, 0.865759789943695068359375, 0.500459730625152587890625, 0.0), vec4(0.0, 0.0, 0.0, 1.0))) * mat4(vec4(0.500459730625152587890625, -0.865759789943695068359375, 0.0, 0.0), vec4(0.865759789943695068359375, 0.500459730625152587890625, 0.0, 0.0), vec4(0.0, 0.0, 1.0, 0.0), vec4(0.0, 0.0, 0.0, 1.0))) * position;
    color = color0;
}
\0"#.as_ptr() as _;
    desc.vs.entry = b"main\0".as_ptr() as _;
    desc.fs.source = br#"
#version 330
        
layout(location = 0) out vec4 frag_color;
in vec4 color;

void main()
{
    frag_color = color;
}
\0"#.as_ptr() as _;
    desc.fs.entry = b"main\0".as_ptr() as _;
    desc.label = b"cube_shader\0".as_ptr() as _;

    desc
}

#[derive(Default)]
struct State {
    bind: sg_bindings,
    pipe: sg_pipeline,
}

/* cube vertex buffer */
const VERTICIES: [f32; 28 * 6] = [
    -1.0, -1.0, -1.0,   1.0, 0.0, 0.0, 1.0,
     1.0, -1.0, -1.0,   1.0, 0.0, 0.0, 1.0,
     1.0,  1.0, -1.0,   1.0, 0.0, 0.0, 1.0,
    -1.0,  1.0, -1.0,   1.0, 0.0, 0.0, 1.0,

    -1.0, -1.0,  1.0,   0.0, 1.0, 0.0, 1.0,
     1.0, -1.0,  1.0,   0.0, 1.0, 0.0, 1.0,
     1.0,  1.0,  1.0,   0.0, 1.0, 0.0, 1.0,
    -1.0,  1.0,  1.0,   0.0, 1.0, 0.0, 1.0,

    -1.0, -1.0, -1.0,   0.0, 0.0, 1.0, 1.0,
    -1.0,  1.0, -1.0,   0.0, 0.0, 1.0, 1.0,
    -1.0,  1.0,  1.0,   0.0, 0.0, 1.0, 1.0,
    -1.0, -1.0,  1.0,   0.0, 0.0, 1.0, 1.0,

    1.0, -1.0, -1.0,    1.0, 0.5, 0.0, 1.0,
    1.0,  1.0, -1.0,    1.0, 0.5, 0.0, 1.0,
    1.0,  1.0,  1.0,    1.0, 0.5, 0.0, 1.0,
    1.0, -1.0,  1.0,    1.0, 0.5, 0.0, 1.0,

    -1.0, -1.0, -1.0,   0.0, 0.5, 1.0, 1.0,
    -1.0, -1.0,  1.0,   0.0, 0.5, 1.0, 1.0,
     1.0, -1.0,  1.0,   0.0, 0.5, 1.0, 1.0,
     1.0, -1.0, -1.0,   0.0, 0.5, 1.0, 1.0,

    -1.0,  1.0, -1.0,   1.0, 0.0, 0.5, 1.0,
    -1.0,  1.0,  1.0,   1.0, 0.0, 0.5, 1.0,
     1.0,  1.0,  1.0,   1.0, 0.0, 0.5, 1.0,
     1.0,  1.0, -1.0,   1.0, 0.0, 0.5, 1.0
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
        label: b"cube-vertices\0".as_ptr() as _,
        ..<_>::default()
    };

    let vbuf = unsafe{ sg_make_buffer(&v_buffer_desc) };

    let i_buffer_desc = sg_buffer_desc{
        type_: sg_buffer_type_SG_BUFFERTYPE_INDEXBUFFER,
        data: range!(INDICES),
        label: b"cube-indices\0".as_ptr() as _,
        ..<_>::default()
    };

    let ibuf = unsafe { sg_make_buffer(&i_buffer_desc) };

    let shader_desc = cube_shader_desc(query_backend());
    let shader = unsafe { sg_make_shader(&shader_desc) };

    let mut layout = sg_layout_desc::default();
    /* test to provide buffer stride, but no attr offsets */
    layout.buffers[0].stride = 28;
    layout.attrs[ATTR_VS_POSITION as usize].format = sg_vertex_format_SG_VERTEXFORMAT_FLOAT3;
    layout.attrs[ATTR_VS_COLOR0 as usize].format = sg_vertex_format_SG_VERTEXFORMAT_FLOAT4;

    let mut depth = sg_depth_state::default();
    depth.write_enabled = true;
    depth.compare = sg_compare_func_SG_COMPAREFUNC_LESS_EQUAL;

    let pipeline_desc = sg_pipeline_desc{
        layout,
        shader,
        index_type: sg_index_type_SG_INDEXTYPE_UINT16,
        cull_mode: sg_cull_mode_SG_CULLMODE_BACK,
        depth,
        label: b"cube-pipeline\0".as_ptr() as _,
        ..sg_pipeline_desc::default()
    };
    /* create pipeline object */
    state.pipe = unsafe { sg_make_pipeline(&pipeline_desc) };

    /* setup resource bindings */
    state.bind = sg_bindings::default();
    state.bind.vertex_buffers[0] = vbuf;
    state.bind.index_buffer = ibuf;
}

fn frame(state: &mut State) {
    let mut pass_action = PassAction::default();
    pass_action.colors[0] = ColorAttachmentAction {
        action: Action::Clear,
        value: Color{ r: 0.25, g: 0.5, b: 0.75, a: 1. },
    };

    begin_default_pass(&pass_action, sapp::width(), sapp::height());
    unsafe {
        sg_apply_pipeline(state.pipe);
        sg_apply_bindings(&state.bind);
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
