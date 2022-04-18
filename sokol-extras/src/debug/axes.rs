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
    mat4::Mat4,
};
use crate::shaders::{self, basic};

#[derive(Default)]
pub struct State {
    pub bind: Bindings,
    pub pipe: Pipeline,
}

const CYLINDER_INDEX_START: shaders::Index = 0;
const CYLINDER_INDEX_ONE_PAST_END: shaders::Index = CYLINDER_INDEX_START + math::geom::CYLINDER_INDEX_COUNT as shaders::Index;
const INDEX_LEN: usize = CYLINDER_INDEX_ONE_PAST_END as usize;

const CYLINDER_VERTEX_START: shaders::Index = 0;
const CYLINDER_VERTEX_ONE_PAST_END: shaders::Index = CYLINDER_VERTEX_START + math::geom::CYLINDER_POINT_COUNT as shaders::Index;
const VERTEX_LEN: usize = CYLINDER_VERTEX_ONE_PAST_END as usize;
struct IndexedMesh {
    pub vertices: [basic::Vertex; VERTEX_LEN],
    pub indices: [shaders::Index; INDEX_LEN],
}

fn gen_mesh() -> IndexedMesh {
    let mut vertices = [basic::VERTEX_DEFAULT; VERTEX_LEN];
    let mut indices = [0; INDEX_LEN];

    let cylinder_mesh = math::geom::gen_cylinder_mesh(Scale {
        x: 1./8.,
        y: 1./8.,
        z: 1./4.,
    });

    for i in CYLINDER_VERTEX_START..CYLINDER_VERTEX_ONE_PAST_END {
        let read_i = (i - CYLINDER_VERTEX_START) as usize;
        let point = cylinder_mesh.points[read_i];

        vertices[i as usize] = basic::vertex!{
            point.x,
            point.y,
            point.z,
            0xFFFFFFFF,
        };
    }

    for i in CYLINDER_INDEX_START..CYLINDER_INDEX_ONE_PAST_END {
        let i = i as usize;
        indices[i] = cylinder_mesh.indices[i - CYLINDER_INDEX_START as usize]
            + CYLINDER_VERTEX_START as shaders::Index;
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