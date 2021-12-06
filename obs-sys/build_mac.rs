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
        PathBuf::from("/Applications/OBS.app/Contents/Frameworks"),
        PathBuf::from("/Applications/OBS.app/Contents/MacOS"),
    ];

    for c in candidates.iter() {
        if let Ok(meta) = fs::metadata(c.join("libobs.0.dylib")) {
            if meta.is_file() {
                println!("cargo:rustc-link-search=native={}", c.display());
                return;
            }
        }
    }

    panic!("could not find libobs - install OBS or set LIBOBS_PATH");
}
