use sokol_bindings_sys as sys;
use crate::Int;

pub type Desc = sys::sapp_desc;
pub type Event = sys::sapp_event;
pub type IconDesc = sys::sapp_icon_desc;

/// This macro calls sapp::run for you. It syntactically ensures that the same
/// userdata type is used for each callback passed in the cbs section. It
/// assumes your userdata type implements `Default`. It also restrcts you to using
/// only userdata callbacks, but you can just ignore the parameter if you like. The
/// only userdata callbacks restriciton is mainly to simplfy the macro itself.
#[macro_export]
macro_rules! _run_with_userdata {
    (
        cbs: {
            type: $type: ty,
            $(init: $init: ident,)?
            $(frame: $frame: ident,)?
            $(cleanup: $cleanup: ident,)?
            $(event: $event: ident,)?
            $(fail: $fail: ident,)?
        },
        $desc: expr $(,)?
    ) => {
        let mut desc: $crate::sapp::Desc = $desc;

        {
            // Don't allow callbacks not passed in through the cbs section to avoid
            // possible undefined behaviour.
            macro_rules! a {
                // Can't do $($field: ident)+ in inner macros
                // See https://github.com/rust-lang/rust/issues/35853
                ($field: ident) => {
                    assert!(
                        desc.$field.is_none(),
                        "Only pass callbacks through the cbs section"
                    );
                }
            }
            a!(init_cb);
            a!(frame_cb);
            a!(cleanup_cb);
            a!(event_cb);
            a!(fail_cb);
            a!(init_userdata_cb);
            a!(frame_userdata_cb);
            a!(cleanup_userdata_cb);
            a!(event_userdata_cb);
            a!(fail_userdata_cb);
        }

        desc.user_data = &mut <$type>::default() as *mut $type as _;

        $( desc.init_userdata_cb = $crate::cb_wrap_userdata!($init : fn(&mut $type)); )?
        $( desc.frame_userdata_cb = $crate::cb_wrap_userdata!($frame : fn(&mut $type)); )?
        $( desc.cleanup_userdata_cb = $crate::cb_wrap_userdata!($cleanup : fn(&mut $type)); )?
        $(
            desc.event_userdata_cb = {
                unsafe extern "C" fn cb_extern(
                    event: *const $crate::sapp::Event,
                    userdata: *mut ::std::os::raw::c_void
                ) {
                    let event_paramter: &$crate::sapp::Event = &*event;
                    // SAFETY: The macro containing this code prevents the userdata
                    // from being used as a different type, which prevents it from
                    // being the wrong size.
                    let userdata_parameter: &mut $type = unsafe { &mut*(userdata as *mut $type) };


                    $event(event_paramter, userdata_parameter)
                }

                Some(cb_extern)
            };
        )?
        $(
            desc.fail_userdata_cb = {
                unsafe extern "C" fn cb_extern(
                    msg: *const ::std::os::raw::c_char,
                    userdata: *mut ::std::os::raw::c_void
                ) {
                    // SAFETY: Sokol passes us a valid, nul-terminated pointer with 
                    // an appropriate lifetime. As of this writing, only C string 
                    // literals are ever passed down, so we're safe there.
                    let msg_parameter = unsafe { std::ffi::CStr::from_ptr(msg) };
                    // SAFETY: The macro containing this code prevents the userdata
                    // from being used as a different type, which prevents it from
                    // being the wrong size.
                    let userdata_parameter: &mut $type = unsafe { &mut*(userdata as *mut $type) };


                    $fail(msg_parameter, userdata_parameter)
                }

                Some(cb_extern)
            };
        )?

        // SAFETY: The macro containing this code prevents the userdata from being
        // used as a different type, which prevents it from being the wrong size.
        unsafe { $crate::sapp::run(&desc); }
    };
}
pub use _run_with_userdata as run_with_userdata;

pub use sys::sapp_run as run;

pub fn width() -> Int {
    // SAFETY: There are no currently known safety issues with this fn.
    unsafe{ sys::sapp_width() }
}

pub fn height() -> Int {
    // SAFETY: There are no currently known safety issues with this fn.
    unsafe{ sys::sapp_height() }
}

