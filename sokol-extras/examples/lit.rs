use sokol_bindings::{
    cstr,
    sapp::{self, IconDesc},
    setup_default_context,
    sg::{self, begin_default_pass, end_pass, commit, query_backend, Action, Bindings, Color, ColorAttachmentAction, PassAction, Pipeline, PipelineDesc},
};
use sokol_extras::{shaders::lit, checkerboard_image};

#[derive(Default)]
struct State {
    bind: Bindings,
    pipe: Pipeline,
}

const INDICES: [u16; 3] = [0, 1, 2];

fn init(state: &mut State) {
    setup_default_context();

    const VERTICES: [lit::Vertex; 3] = lit::vertex_array![
        /* pos                    color       uvs */
        { -1./4., -1./4., -1./4., 0xFFFF0000,     0,     0 },
        {     0.,  1./2., -1./4., 0xFF00FF00, 32767, 32767 },
        {  1./4., -1./4., -1./4., 0xFF0000FF, 32767,     0 },
    ];

    state.bind.vertex_buffers[0] = sg::make_immutable_vertex_buffer!(
        VERTICES
        "vertices"
    );

    state.bind.index_buffer = sg::make_immutable_index_buffer!(
        INDICES,
        "indices"
    );

    state.bind.fs_images[lit::SLOT_TEX as usize] = checkerboard_image::make();

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
        value: Color{ r: 0.25, g: 0.5, b: 0.75, a: 1. },
    };

    let w = sapp::width();
    let h = sapp::height();

    begin_default_pass(&pass_action, w, h);

    unsafe {
        sg::apply_pipeline(state.pipe);
        sg::apply_bindings(&state.bind);
    }

    lit::apply_uniforms([
        1., 0., 0., 0.,
        0., 1., 0., 0.,
        0., 0., 1., 0.,
        0., 0., 0., 1.,
    ]);

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
