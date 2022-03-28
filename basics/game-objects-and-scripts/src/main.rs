use sokol_bindings::{
    cstr,
    sapp::{self, IconDesc},
    setup_default_context,
    sg::{self, begin_default_pass, end_pass, commit, query_backend, Action, Bindings, Color, ColorAttachmentAction, PassAction, Pipeline, PipelineDesc},
    Int,
};
use math::{
    mat4::Mat4,
    vec3::{Vec3, vec3},
};
use sokol_extras::{textured, white_image};

mod skybox;
mod decoded;

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

fn gen_mesh() -> textured::IndexedMesh<
    {math::geom::CYLINDER_POINT_COUNT_USIZE},
    {math::geom::CYLINDER_INDEX_COUNT_USIZE},
> {
    use math::geom::Scale;
    let mesh = math::geom::gen_cylinder_mesh(Scale {
        x: 1./8.,
        y: 1./8.,
        z: 1./8.,
    });

    let mut vertices = [textured::VERTEX_DEFAULT; math::geom::CYLINDER_POINT_COUNT_USIZE];
    for (i, point) in mesh.points.iter().enumerate() {
        vertices[i] = textured::vertex!{
            point.x,
            point.y,
            point.z,
            0xFF000000 + ((i / 4) as textured::ABGR * 0x40 * 0x40) + ((i % 4) as textured::ABGR * 0x40),
            0,
            0,
        };
    }

    textured::IndexedMesh {
        vertices,
        indices: mesh.indices,
    }
}

fn init(state: &mut State) {
    state.eye = vec3!(0., 1.5, 1./16.);
    state.center = vec3!();

    setup_default_context();

    skybox::init(&mut state.skybox);

    let mesh = gen_mesh();

    let vertices = mesh.vertices;

    state.model.bind.vertex_buffers[0] = sg::make_immutable_vertex_buffer!(
        vertices,
        "model-vertices"
    );

    let indices = mesh.indices;

    state.model.bind.index_buffer = sg::make_immutable_index_buffer!(
        indices,
        "model-indices"
    );

    state.model.bind.fs_images[textured::SLOT_TEX as usize] = white_image::make();

    let (shader, layout, depth) = textured::make_shader_etc(query_backend());

    let pipeline_desc = PipelineDesc{
        layout,
        shader,
        index_type: sg::IndexType::UInt16 as _,
        cull_mode: sg::CullMode::Back as _,
        depth,
        label: cstr!("model-pipeline"),
        ..PipelineDesc::default()
    };
    state.model.pipe = unsafe { sg::make_pipeline(&pipeline_desc) };
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

fn draw_model(model: &ModelState, view_proj: Mat4) {
    unsafe {
        sg::apply_pipeline(model.pipe);
        sg::apply_bindings(&model.bind);
    }

    let mvp = view_proj;
    textured::apply_uniforms(mvp.to_column_major());

    unsafe { sg::draw(0, math::geom::CYLINDER_INDEX_COUNT as Int, 1); }
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
