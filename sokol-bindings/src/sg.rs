
use sokol_bindings_sys as sys;
use crate::Int;

pub use sys::sg_context_desc as ContextDesc;

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