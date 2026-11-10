//! The license text the binary prints, fetched from the upstream branch.
use std::process::Command;

const URL: &str = "https://raw.githubusercontent.com/rust-lang/rust/main/LICENSE-MIT";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let file = format!("{}/LICENSE-MIT", std::env::var("OUT_DIR").unwrap());
    let fetched = Command::new("curl")
        .args(["-fsSL", "-m", "30", "-o", &file, URL])
        .status()
        .expect("curl");
    assert!(fetched.success(), "could not fetch {URL}");
}
