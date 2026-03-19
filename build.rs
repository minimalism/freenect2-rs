use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    cc::Build::new()
        .cpp(true)
        .flag_if_supported("-std=c++11")
        .file(manifest_dir.join("csrc/freenect2_shim.cpp"))
        .include(manifest_dir.join("vendor/include"))
        .include(manifest_dir.join("csrc"))
        .compile("freenect2_shim");

    let lib_dir = env::var("FREENECT2_LIB_DIR").unwrap_or_else(|_| {
        panic!(
            "FREENECT2_LIB_DIR must be set to the directory containing libfreenect2.so \
             (e.g. the `lib` folder under your libfreenect2 build tree)"
        )
    });
    println!("cargo:rerun-if-env-changed=FREENECT2_LIB_DIR");
    println!("cargo:rustc-link-search=native={lib_dir}");
    println!("cargo:rustc-link-lib=freenect2");

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
