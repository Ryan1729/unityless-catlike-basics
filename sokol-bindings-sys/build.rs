fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=wrapper.c");
    println!("cargo:rerun-if-changed=third-party/sokol_gfx.h");
    println!("cargo:rerun-if-changed=third-party/sokol_app.h");
    println!("cargo:rerun-if-changed=third-party/sokol_glue.h");

    // TODO link to different platform specific libraries on other platforms
    println!("cargo:rustc-link-lib=GL");
    println!("cargo:rustc-link-lib=dl");
    println!("cargo:rustc-link-lib=m");
    println!("cargo:rustc-link-lib=X11");
    println!("cargo:rustc-link-lib=asound");
    println!("cargo:rustc-link-lib=Xi");
    println!("cargo:rustc-link-lib=Xcursor");

    // TODO select this with a feature flag. Bindings generation will also need to
    // be updated.
    const BACKEND: &str = "SOKOL_GLCORE33";

    cc::Build::new()
        .file("wrapper.c")
        .define(BACKEND, None)
        .flag("-pthread")
        .include("../third-party")
        .compile("wrapper");
}