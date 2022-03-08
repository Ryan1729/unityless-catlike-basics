fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=wrapper.c");
    println!("cargo:rerun-if-changed=third-party/sokol_gfx.h");
    println!("cargo:rerun-if-changed=third-party/sokol_app.h");
    println!("cargo:rerun-if-changed=third-party/sokol_glue.h");

    // TODO select this with a feature flag. Bindings generation will also need to
    // be updated.
    const BACKEND: &str = "SOKOL_GLCORE33";

    cc::Build::new()
        .file("wrapper.c")
        .define(BACKEND, None)
        .compile("wrapper");
}