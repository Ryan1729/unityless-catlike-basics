pub mod axes;

use sokol_bindings::{
    sg::{Action, PassAction, DepthAttachmentAction},
};

/// This pass action clears the depth buffer so that the debug visualizations
/// are drwan over everything.
pub fn pass_action() -> PassAction {
    let mut pass_action = PassAction::default();

    pass_action.depth = DepthAttachmentAction {
        action: Action::Clear,
        value: 0.,
    };

    pass_action
}