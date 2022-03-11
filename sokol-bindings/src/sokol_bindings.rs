
///! TODO expose everything with more idomatic names and types
#[allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
pub use sokol_bindings_sys::*;

/// An alias for `c_int`. So in practice probably 32 bits, but technically allowed
/// to be as low as 16 on some platforms.
pub type Int = ::std::os::raw::c_int;

/// This convenience macro creates a userdata callback, for example for the sapp
/// desc struct. Note that the paramter type indicated *must* be the same as the
/// original type of the userdata field passed elsewhere, otherwise undefined
/// behavior could occur. (Technically if they are the same size it might work,
/// but if you want a union or whatever, there are better places to do that than
/// here.)
#[macro_export]
macro_rules! cb_wrap_userdata {
    ($cb: ident : fn(&mut $type: ty)) => {{
        unsafe extern "C" fn cb_extern(userdata: *mut ::std::os::raw::c_void) {
            // SAFTEY: This must be the same type that the userdata was initially
            // passed to `run` as.
            let paramter: &mut $type = unsafe { &mut*(userdata as *mut $type) };

            $cb(paramter)
        }

        Some(cb_extern)
    }}
}

pub mod sapp;
pub mod sg;
pub mod glue;

pub fn setup_default_context() {
    let desc = &sg_desc{
        context: glue::sapp_sgcontext(),
        ..<_>::default()
    };
    // SAFTEY: The desc value is valid. Using `sapp_sg_context` is explicitly
    // recommended by the header docs.
    unsafe { sg_setup(desc); }
}