use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{PathBuf, Path};
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    if env::var_os("TARGET") == env::var_os("HOST") {
        match pkg_config::find_library("libevdev") {
            Ok(lib) => {
                for path in &lib.include_paths {
                    println!("cargo:include={}", path.display());
                }
                return Ok(());
            },
            Err(e) => {
                eprintln!(
                    "Couldn't find libevdev from pkgconfig ({:?}), \
                     compiling it from source...",
                    e
                );
            }
        };
    }

    if !Path::new("libevdev/.git").exists() {
        let mut download = Command::new("git");
        download.args(&["submodule", "update", "--init", "--depth", "1"]);
        run(&mut download)?;
    }

    let dst = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let src = env::current_dir()?;

    println!("cargo:rustc-link-search={}/lib", dst.display());
    println!("cargo:root={}", dst.display());
    println!("cargo:include={}/include", dst.display());
    println!("cargo:rerun-if-changed=libevdev/autogen.sh");

    println!("cargo:rustc-link-lib=static=evdev");
    let cfg = cc::Build::new();
    let compiler = cfg.get_compiler();

    fs::create_dir(&dst.join("build"))?;

    let mut autogen = Command::new("sh");
    let mut cflags = OsString::new();
    for arg in compiler.args() {
        cflags.push(arg);
        cflags.push(" ");
    }
    autogen.env("CC", compiler.path())
       .env("CFLAGS", cflags)
       .current_dir(&dst.join("build"))
       .arg(src.join("libevdev/autogen.sh").to_str().unwrap()
               .replace("C:\\", "/c/")
               .replace("\\", "/"));
    if let Ok(h) = env::var("HOST") {
        autogen.arg(format!("--host={}", h));
    }
    if let Ok(t) = env::var("TARGET") {
        autogen.arg(format!("--target={}", t));
    }
    autogen.arg(format!("--prefix={}", sanitize_sh(&dst)));
    run(&mut autogen)?;

    let mut make = Command::new("make");
    make.arg(&format!("-j{}", env::var("NUM_JOBS").unwrap()))
        .current_dir(&dst.join("build"));
    run(&mut make)?;

    let mut install = Command::new("make");
    install.arg("install").current_dir(&dst.join("build"));
    run(&mut install)?;
    Ok(())
}

fn run(cmd: &mut Command) -> std::io::Result<()> {
    println!("running: {:?}", cmd);
    assert!(cmd.status()?.success());
    Ok(())
}

fn sanitize_sh(path: &Path) -> String {
    let path = path.to_str().unwrap().replace("\\", "/");
    return change_drive(&path).unwrap_or(path);

    fn change_drive(s: &str) -> Option<String> {
        let mut ch = s.chars();
        let drive = ch.next().unwrap_or('C');
        if ch.next() != Some(':') {
            return None
        }
        if ch.next() != Some('/') {
            return None
        }
        Some(format!("/{}/{}", drive, &s[drive.len_utf8() + 2..]))
    }
}
