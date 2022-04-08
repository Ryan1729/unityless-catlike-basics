use sokol_bindings::{
    cstr,
    sg::{self, Backend, DepthState, LayoutDesc, ShaderDesc},
};

use crate::shaders::{ABGR, Index};
use math::{
    mat4::Mat4,
    vec3::{vec3, Vec3},
};

#[repr(C)]
#[derive(Debug)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub color: ABGR,
    pub u: i16,
    pub v: i16,
}

pub const VERTEX_DEFAULT: Vertex = Vertex {
    position: vec3!(),
    normal: vec3!(),
    color: 0xFFFFFFFF,
    u: 0,
    v: 0,
};

impl Default for Vertex {
    fn default() -> Self {
        VERTEX_DEFAULT
    }
}

#[macro_export]
macro_rules! _textured_lit_vertex {
    (
        $px: expr, $py: expr, $pz: expr,
        $nx: expr, $ny: expr, $nz: expr,
        $color: expr, $u: expr, $v: expr $(,)?
    ) => {
        $crate::shaders::textured_lit::Vertex {
            position: vec3!($px, $py, $pz),
            normal: vec3!($nx, $ny, $nz),
            color: $color,
            u: $u,
            v: $v,
        }
    }
}
pub use _textured_lit_vertex as vertex;


#[macro_export]
macro_rules! _textured_lit_vertex_array {
    (
        $(
            {
                $px: expr, $py: expr, $pz: expr,
                $nx: expr, $ny: expr, $nz: expr,
                $color: expr, $u: expr, $v: expr $(,)?
            }
        ),*

        $(,)?
    ) => {
        [
            $(
                $crate::shaders::textured_lit::Vertex {
                    position: vec3!($px, $py, $pz),
                    normal: vec3!($nx, $ny, $nz),
                    color: $color,
                    u: $u,
                    v: $v,
                }
            ),*
        ]
    }
}
pub use _textured_lit_vertex_array as vertex_array;

const ATTR_VS_POSITION: u8 = 0;
const ATTR_VS_NORMAL: u8 = 1;
const ATTR_VS_COLOR0: u8 = 2;
const ATTR_VS_TEXCOORD0: u8 = 3;
pub const SLOT_TEX: u8 = 0;
pub const SLOT_FS_PARAMS: u8 = 0;
pub const SLOT_VS_PARAMS: u8 = 0;

fn shader_desc(backend: Backend) -> ShaderDesc {
    // TODO support other backends besides `GlCore3_3`
    assert_eq!(backend, Backend::GlCore3_3);

    let mut desc = ShaderDesc::default();

    desc.attrs[ATTR_VS_POSITION as usize].name = cstr!("position");
    desc.attrs[ATTR_VS_NORMAL as usize].name = cstr!("normal");
    desc.attrs[ATTR_VS_COLOR0 as usize].name = cstr!("color0");
    desc.attrs[ATTR_VS_TEXCOORD0 as usize].name = cstr!("texcoord0");

    desc.vs.source = cstr!("#version 330

uniform vec4 vs_params[9];
layout(location = 0) in vec4 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec4 color0;
layout(location = 3) in vec2 texcoord0;
out vec4 vertexColor;
out vec2 uv;
out vec4 P;
out vec3 N;
out vec3 diffuseColor;

void main()
{
    mat4 mvp = mat4(vs_params[4], vs_params[5], vs_params[6], vs_params[7]);
    gl_Position = mvp * position;
    mat4 model = mat4(vs_params[0], vs_params[1], vs_params[2], vs_params[3]);
    P = model * position;
    N = (model * vec4(normal, 0.0)).xyz;
    diffuseColor = vs_params[8].xyz;

    vertexColor = color0;
    uv = texcoord0;
}");
    desc.vs.uniform_blocks[0].size = 144;
    desc.vs.uniform_blocks[0].layout = sg::UniformLayout::Std140 as _;
    desc.vs.uniform_blocks[0].uniforms[0].name = cstr!("vs_params");
    desc.vs.uniform_blocks[0].uniforms[0].type_ = sg::UniformType::Float4 as _;
    desc.vs.uniform_blocks[0].uniforms[0].array_count = 9;
    desc.vs.entry = cstr!("main");

    desc.fs.source = cstr!("#version 330

uniform vec4 fs_params[2];
uniform sampler2D tex;
in vec3 N;
in vec4 P;
layout(location = 0) out vec4 fragColor;
in vec4 vertexColor;
in vec2 uv;
in vec3 diffuseColor;

vec4 linearToGamma(vec4 c)
{
    return vec4(pow(c.xyz, vec3(1/2.2)), c.w);
}

float gammaToLinear(float c)
{
    return pow(c, 2.2);
}

void main()
{
    vec3 lightDir = normalize(fs_params[0].xyz);
    vec3 normal = normalize(N);
    float incidentLightFrac = dot(normal, lightDir);
    if (incidentLightFrac > 0.0)
    {
        vec3 eye = fs_params[1].xyz;
        float reflectedLightFrac = dot(
            reflect(-lightDir, normal),
            normalize(eye - P.xyz)
        );
        fragColor = vec4(
            (
                gammaToLinear(max(reflectedLightFrac, 0.0))
                * incidentLightFrac
            )
            + (diffuseColor * (incidentLightFrac + 0.25)),
            1.0
        );
    } else {
        fragColor = vec4(diffuseColor * 0.25, 1.0);
    }
    fragColor = linearToGamma(fragColor);

    fragColor *= texture(tex, uv) * vertexColor;
}");

    desc.fs.entry = cstr!("main");
    desc.fs.uniform_blocks[0].size = 32;
    desc.fs.uniform_blocks[0].layout = sg::UniformLayout::Std140 as _;
    desc.fs.uniform_blocks[0].uniforms[0].name = cstr!("fs_params");
    desc.fs.uniform_blocks[0].uniforms[0].type_ = sg::UniformType::Float4 as _;
    desc.fs.uniform_blocks[0].uniforms[0].array_count = 2;
    desc.fs.images[0].name = cstr!("tex");
    desc.fs.images[0].image_type = sg::ImageType::_2D as _;
    desc.fs.images[0].sampler_type = sg::SamplerType::Float as _;
    desc.label = cstr!("lit_shader");

    desc
}

fn layout_desc() -> LayoutDesc {
    use sg::VertexFormat;
    let mut layout = LayoutDesc::default();

    layout.attrs[ATTR_VS_POSITION as usize].format = VertexFormat::Float3 as _;
    layout.attrs[ATTR_VS_NORMAL as usize].format = VertexFormat::Float3 as _;
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

pub struct FSParams {
    pub light_dir: Vec3,
    pub eye_pos: Vec3,
}

pub struct VSParams {
    pub model: Mat4,
    pub mvp: Mat4,
    pub diffuse_colour: Vec3,
}

pub fn apply_uniforms(
    VSParams { model, mvp, diffuse_colour: d }: VSParams,
    FSParams { light_dir: l, eye_pos: e }: FSParams,
) {
    let m1 = model.to_column_major();
    let m2 = mvp.to_column_major();

    let vs_params_array = [
        m1[ 0], m1[ 1], m1[ 2], m1[ 3],
        m1[ 4], m1[ 5], m1[ 6], m1[ 7],
        m1[ 8], m1[ 9], m1[10], m1[11],
        m1[12], m1[13], m1[14], m1[15],
        m2[ 0], m2[ 1], m2[ 2], m2[ 3],
        m2[ 4], m2[ 5], m2[ 6], m2[ 7],
        m2[ 8], m2[ 9], m2[10], m2[11],
        m2[12], m2[13], m2[14], m2[15],
        d.x, d.y, d.z, 0.,
    ];

    let fs_params_array = [
        l.x, l.y, l.z, 0.,
        e.x, e.y, e.z, 0.,
    ];

    unsafe {
        sg::apply_uniforms(
            sg::ShaderStage::VS as _,
            SLOT_VS_PARAMS as _,
            &sg::range!(vs_params_array)
        );

        sg::apply_uniforms(
            sg::ShaderStage::FS as _,
            SLOT_FS_PARAMS as _,
            &sg::range!(fs_params_array)
        );
    }
}

pub struct IndexedMesh<const VERTEX_COUNT: usize, const INDEX_COUNT: usize> {
    pub vertices: [Vertex; VERTEX_COUNT],
    pub indices: [Index; INDEX_COUNT],
}