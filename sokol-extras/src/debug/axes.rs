use sokol_bindings::{
    cstr,
    sg::{
        self,
        Bindings,
        Pipeline,
        PipelineDesc,
    },
    Int,
};
use math::{
    Scale,
    angle::{Radians, TAU},
    mat4::Mat4,
    vec3::vec3,
    vec4::vec4,
};
use crate::shaders::{self, basic};

#[derive(Default)]
pub struct State {
    pub bind: Bindings,
    pub pipe: Pipeline,
}

const Z_CYLINDER_INDEX_START: shaders::Index = 0;
const Z_CYLINDER_INDEX_ONE_PAST_END: shaders::Index = Z_CYLINDER_INDEX_START + math::geom::CYLINDER_INDEX_COUNT as shaders::Index;
const Y_CYLINDER_INDEX_START: shaders::Index = Z_CYLINDER_INDEX_ONE_PAST_END;
const Y_CYLINDER_INDEX_ONE_PAST_END: shaders::Index = Y_CYLINDER_INDEX_START + math::geom::CYLINDER_INDEX_COUNT as shaders::Index;
const X_CYLINDER_INDEX_START: shaders::Index = Y_CYLINDER_INDEX_ONE_PAST_END;
const X_CYLINDER_INDEX_ONE_PAST_END: shaders::Index = X_CYLINDER_INDEX_START + math::geom::CYLINDER_INDEX_COUNT as shaders::Index;
const INDEX_LEN: usize = X_CYLINDER_INDEX_ONE_PAST_END as usize;

const Z_CYLINDER_VERTEX_START: shaders::Index = 0;
const Z_CYLINDER_VERTEX_ONE_PAST_END: shaders::Index = Z_CYLINDER_VERTEX_START + math::geom::CYLINDER_POINT_COUNT as shaders::Index;
const Y_CYLINDER_VERTEX_START: shaders::Index = Z_CYLINDER_VERTEX_ONE_PAST_END;
const Y_CYLINDER_VERTEX_ONE_PAST_END: shaders::Index = Y_CYLINDER_VERTEX_START + math::geom::CYLINDER_POINT_COUNT as shaders::Index;
const X_CYLINDER_VERTEX_START: shaders::Index = Y_CYLINDER_VERTEX_ONE_PAST_END;
const X_CYLINDER_VERTEX_ONE_PAST_END: shaders::Index = X_CYLINDER_VERTEX_START + math::geom::CYLINDER_POINT_COUNT as shaders::Index;
const VERTEX_LEN: usize = X_CYLINDER_VERTEX_ONE_PAST_END as usize;
struct IndexedMesh {
    pub vertices: [basic::Vertex; VERTEX_LEN],
    pub indices: [shaders::Index; INDEX_LEN],
}

fn gen_mesh() -> IndexedMesh {
    let mut vertices = [basic::VERTEX_DEFAULT; VERTEX_LEN];
    let mut indices = [0; INDEX_LEN];

    let long_length = 1./16.;
    let short_length = 1./32.;

    let cylinder_mesh = math::geom::gen_cylinder_mesh(Scale {
        x: short_length,
        y: short_length,
        z: long_length,
    });

    for i in Z_CYLINDER_VERTEX_START..Z_CYLINDER_VERTEX_ONE_PAST_END {
        let read_i = (i - Z_CYLINDER_VERTEX_START) as usize;
        let point = cylinder_mesh.points[read_i];

        vertices[i as usize] = basic::vertex!{
            point.x,
            point.y,
            point.z + long_length + short_length,
            0xFFFF0000,
        };
    }

    for i in Z_CYLINDER_INDEX_START..Z_CYLINDER_INDEX_ONE_PAST_END {
        let i = i as usize;
        indices[i] = cylinder_mesh.indices[i - Z_CYLINDER_INDEX_START as usize]
            + Z_CYLINDER_VERTEX_START as shaders::Index;
    }

    let rotate_x = Mat4::rotation(Radians(TAU / 4.), vec3!(x));

    for i in Y_CYLINDER_VERTEX_START..Y_CYLINDER_VERTEX_ONE_PAST_END {
        let read_i = (i - Y_CYLINDER_VERTEX_START) as usize;
        let point = cylinder_mesh.points[read_i];

        let point = rotate_x * vec4!(point.x, point.y, point.z, 1.);

        vertices[i as usize] = basic::vertex!{
            point.x,
            point.y + long_length + short_length,
            point.z,
            0xFF00FF00,
        };
    }

    for i in Y_CYLINDER_INDEX_START..Y_CYLINDER_INDEX_ONE_PAST_END {
        let i = i as usize;
        indices[i] = cylinder_mesh.indices[i - Y_CYLINDER_INDEX_START as usize]
            + Y_CYLINDER_VERTEX_START as shaders::Index;
    }

    let rotate_y = Mat4::rotation(Radians(TAU / 4.), vec3!(x));

    for i in X_CYLINDER_VERTEX_START..X_CYLINDER_VERTEX_ONE_PAST_END {
        let read_i = (i - X_CYLINDER_VERTEX_START) as usize;
        let point = cylinder_mesh.points[read_i];

        let point = rotate_y * vec4!(point.x, point.y, point.z, 1.);

        vertices[i as usize] = basic::vertex!{
            point.x + long_length + short_length,
            point.y,
            point.z,
            0xFF000000 | ((point.x * 16_000_000_000.) as u32),
        };
    }

    for i in X_CYLINDER_INDEX_START..X_CYLINDER_INDEX_ONE_PAST_END {
        let i = i as usize;
        indices[i] = cylinder_mesh.indices[i - X_CYLINDER_INDEX_START as usize]
            + X_CYLINDER_VERTEX_START as shaders::Index;
    }

    IndexedMesh {
        vertices,
        indices,
    }
}

pub fn init(axes: &mut State) {
    let mesh = gen_mesh();

    let vertices = mesh.vertices;

    axes.bind.vertex_buffers[0] = sg::make_immutable_vertex_buffer!(
        vertices,
        "axes-vertices"
    );

    let indices = mesh.indices;

    axes.bind.index_buffer = sg::make_immutable_index_buffer!(
        indices,
        "axes-indices"
    );

    let (shader, layout, depth) = basic::make_shader_etc(sg::query_backend());

    let pipeline_desc = PipelineDesc{
        shader,
        layout,
        depth,
        index_type: sg::IndexType::UInt16 as _,
        cull_mode: sg::CullMode::Front as _,
        label: cstr!("axes-pipeline"),
        ..PipelineDesc::default()
    };
    /* create pipeline objects */
    axes.pipe = unsafe { sg::make_pipeline(&pipeline_desc) };
}

pub fn draw(axes: &State, mvp: Mat4) {
    unsafe {
        sg::apply_pipeline(axes.pipe);
        sg::apply_bindings(&axes.bind);
    }

    basic::apply_uniforms(mvp.to_column_major());

    unsafe { sg::draw(0, INDEX_LEN as Int, 1); }
}