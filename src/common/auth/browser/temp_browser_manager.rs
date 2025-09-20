//! Temporary browser manager for browser authentication
//!
//! This module creates temporary browser instances containing only the cookies database
//! from the original browser. This avoids conflicts when CDP tries to use a
//! browser that's already being used by a running browser instance.

use crate::common::auth::browser::profile_detector::{BrowserProfile, BrowserType};
use crate::common::error::{CsCliError, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

pub struct TempBrowserManager {
    temp_dir: PathBuf,
    created_browsers: Vec<PathBuf>,
}

impl TempBrowserManager {
    /// Create a new temporary browser manager using common utilities
    pub fn new() -> Result<Self> {
        let temp_dir = crate::common::file_io::create_temp_directory("cs-cli-temp-browsers")
            .map_err(|e| CsCliError::Authentication(format!("Failed to create temp browsers directory: {}", e)))?;

        Ok(Self {
            temp_dir,
            created_browsers: Vec::new(),
        })
    }

    /// Create a temporary browser by copying minimal browser folder structure
    pub fn create_temp_browser(&mut self, original_profile: &BrowserProfile) -> Result<BrowserProfile> {
        let temp_browser_id = format!("temp-{}-{}",
            original_profile.profile_id.replace(['/', '\\'], "-"),
            jiff::Timestamp::now().as_second()
        );

        let temp_browser_path = self.temp_dir.join(&temp_browser_id);
        let temp_profile_path = temp_browser_path.join(&original_profile.profile_id);

        info!("Creating minimal temporary browser folder: {} -> {}",
            original_profile.profile_path.parent().unwrap_or(&original_profile.profile_path).display(),
            temp_browser_path.display()
        );

        // Create the temporary browser directory structure
        fs::create_dir_all(&temp_profile_path)
            .map_err(|e| CsCliError::Authentication(format!("Failed to create temp browser directory: {e}")))?;

        // Copy minimal browser folder structure with essential authentication files
        match original_profile.browser_type {
            BrowserType::Chrome | BrowserType::Brave | BrowserType::Edge |
            BrowserType::Arc | BrowserType::Chromium | BrowserType::Opera |
            BrowserType::OperaGX | BrowserType::Vivaldi | BrowserType::Comet => {
                self.copy_minimal_chromium_browser(&original_profile.profile_path, &temp_browser_path, &temp_profile_path)?;
            }
            BrowserType::Firefox | BrowserType::LibreWolf | BrowserType::Zen => {
                self.copy_essential_gecko_files(&original_profile.profile_path, &temp_profile_path)?;
            }
            BrowserType::Safari => {
                return Err(CsCliError::Authentication(
                    "Safari does not support minimal browser creation. Use Safari binary cookie extraction or fallback to Chrome for guided authentication.".to_string()
                ));
            }
        }

        // Add to cleanup list
        self.created_browsers.push(temp_browser_path.clone());

        // Create a new BrowserProfile pointing to the temporary location
        Ok(BrowserProfile {
            browser_type: original_profile.browser_type,
            profile_id: temp_browser_id.clone(),
            profile_name: format!("Temp-{}", original_profile.profile_name),
            email: original_profile.email.clone(),
            profile_path: temp_profile_path,
            last_active: original_profile.last_active,
            is_managed: original_profile.is_managed,
            hosted_domain: original_profile.hosted_domain.clone(),
        })
    }

    /// Copy minimal Chromium browser folder structure (following create_minimal_brave_profile.sh approach)
    fn copy_minimal_chromium_browser(&self, source_profile_path: &Path, dest_browser_path: &Path, dest_profile_path: &Path) -> Result<()> {
        info!("Copying minimal Chromium browser folder structure...");

        let mut copied_files = 0;
        let mut total_bytes = 0u64;

        // Get the browser root directory (parent of profile)
        let source_browser_dir = source_profile_path.parent()
            .ok_or_else(|| CsCliError::Authentication("Cannot determine browser root directory".to_string()))?;

        // Step 1: Copy browser root files (Local State with encryption keys)
        let browser_root_files = [
            "Local State",  // Contains encryption keys essential for cookie decryption
        ];

        for file_name in &browser_root_files {
            let source_path = source_browser_dir.join(file_name);
            let dest_path = dest_browser_path.join(file_name);

            if source_path.exists() {
                match fs::copy(&source_path, &dest_path) {
                    Ok(bytes) => {
                        info!("✓ Copied browser root {}: {} bytes", file_name, bytes);
                        copied_files += 1;
                        total_bytes += bytes;
                    }
                    Err(e) => {
                        warn!("Failed to copy browser root {}: {}", file_name, e);
                    }
                }
            } else {
                warn!("Browser root file not found: {}", source_path.display());
            }
        }

        // Step 2: Copy minimal essential profile files for authentication only
        let profile_files = [
            "Cookies",                   // Main cookies database
            "Login Data",                // Stored login credentials
            "Login Data For Account",    // Account-specific login data
            "Web Data",                  // Web form data and autofill
            "Preferences",               // Profile settings (will be cleaned of extensions)
            "Bookmarks",                 // User bookmarks
        ];

        for file_name in &profile_files {
            let source_path = source_profile_path.join(file_name);
            let dest_path = dest_profile_path.join(file_name);

            if source_path.exists() {
                match fs::copy(&source_path, &dest_path) {
                    Ok(bytes) => {
                        info!("✓ Copied profile {}: {} bytes", file_name, bytes);
                        copied_files += 1;
                        total_bytes += bytes;
                    }
                    Err(e) => {
                        warn!("Failed to copy profile {}: {}", file_name, e);
                    }
                }
            } else {
                debug!("Profile file not found (optional): {}", source_path.display());
            }
        }

        // Step 3: Clean extension data from Preferences if it exists
        let prefs_path = dest_profile_path.join("Preferences");
        if prefs_path.exists() {
            if let Err(e) = self.clean_extensions_from_preferences(&prefs_path) {
                warn!("Failed to clean extensions from Preferences: {}", e);
            }
        }

        if copied_files == 0 {
            return Err(CsCliError::Authentication(
                "No essential authentication files found to copy from Chromium browser".to_string()
            ));
        }

        info!("Successfully copied {} minimal browser files ({} total bytes) for authentication",
               copied_files, total_bytes);

        Ok(())
    }

    /// Clean extension data from Chromium Preferences file (following create_minimal_brave_profile.sh approach)
    fn clean_extensions_from_preferences(&self, prefs_path: &Path) -> Result<()> {
        info!("Cleaning extension data from Preferences file...");

        // Read the preferences file
        let prefs_content = fs::read_to_string(prefs_path)
            .map_err(|e| CsCliError::Authentication(format!("Failed to read Preferences file: {e}")))?;

        // Parse as JSON
        let mut prefs: serde_json::Value = serde_json::from_str(&prefs_content)
            .map_err(|e| CsCliError::Authentication(format!("Failed to parse Preferences JSON: {e}")))?;

        // Remove specific extension-related keys (following jq commands from shell script)
        if let Some(obj) = prefs.as_object_mut() {
            // Remove main extension keys
            obj.remove("extensions");
            obj.remove("extensions_ui_developer_mode");
            obj.remove("ack_existing_ntp_extensions");

            // Remove account_values.extensions
            if let Some(account_values) = obj.get_mut("account_values") {
                if let Some(account_obj) = account_values.as_object_mut() {
                    account_obj.remove("extensions");

                    // Remove brave.default_private_search_provider_data
                    if let Some(brave) = account_obj.get_mut("brave") {
                        if let Some(brave_obj) = brave.as_object_mut() {
                            brave_obj.remove("default_private_search_provider_data");
                        }
                    }
                }
            }

            // Remove any remaining keys containing "extension" (case insensitive)
            self.remove_extension_keys_recursive(&mut prefs);
        }

        // Write the cleaned preferences back
        let cleaned_json = serde_json::to_string_pretty(&prefs)
            .map_err(|e| CsCliError::Authentication(format!("Failed to serialize cleaned Preferences: {e}")))?;

        fs::write(prefs_path, cleaned_json)
            .map_err(|e| CsCliError::Authentication(format!("Failed to write cleaned Preferences: {e}")))?;

        info!("✓ Successfully cleaned extension data from Preferences");
        Ok(())
    }

    /// Recursively remove keys containing "extension" (case insensitive)
    fn remove_extension_keys_recursive(&self, value: &mut serde_json::Value) {
        match value {
            serde_json::Value::Object(map) => {
                // Collect keys to remove (can't modify while iterating)
                let keys_to_remove: Vec<String> = map
                    .keys()
                    .filter(|key| key.to_lowercase().contains("extension"))
                    .cloned()
                    .collect();

                // Remove the keys
                for key in keys_to_remove {
                    map.remove(&key);
                }

                // Recursively clean remaining values
                for (_, v) in map.iter_mut() {
                    self.remove_extension_keys_recursive(v);
                }
            }
            serde_json::Value::Array(arr) => {
                // Recursively clean array elements
                for item in arr.iter_mut() {
                    self.remove_extension_keys_recursive(item);
                }
            }
            _ => {
                // Primitive values, nothing to clean
            }
        }
    }

    /// Copy essential Gecko-based browser files for authentication (Firefox, LibreWolf, Zen)
    fn copy_essential_gecko_files(&self, source_profile: &Path, dest_profile: &Path) -> Result<()> {
        info!("Copying essential Gecko browser authentication files...");

        // Essential files within the profile directory
        let profile_files = [
            "cookies.sqlite",        // Main cookies database
            "cookies.sqlite-wal",    // Write-ahead log (if exists)
            "cookies.sqlite-shm",    // Shared memory (if exists)
            "prefs.js",              // User preferences
            "user.js",               // User configuration (if exists)
            "key4.db",               // Encryption keys
            "cert9.db",              // Certificate database
            "logins.json",           // Login data (may contain tokens)
        ];

        // Essential directories within the profile
        let _profile_directories = [
            "storage",               // IndexedDB equivalent for Firefox
            "webappsstore.sqlite",   // Local storage (but this is a file)
        ];

        let mut copied_files = 0;
        let mut total_bytes = 0u64;

        // Copy individual files
        for file_name in &profile_files {
            let source_path = source_profile.join(file_name);
            let dest_path = dest_profile.join(file_name);

            if source_path.exists() {
                match fs::copy(&source_path, &dest_path) {
                    Ok(bytes) => {
                        info!("✓ Copied {}: {} bytes", file_name, bytes);
                        copied_files += 1;
                        total_bytes += bytes;
                    }
                    Err(e) => {
                        warn!("Failed to copy {}: {}", file_name, e);
                    }
                }
            } else {
                debug!("File not found (optional): {}", source_path.display());
            }
        }

        // Copy storage directory if it exists
        let storage_dir = source_profile.join("storage");
        if storage_dir.exists() && storage_dir.is_dir() {
            let dest_storage = dest_profile.join("storage");
            match self.copy_directory_recursive(&storage_dir, &dest_storage) {
                Ok(bytes) => {
                    info!("✓ Copied storage directory: {} bytes", bytes);
                    copied_files += 1;
                    total_bytes += bytes;
                }
                Err(e) => {
                    warn!("Failed to copy storage directory: {}", e);
                }
            }
        }

        if copied_files == 0 {
            return Err(CsCliError::Authentication(
                "No essential authentication files found to copy from Gecko browser profile".to_string()
            ));
        }

        info!("Successfully copied {} essential files ({} total bytes) for authentication",
               copied_files, total_bytes);

        Ok(())
    }

    /// Recursively copy a directory using common utilities
    fn copy_directory_recursive(&self, source: &Path, dest: &Path) -> Result<u64> {
        crate::common::file_io::copy_directory_recursive(source, dest)
            .map_err(|e| CsCliError::Authentication(format!("Failed to copy directory {} to {}: {}", source.display(), dest.display(), e)))
    }

    /// Get the temporary directory path
    pub fn get_temp_dir(&self) -> &Path {
        &self.temp_dir
    }

    /// Clean up all created temporary browsers using common utilities
    pub fn cleanup(&mut self) -> Result<()> {
        info!("Cleaning up {} temporary browsers", self.created_browsers.len());

        for browser_path in &self.created_browsers {
            if browser_path.exists() {
                match crate::common::file_io::remove_directory(browser_path) {
                    Ok(()) => {
                        debug!("Removed temporary browser: {}", browser_path.display());
                    }
                    Err(e) => {
                        warn!("Failed to remove temporary browser {}: {}", browser_path.display(), e);
                    }
                }
            }
        }

        // Clean up the main temp directory if empty
        if self.temp_dir.exists() {
            match crate::common::file_io::is_directory_empty(&self.temp_dir) {
                Ok(true) => {
                    crate::common::file_io::remove_directory(&self.temp_dir)?;
                    debug!("Removed temp directory: {}", self.temp_dir.display());
                }
                Ok(false) => {
                    debug!("Temp directory not empty, keeping: {}", self.temp_dir.display());
                }
                Err(e) => {
                    debug!("Failed to check if temp directory is empty: {}", e);
                }
            }
        }

        self.created_browsers.clear();
        Ok(())
    }
}

impl Drop for TempBrowserManager {
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
    fn test_temp_browser_manager_creation() {
        let manager = TempBrowserManager::new().unwrap();
        assert!(manager.temp_dir.exists());
        assert!(manager.created_browsers.is_empty());
    }

    #[test]
    fn test_chromium_minimal_browser_copy() {
        let mut manager = TempBrowserManager::new().unwrap();

        // Create a fake browser directory structure
        let browser_dir = std::env::temp_dir().join("test-browser");
        let source_profile_dir = browser_dir.join("Default");
        fs::create_dir_all(&source_profile_dir).unwrap();

        // Create fake browser root files
        let local_state_path = browser_dir.join("Local State");
        let mut local_state_file = File::create(&local_state_path).unwrap();
        local_state_file.write_all(b"fake local state data").unwrap();

        // Create fake profile files
        let cookies_path = source_profile_dir.join("Cookies");
        let mut cookies_file = File::create(&cookies_path).unwrap();
        cookies_file.write_all(b"fake cookies data").unwrap();

        let prefs_path = source_profile_dir.join("Preferences");
        let mut prefs_file = File::create(&prefs_path).unwrap();
        prefs_file.write_all(b"{\"test\": \"data\"}").unwrap();

        // Create a fake browser profile
        let profile = BrowserProfile {
            browser_type: BrowserType::Chrome,
            profile_id: "Default".to_string(),
            profile_name: "Test Profile".to_string(),
            email: None,
            profile_path: source_profile_dir.clone(),
            last_active: None,
            is_managed: false,
            hosted_domain: None,
        };

        // Test temp browser creation
        let temp_profile = manager.create_temp_browser(&profile).unwrap();
        assert!(temp_profile.profile_path.exists());
        assert!(temp_profile.profile_path.join("Cookies").exists());

        // Check that browser root structure was created
        let temp_browser_path = temp_profile.profile_path.parent().unwrap();
        assert!(temp_browser_path.join("Local State").exists());

        // Cleanup
        fs::remove_dir_all(&browser_dir).unwrap();
        manager.cleanup().unwrap();
    }
}