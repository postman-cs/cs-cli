//! Smart Keychain Management
//!
//! Simple macOS-only keychain management that implements two flows:
//! 1. If com.postman.cs-cli exists and accessible -> use it to unlock keychain
//! 2. If not -> prompt for password, validate, store, and unlock keychain

use crate::common::auth::cli_unlock::{
    get_stored_password, has_stored_password, store_password, unlock_keychain_with_password,
};
use crate::{CsCliError, Result};
use owo_colors::OwoColorize;
use tracing::{debug, info};

/// Simple keychain manager for macOS-only operation
pub struct SmartKeychainManager;

impl SmartKeychainManager {
    /// Create a new SmartKeychainManager (macOS only)
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    /// Main entry point: Handle keychain unlock with the two flows
    /// FLOW 1: Check for stored password -> use it -> unlock keychain
    /// FLOW 2: No stored password -> prompt -> validate -> store -> unlock keychain
    pub fn ensure_keychain_access(&self) -> Result<()> {
        info!("Checking keychain access...");

        // FLOW 1: Try to use stored password
        if has_stored_password() {
            debug!("Found com.postman.cs-cli keychain item, attempting to use stored password");
            
            match get_stored_password() {
                Ok(stored_password) => {
                    match unlock_keychain_with_password(&stored_password) {
                        Ok(()) => {
                            info!("Successfully unlocked keychain using stored password");
                            return Ok(());
                        }
                        Err(e) => {
                            debug!("Stored password failed to unlock keychain: {}", e);
                            info!("Stored password appears to be invalid, will prompt for new password");
                            // Continue to FLOW 2
                        }
                    }
                }
                Err(e) => {
                    debug!("Could not retrieve stored password: {}", e);
                    // Continue to FLOW 2
                }
            }
        } else {
            debug!("No com.postman.cs-cli keychain item found");
        }

        // FLOW 2: Prompt user for password and validate
        info!("Prompting for keychain password...");
        println!();
        println!("{}", "Keychain Access Required".truecolor(255, 142, 100));
        println!("{}", "CS-CLI needs your Mac login password to authenticate".truecolor(230, 230, 230));
        println!();

        loop {
            let password = rpassword::prompt_password(format!(
                "{}",
                "Enter your Mac login password: ".truecolor(111, 44, 186)
            ))
            .map_err(|e| CsCliError::Authentication(format!("Failed to read password: {e}")))?;

            // Validate password by attempting keychain unlock
            match unlock_keychain_with_password(&password) {
                Ok(()) => {
                    info!("Password validated successfully, keychain unlocked");

                    // Store password for future use
                    match store_password(&password) {
                        Ok(()) => {
                            info!("Password stored in keychain for future automatic access");
                        }
                        Err(e) => {
                            debug!("Failed to store password (non-fatal): {}", e);
                            // This is non-fatal - keychain is unlocked which is what matters
                        }
                    }

                    return Ok(());
                }
                Err(_) => {
                    println!(
                        "{}",
                        "Invalid password. Please try again.".red()
                    );
                    continue;
                }
            }
        }
    }
}

impl Default for SmartKeychainManager {
    fn default() -> Self {
        Self::new().unwrap_or(Self)
    }
}