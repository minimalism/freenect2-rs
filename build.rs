use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    let lib = pkg_config::Config::new()
        .atleast_version("0.2")
        .probe("freenect2")
        .expect("freenect2 not found via pkg-config; is it installed to /usr/local?");

    let mut build = cc::Build::new();
    build.cpp(true)
        .flag_if_supported("-std=c++11")
        .file(manifest_dir.join("csrc/freenect2_shim.cpp"))
        .include(manifest_dir.join("vendor/include"))
        .include(manifest_dir.join("csrc"));
    for path in &lib.include_paths {
        build.include(path);
    }
    build.compile("freenect2_shim");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs");
    bindgen::Builder::default()
        .header(manifest_dir.join("csrc/freenect2_shim.h").to_str().unwrap())
        .clang_arg(format!("-I{}", manifest_dir.join("vendor/include").display()))
        .clang_arg(format!("-I{}", manifest_dir.join("csrc").display()))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_path)
        .expect("Couldn't write bindings");
}