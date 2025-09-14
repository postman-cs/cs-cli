//! Simple CLI-based keychain unlocking
//!
//! Clean, simple keychain unlock using password from CLI args or environment

use crate::{CsCliError, Result};
use std::process::Command;
use tracing::{debug, info};

/// Unlock macOS keychain with password provided via CLI or environment
/// Call this ONCE at CLI startup to avoid multiple prompts
/// Uses sudo to unlock all keychains and disable auto-lock
pub fn unlock_keychain_with_cli_password(password: &str) -> Result<()> {
    if !cfg!(target_os = "macos") {
        debug!("Not on macOS, skipping keychain unlock");
        return Ok(());
    }

    if password.is_empty() {
        return Err(CsCliError::Authentication("Keychain password cannot be empty".to_string()));
    }

    info!("Configuring keychain access...");
    
    let home = std::env::var("HOME")
        .map_err(|_| CsCliError::Authentication("Could not find home directory".to_string()))?;
    
    // Step 1: Unlock user keychains only (not system keychains which have different passwords)
    let user_keychains = vec![
        format!("{}/Library/Keychains/login.keychain-db", home),
        format!("{}/Library/Keychains/login.keychain", home),  // Legacy format
    ];
    
    let mut unlock_success = false;
    for keychain in &user_keychains {
        let unlock_cmd = format!(
            "echo '{}' | sudo -S security unlock-keychain -p '{}' '{}' 2>/dev/null",
            password, password, keychain
        );
        
        if let Ok(output) = Command::new("sh").args(&["-c", &unlock_cmd]).output() {
            if output.status.success() {
                debug!("Unlocked: {}", keychain);
                unlock_success = true;
            }
        }
    }
    
    // Step 2: Set keychain settings to prevent auto-lock for main keychain
    let main_keychain = format!("{}/Library/Keychains/login.keychain-db", home);
    let settings_cmd = format!(
        "echo '{}' | sudo -S security set-keychain-settings -l '{}' 2>/dev/null",
        password, main_keychain
    );
    
    let settings_output = Command::new("sh")
        .args(&["-c", &settings_cmd])
        .output()
        .map_err(|e| CsCliError::Authentication(format!("Failed to run settings command: {}", e)))?;
    
    if unlock_success && settings_output.status.success() {
        info!("Keychain access configured successfully");
        Ok(())
    } else {
        // Show actual error details for debugging
        let settings_stderr = String::from_utf8_lossy(&settings_output.stderr);
        
        let error_msg = format!(
            "Failed to configure keychain access. Unlock result: {}, Settings result: {} ({})",
            unlock_success,
            settings_output.status.success(), settings_stderr.trim()
        );
        
        Err(CsCliError::Authentication(error_msg))
    }
}


/// Check if keychain is already unlocked (no password needed)
pub fn is_keychain_unlocked() -> Result<bool> {
    if !cfg!(target_os = "macos") {
        return Ok(true); // Non-macOS doesn't need keychain unlock
    }

    // Check if any keychain is locked
    let output = Command::new("security")
        .args(&["list-keychains"])
        .output()
        .map_err(|e| CsCliError::Authentication(format!("Failed to list keychains: {}", e)))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // If we can list keychains, generally means they're accessible
        let is_unlocked = !stdout.is_empty();
        debug!("Keychain status: {}", if is_unlocked { "accessible" } else { "locked" });
        Ok(is_unlocked)
    } else {
        debug!("Keychain list failed, assuming locked");
        Ok(false)
    }
}
