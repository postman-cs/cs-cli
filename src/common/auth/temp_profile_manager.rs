//! Temporary profile manager for browser authentication
//!
//! This module creates temporary browser profiles containing only the cookies database
//! from the original profile. This avoids conflicts when WebDriver tries to use a
//! profile that's already being used by a running browser instance.

use crate::common::auth::profile_detector::{BrowserProfile, BrowserType};
use crate::common::error::{CsCliError, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

pub struct TempProfileManager {
    temp_dir: PathBuf,
    created_profiles: Vec<PathBuf>,
}

impl TempProfileManager {
    /// Create a new temporary profile manager
    pub fn new() -> Result<Self> {
        let temp_dir = std::env::temp_dir()
            .join("cs-cli-temp-profiles")
            .join(format!("session-{}", std::process::id()));

        fs::create_dir_all(&temp_dir)
            .map_err(|e| CsCliError::Authentication(format!("Failed to create temp profiles directory: {e}")))?;

        info!("Created temporary profiles directory: {}", temp_dir.display());

        Ok(Self {
            temp_dir,
            created_profiles: Vec::new(),
        })
    }

    /// Create a temporary profile with only the cookies database from the original profile
    pub fn create_temp_profile(&mut self, original_profile: &BrowserProfile) -> Result<BrowserProfile> {
        let temp_profile_id = format!("temp-{}-{}",
            original_profile.profile_id.replace(['/', '\\'], "-"),
            jiff::Timestamp::now().as_second()
        );

        let temp_profile_path = self.temp_dir.join(&temp_profile_id);

        info!("Creating temporary profile: {} -> {}",
            original_profile.profile_path.display(),
            temp_profile_path.display()
        );

        // Create the temporary profile directory
        fs::create_dir_all(&temp_profile_path)
            .map_err(|e| CsCliError::Authentication(format!("Failed to create temp profile directory: {e}")))?;

        // Copy cookies database based on browser type
        match original_profile.browser_type {
            BrowserType::Chrome | BrowserType::Brave | BrowserType::Edge |
            BrowserType::Arc | BrowserType::Chromium | BrowserType::Opera |
            BrowserType::OperaGX | BrowserType::Vivaldi | BrowserType::Comet => {
                self.copy_chromium_cookies(&original_profile.profile_path, &temp_profile_path)?;
            }
            BrowserType::Firefox | BrowserType::LibreWolf | BrowserType::Zen => {
                self.copy_firefox_cookies(&original_profile.profile_path, &temp_profile_path)?;
            }
            BrowserType::Safari => {
                // Safari uses system keychain, no profile copying needed
                warn!("Safari profile copying not supported - using original profile");
            }
        }

        // Add to cleanup list
        self.created_profiles.push(temp_profile_path.clone());

        // Create a new BrowserProfile pointing to the temporary location
        Ok(BrowserProfile {
            browser_type: original_profile.browser_type,
            profile_id: temp_profile_id,
            profile_name: format!("Temp-{}", original_profile.profile_name),
            email: original_profile.email.clone(),
            profile_path: temp_profile_path,
            last_active: original_profile.last_active,
            is_managed: original_profile.is_managed,
            hosted_domain: original_profile.hosted_domain.clone(),
        })
    }

    /// Copy Chromium-based browser cookies database
    fn copy_chromium_cookies(&self, source_profile: &Path, dest_profile: &Path) -> Result<()> {
        let cookies_files = [
            "Cookies",           // Main cookies file
            "Cookies-journal",   // SQLite journal file (if exists)
            "Network/Cookies",   // Alternative location for some browsers
        ];

        let mut copied_any = false;

        for cookies_file in &cookies_files {
            let source_path = source_profile.join(cookies_file);
            let dest_path = dest_profile.join(cookies_file);

            if source_path.exists() {
                // Create parent directory if needed
                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| CsCliError::Authentication(format!("Failed to create cookies directory: {e}")))?;
                }

                match fs::copy(&source_path, &dest_path) {
                    Ok(bytes) => {
                        debug!("Copied {} ({} bytes): {} -> {}",
                            cookies_file, bytes, source_path.display(), dest_path.display());
                        copied_any = true;
                    }
                    Err(e) => {
                        warn!("Failed to copy {}: {}", cookies_file, e);
                    }
                }
            } else {
                debug!("Cookies file not found: {}", source_path.display());
            }
        }

        if !copied_any {
            return Err(CsCliError::Authentication(
                "No cookies files found to copy from Chromium profile".to_string()
            ));
        }

        // Also copy essential browser files that might be needed
        let essential_files = [
            "Preferences",       // User preferences
            "Local State",       // Browser state (if at profile level)
        ];

        for essential_file in &essential_files {
            let source_path = source_profile.join(essential_file);
            let dest_path = dest_profile.join(essential_file);

            if source_path.exists() {
                if let Err(e) = fs::copy(&source_path, &dest_path) {
                    debug!("Failed to copy {} (non-critical): {}", essential_file, e);
                } else {
                    debug!("Copied essential file: {}", essential_file);
                }
            }
        }

        Ok(())
    }

    /// Copy Firefox-based browser cookies database
    fn copy_firefox_cookies(&self, source_profile: &Path, dest_profile: &Path) -> Result<()> {
        let cookies_files = [
            "cookies.sqlite",        // Main cookies database
            "cookies.sqlite-wal",    // Write-ahead log (if exists)
            "cookies.sqlite-shm",    // Shared memory (if exists)
        ];

        let mut copied_any = false;

        for cookies_file in &cookies_files {
            let source_path = source_profile.join(cookies_file);
            let dest_path = dest_profile.join(cookies_file);

            if source_path.exists() {
                match fs::copy(&source_path, &dest_path) {
                    Ok(bytes) => {
                        debug!("Copied {} ({} bytes): {} -> {}",
                            cookies_file, bytes, source_path.display(), dest_path.display());
                        copied_any = true;
                    }
                    Err(e) => {
                        warn!("Failed to copy {}: {}", cookies_file, e);
                    }
                }
            } else {
                debug!("Firefox cookies file not found: {}", source_path.display());
            }
        }

        if !copied_any {
            return Err(CsCliError::Authentication(
                "No cookies files found to copy from Firefox profile".to_string()
            ));
        }

        // Copy essential Firefox files
        let essential_files = [
            "prefs.js",          // User preferences
            "user.js",           // User configuration (if exists)
        ];

        for essential_file in &essential_files {
            let source_path = source_profile.join(essential_file);
            let dest_path = dest_profile.join(essential_file);

            if source_path.exists() {
                if let Err(e) = fs::copy(&source_path, &dest_path) {
                    debug!("Failed to copy {} (non-critical): {}", essential_file, e);
                } else {
                    debug!("Copied essential Firefox file: {}", essential_file);
                }
            }
        }

        Ok(())
    }

    /// Get the temporary directory path
    pub fn get_temp_dir(&self) -> &Path {
        &self.temp_dir
    }

    /// Clean up all created temporary profiles
    pub fn cleanup(&mut self) -> Result<()> {
        info!("Cleaning up {} temporary profiles", self.created_profiles.len());

        for profile_path in &self.created_profiles {
            if profile_path.exists() {
                match fs::remove_dir_all(profile_path) {
                    Ok(()) => {
                        debug!("Removed temporary profile: {}", profile_path.display());
                    }
                    Err(e) => {
                        warn!("Failed to remove temporary profile {}: {}", profile_path.display(), e);
                    }
                }
            }
        }

        // Clean up the main temp directory if empty
        if self.temp_dir.exists() {
            if let Ok(entries) = fs::read_dir(&self.temp_dir) {
                if entries.count() == 0 {
                    if let Err(e) = fs::remove_dir(&self.temp_dir) {
                        debug!("Failed to remove temp directory {}: {}", self.temp_dir.display(), e);
                    } else {
                        debug!("Removed temp directory: {}", self.temp_dir.display());
                    }
                }
            }
        }

        self.created_profiles.clear();
        Ok(())
    }
}

impl Drop for TempProfileManager {
    fn drop(&mut self) {
        // Clean up on drop, but don't panic if it fails
        let _ = self.cleanup();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_temp_profile_manager_creation() {
        let manager = TempProfileManager::new().unwrap();
        assert!(manager.temp_dir.exists());
        assert!(manager.created_profiles.is_empty());
    }

    #[test]
    fn test_chromium_cookies_copy() {
        let mut manager = TempProfileManager::new().unwrap();

        // Create a fake source profile with cookies
        let source_dir = std::env::temp_dir().join("test-source-profile");
        fs::create_dir_all(&source_dir).unwrap();

        let cookies_path = source_dir.join("Cookies");
        let mut cookies_file = File::create(&cookies_path).unwrap();
        cookies_file.write_all(b"fake cookies data").unwrap();

        // Create a fake browser profile
        let profile = BrowserProfile {
            browser_type: BrowserType::Chrome,
            profile_id: "test".to_string(),
            profile_name: "Test Profile".to_string(),
            email: None,
            profile_path: source_dir.clone(),
            last_active: None,
            is_managed: false,
            hosted_domain: None,
        };

        // Test temp profile creation
        let temp_profile = manager.create_temp_profile(&profile).unwrap();
        assert!(temp_profile.profile_path.exists());
        assert!(temp_profile.profile_path.join("Cookies").exists());

        // Cleanup
        fs::remove_dir_all(&source_dir).unwrap();
        manager.cleanup().unwrap();
    }
}