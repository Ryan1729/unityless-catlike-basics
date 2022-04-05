use sokol_bindings::{
    cstr,
    sapp::{self, IconDesc},
    setup_default_context,
    sg::{self, begin_default_pass, end_pass, commit, query_backend, Action, Bindings, Color, ColorAttachmentAction, PassAction, Pipeline, PipelineDesc},
    Int,
};
use sokol_extras::shaders::lit;
use math::{
    mat4::Mat4,
    vec3::vec3,
    vec4::vec4,
    angle::Radians,
};

#[derive(Default)]
struct State {
    bind: Bindings,
    pipe: Pipeline,
    ry: f32,
}

const CUBE_INDEX_COUNT: Int = 36;

const INDICES: [u16; CUBE_INDEX_COUNT as usize] = [
    0, 1, 2,  0, 2, 3,
    6, 5, 4,  7, 6, 4,
    8, 9, 10,  8, 10, 11,
    14, 13, 12,  15, 14, 12,
    16, 17, 18,  16, 18, 19,
    22, 21, 20,  23, 22, 20
];

fn init(state: &mut State) {
    setup_default_context();

    const VERTICES: [lit::Vertex; 24] = {
        macro_rules! m {
            (0/1) => {0};
            (1/1) => {32767};
            (1/4) => {32767/4};
            (1/3) => {32767/3};
            (1/2) => {32767/2};
            (2/3) => {m!(1/3) * 2};
            (3/4) => {m!(1/4) * 3};
        }
        lit::vertex_array![
            /* pos                  color       uvs */
            { -1., -1., -1.,  0xFF00FF00, m!(0/1), m!(1/3) },
            {  1., -1., -1.,  0xFF00FF00, m!(1/4), m!(1/3) },
            {  1.,  1., -1.,  0xFF00FF00, m!(1/2), m!(1/3) },
            { -1.,  1., -1.,  0xFF00FF00, m!(3/4), m!(1/3) },

            { -1., -1.,  1.,  0xFF00FF00, m!(0/1), m!(2/3) },
            {  1., -1.,  1.,  0xFF00FF00, m!(1/4), m!(2/3) },
            {  1.,  1.,  1.,  0xFF00FF00, m!(1/2), m!(2/3) },
            { -1.,  1.,  1.,  0xFF00FF00, m!(3/4), m!(2/3) },

            { -1., -1., -1.,  0xFF00FF00, m!(3/4), m!(1/3) },
            { -1.,  1., -1.,  0xFF00FF00, m!(3/4), m!(1/3) },
            { -1.,  1.,  1.,  0xFF00FF00, m!(1/1), m!(2/3) },
            { -1., -1.,  1.,  0xFF00FF00, m!(1/1), m!(2/3) },

            {  1., -1., -1.,  0xFF00FF00, m!(1/4), m!(1/3) },
            {  1.,  1., -1.,  0xFF00FF00, m!(1/2), m!(1/3) },
            {  1.,  1.,  1.,  0xFF00FF00, m!(1/2), m!(2/3) },
            {  1., -1.,  1.,  0xFF00FF00, m!(1/4), m!(2/3) },

            { -1., -1., -1.,  0xFF00FF00, m!(0/1), m!(1/3) },
            { -1., -1.,  1.,  0xFF00FF00, m!(0/1), m!(2/3) },
            {  1., -1.,  1.,  0xFF00FF00, m!(1/4), m!(2/3) },
            {  1., -1., -1.,  0xFF00FF00, m!(1/4), m!(1/3) },

            { -1.,  1., -1.,  0xFF00FF00, m!(3/4), m!(1/3) },
            { -1.,  1.,  1.,  0xFF00FF00, m!(3/4), m!(2/3) },
            {  1.,  1.,  1.,  0xFF00FF00, m!(1/2), m!(2/3) },
            {  1.,  1., -1.,  0xFF00FF00, m!(1/2), m!(1/3) },
        ]
    };

    state.bind.vertex_buffers[0] = sg::make_immutable_vertex_buffer!(
        VERTICES
        "vertices"
    );

    state.bind.index_buffer = sg::make_immutable_index_buffer!(
        INDICES,
        "indices"
    );

    let (shader, layout, depth) = lit::make_shader_etc(query_backend());

    let pipeline_desc = PipelineDesc{
        layout,
        shader,
        index_type: sg::IndexType::UInt16 as _,
        cull_mode: sg::CullMode::Back as _,
        depth,
        label: cstr!("cube-pipeline"),
        ..PipelineDesc::default()
    };
    state.pipe = unsafe { sg::make_pipeline(&pipeline_desc) };
}

fn frame(state: &mut State) {
    let mut pass_action = PassAction::default();
    pass_action.colors[0] = ColorAttachmentAction {
        action: Action::Clear,
        value: Color{ r: 0., g: 0.25, b: 1., a: 1. },
    };

    let w = sapp::width();
    let h = sapp::height();

    begin_default_pass(&pass_action, w, h);

    unsafe {
        sg::apply_pipeline(state.pipe);
        sg::apply_bindings(&state.bind);
    }

    let rym = Mat4::rotation(Radians(state.ry), vec3!(y));
    let light_dir = rym * vec4!(50., 50., -50., 0.);

    let eye_pos = vec3!(5., 5., 5.);

    let model = Mat4::identity();
    let proj = Mat4::perspective(60., w as f32/h as f32, (0.01, 100.));
    let view = Mat4::look_at(eye_pos, vec3!(), vec3!(y));
    let mvp = model * (proj * view);

    let vs_params = lit::VSParams {
        model,
        mvp,
        diffuse_colour: vec3!(1., 1., 0.),
    };

    let fs_params = lit::FSParams {
        light_dir: light_dir.xyz().normalize(),
        eye_pos,
    };

    lit::apply_uniforms(vs_params, fs_params);

    unsafe { sg::draw(0, INDICES.len() as _, 1); }

    end_pass();

    commit();
}

fn cleanup(_state: &mut State) {
    sg::shutdown()
}

fn main() {
    sapp::run_with_userdata!(
        cbs: {
            type: State,
            init: init,
            frame: frame,
            cleanup: cleanup,
        },
        sapp::Desc{
            width: 800,
            height: 600,
            sample_count: 4,
            window_title: concat!(file!(), "\0"),
            icon: IconDesc {
                sokol_default: true,
                ..<_>::default()
            },
            ..<_>::default()
        }
    );
}
