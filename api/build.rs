//! Warns at build time when a newer stable Rust is out than the one
//! building: a version check that reads the channel manifest and sends
//! nothing about the build.
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let manifest = Command::new("curl")
        .args([
            "-fsS",
            "-m",
            "10",
            "https://static.rust-lang.org/dist/channel-rust-stable.toml",
        ])
        .output();
    let latest = manifest
        .ok()
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .and_then(|text| {
            text.lines()
                .skip_while(|line| *line != "[pkg.rustc]")
                .find_map(|line| {
                    line.strip_prefix("version = \"")
                        .map(|v| v.split(' ').next().unwrap_or("").to_string())
                })
        });
    if let Some(latest) = latest {
        println!("cargo:warning=the latest stable Rust is {latest}");
    }
}
