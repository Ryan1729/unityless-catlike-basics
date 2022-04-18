use sokol_bindings::{
    cstr,
    sapp::{self, IconDesc},
    setup_default_context,
    sg::{self, begin_default_pass, end_pass, commit, query_backend, Action, Bindings, Color, ColorAttachmentAction, PassAction, Pipeline, PipelineDesc},
    Int,
};
use math::{
    angle::{Angle, Radians},
    mat4::Mat4,
    vec3::{Vec3, vec3},
};
use sokol_extras::{
    images::white,
    shaders::{self, textured_lit},
};

mod skybox;
mod decoded;

#[derive(Default)]
struct ModelState {
    bind: Bindings,
    pipe: Pipeline,
}

type Radius = f32;

#[derive(Default)]
struct Eye {
    x: Radians,
    y: Radians,
    z: Radians,
    radius: Radius
}

impl Eye {
    fn to_vec3(&self) -> Vec3 {
        vec3!(
            self.x.raw_radians().cos() * self.radius,
            self.y.raw_radians().cos() * self.radius,
            self.z.raw_radians().cos() * self.radius,
        )
    }
}

#[derive(Default)]
struct State {
    skybox: skybox::State,
    model: ModelState,
    eye: Eye,
    light_dir: Vec3,
    center: Vec3,
    time: f32,
}

// Near/Far clipping plane distances along z.
const NEAR: f32 = 0.01;
// An f32 has 24 mantissa bits, so 2 to the 24th power seems reasonable here.
const FAR: f32 = 16777216.0;

const CYLINDER_INDEX_START: shaders::Index = 0;
const CYLINDER_INDEX_ONE_PAST_END: shaders::Index = CYLINDER_INDEX_START + math::geom::CYLINDER_INDEX_COUNT as shaders::Index;
const CUBE_INDEX_START: shaders::Index = CYLINDER_INDEX_ONE_PAST_END;
const CUBE_INDEX_ONE_PAST_END: shaders::Index = CUBE_INDEX_START + math::geom::CUBE_INDEX_COUNT as shaders::Index;
const INDEX_LEN: usize = CUBE_INDEX_ONE_PAST_END as usize;

const CYLINDER_VERTEX_START: shaders::Index = 0;
const CYLINDER_VERTEX_ONE_PAST_END: shaders::Index = CYLINDER_VERTEX_START + math::geom::CYLINDER_POINT_COUNT as shaders::Index;
const CUBE_VERTEX_START: shaders::Index = CYLINDER_VERTEX_ONE_PAST_END;
const CUBE_VERTEX_ONE_PAST_END: shaders::Index = CUBE_VERTEX_START + math::geom::CUBE_POINT_COUNT as shaders::Index;
const VERTEX_LEN: usize = CUBE_VERTEX_ONE_PAST_END as usize;

struct IndexedMesh {
    pub vertices: [textured_lit::Vertex; VERTEX_LEN],
    pub indices: [shaders::Index; INDEX_LEN],
}

fn gen_mesh() -> IndexedMesh {
    use math::geom::Scale;

    let mut vertices = [textured_lit::VERTEX_DEFAULT; VERTEX_LEN];
    let mut indices = [0; INDEX_LEN];

    let cylinder_mesh = math::geom::gen_cylinder_mesh(Scale {
        x: 1./8.,
        y: 1./8.,
        z: 1./4.,
    });

    for i in CYLINDER_VERTEX_START..CYLINDER_VERTEX_ONE_PAST_END {
        let read_i = (i - CYLINDER_VERTEX_START) as usize;
        let point = cylinder_mesh.points[read_i];
        let normal = Vec3::from(cylinder_mesh.normals[read_i]);

        vertices[i as usize] = textured_lit::vertex!{
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

    for i in CYLINDER_INDEX_START..CYLINDER_INDEX_ONE_PAST_END {
        let i = i as usize;
        indices[i] = cylinder_mesh.indices[i - CYLINDER_INDEX_START as usize]
            + CYLINDER_VERTEX_START as shaders::Index;
    }

    let cube_mesh = math::geom::gen_cube_mesh(1./8.);

    for i in CUBE_VERTEX_START..CUBE_VERTEX_ONE_PAST_END {
        let read_i = (i - CUBE_VERTEX_START) as usize;
        let point = cube_mesh.points[read_i];
        let normal = Vec3::from(cube_mesh.normals[read_i]);

        vertices[i as usize] = textured_lit::vertex!{
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

    for i in CUBE_INDEX_START..CUBE_INDEX_ONE_PAST_END {
        let i = i as usize;
        indices[i] = cube_mesh.indices[i - CUBE_INDEX_START as usize]
            + CUBE_VERTEX_START;
    }

    IndexedMesh {
        vertices,
        indices,
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

    state.eye.x = Radians(-math::angle::TAU / 4.);
    state.eye.y = Radians(4.0);
    state.eye.z = Radians(4.375);
    state.eye.radius = 10.;
    state.light_dir = vec3!(1., -1., -1.);
    state.center = vec3!();
}

fn frame(state: &mut State) {
    state.time += sapp::frame_duration() as f32;

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

    draw_model(&state, view_proj);

    end_pass();

    commit();
}

fn draw_model(state: &State, view_proj: Mat4) {
    let model = &state.model;
    let eye_pos = state.eye.to_vec3();
    let light_dir = state.light_dir;

    unsafe {
        sg::apply_pipeline(model.pipe);
        sg::apply_bindings(&model.bind);
    }

    let fs_params = textured_lit::FSParams {
        light_dir,
        eye_pos,
    };

    let diffuse_colour = vec3!(1., 1., 1.);

    {
        let model = Mat4::rotation(Radians(math::angle::PI/2.), vec3!(x))
            * Mat4::scale(vec3!(10., 10., 0.2));

        textured_lit::apply_uniforms(
            textured_lit::VSParams {
                model,
                mvp: view_proj * model,
                diffuse_colour,
            },
            fs_params
        );

        unsafe {
            sg::draw(CYLINDER_INDEX_START as Int, CYLINDER_INDEX_ONE_PAST_END as Int, 1);
        }
    }

    {
        let model = Mat4::translate(vec3!(0., 4., -0.25)) *
            Mat4::scale(vec3!(0.5, 1., 0.1));

        textured_lit::apply_uniforms(
            textured_lit::VSParams {
                model,
                mvp: view_proj * model,
                diffuse_colour,
            },
            fs_params
        );

        unsafe {
            sg::draw(CUBE_INDEX_START as Int, CUBE_INDEX_ONE_PAST_END as Int, 1);
        }
    }
}

fn cleanup(_state: &mut State) {
    sg::shutdown()
}

fn event(event: &sapp::Event, state: &mut State) {
    use sapp::{EventKind, KeyCode, CTRL, SHIFT};

    const RADIUS_MOVE_SCALE: f32 = 1./16.;
    const ANGLE_MOVE_SCALE: Radians = Radians(1./8.);
    const LIGHT_DIR_SCALE: f32 = 1./32.;
    const CENTER_MOVE_SCALE: f32 = 1./32.;

    match event.kind {
        EventKind::KeyDown { key_code, modifiers, .. } => {
            macro_rules! do_move {
                () => {
                    match key_code {
                        KeyCode::Up => {
                            state.eye.radius += RADIUS_MOVE_SCALE;
                        },
                        KeyCode::Down => {
                            state.eye.radius -= RADIUS_MOVE_SCALE;
                        },
                        KeyCode::Right => match modifiers {
                            0 => {state.eye.x -= ANGLE_MOVE_SCALE;},
                            CTRL => {state.eye.y -= ANGLE_MOVE_SCALE;},
                            SHIFT => {state.eye.z -= ANGLE_MOVE_SCALE;},
                            _ => {}
                        },
                        KeyCode::Left => match modifiers {
                            0 => {state.eye.x += ANGLE_MOVE_SCALE;},
                            CTRL => {state.eye.y += ANGLE_MOVE_SCALE;},
                            SHIFT => {state.eye.z += ANGLE_MOVE_SCALE;},
                            _ => {}
                        },
                        KeyCode::D | KeyCode::S => match modifiers {
                            0 => {state.center.x /= CENTER_MOVE_SCALE;},
                            CTRL => {state.center.y /= CENTER_MOVE_SCALE;},
                            SHIFT => {state.center.z /= CENTER_MOVE_SCALE;},
                            _ => {}
                        },
                        KeyCode::A | KeyCode::W => match modifiers {
                            0 => {state.center.x *= CENTER_MOVE_SCALE;},
                            CTRL => {state.center.y *= CENTER_MOVE_SCALE;},
                            SHIFT => {state.center.z *= CENTER_MOVE_SCALE;},
                            _ => {}
                        },
                        KeyCode::V => match modifiers {
                            0 => {state.light_dir.x -= LIGHT_DIR_SCALE;},
                            CTRL => {state.light_dir.y -= LIGHT_DIR_SCALE;},
                            SHIFT => {state.light_dir.z -= LIGHT_DIR_SCALE;},
                            _ => {}
                        },
                        KeyCode::B => match modifiers {
                            0 => {state.light_dir.x += LIGHT_DIR_SCALE;},
                            CTRL => {state.light_dir.y += LIGHT_DIR_SCALE;},
                            SHIFT => {state.light_dir.z += LIGHT_DIR_SCALE;},
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
    Mat4::look_at(state.eye.to_vec3(), state.center, vec3!(y))
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
