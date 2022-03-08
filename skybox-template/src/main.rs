use sokol_bindings::*;

#[derive(Default)]
struct State {
    pass_action: sg_pass_action,
}

fn init(user_data: *mut ::std::os::raw::c_void) {
    let desc = &sg_desc{
        context: unsafe { sapp_sgcontext() },
        ..<_>::default()
    };
    unsafe { sg_setup(desc); }

    // SAFTEY: Neither we or sokol has switched the pointer out for another type.
    let state = unsafe { &mut*(user_data as *mut State) };

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
    }
}

fn frame(user_data: *mut ::std::os::raw::c_void) {
    // SAFTEY: Neither we or sokol has switched the pointer out for another type.
    let state = unsafe { &mut*(user_data as *mut State) };
    let pass_action = &mut state.pass_action;

    let g = pass_action.colors[0].value.g + 0.01;

    pass_action.colors[0].value.g = if g > 1. { 0. } else { g };
    unsafe {
        sg_begin_default_pass(pass_action as _, sapp_width(), sapp_height());
        sg_end_pass();
        sg_commit();
    }
}

fn main() {
    let window_title = concat!(env!("CARGO_CRATE_NAME"), "\0");

    unsafe extern "C" fn init_extern(user_data: *mut ::std::os::raw::c_void) {
        init(user_data)
    }

    unsafe extern "C" fn frame_extern(user_data: *mut ::std::os::raw::c_void) {
        frame(user_data)
    }

    unsafe extern "C" fn cleanup_extern(_user_data: *mut ::std::os::raw::c_void) {
        sg_shutdown()
    }

    let desc = &sapp_desc{
        user_data: &mut State::default() as *mut State as _,
        init_userdata_cb: Some(init_extern),
        frame_userdata_cb: Some(frame_extern),
        cleanup_userdata_cb: Some(cleanup_extern),
        width: 800,
        height: 600,
        sample_count: 4,
        window_title: window_title.as_ptr() as _,
        icon: sapp_icon_desc {
            sokol_default: true,
            ..<_>::default()
        },
        ..<_>::default()
    };

    unsafe{ sapp_run(desc) };
}
