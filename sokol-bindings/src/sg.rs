
use sokol_bindings_sys as sys;
use crate::Int;

pub use sys::sg_range as Range;

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

pub use sys::sg_context_desc as ContextDesc;
pub use sys::sg_shader_desc as ShaderDesc;
pub use sys::sg_image_desc as ImageDesc;

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
pub enum Usage {
    Default = 0,
    Immutable = 1,
    Dynamic = 2,
    Stream = 3,
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
