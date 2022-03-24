
use sokol_bindings_sys as sys;
use crate::Int;

pub use sys::sg_range as Range;
pub use sys::sg_bindings as Bindings;
pub use sys::sg_context_desc as ContextDesc;
pub use sys::sg_depth_state as DepthState;
pub use sys::sg_image as Image;
pub use sys::sg_image_desc as ImageDesc;
pub use sys::sg_layout_desc as LayoutDesc;
pub use sys::sg_pipeline as Pipeline;
pub use sys::sg_pipeline_desc as PipelineDesc;
pub use sys::sg_shader as Shader;
pub use sys::sg_shader_desc as ShaderDesc;

pub use sys::sg_make_image as make_image;
pub use sys::sg_make_pipeline as make_pipeline;
pub use sys::sg_make_shader as make_shader;
pub use sys::sg_apply_uniforms as apply_uniforms;

// TODO wrap everywhere we'd want to use this with things that use slices instead.
#[macro_export]
macro_rules! _range {
    // If you pass an ident, we can insert the `&` for you.
    ($arr_name: ident) => {
        $crate::sg::Range {
            size: core::mem::size_of_val(&$arr_name),
            ptr: &$arr_name as *const _ as _,
        }
    };
    // If you pass an expr, we don't want to evaluate the expr twice, so AFAIK
    // the cleanest way to do that is for you to add the `&`.
    ($arr_ref: expr) => {{
        let arr_ref: &[u8] = $arr_ref;
        $crate::sg::Range {
            size: core::mem::size_of_val(arr_ref),
            ptr: arr_ref as *const _ as _,
        }
    }}
}
pub use _range as range;

pub fn shutdown() {
    // SAFETY: There are no currently known safety issues with this fn.
    unsafe{ sys::sg_shutdown() }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Action {
    Default = sys::sg_action__SG_ACTION_DEFAULT,
    Clear = sys::sg_action_SG_ACTION_CLEAR,
    Load = sys::sg_action_SG_ACTION_LOAD,
    DontCare = sys::sg_action_SG_ACTION_DONTCARE,
}

impl Default for Action {
    fn default() -> Self {
        Self::Default
    }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Backend {
    GlCore3_3 = sys::sg_backend_SG_BACKEND_GLCORE33,
    Gles2 = sys::sg_backend_SG_BACKEND_GLES2,
    Gles3 = sys::sg_backend_SG_BACKEND_GLES3,
    D3D11 = sys::sg_backend_SG_BACKEND_D3D11,
    MetalIos = sys::sg_backend_SG_BACKEND_METAL_IOS,
    MetalMacos = sys::sg_backend_SG_BACKEND_METAL_MACOS,
    MetalSimulator = sys::sg_backend_SG_BACKEND_METAL_SIMULATOR,
    Wgpu = sys::sg_backend_SG_BACKEND_WGPU,
    Dummy = sys::sg_backend_SG_BACKEND_DUMMY,
}

pub fn query_backend() -> Backend {
    use Backend::*;

    // SAFETY: There are no currently known safety issues with this fn.
    let backend_int = unsafe{ sys::sg_query_backend() };

    match backend_int {
        sys::sg_backend_SG_BACKEND_GLCORE33 => GlCore3_3,
        sys::sg_backend_SG_BACKEND_GLES2 => Gles2,
        sys::sg_backend_SG_BACKEND_GLES3 => Gles3,
        sys::sg_backend_SG_BACKEND_D3D11 => D3D11,
        sys::sg_backend_SG_BACKEND_METAL_IOS => MetalIos,
        sys::sg_backend_SG_BACKEND_METAL_MACOS => MetalMacos,
        sys::sg_backend_SG_BACKEND_METAL_SIMULATOR => MetalSimulator,
        sys::sg_backend_SG_BACKEND_WGPU => Wgpu,
        sys::sg_backend_SG_BACKEND_DUMMY => Dummy,
        _ => unreachable!(),
    }
}

#[repr(u32)]
pub enum CompareFunc {
    Default = sys::sg_compare_func__SG_COMPAREFUNC_DEFAULT,
    Never = sys::sg_compare_func_SG_COMPAREFUNC_NEVER,
    Less = sys::sg_compare_func_SG_COMPAREFUNC_LESS,
    Equal = sys::sg_compare_func_SG_COMPAREFUNC_EQUAL,
    LessEqual = sys::sg_compare_func_SG_COMPAREFUNC_LESS_EQUAL,
    Greater = sys::sg_compare_func_SG_COMPAREFUNC_GREATER,
    NotEqual = sys::sg_compare_func_SG_COMPAREFUNC_NOT_EQUAL,
    GreaterEqual = sys::sg_compare_func_SG_COMPAREFUNC_GREATER_EQUAL,
    Always = sys::sg_compare_func_SG_COMPAREFUNC_ALWAYS,
}

impl Default for CompareFunc {
    fn default() -> Self {
        Self::Default
    }
}

#[repr(u32)]
pub enum ImageType {
    Default = sys::sg_image_type__SG_IMAGETYPE_DEFAULT,
    _2D = sys::sg_image_type_SG_IMAGETYPE_2D,
    Cube = sys::sg_image_type_SG_IMAGETYPE_CUBE,
    _3D = sys::sg_image_type_SG_IMAGETYPE_3D,
    Array = sys::sg_image_type_SG_IMAGETYPE_ARRAY,
}

impl Default for ImageType {
    fn default() -> Self {
        Self::Default
    }
}

#[repr(u32)]
pub enum SamplerType {
    Default = sys::sg_sampler_type__SG_SAMPLERTYPE_DEFAULT,
    Float = sys::sg_sampler_type_SG_SAMPLERTYPE_FLOAT,
    SInt = sys::sg_sampler_type_SG_SAMPLERTYPE_SINT,
    UInt = sys::sg_sampler_type_SG_SAMPLERTYPE_UINT,
}

impl Default for SamplerType {
    fn default() -> Self {
        Self::Default
    }
}

#[repr(u32)]
pub enum ShaderStage {
    VS = sys::sg_shader_stage_SG_SHADERSTAGE_VS,
    FS = sys::sg_shader_stage_SG_SHADERSTAGE_FS,
}

#[repr(u32)]
pub enum UniformLayout {
    Default = sys::sg_uniform_layout__SG_UNIFORMLAYOUT_DEFAULT,
    Native = sys::sg_uniform_layout_SG_UNIFORMLAYOUT_NATIVE,
    Std140 = sys::sg_uniform_layout_SG_UNIFORMLAYOUT_STD140,
}

impl Default for UniformLayout {
    fn default() -> Self {
        Self::Default
    }
}

#[repr(u32)]
pub enum UniformType {
    Invalid = sys::sg_uniform_type_SG_UNIFORMTYPE_INVALID,
    Float = sys::sg_uniform_type_SG_UNIFORMTYPE_FLOAT,
    Float2 = sys::sg_uniform_type_SG_UNIFORMTYPE_FLOAT2,
    Float3 = sys::sg_uniform_type_SG_UNIFORMTYPE_FLOAT3,
    Float4 = sys::sg_uniform_type_SG_UNIFORMTYPE_FLOAT4,
    Int = sys::sg_uniform_type_SG_UNIFORMTYPE_INT,
    Int2 = sys::sg_uniform_type_SG_UNIFORMTYPE_INT2,
    Int3 = sys::sg_uniform_type_SG_UNIFORMTYPE_INT3,
    Int4 = sys::sg_uniform_type_SG_UNIFORMTYPE_INT4,
    Mat4 = sys::sg_uniform_type_SG_UNIFORMTYPE_MAT4,
}

impl Default for UniformType {
    fn default() -> Self {
        Self::Invalid
    }
}

#[repr(u32)]
pub enum Usage {
    Default = sys::sg_usage__SG_USAGE_DEFAULT,
    Immutable = sys::sg_usage_SG_USAGE_IMMUTABLE,
    Dynamic = sys::sg_usage_SG_USAGE_DYNAMIC,
    Stream = sys::sg_usage_SG_USAGE_STREAM,
}

impl Default for Usage {
    fn default() -> Self {
        Self::Default
    }
}

#[repr(u32)]
pub enum VertexFormat {
    Invalid = sys::sg_vertex_format_SG_VERTEXFORMAT_INVALID,
    Float = sys::sg_vertex_format_SG_VERTEXFORMAT_FLOAT,
    Float2 = sys::sg_vertex_format_SG_VERTEXFORMAT_FLOAT2,
    Float3 = sys::sg_vertex_format_SG_VERTEXFORMAT_FLOAT3,
    Float4 = sys::sg_vertex_format_SG_VERTEXFORMAT_FLOAT4,
    Byte4 = sys::sg_vertex_format_SG_VERTEXFORMAT_BYTE4,
    Byte4N = sys::sg_vertex_format_SG_VERTEXFORMAT_BYTE4N,
    UByte4 = sys::sg_vertex_format_SG_VERTEXFORMAT_UBYTE4,
    UByte4N = sys::sg_vertex_format_SG_VERTEXFORMAT_UBYTE4N,
    Short2 = sys::sg_vertex_format_SG_VERTEXFORMAT_SHORT2,
    Short2N = sys::sg_vertex_format_SG_VERTEXFORMAT_SHORT2N,
    UShort2N = sys::sg_vertex_format_SG_VERTEXFORMAT_USHORT2N,
    Short4 = sys::sg_vertex_format_SG_VERTEXFORMAT_SHORT4,
    Short4N = sys::sg_vertex_format_SG_VERTEXFORMAT_SHORT4N,
    UShort4N = sys::sg_vertex_format_SG_VERTEXFORMAT_USHORT4N,
    UInt10N2 = sys::sg_vertex_format_SG_VERTEXFORMAT_UINT10_N2,
}

impl Default for VertexFormat {
    fn default() -> Self {
        Self::Invalid
    }
}

#[macro_export]
macro_rules! _make_immutable_vertex_buffer {
    (
        $vertex: ident $(,)?
        $label: literal $(,)?
    ) => {{
        let v_buffer_desc = $crate::sg::BufferDesc {
            usage: $crate::sg::Usage::Immutable as _,
            data: $crate::sg::range!($vertex),
            label: $crate::cstr!($label),
            ..<_>::default()
        };

        // SAFETY: The types of the arguments to this macro, and the macros those 
        // arguments are passed to, ensure that the `v_buffer_desc` is correct, 
        // for at least some given vertex size/type.
        unsafe{ $crate::sg::make_buffer(&v_buffer_desc) }
    }}
}

pub use _make_immutable_vertex_buffer as make_immutable_vertex_buffer;

pub type BufferDesc = sys::sg_buffer_desc;

pub use sys::sg_make_buffer as make_buffer;

pub use sys::sg_color as Color;

#[derive(Clone, Copy, Debug, Default)]
pub struct ColorAttachmentAction {
    pub action: Action,
    pub value: Color,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct DepthAttachmentAction {
    pub action: Action,
    pub value: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StencilAttachmentAction {
    pub action: Action,
    pub value: u8,
}

pub const MAX_COLOR_ATTACHMENTS: u8 = 4;

#[derive(Clone, Copy, Debug, Default)]
pub struct PassAction {
    pub colors: [ColorAttachmentAction; MAX_COLOR_ATTACHMENTS as usize],
    pub depth: DepthAttachmentAction,
    pub stencil: StencilAttachmentAction,
}

pub fn begin_default_pass(pass_action: &PassAction, width: Int, height: Int) {
    type ActionInternal = u32;

    let mut pass_action_parameter = sys::sg_pass_action::default();
    pass_action_parameter.colors = [
        sys::sg_color_attachment_action {
            action: pass_action.colors[0].action as ActionInternal,
            value: pass_action.colors[0].value,
        },
        sys::sg_color_attachment_action {
            action: pass_action.colors[1].action as ActionInternal,
            value: pass_action.colors[1].value,
        },
        sys::sg_color_attachment_action {
            action: pass_action.colors[2].action as ActionInternal,
            value: pass_action.colors[2].value,
        },
        sys::sg_color_attachment_action {
            action: pass_action.colors[3].action as ActionInternal,
            value: pass_action.colors[3].value,
        },
    ];
    pass_action_parameter.depth = sys::sg_depth_attachment_action {
        action: pass_action.depth.action as ActionInternal,
        value: pass_action.depth.value,
    };
    pass_action_parameter.stencil = sys::sg_stencil_attachment_action {
        action: pass_action.stencil.action as ActionInternal,
        value: pass_action.stencil.value,
    };

    // SAFETY: The PassAction type ensures that the parameter made above is valid.
    unsafe { sys::sg_begin_default_pass(&pass_action_parameter as _, width, height); }
}

pub fn end_pass() {
    // SAFETY: There are no currently known safety issues with this fn.
    unsafe{ sys::sg_end_pass() }
}

pub fn commit() {
    // SAFETY: There are no currently known safety issues with this fn.
    unsafe{ sys::sg_commit() }
}
