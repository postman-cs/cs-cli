//! Smart Keychain Management
//!
//! Simple macOS-only keychain management that ensures proper ACL permissions
//! for accessing browser cookies without password prompts.

use crate::{CsCliError, Result};
use std::process::Command;
use tracing::{debug, info};

/// Simple keychain manager for macOS-only operation
pub struct SmartKeychainManager;

impl SmartKeychainManager {
    /// Create a new SmartKeychainManager (macOS only)
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    /// Ensure keychain access with proper ACL permissions (no password prompts)
    pub fn ensure_keychain_access(&self) -> Result<()> {
        info!("Ensuring keychain access with proper ACL permissions...");

        // Check if we can access keychain without prompts
        match self.test_keychain_access() {
            Ok(()) => {
                info!("Keychain access available");
                return Ok(());
            }
            Err(e) => {
                debug!("Initial keychain access test failed: {}", e);
                // Continue to setup proper permissions
            }
        }

        // Set up keychain permissions for our application
        self.setup_keychain_permissions()?;

        // Verify access now works
        self.test_keychain_access()
            .map_err(|e| CsCliError::Authentication(format!("Keychain setup failed: {}", e)))?;

        info!("Keychain access configured successfully");
        Ok(())
    }

    /// Test if we can access keychain without password prompts
    fn test_keychain_access(&self) -> Result<()> {
        debug!("Testing keychain access...");

        // Try to list keychain items - this should work without prompts if ACL is set correctly
        let output = Command::new("security")
            .args(["dump-keychain", "-d"])
            .output()
            .map_err(|e| CsCliError::Authentication(format!("Failed to test keychain access: {e}")))?;

        if output.status.success() {
            debug!("Keychain access test successful");
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(CsCliError::Authentication(format!(
                "Keychain access test failed: {}",
                stderr.trim()
            )))
        }
    }

    /// Set up keychain permissions for passwordless access
    fn setup_keychain_permissions(&self) -> Result<()> {
        info!("Setting up keychain permissions for cs-cli...");

        // Get current executable path for ACL (not currently used but may be needed for future ACL setup)
        let _current_exe = std::env::current_exe().map_err(|e| {
            CsCliError::Authentication(format!("Could not determine executable path: {e}"))
        })?;

        let home = std::env::var("HOME")
            .map_err(|_| CsCliError::Authentication("Could not find home directory".to_string()))?;

        let main_keychain = format!("{home}/Library/Keychains/login.keychain-db");

        // Create an ACL entry for our application
        info!("Adding ACL entry for cs-cli to keychain...");

        // Set up keychain permissions using signature-based trust for our signed app
        let output = Command::new("security")
            .args([
                "set-key-partition-list",
                "-S",
                "apple-tool:,apple:,com.postman.cs-cli", // Include our bundle identifier
                "-s",
                "-k",
                "", // Empty password - will use current session
                &main_keychain,
            ])
            .output()
            .map_err(|e| CsCliError::Authentication(format!("Failed to setup keychain permissions: {e}")))?;

        if output.status.success() {
            info!("Keychain permissions configured successfully");
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            debug!("Keychain permission setup result: {}", stderr);
            // This might "fail" but still work, so we'll test access next
            Ok(())
        }
    }
}

impl Default for SmartKeychainManager {
    fn default() -> Self {
        Self::new().unwrap_or(Self)
    }
}
