pub mod axes;

use sokol_bindings::{
    sg::{
        Action,
        PassAction,
        DepthAttachmentAction,
        DEFAULT_CLEAR_DEPTH,
    },
};

/// This pass action clears the depth buffer so that the debug visualizations
/// are drawn over everything.
pub fn pass_action() -> PassAction {
    let mut pass_action = PassAction::load();

    pass_action.depth = DepthAttachmentAction {
        action: Action::Clear,
        value: DEFAULT_CLEAR_DEPTH,
    };

    pass_action
}