//! Build script for cs-cli
//!
//! Downloads lightpanda binary at build time if not present,
//! enabling include_bytes! to embed it in the final executable.

use std::path::Path;
use std::process::Command;

const LIGHTPANDA_URL: &str = "https://github.com/lightpanda-io/browser/releases/download/nightly/lightpanda-aarch64-macos";
const LIGHTPANDA_PATH: &str = "bundled/lightpanda/lightpanda-aarch64-macos";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=bundled/lightpanda/");

    // Only download on Apple Silicon macOS
    if cfg!(not(target_os = "macos")) || cfg!(not(target_arch = "aarch64")) {
        println!("cargo:warning=Lightpanda binary only supported on Apple Silicon macOS");
        return;
    }

    // Check if binary already exists
    if Path::new(LIGHTPANDA_PATH).exists() {
        println!("cargo:warning=Using existing lightpanda binary");
        return;
    }

    // Create bundled directory if it doesn't exist
    if let Err(e) = std::fs::create_dir_all("bundled/lightpanda") {
        panic!("Failed to create bundled directory: {}", e);
    }

    println!("cargo:warning=Downloading lightpanda binary for embedding...");

    // Download lightpanda binary
    let output = Command::new("curl")
        .args([
            "-L",
            "-o", LIGHTPANDA_PATH,
            LIGHTPANDA_URL
        ])
        .output()
        .expect("Failed to execute curl command");

    if !output.status.success() {
        panic!(
            "Failed to download lightpanda binary: {}", 
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Verify the downloaded file exists and has reasonable size
    let metadata = std::fs::metadata(LIGHTPANDA_PATH)
        .expect("Failed to read downloaded binary metadata");
    
    if metadata.len() < 1_000_000 {  // Less than 1MB is suspicious
        panic!("Downloaded lightpanda binary is too small ({}bytes), download may have failed", metadata.len());
    }

    println!("cargo:warning=Successfully downloaded lightpanda binary ({} bytes)", metadata.len());
}