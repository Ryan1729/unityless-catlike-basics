
use sokol_bindings_sys as sys;

pub use sys::sg_context_desc as ContextDesc;

pub fn shutdown() {
    // SAFETY: There are no currently known safety issues with this fn.
    unsafe{ sys::sg_shutdown() }
}

pub fn end_pass() {
    // SAFETY: There are no currently known safety issues with this fn.
    unsafe{ sys::sg_end_pass() }
}

pub fn commit() {
    // SAFETY: There are no currently known safety issues with this fn.
    unsafe{ sys::sg_commit() }
}