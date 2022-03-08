use sokol_bindings::*;

#[derive(Default)]
struct State {
    pass_action: sg_pass_action,
}

unsafe extern "C" fn init(user_data: *mut ::std::os::raw::c_void) {
    sg_setup(&sg_desc{
        context: sapp_sgcontext(),
        ..<_>::default()
    });

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

unsafe extern "C" fn frame(user_data: *mut ::std::os::raw::c_void) {
    // SAFTEY: Neither we or sokol has switched the pointer out for another type.
    let state = unsafe { &mut*(user_data as *mut State) };
    let pass_action = &mut state.pass_action;

    let g = pass_action.colors[0].value.g + 0.01;

    pass_action.colors[0].value.g = if g > 1. { 0. } else { g };
    sg_begin_default_pass(pass_action as _, sapp_width(), sapp_height());
    sg_end_pass();
    sg_commit();
}

unsafe extern "C" fn cleanup(_user_data: *mut ::std::os::raw::c_void) {
    sg_shutdown();
}

fn main() {
    let window_title = std::ffi::CString::new(env!("CARGO_CRATE_NAME"))
        .expect(concat!(
            "CARGO_CRATE_NAME contained a nul byte!:\n",
            env!("CARGO_CRATE_NAME")
        ));

    let desc = &sapp_desc{
        user_data: &mut State::default() as *mut State as _,
        init_userdata_cb: Some(init),
        frame_userdata_cb: Some(frame),
        cleanup_userdata_cb: Some(cleanup),
        width: 800,
        height: 600,
        sample_count: 4,
        window_title: window_title.as_ptr(),
        icon: sapp_icon_desc {
            sokol_default: true,
            ..<_>::default()
        },
        ..<_>::default()
    };

    unsafe{ sapp_run(desc) };
}
