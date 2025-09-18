//! Simple macOS keychain unlocking
//!
//! Clean, simple keychain unlock using password from stored keychain item

use crate::{CsCliError, Result};
use std::process::Command;
use tracing::{debug, info, warn};

const KEYCHAIN_SERVICE_NAME: &str = "com.postman.cs-cli";
const KEYCHAIN_ACCOUNT_NAME: &str = "login-keychain-password";

/// Unlock macOS keychain and prevent auto-lock
/// This is the main entry point that handles both password validation and keychain unlock
pub fn unlock_keychain_with_password(password: &str) -> Result<()> {
    if password.is_empty() {
        return Err(CsCliError::Authentication(
            "Keychain password cannot be empty".to_string(),
        ));
    }

    info!("Unlocking keychain...");

    let home = std::env::var("HOME")
        .map_err(|_| CsCliError::Authentication("Could not find home directory".to_string()))?;

    // Main keychain path
    let main_keychain = format!("{home}/Library/Keychains/login.keychain-db");

    // Step 1: Unlock keychain with sudo
    let unlock_cmd = format!(
        "echo '{password}' | sudo -S security unlock-keychain -p '{password}' '{main_keychain}' 2>/dev/null"
    );

    let unlock_output = Command::new("sh")
        .args(["-c", &unlock_cmd])
        .output()
        .map_err(|e| CsCliError::Authentication(format!("Failed to run unlock command: {e}")))?;

    if !unlock_output.status.success() {
        let stderr = String::from_utf8_lossy(&unlock_output.stderr);
        return Err(CsCliError::Authentication(format!(
            "Failed to unlock keychain: {}",
            stderr.trim()
        )));
    }

    // Step 2: Set keychain to never auto-lock
    let settings_cmd = format!(
        "echo '{password}' | sudo -S security set-keychain-settings -l '{main_keychain}' 2>/dev/null"
    );

    let settings_output = Command::new("sh")
        .args(["-c", &settings_cmd])
        .output()
        .map_err(|e| CsCliError::Authentication(format!("Failed to run settings command: {e}")))?;

    if !settings_output.status.success() {
        warn!("Failed to set keychain no-lock setting, but keychain is unlocked");
    }

    // Done - keychain is unlocked and set to not auto-lock
    // Users can click "Allow Always" when rookie prompts for browser Safe Storage access

    info!("Keychain unlocked successfully");
    Ok(())
}

/// Try to retrieve password from stored keychain item
pub fn get_stored_password() -> Result<String> {
    debug!("Attempting to retrieve stored password from keychain");
    
    let output = Command::new("security")
        .args([
            "find-generic-password",
            "-a", KEYCHAIN_ACCOUNT_NAME,
            "-s", KEYCHAIN_SERVICE_NAME,
            "-w"  // Return password only
        ])
        .output()
        .map_err(|e| CsCliError::Authentication(format!("Failed to access keychain: {e}")))?;

    if output.status.success() {
        let password = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !password.is_empty() {
            debug!("Successfully retrieved stored password from keychain");
            Ok(password)
        } else {
            Err(CsCliError::Authentication("Retrieved empty password from keychain".to_string()))
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        debug!("Keychain password retrieval failed: {}", stderr);
        Err(CsCliError::Authentication("Could not retrieve stored password from keychain".to_string()))
    }
}

/// Store password in keychain for future use
pub fn store_password(password: &str) -> Result<()> {
    if password.is_empty() {
        return Err(CsCliError::Authentication("Cannot store empty password".to_string()));
    }

    info!("Storing password in keychain for future use...");

    // Get current executable path for ACL
    let current_exe = std::env::current_exe()
        .map_err(|e| CsCliError::Authentication(format!("Could not determine executable path: {e}")))?;

    // First, try to delete existing item (ignore errors)
    let _ = Command::new("security")
        .args([
            "delete-generic-password",
            "-a", KEYCHAIN_ACCOUNT_NAME,
            "-s", KEYCHAIN_SERVICE_NAME,
        ])
        .output();

    // Add new keychain item with ACL for our binary
    let output = Command::new("security")
        .args([
            "add-generic-password",
            "-a", KEYCHAIN_ACCOUNT_NAME,
            "-s", KEYCHAIN_SERVICE_NAME,
            "-w", password,
            "-T", current_exe.to_str().unwrap_or(""),
            "-T", "/usr/bin/security", // Allow security command access
        ])
        .output()
        .map_err(|e| CsCliError::Authentication(format!("Failed to store password: {e}")))?;

    if output.status.success() {
        info!("Password stored successfully in keychain");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(CsCliError::Authentication(format!(
            "Failed to store password in keychain: {}",
            stderr.trim()
        )))
    }
}

/// Check if stored password exists in keychain
pub fn has_stored_password() -> bool {
    Command::new("security")
        .args([
            "find-generic-password",
            "-a", KEYCHAIN_ACCOUNT_NAME,
            "-s", KEYCHAIN_SERVICE_NAME,
        ])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}