
use sokol_bindings_sys as sys;

pub fn shutdown() {
    // SAFETY: There are no currently known safety issues with this fn.
    unsafe{ sys::sg_shutdown() }
}