//! Build script to embed GitHub OAuth credentials at compile time
//!
//! Reads OAuth app client secret from environment variable and makes it
//! available to the application. This keeps credentials out of source code.

use std::env;

fn main() {
    // Load .env file if it exists (silent on failure)
    let _ = dotenvy::dotenv();

    // Rebuild if .env file changes
    println!("cargo:rerun-if-changed=.env");
    println!("cargo:rerun-if-env-changed=GITHUB_CLIENT_SECRET");
    println!("cargo:rerun-if-env-changed=GITHUB_CLIENT_ID");

    // Read GitHub OAuth client ID from environment
    if let Ok(client_id) = env::var("GITHUB_CLIENT_ID") {
        println!("cargo:rustc-env=GITHUB_CLIENT_ID={}", client_id);
    } else {
        // For development builds, provide a placeholder
        println!("cargo:rustc-env=GITHUB_CLIENT_ID=dev_placeholder_client_id");
        println!("cargo:warning=GITHUB_CLIENT_ID not set - OAuth will not work");
    }

    // Read GitHub OAuth client secret from environment
    if let Ok(client_secret) = env::var("GITHUB_CLIENT_SECRET") {
        println!("cargo:rustc-env=GITHUB_CLIENT_SECRET={}", client_secret);
    } else {
        // For development builds, provide a placeholder
        println!("cargo:rustc-env=GITHUB_CLIENT_SECRET=dev_placeholder_secret");
        println!("cargo:warning=GITHUB_CLIENT_SECRET not set - OAuth will not work");
    }

    // Ensure build fails in release mode without proper credentials
    if env::var("PROFILE").unwrap_or_default() == "release" {
        if env::var("GITHUB_CLIENT_ID").is_err() || env::var("GITHUB_CLIENT_SECRET").is_err() {
            panic!("GITHUB_CLIENT_ID and GITHUB_CLIENT_SECRET must be set for release builds");
        }
    }
}
