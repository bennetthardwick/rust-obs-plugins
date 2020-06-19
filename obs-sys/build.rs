extern crate bindgen;

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=wrapper.h");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs");

    if let Ok(bindings) = bindgen::Builder::default()
        .header("wrapper.h")
        .blacklist_type("_bindgen_ty_2")
        .derive_default(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
    {
        bindings
            .write_to_file(&out_path)
            .expect("Couldn't write bindings!");
        fs::copy(&out_path, "generated/bindings.rs").expect("Could not copy bindings!");
    } else {
        if let Ok(should_fail) = env::var("FAIL_BUILD_IN_CI") {
            if should_fail == "true" {
                panic!("Could not find obs headers - aborting!");
            }
        }

        println!("cargo:warning=Could not find obs headers - using pre-compiled.");
        println!("cargo:warning=This could result in a library that doesn't work.");
        fs::copy("generated/bindings.rs", out_path).expect("Could not copy bindings!");
    }
}
