use sokol_bindings::{
    *, // TODO Remove
    cstr,
    sapp::{self, IconDesc},
    setup_default_context,
    sg::{self, begin_default_pass, end_pass, commit, make_immutable_vertex_buffer, query_backend, range, Action, Bindings, Color, ColorAttachmentAction, ImageDesc, PassAction, Pipeline, PipelineDesc},
    Int,
};
use math::{
    mat4::Mat4,
    vec3::{Vec3, vec3},
};

mod textured {
    use sokol_bindings::{
        cstr,
        sg::{self, Backend, DepthState, LayoutDesc, ShaderDesc},
    };

    /// From most significant to least significant. So in a hex literal that's
    /// `0xAABBGGRR`, so `0xFFC08040` has full alpha, around 3/4 blue, around half green
    /// and adound 1/4 red.
    pub type ABGR = u32;

    #[derive(Default)]
    pub struct Vertex {
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

    #[macro_export]
    macro_rules! _vertex_array {
        (
            $(
                {$x: expr, $y: expr, $z: expr, $color: literal, $u: expr, $v: expr $(,)?}
            ),*

            $(,)?
        ) => {
            [
                $(
                    $crate::textured::Vertex {
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
    pub use _vertex_array as vertex_array;

    const ATTR_VS_POSITION: u8 = 0;
    const ATTR_VS_COLOR0: u8 = 1;
    const ATTR_VS_TEXCOORD0: u8 = 2;
    pub const SLOT_TEX: u8 = 0;
    pub const SLOT_VS_PARAMS: u8 = 0;

    fn shader_desc(backend: Backend) -> ShaderDesc {
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
        desc.vs.uniform_blocks[0].layout = sg::UniformLayout::Std140 as _;
        desc.vs.uniform_blocks[0].uniforms[0].name = cstr!("vs_params");
        desc.vs.uniform_blocks[0].uniforms[0].type_ = sg::UniformType::Float4 as _;
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
        desc.fs.images[0].image_type = sg::ImageType::_2D as _;
        desc.fs.images[0].sampler_type = sg::SamplerType::Float as _;
        desc.label = cstr!("textured_shader");

        desc
    }

    fn layout_desc() -> LayoutDesc {
        use sg::VertexFormat;
        let mut layout = LayoutDesc::default();

        layout.attrs[ATTR_VS_POSITION as usize].format = VertexFormat::Float3 as _;
        layout.attrs[ATTR_VS_COLOR0 as usize].format = VertexFormat::UByte4N as _;
        layout.attrs[ATTR_VS_TEXCOORD0 as usize].format = VertexFormat::Short2N as _;

        layout
    }

    fn depth_state() -> DepthState {
        let mut depth = DepthState::default();
        depth.write_enabled = true;
        depth.compare = sg::CompareFunc::LessEqual as _;
        depth
    }

    /// A `Shader` and some other parts of a `sg::Pipeline` that one is unlikely to
    /// change without also changing the shader code.
    pub type ShaderEtc = (sg::Shader, LayoutDesc, DepthState);

    pub fn make_shader_etc(backend: Backend) -> ShaderEtc {
        let shader_desc = shader_desc(backend);
        let shader = unsafe { sg::make_shader(&shader_desc) };

        (
            shader,
            layout_desc(),
            depth_state(),
        )
    }

    pub type VSParams = [f32; 4 * 4];

    pub fn apply_uniforms(vs_params: VSParams) {
        unsafe {
            sg::apply_uniforms(
                sg::ShaderStage::VS as _,
                SLOT_VS_PARAMS as _,
                &sg::range!(vs_params)
            );
        }
    }
}

#[derive(Default)]
struct ModelState {
    bind: Bindings,
    pipe: Pipeline,
}

#[derive(Default)]
struct State {
    skybox: skybox::State,
    model: ModelState,
    eye: Vec3,
    center: Vec3,
}

// Near/Far clipping plane distances along z.
const NEAR: f32 = 0.01;
// An f32 has 24 mantissa bits, so 2 to the 24th power seems reasonable here.
const FAR: f32 = 16777216.0;

const MODEL_VERTICIES: [textured::Vertex; 24] = {
    // Short for Cube Scale.
    const C_S: f32 = 1./8.;
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
        { -C_S, -C_S, -C_S,  0xFF00FF00, m!(0/1), m!(1/3) },
        {  C_S, -C_S, -C_S,  0xFF00FF00, m!(1/4), m!(1/3) },
        {  C_S,  C_S, -C_S,  0xFF00FF00, m!(1/2), m!(1/3) },
        { -C_S,  C_S, -C_S,  0xFF00FF00, m!(3/4), m!(1/3) },

        { -C_S, -C_S,  C_S,  0xFF00FF00, m!(0/1), m!(2/3) },
        {  C_S, -C_S,  C_S,  0xFF00FF00, m!(1/4), m!(2/3) },
        {  C_S,  C_S,  C_S,  0xFF00FF00, m!(1/2), m!(2/3) },
        { -C_S,  C_S,  C_S,  0xFF00FF00, m!(3/4), m!(2/3) },

        { -C_S, -C_S, -C_S,  0xFF00FF00, m!(3/4), m!(1/3) },
        { -C_S,  C_S, -C_S,  0xFF00FF00, m!(3/4), m!(1/3) },
        { -C_S,  C_S,  C_S,  0xFF00FF00, m!(1/1), m!(2/3) },
        { -C_S, -C_S,  C_S,  0xFF00FF00, m!(1/1), m!(2/3) },

        {  C_S, -C_S, -C_S,  0xFF00FF00, m!(1/4), m!(1/3) },
        {  C_S,  C_S, -C_S,  0xFF00FF00, m!(1/2), m!(1/3) },
        {  C_S,  C_S,  C_S,  0xFF00FF00, m!(1/2), m!(2/3) },
        {  C_S, -C_S,  C_S,  0xFF00FF00, m!(1/4), m!(2/3) },

        { -C_S, -C_S, -C_S,  0xFF00FF00, m!(0/1), m!(1/3) },
        { -C_S, -C_S,  C_S,  0xFF00FF00, m!(0/1), m!(2/3) },
        {  C_S, -C_S,  C_S,  0xFF00FF00, m!(1/4), m!(2/3) },
        {  C_S, -C_S, -C_S,  0xFF00FF00, m!(1/4), m!(1/3) },

        { -C_S,  C_S, -C_S,  0xFF00FF00, m!(3/4), m!(1/3) },
        { -C_S,  C_S,  C_S,  0xFF00FF00, m!(3/4), m!(2/3) },
        {  C_S,  C_S,  C_S,  0xFF00FF00, m!(1/2), m!(2/3) },
        {  C_S,  C_S, -C_S,  0xFF00FF00, m!(1/2), m!(1/3) },
    ]
};

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
    state.eye = vec3!(0., 1.5, 1./16.);
    state.center = vec3!();

    setup_default_context();

    skybox::init(&mut state.skybox);

    state.model.bind.vertex_buffers[0] = make_immutable_vertex_buffer!(
        MODEL_VERTICIES
        "model-vertices"
    );

    state.model.bind.index_buffer = cube::make_index_buffer();

    const WHITE_TEXTURE: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];

    let mut white_image_desc = ImageDesc::default();
    white_image_desc.width = 1;
    white_image_desc.height = 1;
    white_image_desc.data.subimage[0][0] = range!(WHITE_TEXTURE);
    white_image_desc.label = cstr!("white-texture");

    state.model.bind.fs_images[textured::SLOT_TEX as usize]
        = unsafe { sg::make_image(&white_image_desc) };

    let (shader, layout, depth) = textured::make_shader_etc(query_backend());

    let pipeline_desc = PipelineDesc{
        layout,
        shader,
        index_type: sg_index_type_SG_INDEXTYPE_UINT16,
        cull_mode: sg_cull_mode_SG_CULLMODE_BACK,
        depth,
        label: cstr!("cube-pipeline"),
        ..PipelineDesc::default()
    };
    state.model.pipe = unsafe { sg_make_pipeline(&pipeline_desc) };
}

fn draw_model(model: &ModelState, view_proj: Mat4) {
    unsafe {
        sg_apply_pipeline(model.pipe);
        sg_apply_bindings(&model.bind);
    }

    let mvp = view_proj;
    textured::apply_uniforms(mvp.to_column_major());

    unsafe { sg_draw(0, cube::INDEX_COUNT, 1); }
}

fn frame(state: &mut State) {
    let mut pass_action = PassAction::default();
    pass_action.colors[0] = ColorAttachmentAction {
        action: Action::Clear,
        value: Color{ r: 0.25, g: 0.5, b: 0.75, a: 1. },
    };

    let w = sapp::width();
    let h = sapp::height();

    /* compute model-view-projection matrix for vertex shader */
    let proj = Mat4::perspective(60., w as f32/h as f32, (NEAR, FAR));
    let view = get_view_matrix(state);
    let view_proj = proj * view;

    begin_default_pass(&pass_action, sapp::width(), sapp::height());

    skybox::draw(&state.skybox, view_proj);

    draw_model(&state.model, view_proj);

    end_pass();

    commit();
}

fn cleanup(_state: &mut State) {
    sg::shutdown()
}

fn event(event: &sapp::Event, state: &mut State) {
    use sapp::{EventKind, KeyCode};

    const MOVE_SCALE: f32 = 1./16.;

    match event.kind {
        EventKind::KeyDown { key_code, .. } => {
            macro_rules! do_move {
                () => {
                    match key_code {
                        KeyCode::Right => {
                            state.eye += vec3!(x) * MOVE_SCALE;
                        },
                        KeyCode::Left => {
                            state.eye -= vec3!(x) * MOVE_SCALE;
                        },
                        KeyCode::Down => {
                            state.eye -= vec3!(z) * MOVE_SCALE;
                        },
                        KeyCode::Up => {
                            state.eye += vec3!(z) * MOVE_SCALE;
                        },
                        KeyCode::D => {
                            state.center += vec3!(x) * MOVE_SCALE;
                        },
                        KeyCode::A => {
                            state.center -= vec3!(x) * MOVE_SCALE;
                        },
                        KeyCode::S => {
                            state.center -= vec3!(z) * MOVE_SCALE;
                        },
                        KeyCode::W => {
                            state.center += vec3!(z) * MOVE_SCALE;
                        },
                        _ => {}
                    }
                }
            }

            do_move!();

            let view = get_view_matrix(state);
            // Certain cases make the view matrix become degenerate. This can cause issues,
            // for example the skybox disappearing. In at least some of these cases,
            // doing the move again "fixes" the issue. So we  use that workaround.
            if (
                view.x_axis() == vec3!()
            ) || (
                view.y_axis() == vec3!()
            ) || (
                view.z_axis() == vec3!()
            ) {
                do_move!();
            }
        }
        _ => {}
    }
}

fn fail(_msg: &std::ffi::CStr, _state: &mut State) {

}

fn get_view_matrix(state: &State) -> Mat4 {
    Mat4::look_at(state.eye, state.center, vec3!(y))
}

mod cube {
    use sokol_bindings::{
        *, // TODO Remove
        cstr,
        sg::{
            make_buffer,
            range,
        },
        Int,
    };

    pub const INDEX_COUNT: Int = 36;

    /* create an index buffer for the cubes */
    pub const INDICES: [u16; INDEX_COUNT as usize] = [
        0, 1, 2,  0, 2, 3,
        6, 5, 4,  7, 6, 4,
        8, 9, 10,  8, 10, 11,
        14, 13, 12,  15, 14, 12,
        16, 17, 18,  16, 18, 19,
        22, 21, 20,  23, 22, 20
    ];

    pub fn make_index_buffer() -> sg_buffer {
        let i_buffer_desc = sg_buffer_desc{
            type_: sg_buffer_type_SG_BUFFERTYPE_INDEXBUFFER,
            data: range!(INDICES),
            label: cstr!("cube-indices"),
            ..<_>::default()
        };

        unsafe { make_buffer(&i_buffer_desc) }
    }
}

mod skybox {
    use crate::{textured, cube};
    use sokol_bindings::{
        *, // TODO Remove
        cstr,
        sg::{
            self,
            make_immutable_vertex_buffer,
            range,
            Bindings,
            Pipeline,
            PipelineDesc,
        },
    };
    use math::mat4::Mat4;

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

    pub fn init(skybox: &mut State) {
        skybox.bind.vertex_buffers[0] = make_immutable_vertex_buffer!(
            SKYBOX_VERTICIES
            "skybox-vertices"
        );

        skybox.bind.index_buffer = cube::make_index_buffer();

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
            index_type: sg_index_type_SG_INDEXTYPE_UINT16,
            cull_mode: sg_cull_mode_SG_CULLMODE_FRONT,
            label: cstr!("skybox-pipeline"),
            ..PipelineDesc::default()
        };
        /* create pipeline objects */
        skybox.pipe = unsafe { sg::make_pipeline(&pipeline_desc) };
    }

    pub fn draw(skybox: &State, view_proj: Mat4) {
        unsafe {
            sg_apply_pipeline(skybox.pipe);
            sg_apply_bindings(&skybox.bind);
        }

        textured::apply_uniforms(view_proj.to_column_major());

        unsafe { sg_draw(0, cube::INDEX_COUNT, 1); }
    }
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
            window_title: WINDOW_TITLE,
            icon: IconDesc {
                sokol_default: true,
                ..<_>::default()
            },
            ..<_>::default()
        }
    );
}
