// FILE: src/common/self_update.rs
//
// PURPOSE:
//   Implements auto-update functionality for CS-CLI. Handles downloading,
//   verifying, and installing signed PKG updates via external installer.
//
// KEY DEPENDENCIES:
//   - reqwest for HTTP downloads
//   - sha2 for PKG integrity verification
//   - System tools: spctl, pkgutil, installer, osascript
//
// CORE EXPORTS:
//   - run(): Main update flow with channel support
//   - rollback(): Atomic rollback to previous version
//   - list_versions(): Show available installed versions
//

use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::Command;
use serde::Deserialize;
use sha2::{Sha256, Digest};
use tempfile::NamedTempFile;
use reqwest::Client;
use std::time::Duration;
use anyhow::{Result, Context};

const TEAM_ID: &str = "RGSZTAM229";
const PKG_IDENTIFIER: &str = "com.postman.cs-cli";
const DEFAULT_CHANNEL: &str = "stable";
const MANIFEST_BASE_URL: &str = "https://raw.githubusercontent.com/postman-cs/cs-cli/main/updates";
// Removed unused INSTALL_ROOT constant
const VERSIONS_DIR: &str = "/Library/CS-CLI/versions";

#[derive(Debug, Deserialize)]
struct Manifest {
    #[allow(dead_code)]
    channel: String,
    latest: Release,
}

#[derive(Debug, Deserialize)]
struct Release {
    version: String,
    #[allow(dead_code)]
    min_macos: Option<String>,
    #[allow(dead_code)]
    universal: Option<bool>,
    artifacts: Vec<Artifact>,
    #[allow(dead_code)]
    pkg_identifier: Option<String>,
    release_notes_url: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct Artifact {
    arch: String,             // "arm64" | "x86_64" | "universal"
    pkg_url: String,
    sha256: String,
    #[allow(dead_code)]
    size: Option<u64>,
}

/// Main update flow: check -> download -> verify -> install
pub async fn run(channel: Option<String>, assume_yes: bool, background: bool) -> Result<()> {
    let channel = channel.unwrap_or_else(|| get_configured_channel());
    let arch = current_arch();
    let client = http_client()?;
    
    let manifest = fetch_manifest(&client, &channel).await
        .context("Failed to fetch update manifest")?;
    
    let artifact = select_artifact(&manifest.latest.artifacts, &arch)
        .context("No compatible artifact found")?;

    if !is_upgrade_needed(&manifest.latest.version)? {
        if !background {
            println!("Already at latest version {}", current_version()?);
        }
        return Ok(());
    }

    if !background {
        println!("Update available: {} -> {}", current_version()?, manifest.latest.version);
        if let Some(notes_url) = &manifest.latest.release_notes_url {
            println!("Release notes: {}", notes_url);
        }
    }

    let pkg_path = download_pkg(&client, &artifact).await?;
    verify_pkg_integrity(&pkg_path, &artifact.sha256)?;
    verify_pkg_signature(&pkg_path)?;
    install_pkg(&pkg_path, assume_yes, background)?;
    
    if !background {
        println!("✅ Successfully updated to version {}", manifest.latest.version);
    }
    
    Ok(())
}

/// Rollback to the previous installed version
pub fn rollback() -> Result<()> {
    let versions = list_installed_versions()?;
    if versions.len() < 2 {
        anyhow::bail!("No previous version available for rollback");
    }
    
    let current = current_version()?;
    let previous = versions.iter()
        .rev()
        .find(|&v| v != &current)
        .context("Could not find previous version")?;
    
    println!("Rolling back from {} to {}", current, previous);
    set_current_version(previous)?;
    
    // Verify rollback worked
    let new_current = current_version()?;
    if new_current == *previous {
        println!("✅ Successfully rolled back to version {}", previous);
    } else {
        anyhow::bail!("Rollback verification failed: expected {}, got {}", previous, new_current);
    }
    
    Ok(())
}

/// List all installed versions
pub fn list_versions() -> Result<()> {
    let versions = list_installed_versions()?;
    let current = current_version()?;
    
    println!("Installed versions:");
    for version in versions {
        let marker = if version == current { " (current)" } else { "" };
        println!("  {}{}", version, marker);
    }
    
    Ok(())
}

fn http_client() -> Result<Client> {
    Ok(Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent(&format!("cs-cli-updater/{}", env!("CARGO_PKG_VERSION")))
        .build()?)
}

async fn fetch_manifest(client: &Client, channel: &str) -> Result<Manifest> {
    let url = format!("{}/{}.json", MANIFEST_BASE_URL, channel);
    let resp = client.get(&url).send().await?.error_for_status()?;
    let manifest: Manifest = resp.json().await?;
    Ok(manifest)
}

fn select_artifact(artifacts: &[Artifact], arch: &str) -> Result<Artifact> {
    // Prefer exact arch match, fallback to universal
    if let Some(artifact) = artifacts.iter().find(|a| a.arch == arch) {
        return Ok(artifact.clone());
    }
    if let Some(universal) = artifacts.iter().find(|a| a.arch == "universal") {
        return Ok(universal.clone());
    }
    anyhow::bail!("No artifact available for architecture {}", arch)
}

fn current_arch() -> &'static str {
    if cfg!(target_arch = "aarch64") { "arm64" }
    else if cfg!(target_arch = "x86_64") { "x86_64" }
    else { "unknown" }
}

fn current_version() -> Result<String> {
    let output = Command::new(std::env::current_exe()?)
        .arg("--version")
        .output()?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let version = stdout
        .split_whitespace()
        .last()
        .context("Could not parse version from --version output")?
        .to_string();
    
    Ok(version)
}

fn is_upgrade_needed(latest: &str) -> Result<bool> {
    let current = current_version()?;
    let current_ver = semver::Version::parse(&current)?;
    let latest_ver = semver::Version::parse(latest)?;
    Ok(latest_ver > current_ver)
}

async fn download_pkg(client: &Client, artifact: &Artifact) -> Result<PathBuf> {
    println!("Downloading update package...");
    let response = client.get(&artifact.pkg_url).send().await?.error_for_status()?;
    
    let temp_file = NamedTempFile::new()?.into_temp_path().keep()?;
    let mut file = File::create(&temp_file)?;
    
    let bytes = response.bytes().await?;
    io::Write::write_all(&mut file, &bytes)?;
    
    Ok(temp_file)
}

fn verify_pkg_integrity(path: &Path, expected_sha256: &str) -> Result<()> {
    println!("Verifying package integrity...");
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    
    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 { break; }
        hasher.update(&buffer[..bytes_read]);
    }
    
    let computed_hash = format!("{:x}", hasher.finalize());
    if computed_hash != expected_sha256.to_lowercase() {
        anyhow::bail!(
            "Package integrity check failed: expected {}, got {}", 
            expected_sha256, 
            computed_hash
        );
    }
    
    Ok(())
}

fn verify_pkg_signature(path: &Path) -> Result<()> {
    println!("Verifying package signature...");
    
    // Verify Gatekeeper assessment
    let spctl_status = Command::new("/usr/sbin/spctl")
        .args(["--assess", "--type", "install", "--verbose"])
        .arg(path)
        .status()?;
    
    if !spctl_status.success() {
        anyhow::bail!("Package failed Gatekeeper assessment (spctl)");
    }
    
    // Verify signature and Team ID
    let pkgutil_output = Command::new("/usr/sbin/pkgutil")
        .args(["--check-signature"])
        .arg(path)
        .output()?;
    
    let stdout = String::from_utf8_lossy(&pkgutil_output.stdout);
    if !stdout.contains(TEAM_ID) {
        anyhow::bail!("Package signature verification failed: Team ID {} not found", TEAM_ID);
    }
    
    // Optional: Check package identifier
    if !stdout.contains(PKG_IDENTIFIER) {
        eprintln!("Warning: Package identifier {} not found in signature output", PKG_IDENTIFIER);
    }
    
    Ok(())
}

fn install_pkg(path: &Path, assume_yes: bool, background: bool) -> Result<()> {
    if nix::unistd::Uid::effective().is_root() {
        return run_installer_direct(path);
    }
    
    if !assume_yes && atty::is(atty::Stream::Stdin) && !background {
        println!("Administrator privileges required to install update.");
        println!("You can: (1) re-run with sudo, or (2) authenticate via system dialog.");
    }
    
    // Try AppleScript escalation for GUI environments
    let script = format!(
        r#"do shell script "/usr/sbin/installer -pkg {} -target /" with administrator privileges"#,
        path.to_string_lossy().replace('"', "\\\"")
    );
    
    let applescript_result = Command::new("/usr/bin/osascript")
        .arg("-e")
        .arg(&script)
        .status();
    
    match applescript_result {
        Ok(status) if status.success() => {
            println!("Package installed successfully");
            Ok(())
        }
        _ => {
            if !background {
                println!("Falling back to sudo. Please enter your password if prompted.");
            }
            run_installer_direct(path)
        }
    }
}

fn run_installer_direct(path: &Path) -> Result<()> {
    let status = Command::new("/usr/sbin/installer")
        .args(["-pkg", &path.to_string_lossy(), "-target", "/"])
        .status()?;
    
    if !status.success() {
        anyhow::bail!("Package installation failed");
    }
    
    Ok(())
}

fn list_installed_versions() -> Result<Vec<String>> {
    let versions_path = Path::new(VERSIONS_DIR);
    if !versions_path.exists() {
        return Ok(vec![]);
    }
    
    let mut versions = fs::read_dir(versions_path)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .filter(|name| semver::Version::parse(name).is_ok())
        .collect::<Vec<_>>();
    
    versions.sort_by(|a, b| {
        semver::Version::parse(a).unwrap().cmp(&semver::Version::parse(b).unwrap())
    });
    
    Ok(versions)
}

fn set_current_version(version: &str) -> Result<()> {
    let current_link = Path::new("/Library/CS-CLI/current");
    let target = format!("/Library/CS-CLI/versions/{}", version);
    
    // Verify target exists
    if !Path::new(&target).join("cs-cli").exists() {
        anyhow::bail!("Version {} binary not found at {}/cs-cli", version, target);
    }
    
    // Remove existing symlink
    let _ = fs::remove_file(current_link);
    
    // Create new symlink
    std::os::unix::fs::symlink(&target, current_link)
        .context("Failed to create current version symlink")?;
    
    Ok(())
}

fn get_configured_channel() -> String {
    // Try to read channel from config file
    let config_path = Path::new("/Library/CS-CLI/config.json");
    if let Ok(content) = fs::read_to_string(config_path) {
        if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(channel) = config.get("channel").and_then(|c| c.as_str()) {
                return channel.to_string();
            }
        }
    }
    DEFAULT_CHANNEL.to_string()
}
