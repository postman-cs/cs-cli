//! Gist Configuration Manager
//!
//! Handles local configuration storage and management for GitHub gist
//! storage operations.

use super::github_gist_errors::{GistStorageError, GistResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{debug, info, warn};

/// Configuration stored locally to track gist
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GistConfig {
    pub gist_id: String,
    pub github_username: String,
    pub token_hash: String, // SHA-256 hash of token for validation
    pub last_sync: String,
    pub created_at: String,
    pub version: u32,
}

impl GistConfig {
    /// Create new gist configuration
    pub fn new(gist_id: String, github_username: String, token_hash: String) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            gist_id,
            github_username,
            token_hash,
            last_sync: now.clone(),
            created_at: now,
            version: 1,
        }
    }

    /// Update the last sync timestamp
    pub fn update_sync_time(&mut self) {
        self.last_sync = chrono::Utc::now().to_rfc3339();
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), GistStorageError> {
        if self.gist_id.is_empty() {
            return Err(GistStorageError::ConfigError {
                field: "gist_id".to_string(),
                reason: "Gist ID cannot be empty".to_string(),
            });
        }

        if self.github_username.is_empty() {
            return Err(GistStorageError::ConfigError {
                field: "github_username".to_string(),
                reason: "GitHub username cannot be empty".to_string(),
            });
        }

        if self.token_hash.len() != 64 {
            return Err(GistStorageError::ConfigError {
                field: "token_hash".to_string(),
                reason: "Token hash must be 64 characters (SHA-256)".to_string(),
            });
        }

        // Validate timestamps
        if let Err(e) = chrono::DateTime::parse_from_rfc3339(&self.last_sync) {
            return Err(GistStorageError::ConfigError {
                field: "last_sync".to_string(),
                reason: format!("Invalid timestamp format: {}", e),
            });
        }

        if let Err(e) = chrono::DateTime::parse_from_rfc3339(&self.created_at) {
            return Err(GistStorageError::ConfigError {
                field: "created_at".to_string(),
                reason: format!("Invalid timestamp format: {}", e),
            });
        }

        Ok(())
    }

    /// Check if configuration is recent (within last 30 days)
    pub fn is_recent(&self) -> bool {
        if let Ok(last_sync) = chrono::DateTime::parse_from_rfc3339(&self.last_sync) {
            let now = chrono::Utc::now();
            let thirty_days_ago = now - chrono::Duration::days(30);
            last_sync.with_timezone(&chrono::Utc) > thirty_days_ago
        } else {
            false
        }
    }
}

/// Configuration manager for gist storage
pub struct GistConfigManager {
    config_path: PathBuf,
}

impl GistConfigManager {
    /// Create new configuration manager
    pub fn new() -> GistResult<Self> {
        let config_path = Self::get_config_path()?;
        Ok(Self { config_path })
    }

    /// Load gist configuration from local storage
    pub fn load(&self) -> GistResult<Option<GistConfig>> {
        if !self.config_path.exists() {
            debug!("Configuration file does not exist: {:?}", self.config_path);
            return Ok(None);
        }

        let config_data = std::fs::read_to_string(&self.config_path)
            .map_err(|e| GistStorageError::ConfigError {
                field: "file_read".to_string(),
                reason: format!("Failed to read config file: {}", e),
            })?;

        // Check if file is empty
        if config_data.trim().is_empty() {
            debug!("Configuration file is empty: {:?}", self.config_path);
            return Ok(None);
        }

        let config: GistConfig = serde_json::from_str(&config_data)
            .map_err(|e| GistStorageError::ConfigError {
                field: "json_parse".to_string(),
                reason: format!("Invalid configuration format: {}", e),
            })?;

        // Validate the loaded configuration
        config.validate()?;

        info!("Loaded gist configuration for user: {}", config.github_username);
        Ok(Some(config))
    }

    /// Save gist configuration to local storage
    pub fn save(&self, config: &GistConfig) -> GistResult<()> {
        // Validate before saving
        config.validate()?;

        // Create config directory if needed
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| GistStorageError::ConfigError {
                    field: "directory_creation".to_string(),
                    reason: format!("Failed to create config directory: {}", e),
                })?;
        }

        let config_data = serde_json::to_string_pretty(config)
            .map_err(|e| GistStorageError::ConfigError {
                field: "json_serialization".to_string(),
                reason: format!("Failed to serialize config: {}", e),
            })?;

        std::fs::write(&self.config_path, config_data)
            .map_err(|e| GistStorageError::ConfigError {
                field: "file_write".to_string(),
                reason: format!("Failed to save config: {}", e),
            })?;

        debug!("Saved gist configuration to: {:?}", self.config_path);
        Ok(())
    }

    /// Remove gist configuration
    pub fn remove(&self) -> GistResult<()> {
        if self.config_path.exists() {
            std::fs::remove_file(&self.config_path)
                .map_err(|e| GistStorageError::ConfigError {
                    field: "file_removal".to_string(),
                    reason: format!("Failed to remove config: {}", e),
                })?;
            
            info!("Removed gist configuration");
        }
        Ok(())
    }

    /// Check if configuration exists
    pub fn exists(&self) -> bool {
        self.config_path.exists()
    }

    /// Get configuration file path
    pub fn config_path(&self) -> &PathBuf {
        &self.config_path
    }

    /// Get configuration file path
    fn get_config_path() -> GistResult<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| GistStorageError::ConfigError {
                field: "config_directory".to_string(),
                reason: "Unable to determine config directory".to_string(),
            })?
            .join("cs-cli");

        Ok(config_dir.join("github-gist-config.json"))
    }

    /// Backup existing configuration
    pub fn backup(&self) -> GistResult<Option<PathBuf>> {
        if !self.config_path.exists() {
            return Ok(None);
        }

        let backup_path = self.config_path.with_extension("json.backup");
        std::fs::copy(&self.config_path, &backup_path)
            .map_err(|e| GistStorageError::ConfigError {
                field: "backup_creation".to_string(),
                reason: format!("Failed to create backup: {}", e),
            })?;

        info!("Created configuration backup: {:?}", backup_path);
        Ok(Some(backup_path))
    }

    /// Restore from backup
    pub fn restore_from_backup(&self) -> GistResult<()> {
        let backup_path = self.config_path.with_extension("json.backup");
        
        if !backup_path.exists() {
            return Err(GistStorageError::ConfigError {
                field: "backup_restore".to_string(),
                reason: "No backup file found".to_string(),
            });
        }

        std::fs::copy(&backup_path, &self.config_path)
            .map_err(|e| GistStorageError::ConfigError {
                field: "backup_restore".to_string(),
                reason: format!("Failed to restore from backup: {}", e),
            })?;

        info!("Restored configuration from backup");
        Ok(())
    }

    /// Validate configuration file integrity
    pub fn validate_file(&self) -> GistResult<bool> {
        match self.load() {
            Ok(Some(config)) => {
                config.validate()?;
                Ok(true)
            }
            Ok(None) => Ok(false), // File doesn't exist, which is valid
            Err(e) => {
                warn!("Configuration file validation failed: {}", e);
                Ok(false)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_gist_config_creation() {
        let config = GistConfig::new(
            "test-gist-id".to_string(),
            "testuser".to_string(),
            "a".repeat(64), // Valid SHA-256 hash length
        );

        assert_eq!(config.gist_id, "test-gist-id");
        assert_eq!(config.github_username, "testuser");
        assert_eq!(config.version, 1);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_gist_config_validation() {
        let mut config = GistConfig::new(
            "".to_string(), // Invalid empty gist_id
            "testuser".to_string(),
            "a".repeat(64),
        );

        assert!(config.validate().is_err());

        config.gist_id = "valid-id".to_string();
        config.token_hash = "invalid".to_string(); // Too short
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_manager_operations() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test-config.json");
        
        // Create a custom config manager with temp path
        let manager = GistConfigManager {
            config_path: config_path.clone(),
        };

        let config = GistConfig::new(
            "test-gist-id".to_string(),
            "testuser".to_string(),
            "a".repeat(64),
        );

        // Test save
        assert!(manager.save(&config).is_ok());
        assert!(manager.exists());

        // Test load
        let loaded = manager.load().unwrap().unwrap();
        assert_eq!(loaded.gist_id, config.gist_id);
        assert_eq!(loaded.github_username, config.github_username);

        // Test remove
        assert!(manager.remove().is_ok());
        assert!(!manager.exists());
    }

    #[test]
    fn test_config_backup_restore() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test-config.json");
        
        let manager = GistConfigManager {
            config_path: config_path.clone(),
        };

        let config = GistConfig::new(
            "test-gist-id".to_string(),
            "testuser".to_string(),
            "a".repeat(64),
        );

        // Save config
        manager.save(&config).unwrap();

        // Create backup
        let backup_path = manager.backup().unwrap().unwrap();
        assert!(backup_path.exists());

        // Remove original
        manager.remove().unwrap();

        // Restore from backup
        manager.restore_from_backup().unwrap();
        assert!(manager.exists());

        // Verify content
        let restored = manager.load().unwrap().unwrap();
        assert_eq!(restored.gist_id, config.gist_id);
    }
}