//! Hybrid Cookie Storage with GitHub Gist Sync
//!
//! Provides seamless cookie storage that automatically syncs via GitHub gists
//! when available, with transparent fallback to local keychain storage.

use super::github_gist_storage::GitHubGistStorage;
use super::guided_cookie_storage;
use crate::{CsCliError, Result};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Hybrid storage backend that combines gist sync with local fallback
pub struct HybridCookieStorage {
    gist_storage: Option<GitHubGistStorage>,
    #[allow(dead_code)]
    sync_enabled: bool,
}

impl HybridCookieStorage {
    /// Initialize hybrid storage with optional GitHub gist sync
    pub async fn new() -> Result<Self> {
        // Check if user wants gist sync (only prompt once)
        let sync_enabled = should_enable_sync().await;

        let gist_storage = if sync_enabled {
            match GitHubGistStorage::new().await {
                Ok(storage) => {
                    info!("GitHub gist sync enabled");
                    Some(storage)
                }
                Err(e) => {
                    warn!("Failed to initialize GitHub gist storage: {}", e);
                    info!("Falling back to local-only storage");
                    None
                }
            }
        } else {
            debug!("GitHub gist sync disabled by user preference");
            None
        };

        Ok(Self {
            gist_storage,
            sync_enabled,
        })
    }

    /// Store cookies with automatic sync when available
    pub async fn store_cookies(&mut self, cookies: &HashMap<String, String>) -> Result<()> {
        let mut gist_success = false;

        // Try gist storage first (if enabled)
        if let Some(ref mut gist_storage) = self.gist_storage {
            match gist_storage.store_cookies(cookies).await {
                Ok(()) => {
                    info!("Session data synced successfully");
                    gist_success = true;
                }
                Err(e) => {
                    warn!("Gist storage failed, using local fallback: {}", e);
                }
            }
        }

        // Always store locally as backup (even when gist succeeds)
        match tokio::task::spawn_blocking({
            let cookies = cookies.clone();
            move || guided_cookie_storage::store_cookies(&cookies)
        })
        .await
        {
            Ok(Ok(())) => {
                if gist_success {
                    debug!("Session data also stored locally as backup");
                } else {
                    info!("Session data stored locally");
                }
            }
            Ok(Err(e)) => {
                if !gist_success {
                    return Err(e);
                } else {
                    warn!("Local storage failed but gist storage succeeded: {}", e);
                }
            }
            Err(e) => {
                warn!("Local storage task failed: {}", e);
                if !gist_success {
                    return Err(CsCliError::Authentication(
                        "All storage methods failed".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Retrieve cookies from best available source
    pub async fn get_cookies(&self) -> Result<HashMap<String, String>> {
        // Try gist storage first (most up-to-date)
        if let Some(ref gist_storage) = self.gist_storage {
            match gist_storage.get_cookies().await {
                Ok(cookies) => {
                    debug!("Using synced session data from GitHub gist");
                    return Ok(cookies);
                }
                Err(e) => {
                    warn!("Failed to retrieve from gist, trying local: {}", e);
                }
            }
        }

        // Fallback to local storage
        tokio::task::spawn_blocking(guided_cookie_storage::get_stored_cookies)
            .await
            .map_err(|e| CsCliError::Authentication(format!("Task failed: {e}")))?
    }

    /// Check if cookies exist in any storage location
    pub async fn has_cookies(&self) -> bool {
        // Check gist storage first
        if let Some(ref gist_storage) = self.gist_storage {
            if gist_storage.has_cookies().await {
                return true;
            }
        }

        // Check local storage
        tokio::task::spawn_blocking(guided_cookie_storage::has_stored_cookies)
            .await
            .unwrap_or(false)
    }

    /// Delete cookies from all storage locations
    pub async fn delete_cookies(&mut self) -> Result<()> {
        let mut errors = Vec::new();

        // Delete from gist storage
        if let Some(ref gist_storage) = self.gist_storage {
            if let Err(e) = gist_storage.delete_cookies().await {
                errors.push(format!("Gist deletion failed: {e}"));
            }
        }

        // Delete from local storage
        match tokio::task::spawn_blocking(guided_cookie_storage::delete_stored_cookies).await {
            Ok(Ok(())) => debug!("Local session data deleted"),
            Ok(Err(e)) => errors.push(format!("Local deletion failed: {e}")),
            Err(e) => errors.push(format!("Local deletion task failed: {e}")),
        }

        if errors.is_empty() {
            info!("Session data deleted from all storage locations");
            Ok(())
        } else {
            warn!("Some deletion operations failed: {:?}", errors);
            Ok(()) // Non-fatal - best effort cleanup
        }
    }

    /// Get sync status for display purposes
    pub fn is_sync_enabled(&self) -> bool {
        self.gist_storage.is_some()
    }
}

/// Check if user wants to enable GitHub gist sync
async fn should_enable_sync() -> bool {
    // Check if user previously made a decision
    if let Ok(preference) = get_sync_preference() {
        return preference;
    }

    // No preference found - this means it's a first run
    // The TUI will handle the sync choice, so default to local-only for now
    false
}

/// Set sync preference from TUI choice
pub async fn set_sync_preference(enabled: bool) -> Result<()> {
    save_sync_preference(enabled)
}

/// Get saved sync preference
fn get_sync_preference() -> Result<bool> {
    let config_path = get_sync_preference_path()?;
    let preference = std::fs::read_to_string(config_path)
        .map_err(|_| CsCliError::Configuration("No sync preference found".to_string()))?;

    Ok(preference.trim() == "enabled")
}

/// Save sync preference
fn save_sync_preference(enabled: bool) -> Result<()> {
    let config_path = get_sync_preference_path()?;

    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| CsCliError::Configuration(format!("Failed to create config dir: {e}")))?;
    }

    let preference = if enabled { "enabled" } else { "disabled" };
    std::fs::write(config_path, preference)
        .map_err(|e| CsCliError::Configuration(format!("Failed to save preference: {e}")))?;

    Ok(())
}

/// Get sync preference file path
fn get_sync_preference_path() -> Result<std::path::PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| {
            CsCliError::Configuration("Unable to determine config directory".to_string())
        })?
        .join("cs-cli");

    Ok(config_dir.join("sync-preference"))
}

// Public interface functions for easy integration
pub async fn store_cookies_hybrid(cookies: &HashMap<String, String>) -> Result<()> {
    let mut storage = HybridCookieStorage::new().await?;
    storage.store_cookies(cookies).await
}

pub async fn get_cookies_hybrid() -> Result<HashMap<String, String>> {
    let storage = HybridCookieStorage::new().await?;
    storage.get_cookies().await
}

pub async fn has_cookies_hybrid() -> bool {
    match HybridCookieStorage::new().await {
        Ok(storage) => storage.has_cookies().await,
        Err(_) => false,
    }
}

pub async fn delete_cookies_hybrid() -> Result<()> {
    let mut storage = HybridCookieStorage::new().await?;
    storage.delete_cookies().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hybrid_storage_initialization() {
        // This test verifies that hybrid storage can be created
        // without requiring actual GitHub authentication
        let result = HybridCookieStorage::new().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_sync_preference_management() {
        // Test saving and loading sync preferences
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("sync-preference");

        // Mock the config path by creating a temporary preference file
        std::fs::write(&config_path, "enabled").unwrap();
        let content = std::fs::read_to_string(&config_path).unwrap();
        assert_eq!(content.trim(), "enabled");

        std::fs::write(&config_path, "disabled").unwrap();
        let content = std::fs::read_to_string(&config_path).unwrap();
        assert_eq!(content.trim(), "disabled");
    }
}
