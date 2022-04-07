use sokol_bindings::{
    cstr,
    sg::{self, Backend, DepthState, LayoutDesc, ShaderDesc},
};

use crate::shaders::{Index, ABGR};

#[repr(C)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub color: ABGR,
    pub u: i16,
    pub v: i16,
}

pub const VERTEX_DEFAULT: Vertex = Vertex {
    x: 0.,
    y: 0.,
    z: 0.,
    color: 0xFF000000,
    u: 0,
    v: 0,
};

impl Default for Vertex {
    fn default() -> Self {
        VERTEX_DEFAULT
    }
}

#[macro_export]
macro_rules! _textured_vertex {
    (
        $x: expr, $y: expr, $z: expr, $color: expr, $u: expr, $v: expr $(,)?
    ) => {
        $crate::shaders::textured::Vertex {
            x: $x,
            y: $y,
            z: $z,
            color: $color,
            u: $u,
            v: $v,
        }
    }
}
pub use _textured_vertex as vertex;


#[macro_export]
macro_rules! _textured_vertex_array {
    (
        $(
            {$x: expr, $y: expr, $z: expr, $color: expr, $u: expr, $v: expr $(,)?}
        ),*

        $(,)?
    ) => {
        [
            $(
                $crate::shaders::textured::Vertex {
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
pub use _textured_vertex_array as vertex_array;

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

pub struct IndexedMesh<const VERTEX_COUNT: usize, const INDEX_COUNT: usize> {
    pub vertices: [Vertex; VERTEX_COUNT],
    pub indices: [Index; INDEX_COUNT],
}