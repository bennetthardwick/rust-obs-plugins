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

    #[cfg(not(target_os = "macos"))]
    println!("cargo:rustc-link-lib=dylib=obs");

    // mac OBS only ships with libobs.0.dylib for some reason
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-lib=dylib=obs.0");

    #[cfg(windows)]
    build_win::find_windows_obs_lib();

    #[cfg(target_os = "macos")]
    build_mac::find_mac_obs_lib();

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
        if env::var("DONT_USE_GENERATED_BINDINGS").is_ok() {
            panic!("Could not find obs headers - aborting!");
        }

        println!("cargo:warning=Could not find obs headers - using pre-compiled.");
        println!("cargo:warning=This could result in a library that doesn't work.");
        fs::copy("generated/bindings.rs", out_path).expect("Could not copy bindings!");
    }
}
