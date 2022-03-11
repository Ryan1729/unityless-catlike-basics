use sokol_bindings::*;

use sokol_bindings::sg::{begin_default_pass, end_pass, commit, Action, Color, ColorAttachmentAction, PassAction};

#[derive(Default)]
struct State {
    pass_action: PassAction,
}

fn init(state: &mut State) {
    setup_default_context();

    state.pass_action = PassAction {
        colors:[
            ColorAttachmentAction {
                action: Action::Clear,
                value: Color { r: 1., g: 0., b: 0., a: 1.},
            },
            <_>::default(),
            <_>::default(),
            <_>::default(),
        ],
        ..<_>::default()
    };
}

fn frame(state: &mut State) {
    let pass_action = &mut state.pass_action;

    let g = pass_action.colors[0].value.g + 0.01;

    pass_action.colors[0].value.g = if g > 1. { 0. } else { g };

    begin_default_pass(pass_action, sapp::width(), sapp::height());
    end_pass();
    commit();
}

fn cleanup(_state: &mut State) {
    sg::shutdown()
}

fn event(_event: &sapp::Event, _state: &mut State) {

}

fn fail(_msg: &std::ffi::CStr, _state: &mut State) {

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
            window_title: WINDOW_TITLE.as_ptr() as _,
            icon: sapp_icon_desc {
                sokol_default: true,
                ..<_>::default()
            },
            ..<_>::default()
        }
    );
}
