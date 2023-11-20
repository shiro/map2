use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(feature = "libevdev-1-10")]
// ver_str is string of the form "major.minor.patch"
fn parse_version(ver_str: &str) -> Option<(u32, u32, u32)> {
    let mut major_minor_patch = ver_str
        .split(".")
        .map(|str| str.parse::<u32>().unwrap());
    let major = major_minor_patch.next()?;
    let minor = major_minor_patch.next()?;
    let patch = major_minor_patch.next()?;
    Some((major, minor, patch))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if env::var_os("TARGET") == env::var_os("HOST") {
        let mut config = pkg_config::Config::new();
        config.print_system_libs(false);

        match config.probe("libevdev") {
            Ok(lib) => {
                // panic if feature 1.10 is enabled and the installed library
                // is older than 1.10
                #[cfg(feature = "libevdev-1-10")]
                {
                    let (major, minor, patch) = parse_version(&lib.version)
                        .expect("Could not parse version information");
                    assert_eq!(major, 1, "evdev-rs works only with libevdev 1");
                    assert!(minor >= 10,
                        "Feature libevdev-1-10 was enabled, when compiling \
                        for a system with libevdev version {}.{}.{}",
                        major,
                        minor,
                        patch,
                    );
                }
                for path in &lib.include_paths {
                    println!("cargo:include={}", path.display());
                }
                return Ok(());
            }
            Err(e) => eprintln!(
                "Couldn't find libevdev from pkgconfig ({:?}), \
                    compiling it from source...",
                e
            ),
        };
    }

    if !Path::new("libevdev/.git").exists() {
        let mut download = Command::new("git");
        download.args(&["submodule", "update", "--init", "--depth", "1"]);
        run_ignore_error(&mut download)?;
    }

    let dst = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let src = env::current_dir()?;
    let mut cp = Command::new("cp");
    cp.arg("-r")
        .arg(&src.join("libevdev/"))
        .arg(&dst)
        .current_dir(&src);
    run(&mut cp)?;

    println!("cargo:rustc-link-search={}/lib", dst.display());
    println!("cargo:root={}", dst.display());
    println!("cargo:include={}/include", dst.display());
    println!("cargo:rerun-if-changed=libevdev");

    println!("cargo:rustc-link-lib=static=evdev");
    let cfg = cc::Build::new();
    let compiler = cfg.get_compiler();

    if !&dst.join("build").exists() {
        fs::create_dir(&dst.join("build"))?;
    }

    let mut autogen = Command::new("sh");
    let mut cflags = OsString::new();
    for arg in compiler.args() {
        cflags.push(arg);
        cflags.push(" ");
    }
    autogen
        .env("CC", compiler.path())
        .env("CFLAGS", cflags)
        .current_dir(&dst.join("build"))
        .arg(
            dst.join("libevdev/autogen.sh")
                .to_str()
                .unwrap()
                .replace("C:\\", "/c/")
                .replace("\\", "/"),
        );
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

fn run_ignore_error(cmd: &mut Command) -> std::io::Result<()> {
    println!("running: {:?}", cmd);
    let _ = cmd.status();
    Ok(())
}

fn sanitize_sh(path: &Path) -> String {
    let path = path.to_str().unwrap().replace("\\", "/");
    return change_drive(&path).unwrap_or(path);

    fn change_drive(s: &str) -> Option<String> {
        let mut ch = s.chars();
        let drive = ch.next().unwrap_or('C');
        if ch.next() != Some(':') {
            return None;
        }
        if ch.next() != Some('/') {
            return None;
        }
        Some(format!("/{}/{}", drive, &s[drive.len_utf8() + 2..]))
    }
}
