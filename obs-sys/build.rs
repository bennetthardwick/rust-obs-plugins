extern crate bindgen;

use std::env;
use std::fs;
use std::path::PathBuf;

#[cfg(windows)]
mod build_win;

#[cfg(target_os = "macos")]
mod build_mac;

fn main() {
    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-env-changed=DONT_USE_GENERATED_BINDINGS");

    #[cfg(not(target_os = "macos"))]
    {
        println!("cargo:rustc-link-lib=dylib=obs");
        println!("cargo:rustc-link-lib=dylib=obs-frontend-api");
    }

    // mac OBS only ships with libobs.0.dylib for some reason
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-lib=dylib=obs.0");

    #[cfg(windows)]
    build_win::find_windows_obs_lib();

    #[cfg(target_os = "macos")]
    build_mac::find_mac_obs_lib();

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs");

    match bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg("-I/usr/include/obs/")
        .blocklist_type("_bindgen_ty_2")
        .blocklist_type("_bindgen_ty_3")
        .blocklist_type("_bindgen_ty_4")
        .derive_default(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
    {
        Ok(bindings) => {
            bindings
                .write_to_file(&out_path)
                .expect("Couldn't write bindings!");
            fs::copy(&out_path, "generated/bindings.rs").expect("Could not copy bindings!");
        }

        Err(e) => {
            if env::var("DONT_USE_GENERATED_BINDINGS").is_ok() {
                panic!("Failed to generate headers with bindgen: {}", e);
            }

            println!("cargo:warning=Could not find obs headers - using pre-compiled.");
            println!("cargo:warning=This could result in a library that doesn't work.");
            fs::copy("generated/bindings.rs", out_path).expect("Could not copy bindings!");
        }
    }
}
