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
use sokol_extras::{shaders::{self, textured_lit}, images::white};

mod skybox;
mod decoded;

#[derive(Default)]
struct ModelState {
    bind: Bindings,
    pipe: Pipeline,
    scale: Vec3,
}

#[derive(Default)]
struct State {
    skybox: skybox::State,
    model: ModelState,
    eye: Vec3,
    center: Vec3,
    time: f32,
}

// Near/Far clipping plane distances along z.
const NEAR: f32 = 0.01;
// An f32 has 24 mantissa bits, so 2 to the 24th power seems reasonable here.
const FAR: f32 = 16777216.0;

struct IndexedMesh {
    pub vertices: [textured_lit::Vertex; math::geom::CYLINDER_POINT_COUNT_USIZE],
    pub indices: [shaders::Index; math::geom::CYLINDER_INDEX_COUNT_USIZE],
}

fn gen_mesh() -> IndexedMesh {
    use math::geom::Scale;
    let mesh = math::geom::gen_cylinder_mesh(Scale {
        x: 1./8.,
        y: 1./8.,
        z: 1./4.,
    });

    let mut vertices = [textured_lit::VERTEX_DEFAULT; math::geom::CYLINDER_POINT_COUNT_USIZE];
    for i in 0..math::geom::CYLINDER_POINT_COUNT_USIZE {
        let point = mesh.points[i];
        let normal = Vec3::from(mesh.normals[i]);

        vertices[i] = textured_lit::vertex!{
            point.x,
            point.y,
            point.z,
            normal.x,
            normal.y,
            normal.z,
            0xFFFFFFFF,
            0,
            0,
        };
    }

    IndexedMesh {
        vertices,
        indices: mesh.indices,
    }
}

fn init(state: &mut State) {
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

    state.model.bind.fs_images[textured_lit::SLOT_TEX as usize] = white::make();

    let (shader, layout, depth) = textured_lit::make_shader_etc(query_backend());

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

    state.model.scale = vec3!(10., 10., 0.2);
    state.eye = vec3!(0., 5.5, 1./64.);
    state.center = vec3!();
}

fn frame(state: &mut State) {
    state.time += sapp::frame_duration() as f32;
    let (sin, cos) = state.time.sin_cos();
    state.eye.x = sin * 10.;
    state.eye.z = cos * 10.;

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

    draw_model(&state.model, state.eye, view_proj);

    end_pass();

    commit();
}

fn draw_model(model: &ModelState, eye_pos: Vec3, view_proj: Mat4) {
    unsafe {
        sg::apply_pipeline(model.pipe);
        sg::apply_bindings(&model.bind);
    }

    let model = Mat4::scale(model.scale);

    let mvp = view_proj * model;

    let vs_params = textured_lit::VSParams {
        model,
        mvp,
        diffuse_colour: vec3!(1., 1., 1.),
    };

    let fs_params = textured_lit::FSParams {
        light_dir: vec3!(-1., 0., 1.),
        eye_pos,
    };

    textured_lit::apply_uniforms(vs_params, fs_params);

    unsafe { sg::draw(0, math::geom::CYLINDER_INDEX_COUNT as Int, 1); }
}

fn cleanup(_state: &mut State) {
    sg::shutdown()
}

fn event(event: &sapp::Event, state: &mut State) {
    use sapp::{EventKind, KeyCode, CTRL, SHIFT};

    const MOVE_SCALE: f32 = 2.;
    const SCALE_SCALE: f32 = 2.;

    match event.kind {
        EventKind::KeyDown { key_code, modifiers, .. } => {
            macro_rules! do_move {
                () => {
                    match key_code {
                        KeyCode::Minus => match modifiers {
                            0 => {state.model.scale.x /= SCALE_SCALE;},
                            CTRL => {state.model.scale.y /= SCALE_SCALE;},
                            SHIFT => {state.model.scale.z /= SCALE_SCALE;},
                            _ => {}
                        },
                        KeyCode::Plus => match modifiers {
                            0 => {state.model.scale.x *= SCALE_SCALE;},
                            CTRL => {state.model.scale.y *= SCALE_SCALE;},
                            SHIFT => {state.model.scale.z *= SCALE_SCALE;},
                            _ => {}
                        },
                        KeyCode::Right | KeyCode::Down => match modifiers {
                            0 => {state.eye.x /= MOVE_SCALE;},
                            CTRL => {state.eye.y /= MOVE_SCALE;},
                            SHIFT => {state.eye.z /= MOVE_SCALE;},
                            _ => {}
                        },
                        KeyCode::Left | KeyCode::Up => match modifiers {
                            0 => {state.eye.x *= MOVE_SCALE;},
                            CTRL => {state.eye.y *= MOVE_SCALE;},
                            SHIFT => {state.eye.z *= MOVE_SCALE;},
                            _ => {}
                        },
                        KeyCode::D | KeyCode::S => match modifiers {
                            0 => {state.center.x /= MOVE_SCALE;},
                            CTRL => {state.center.y /= MOVE_SCALE;},
                            SHIFT => {state.center.z /= MOVE_SCALE;},
                            _ => {}
                        },
                        KeyCode::A | KeyCode::W => match modifiers {
                            0 => {state.center.x *= MOVE_SCALE;},
                            CTRL => {state.center.y *= MOVE_SCALE;},
                            SHIFT => {state.center.z *= MOVE_SCALE;},
                            _ => {}
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
