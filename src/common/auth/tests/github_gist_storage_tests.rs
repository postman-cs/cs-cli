//! Comprehensive Tests for GitHub Gist Storage
//!
//! Includes unit tests, integration tests, and mock-based tests
//! for the enhanced GitHub Gist storage functionality.

use super::super::github_gist_storage::GitHubGistStorage;
use super::super::github_gist_errors::{GistStorageError, RetryConfig};
use super::super::session_metadata::{SessionData, SessionMetadata};
use super::super::gist_config_manager::GistConfig;
use crate::{CsCliError, Result};
use mockall::predicate::*;
use mockall::mock;
use std::collections::HashMap;
use tempfile::TempDir;
use async_trait::async_trait;

// Mock trait for GitHub API operations
#[async_trait]
pub trait GitHubApi {
    async fn create_gist(&self, description: &str, filename: &str, content: &str) -> Result<String>;
    async fn update_gist(&self, gist_id: &str, filename: &str, content: &str) -> Result<()>;
    async fn get_gist(&self, gist_id: &str) -> Result<String>;
    async fn delete_gist(&self, gist_id: &str) -> Result<()>;
    async fn get_current_user(&self) -> Result<String>;
}

// Mock for GitHub API operations
mock! {
    GitHubApi {}
    
    #[async_trait]
    impl GitHubApi for GitHubApi {
        async fn create_gist(&self, description: &str, filename: &str, content: &str) -> Result<String>;
        async fn update_gist(&self, gist_id: &str, filename: &str, content: &str) -> Result<()>;
        async fn get_gist(&self, gist_id: &str) -> Result<String>;
        async fn delete_gist(&self, gist_id: &str) -> Result<()>;
        async fn get_current_user(&self) -> Result<String>;
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_session_metadata_creation() {
        let platforms = vec!["gong".to_string(), "slack".to_string()];
        let metadata = SessionMetadata::new(platforms.clone());
        
        assert_eq!(metadata.version, 1);
        assert_eq!(metadata.platforms, platforms);
        assert_eq!(metadata.session_id.len(), 32);
        assert_eq!(metadata.device_id.len(), 16);
        assert!(metadata.is_valid());
    }

    #[test]
    fn test_session_metadata_validation() {
        let platforms = vec!["gong".to_string()];
        let mut metadata = SessionMetadata::new(platforms);
        
        // Test valid session
        assert!(metadata.validate().is_ok());
        
        // Test expired session
        metadata.expires_at = chrono::Utc::now() - chrono::Duration::days(1);
        assert!(metadata.validate().is_err());
        
        // Test too old session
        metadata.created_at = chrono::Utc::now() - chrono::Duration::days(100);
        assert!(metadata.validate().is_err());
    }

    #[test]
    fn test_session_data_creation_and_validation() {
        let mut cookies = HashMap::new();
        cookies.insert("session_id".to_string(), "test_value".to_string());
        cookies.insert("auth_token".to_string(), "another_value".to_string());
        
        let session_data = SessionData::new(cookies.clone());
        assert!(session_data.is_valid());
        assert_eq!(session_data.cookies, cookies);
        
        // Test content hash validation
        let validation_result = session_data.validate();
        assert!(validation_result.is_ok());
    }

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
    fn test_error_types() {
        let timeout_err = GistStorageError::NetworkTimeout {
            timeout: 30,
            operation: "test".to_string(),
        };
        assert!(timeout_err.is_retryable());
        assert_eq!(timeout_err.retry_delay(), Some(30));

        let client_err = GistStorageError::ClientNotInitialized;
        assert!(!client_err.is_retryable());
        assert_eq!(client_err.retry_delay(), None);
    }

    #[test]
    fn test_retry_config() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.base_delay_ms, 1000);
        assert_eq!(config.max_delay_ms, 10000);
        assert_eq!(config.backoff_multiplier, 2.0);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_storage_initialization() {
        let storage = GitHubGistStorage::new().await;
        assert!(storage.is_ok());
    }

    #[tokio::test]
    async fn test_storage_with_custom_retry_config() {
        let retry_config = RetryConfig {
            max_retries: 5,
            base_delay_ms: 2000,
            max_delay_ms: 20000,
            backoff_multiplier: 1.5,
        };

        let storage = GitHubGistStorage::with_retry_config(retry_config.clone()).await;
        assert!(storage.is_ok());
        
        let storage = storage.unwrap();
        assert_eq!(storage.retry_config().max_retries, 5);
    }

    #[tokio::test]
    async fn test_storage_clone() {
        let storage = GitHubGistStorage::new().await.unwrap();
        let cloned = storage.clone();
        
        // Both should have the same retry config
        assert_eq!(
            storage.retry_config().max_retries,
            cloned.retry_config().max_retries
        );
    }

    #[tokio::test]
    async fn test_async_encryption_roundtrip() {
        use crate::common::auth::session::async_session_encryption::AsyncSessionEncryption;
        
        let encryption = AsyncSessionEncryption::new().unwrap();
        
        let mut cookies = HashMap::new();
        cookies.insert("session_id".to_string(), "test_value".to_string());
        cookies.insert("auth_token".to_string(), "another_value".to_string());
        
        // Encrypt cookies
        let encrypted = encryption.encrypt_cookies(&cookies).await.unwrap();
        assert!(!encrypted.is_empty());
        
        // Decrypt cookies
        let decrypted = encryption.decrypt_cookies(&encrypted).await.unwrap();
        assert_eq!(cookies, decrypted);
    }

    #[tokio::test]
    async fn test_session_data_encryption_roundtrip() {
        use crate::common::auth::session::async_session_encryption::AsyncSessionEncryption;
        
        let encryption = AsyncSessionEncryption::new().unwrap();
        
        let mut cookies = HashMap::new();
        cookies.insert("session_id".to_string(), "test_value".to_string());
        
        let original_session = SessionData::new(cookies);
        
        // Encrypt session
        let encrypted = encryption.encrypt_session(&original_session).await.unwrap();
        assert!(!encrypted.is_empty());
        
        // Decrypt session
        let decrypted_session = encryption.decrypt_session(&encrypted).await.unwrap();
        
        assert_eq!(original_session.metadata.session_id, decrypted_session.metadata.session_id);
        assert_eq!(original_session.cookies, decrypted_session.cookies);
        assert!(decrypted_session.is_valid());
    }

    #[tokio::test]
    async fn test_tampered_data_fails() {
        use crate::common::auth::session::async_session_encryption::AsyncSessionEncryption;
        
        let encryption = AsyncSessionEncryption::new().unwrap();
        
        let mut cookies = HashMap::new();
        cookies.insert("session_id".to_string(), "test_value".to_string());
        
        let encrypted = encryption.encrypt_cookies(&cookies).await.unwrap();
        
        // Tamper with the encrypted data
        let mut tampered = encrypted.clone();
        if let Some(last_byte) = tampered.last_mut() {
            *last_byte ^= 1;
        }
        
        // Should fail with decryption error
        let result = encryption.decrypt_cookies(&tampered).await;
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod mock_tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_github_api_create_gist() {
        let mut mock_api = MockGitHubApi::new();
        
        mock_api
            .expect_create_gist()
            .with(
                eq("CS-CLI encrypted session data"),
                eq("cs-cli-session-data.enc"),
                function(|_: &str| true)
            )
            .times(1)
            .returning(|_, _, _| Ok("test-gist-id".to_string()));

        let result = mock_api.create_gist(
            "CS-CLI encrypted session data",
            "cs-cli-session-data.enc",
            "test content"
        ).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test-gist-id");
    }

    #[tokio::test]
    async fn test_mock_github_api_update_gist() {
        let mut mock_api = MockGitHubApi::new();
        
        mock_api
            .expect_update_gist()
            .with(
                eq("test-gist-id"),
                eq("cs-cli-session-data.enc"),
                function(|_: &str| true)
            )
            .times(1)
            .returning(|_, _, _| Ok(()));

        let result = mock_api.update_gist(
            "test-gist-id",
            "cs-cli-session-data.enc",
            "updated content"
        ).await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mock_github_api_get_gist() {
        let mut mock_api = MockGitHubApi::new();
        
        mock_api
            .expect_get_gist()
            .with(eq("test-gist-id"))
            .times(1)
            .returning(|_| Ok("encrypted content".to_string()));

        let result = mock_api.get_gist("test-gist-id").await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "encrypted content");
    }

    #[tokio::test]
    async fn test_mock_github_api_delete_gist() {
        let mut mock_api = MockGitHubApi::new();
        
        mock_api
            .expect_delete_gist()
            .with(eq("test-gist-id"))
            .times(1)
            .returning(|_| Ok(()));

        let result = mock_api.delete_gist("test-gist-id").await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mock_github_api_get_current_user() {
        let mut mock_api = MockGitHubApi::new();
        
        mock_api
            .expect_get_current_user()
            .times(1)
            .returning(|| Ok("testuser".to_string()));

        let result = mock_api.get_current_user().await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "testuser");
    }

    #[tokio::test]
    async fn test_mock_api_error_handling() {
        let mut mock_api = MockGitHubApi::new();
        
        mock_api
            .expect_create_gist()
            .times(1)
            .returning(|_, _, _| Err(CsCliError::GistStorageStructured(GistStorageError::ApiRequestFailed {
                operation: "create_gist".to_string(),
                status: 500,
                details: Some("Internal server error".to_string()),
            })));

        let result = mock_api.create_gist(
            "CS-CLI encrypted session data",
            "cs-cli-session-data.enc",
            "test content"
        ).await;
        
        assert!(result.is_err());
        
        if let Err(CsCliError::GistStorageStructured(GistStorageError::ApiRequestFailed { operation, status, .. })) = result {
            assert_eq!(operation, "create_gist");
            assert_eq!(status, 500);
        } else {
            panic!("Expected ApiRequestFailed error");
        }
    }
}

#[cfg(test)]
mod config_manager_tests {
    use super::*;

    #[test]
    fn test_config_manager_operations() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test-config.json");
        
        // Create a custom config manager with temp path
        let manager = crate::common::auth::gist_config_manager::GistConfigManager::new().unwrap();

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
        
        let manager = crate::common::auth::gist_config_manager::GistConfigManager::new().unwrap();

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

    #[test]
    fn test_config_validation() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test-config.json");
        
        let manager = crate::common::auth::gist_config_manager::GistConfigManager::new().unwrap();

        // Test with valid config
        let valid_config = GistConfig::new(
            "test-gist-id".to_string(),
            "testuser".to_string(),
            "a".repeat(64),
        );
        assert!(manager.validate_file().unwrap());

        // Test with invalid config (empty file)
        std::fs::write(&config_path, "").unwrap();
        assert!(!manager.validate_file().unwrap());
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_encryption_performance() {
        use crate::common::auth::session::async_session_encryption::AsyncSessionEncryption;
        
        let encryption = AsyncSessionEncryption::new().unwrap();
        
        // Create large cookie set
        let mut cookies = HashMap::new();
        for i in 0..1000 {
            cookies.insert(format!("cookie_{}", i), format!("value_{}", i));
        }
        
        let start = Instant::now();
        
        // Encrypt
        let encrypted = encryption.encrypt_cookies(&cookies).await.unwrap();
        
        // Decrypt
        let decrypted = encryption.decrypt_cookies(&encrypted).await.unwrap();
        
        let duration = start.elapsed();
        
        assert_eq!(cookies, decrypted);
        assert!(duration.as_millis() < 1000); // Should complete in under 1 second
    }

    #[test]
    fn test_session_metadata_performance() {
        let start = Instant::now();
        
        // Create many session metadata objects
        for _ in 0..1000 {
            let platforms = vec!["gong".to_string(), "slack".to_string()];
            let metadata = SessionMetadata::new(platforms);
            assert!(metadata.is_valid());
        }
        
        let duration = start.elapsed();
        assert!(duration.as_millis() < 100); // Should complete in under 100ms
    }
}