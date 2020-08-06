use cc;
use regex::Regex;
use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, BufWriter};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;
use winreg::RegKey;
use winreg::enums::{KEY_WOW64_32KEY, HKEY_LOCAL_MACHINE, KEY_READ};

fn generate_def<P: AsRef<OsStr>, Q: AsRef<Path>>(
    mut dumpbin: Command,
    dll_path: &P,
    def_path: &Q,
) -> Result<(), io::Error> {
    let exports = dumpbin.arg("/EXPORTS").arg(dll_path).output()?;
    assert!(exports.status.success());
    let f = File::create(def_path)?;
    let mut f = BufWriter::new(f);
    f.write(b"EXPORTS\r\n")?;
    let exports = String::from_utf8_lossy(&exports.stdout);
    let pattern = Regex::new(r"(?im)^\s*\d+\s+[0-9a-f]+\s+[0-9a-f]+\s+(\S+)\r?$").unwrap();
    for export in pattern.captures_iter(&exports) {
        f.write_fmt(format_args!("{}\r\n", export.get(1).unwrap().as_str()))?;
    }
    f.flush()?;
    Ok(())
}

pub fn find_windows_obs_lib() {
    if let Some(path) = env::var("LIBOBS_PATH").ok() {
        println!("cargo:rustc-link-search=native={}", path);
        return;
    }
    // MSVC doesn't link against normal libraries,
    // and Windows doesn't have a standard mechanism for locating build-time dependencies.
    // Try to locate an OBS installation using the registry and then generate a .lib file
    // containing all symbols exported by libobs.
    let target = env::var("TARGET").unwrap();
    if let Some((dll_path, arch)) = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags("SOFTWARE\\OBS Studio", KEY_READ | KEY_WOW64_32KEY)
        .ok()
        .and_then(|key| key.get_value("").ok())
        .and_then(|base_path: String| match target.as_str() {
            "i686-pc-windows-msvc" => {
                Some((PathBuf::from(base_path).join("bin\\32bit\\obs.dll"), "X86"))
            }
            "x86_64-pc-windows-msvc" => {
                Some((PathBuf::from(base_path).join("bin\\64bit\\obs.dll"), "X64"))
            }
            _ => None,
        }) {
        let dumpbin = cc::windows_registry::find(&target, "dumpbin.exe");
        let lib = cc::windows_registry::find(&target, "lib.exe");
        match (dumpbin, lib) {
            (Some(mut dumpbin), Some(mut lib)) => {
                let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
                let def_path = out_path.join("obs.def");
                let lib_path = out_path.join("obs.lib");
                if let Ok(()) = generate_def(dumpbin, &dll_path, &def_path) {
                    assert!(
                        lib.arg(format!("/DEF:{}", def_path.to_str().unwrap()))
                            .arg(format!("/OUT:{}", lib_path.to_str().unwrap()))
                            .arg(format!("/MACHINE:{}", arch))
                            .status()
                            .unwrap()
                            .success()
                    );
                    println!(
                        "cargo:rustc-link-search=native={}",
                        out_path.to_str().unwrap()
                    );
                }
            }
            _ => {}
        }
        return;
    }
}
