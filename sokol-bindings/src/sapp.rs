use sokol_bindings_sys as sys;

pub type Desc = sys::sapp_desc;

/// This macro calls sapp::run for you. It syntactically ensures that the same 
/// userdata type is used for each callback passed in the cbs section. It 
/// assumes your userdata type implements `Default`.
#[macro_export]
macro_rules! _run_with_userdata {
    (
        cbs: {
            type: $type: ty,
            $(init: $init: ident,)?
            $(frame: $frame: ident,)?
            $(cleanup: $cleanup: ident,)?
        },
        $desc: expr $(,)?
    ) => {
        let mut desc = $desc;

        desc.user_data = &mut <$type>::default() as *mut $type as _;

        $( desc.init_userdata_cb = cb_wrap_userdata!($init : fn(&mut $type)); )?
        $( desc.frame_userdata_cb = cb_wrap_userdata!($frame : fn(&mut $type)); )?
        $( desc.cleanup_userdata_cb = cb_wrap_userdata!($cleanup : fn(&mut $type)); )?

        unsafe { $crate::sapp::run(&desc); }
    };
}
pub use _run_with_userdata as run_with_userdata;

pub use sys::sapp_run as run;
