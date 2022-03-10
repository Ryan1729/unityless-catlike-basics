use sokol_bindings::*;

#[derive(Default)]
struct State {
    pass_action: sg_pass_action,
}

fn init(state: &mut State) {
    let desc = &sg_desc{
        context: unsafe { sapp_sgcontext() },
        ..<_>::default()
    };
    unsafe { sg_setup(desc); }

    state.pass_action = sg_pass_action {
        colors:[
            sg_color_attachment_action{
                action: sg_action_SG_ACTION_CLEAR,
                value: sg_color { r: 1., g: 0., b: 0., a: 1.},
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
    unsafe {
        sg_begin_default_pass(pass_action as _, sapp_width(), sapp_height());
        sg_end_pass();
        sg_commit();
    }
}

fn cleanup(_state: &mut State) {
    unsafe { sg_shutdown() }
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
