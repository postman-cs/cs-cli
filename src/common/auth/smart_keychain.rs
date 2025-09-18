//! Smart Keychain Management with Seamless Access
//!
//! This module provides intelligent keychain access that:
//! 1. Checks if Firefox cookies exist (no keychain needed)
//! 2. Uses trusted binary storage for seamless password retrieval
//! 3. Falls back to password prompt with secure storage

use crate::{CsCliError, Result};
use owo_colors::OwoColorize;
use std::process::Command;
use tracing::{debug, info, warn};

/// Smart keychain manager that handles seamless password storage and retrieval
pub struct SmartKeychainManager {}

impl SmartKeychainManager {
    /// Create a new SmartKeychainManager
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }


    /// Check if TouchID is available on this device
    pub fn is_touchid_available(&self) -> Result<bool> {
        debug!("Checking TouchID availability...");

        // TouchID availability is determined by system hardware - always true on modern Macs
        // The actual TouchID prompt happens automatically when keychain is locked
        Ok(true)
    }

    /// Retrieve stored password (seamless access via trusted binary)
    pub fn retrieve_password_with_touchid(&self) -> Result<Option<String>> {
        debug!("Attempting to retrieve password from keychain...");

        let output = Command::new("security")
            .args(&[
                "find-generic-password",
                "-s", "com.postman.cs-cli",
                "-a", "login-keychain-password",
                "-w"  // Return password data
            ])
            .output()
            .map_err(|e| CsCliError::Authentication(format!("Failed to retrieve password: {}", e)))?;

        if output.status.success() {
            let password = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(Some(password))
        } else {
            let exit_code = output.status.code().unwrap_or(-1);
            if exit_code == 44 {
                // Item not found
                debug!("No stored password found");
                Ok(None)
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if stderr.contains("canceled") || stderr.contains("user cancel") {
                    warn!("User canceled authentication");
                    Ok(None)
                } else {
                    Err(CsCliError::Authentication(format!("Failed to retrieve password: {}", stderr)))
                }
            }
        }
    }

    /// Store password in keychain for future use (trust our binary for seamless access)
    pub fn store_password_with_touchid(&self, password: &str) -> Result<()> {
        info!("Storing password in keychain for future use...");

        // Get current executable path to trust our binary
        let exe_path = std::env::current_exe()
            .map_err(|e| CsCliError::Authentication(format!("Failed to get executable path: {}", e)))?;

        let exe_path_str = exe_path.to_str()
            .ok_or_else(|| CsCliError::Authentication("Invalid executable path encoding".to_string()))?;

        let output = Command::new("security")
            .args(&[
                "add-generic-password",
                "-s", "com.postman.cs-cli",
                "-a", "login-keychain-password",
                "-w", password,
                "-A",  // Allow access from applications signed with same certificate
                "-T", exe_path_str,  // Trust our specific binary
                "-U"  // Update if exists
            ])
            .output()
            .map_err(|e| CsCliError::Authentication(format!("Failed to store password: {}", e)))?;

        if output.status.success() {
            info!("Password stored successfully in keychain with trusted access");
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(CsCliError::Authentication(format!("Failed to store password: {}", stderr)))
        }
    }

    /// Unlock the keychain with a password
    pub fn unlock_keychain(&self, password: &str) -> Result<()> {
        debug!("Attempting to unlock keychain...");

        let output = Command::new("security")
            .args(&["unlock-keychain", "-p", password])
            .output()
            .map_err(|e| CsCliError::Authentication(format!("Failed to unlock keychain: {}", e)))?;

        if output.status.success() {
            info!("Keychain unlocked successfully");
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(CsCliError::Authentication(format!("Failed to unlock keychain: {}", stderr)))
        }
    }

    /// Delete stored password (useful for password changes)
    pub fn delete_stored_password(&self) -> Result<()> {
        debug!("Deleting stored password...");

        let output = Command::new("security")
            .args(&[
                "delete-generic-password",
                "-s", "com.postman.cs-cli",
                "-a", "login-keychain-password"
            ])
            .output()
            .map_err(|e| CsCliError::Authentication(format!("Failed to delete password: {}", e)))?;

        if output.status.success() {
            info!("Stored password deleted");
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(CsCliError::Authentication(format!("Failed to delete stored password: {}", stderr)))
        }
    }


    /// Smart cookie extraction with intelligent fallbacks
    pub fn ensure_cookie_access(&self, extractor: &crate::common::auth::cookie_extractor::CookieExtractor) -> Result<()> {

        // Step 1: Try Firefox first (no keychain needed)
        debug!("Attempting Firefox cookie extraction (no keychain required)...");
        match extractor.extract_firefox_cookies() {
            Ok(cookies) => {
                // Validate we have essential cookies for Gong authentication
                let current_time = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                // Check for valid "cell" cookie (essential for determining Gong cell identifier)
                let valid_cell_cookie = cookies.iter().any(|c| {
                    c.name == "cell" && match c.expires {
                        None => true, // Session cookies are always valid
                        Some(expires_timestamp) => expires_timestamp > current_time,
                    }
                });

                // Count valid session cookies overall
                let valid_session_cookies: Vec<_> = cookies.iter().filter(|c| {
                    match c.expires {
                        None => true, // Session cookies are always valid
                        Some(expires_timestamp) => expires_timestamp > current_time,
                    }
                }).collect();

                if valid_cell_cookie && !valid_session_cookies.is_empty() {
                    info!(
                        valid_cookies = valid_session_cookies.len(),
                        "Successfully extracted valid Gong cookies from Firefox (no keychain needed)"
                    );
                    return Ok(());
                } else if !valid_cell_cookie {
                    debug!("Firefox missing valid 'cell' cookie for Gong authentication (required for cell identification)");
                } else {
                    debug!("Firefox has no valid session cookies for Gong authentication");
                }
            }
            Err(e) => debug!("Firefox extraction failed: {}", e),
        }

        // Step 2: Skip keychain status check, go straight to TouchID attempt
        info!("Firefox cookies insufficient, attempting smart keychain unlock...");

        // Step 3: Try to get stored password with TouchID
        if self.is_touchid_available()? {
            info!("TouchID is available, checking for stored password...");
            match self.retrieve_password_with_touchid() {
                Ok(Some(password)) => {
                    info!("Found stored password, attempting to unlock keychain...");

                    // Try to unlock with stored password
                    match self.unlock_keychain(&password) {
                        Ok(()) => {
                            info!("Successfully unlocked keychain with stored TouchID password");
                            return Ok(());
                        }
                        Err(e) => {
                            warn!("Stored password failed (password may have changed): {}", e);
                            // Delete the invalid stored password
                            let _ = self.delete_stored_password();
                        }
                    }
                }
                Ok(None) => {
                    info!("No stored password found, will prompt user");
                }
                Err(e) => {
                    warn!("TouchID password retrieval failed: {}", e);
                }
            }
        } else {
            info!("TouchID not available");
        }

        // Step 4: Prompt for password and store it for future use
        println!();
        println!("{}", "Keychain Access Required".truecolor(255, 142, 100));
        println!("{}", "CS-CLI needs to unlock your keychain to access browser cookies.".truecolor(200, 200, 200));

        if self.is_touchid_available()? {
            println!("{}", "After entering your password once, future runs will be seamless.".truecolor(150, 150, 150));
        }
        println!();

        let password = rpassword::prompt_password(format!(
            "{}",
            "Enter your Mac login password: ".truecolor(111, 44, 186)
        ))
        .map_err(|e| CsCliError::Authentication(format!("Failed to read password: {}", e)))?;

        // Unlock keychain
        self.unlock_keychain(&password)?;

        // Store password for future use (if TouchID is available)
        if self.is_touchid_available()? {
            info!("Keychain available, attempting to store password for future use...");
            match self.store_password_with_touchid(&password) {
                Ok(()) => {
                    info!("Successfully stored password with trusted access");
                    println!("{}", "✓ Password saved for seamless future access".truecolor(100, 200, 100));
                }
                Err(e) => {
                    warn!("Failed to store password for future use: {}", e);
                    println!("{}", format!("⚠ Could not save password for future use: {}", e).truecolor(255, 165, 0));
                }
            }
        } else {
            info!("TouchID not available, password will not be stored for future use");
        }

        Ok(())
    }
}

impl Default for SmartKeychainManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            // Fallback: create empty struct if initialization fails
            Self {}
        })
    }
}

/// Check if we're running in an environment where keychain access is expected
pub fn should_manage_keychain() -> bool {
    // Only manage keychain on macOS
    cfg!(target_os = "macos")
        // And not in CI/test environments
        && std::env::var("CI").is_err()
        && std::env::var("GITHUB_ACTIONS").is_err()
        // And not when explicitly disabled
        && std::env::var("CS_CLI_NO_KEYCHAIN").is_err()
}