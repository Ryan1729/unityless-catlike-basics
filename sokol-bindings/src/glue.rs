
pub fn sapp_sgcontext() -> crate::sg::ContextDesc {
    // SAFETY: There are no currently known safety issues with this fn.
    unsafe { sokol_bindings_sys::sapp_sgcontext() }
}
