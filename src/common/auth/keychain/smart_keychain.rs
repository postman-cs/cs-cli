//! Smart Keychain Management
//!
//! Simple macOS-only keychain management that implements two flows:
//! 1. If com.postman.cs-cli exists and accessible -> use it to unlock keychain
//! 2. If not -> prompt for password, validate, store, and unlock keychain

use crate::common::auth::keychain::cli_unlock::{
    get_stored_password, store_password, unlock_keychain_with_password,
};
use crate::{CsCliError, Result};
use owo_colors::OwoColorize;
use tracing::{debug, info};
use inquire::Password;

/// Simple keychain manager for macOS-only operation
pub struct SmartKeychainManager;

impl SmartKeychainManager {
    /// Create a new SmartKeychainManager (macOS only)
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    /// Main entry point: Handle keychain unlock with the two flows
    /// FLOW 1: Check for stored password -> use it -> unlock keychain
    /// Ensure keychain access with stored password or user prompt
    pub fn ensure_keychain_access(&self) -> Result<()> {
        info!("Checking keychain access...");

        // Try stored password first
        if let Ok(stored_password) = get_stored_password() {
            if unlock_keychain_with_password(&stored_password).is_ok() {
                info!("Successfully unlocked keychain using stored password");
                return Ok(());
            }
            debug!("Stored password failed, will prompt for new password");
        }

        // Prompt user for password
        self.prompt_and_validate_password()
    }

    /// Prompt user for password and validate it
    fn prompt_and_validate_password(&self) -> Result<()> {
        info!("Prompting for keychain password...");
        self.show_password_prompt();

        loop {
            let password = self.get_password_input()?;
            
            if unlock_keychain_with_password(&password).is_ok() {
                info!("Password validated successfully, keychain unlocked");
                let _ = store_password(&password); // Ignore storage errors
                return Ok(());
            }
            
            println!("{}", "Invalid password. Please try again.".red());
        }
    }

    /// Show password prompt UI
    fn show_password_prompt(&self) {
        println!();
        println!("{}", "Keychain Access Required".truecolor(255, 142, 100));
        println!("{}", "CS-CLI needs your Mac login password to authenticate".truecolor(230, 230, 230));
        println!();
    }

    /// Get password input from user
    fn get_password_input(&self) -> Result<String> {
        Password::new("Enter your Mac login password:")
            .prompt()
            .map_err(|e| CsCliError::Authentication(format!("Failed to read password: {e}")))
    }
}

impl Default for SmartKeychainManager {
    fn default() -> Self {
        Self::new().unwrap_or(Self)
    }
}
