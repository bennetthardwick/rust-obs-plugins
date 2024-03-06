use std::env;
use std::fs;
use std::path::PathBuf;

pub fn find_mac_obs_lib() {
    if let Some(path) = env::var("LIBOBS_PATH").ok() {
        println!("cargo:rustc-link-search=native={}", path);
        return;
    }

    let candidates = [
        PathBuf::from(&*shellexpand::tilde(
            "~/Applications/OBS.app/Contents/MacOS",
        )),
        PathBuf::from(&*shellexpand::tilde(
            "~/Applications/OBS.app/Contents/Frameworks",
        )),
        PathBuf::from("/Applications/OBS.app/Contents/Frameworks"),
        PathBuf::from("/Applications/OBS.app/Contents/MacOS"),
    ];

    let mut found_obs = false;
    let mut found_obs_frontend = false;

    for c in candidates.iter() {
        if !found_obs {
            if let Ok(meta) = fs::metadata(c.join("obs.dylib")) {
                if meta.is_file() {
                    println!("cargo:rustc-link-search=dylib={}", c.display());
                    println!("cargo:rustc-link-lib=dylib=obs");
                    found_obs = true;
                }
            }

            if let Ok(meta) = fs::metadata(c.join("libobs.framework")) {
                if meta.is_dir() {
                    println!("cargo:rustc-link-search=framework={}", c.display());
                    println!("cargo:rustc-link-lib=framework=libobs.framework");
                    found_obs = true;
                }
            }
        }

        if !found_obs_frontend {
            if let Ok(meta) = fs::metadata(c.join("obs-frontend-api.dylib")) {
                if meta.is_file() {
                    println!("cargo:rustc-link-search=native={}", c.display());
                    println!("cargo:rustc-link-lib=dylib=obs-frontend-api");
                    found_obs_frontend = true;
                }
            }

            if let Ok(meta) = fs::metadata(c.join("libobs-frontend-api.framework")) {
                if meta.is_dir() {
                    println!("cargo:rustc-link-search=framework={}", c.display());
                    println!("cargo:rustc-link-lib=framework=libobs-frontend-api.framework");
                    found_obs_frontend = true;
                }
            }
        }
    }

    if !found_obs {
        panic!("could not find libobs - install OBS or set LIBOBS_PATH");
    }

    if !found_obs_frontend {
        panic!("could not find libobs-frontend-api - install OBS or set LIBOBS_PATH");
    }
}
