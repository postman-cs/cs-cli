//! Enhanced GitHub Gist Storage Backend for Session Data
//!
//! Refactored version with improved error handling, async operations,
//! and separated concerns for better maintainability and testability.

use crate::common::auth::session::async_session_encryption::AsyncSessionEncryption;
use super::github_authenticator::GitHubAuthenticator;
use super::github_gist_errors::{GistStorageError, GistResult, RetryConfig, with_retry};
use super::github_oauth_config::{GIST_DESCRIPTION, GIST_FILENAME};
use super::gist_config_manager::{GistConfig, GistConfigManager};
use crate::common::auth::session::session_metadata::SessionData;
use crate::{CsCliError, Result};
use octocrab::Octocrab;
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Enhanced GitHub Gist storage with improved architecture
pub struct GitHubGistStorage {
    authenticator: GitHubAuthenticator,
    encryption: AsyncSessionEncryption,
    config_manager: GistConfigManager,
    retry_config: RetryConfig,
}

impl GitHubGistStorage {
    /// Initialize gist storage with enhanced error handling
    pub async fn new() -> Result<Self> {
        let authenticator = GitHubAuthenticator::new();
        let encryption = AsyncSessionEncryption::new()
            .map_err(|e| GistStorageError::EncryptionFailed {
                reason: e.to_string(),
            })?;
        let config_manager = GistConfigManager::new()
            .map_err(|e| GistStorageError::ConfigError {
                field: "config_manager".to_string(),
                reason: e.to_string(),
            })?;

        let mut storage = Self {
            authenticator,
            encryption,
            config_manager,
            retry_config: RetryConfig::default(),
        };

        // Try to initialize from existing configuration
        storage.try_initialize_from_config().await?;

        Ok(storage)
    }

    /// Create with custom retry configuration
    pub async fn with_retry_config(retry_config: RetryConfig) -> Result<Self> {
        let mut storage = Self::new().await?;
        storage.retry_config = retry_config;
        Ok(storage)
    }

    /// Try to initialize from existing configuration
    async fn try_initialize_from_config(&mut self) -> Result<()> {
        if let Some(config) = self.config_manager.load()
            .map_err(|e| GistStorageError::ConfigError {
                field: "load_config".to_string(),
                reason: e.to_string(),
            })? {
            
            info!("Found existing gist configuration for user: {}", config.github_username);
            
            // Try to create GitHub client from stored token
            if self.authenticator.try_from_stored_token().await.is_ok() {
                info!("Successfully initialized from existing configuration");
            } else {
                warn!("Stored GitHub token expired, will re-authenticate if needed");
            }
        } else {
            debug!("No existing configuration found");
        }
        
        Ok(())
    }

    /// Store session cookies with enhanced error handling and retry logic
    pub async fn store_cookies(&mut self, cookies: &HashMap<String, String>) -> Result<()> {
        info!("Storing {} cookies to GitHub gist", cookies.len());

        // Perform the operation with retry logic
        for attempt in 1..=self.retry_config.max_retries {
            match self.try_store_operation(cookies).await {
                Ok(_) => {
                    info!("Successfully stored session data to GitHub gist");
                    return Ok(());
                }
                Err(e) if attempt < self.retry_config.max_retries && e.is_retryable() => {
                    let delay = e.retry_delay().unwrap_or(5);
                    tokio::time::sleep(tokio::time::Duration::from_secs(delay)).await;
                    continue;
                }
                Err(e) => return Err(CsCliError::GistStorageStructured(e)),
            }
        }
        
        unreachable!("Loop should have returned or errored")
    }

    /// Try the store operation once
    async fn try_store_operation(&mut self, cookies: &HashMap<String, String>) -> GistResult<()> {
        let client = self.authenticator.ensure_authenticated().await
            .map_err(|e| GistStorageError::AuthenticationRequired { reason: e.to_string() })?;
        let session_data = SessionData::new(cookies.clone());
        let encrypted_data = self.encryption.encrypt_session(&session_data).await?;
        
        let encoded_data = tokio::task::spawn_blocking(move || {
            base64_simd::STANDARD.encode_to_string(&encrypted_data)
        }).await.map_err(|e| GistStorageError::SerializationFailed {
            reason: format!("Base64 encoding failed: {}", e),
        })?;

        self.store_to_gist(client.as_ref(), &encoded_data).await
    }

    /// Retrieve session cookies with validation
    pub async fn get_cookies(&self) -> Result<HashMap<String, String>> {
        let operation = || async {
            // Ensure GitHub client is authenticated
            let client = self.authenticator.ensure_authenticated().await?;

            // Get configuration
            let config = self.config_manager.load()?
                .ok_or_else(|| GistStorageError::ConfigError {
                    field: "gist_config".to_string(),
                    reason: "Gist configuration not found".to_string(),
                })?;

            // Fetch gist content
            let gist_content = self.fetch_gist_content(client.as_ref(), &config.gist_id).await?;

            // Decode and decrypt
            let encrypted_data = tokio::task::spawn_blocking(move || {
                base64_simd::STANDARD.decode_to_vec(&gist_content)
            }).await.map_err(|e| GistStorageError::SerializationFailed {
                reason: format!("Base64 decoding failed: {}", e),
            })?
            .map_err(|_| GistStorageError::InvalidSessionData {
                reason: "Invalid base64 in gist content".to_string(),
            })?;

            // Decrypt session data
            let session_data = self.encryption.decrypt_session(&encrypted_data).await?;

            debug!(
                "Retrieved session data for platforms: {:?}",
                session_data.metadata.platforms
            );

            Ok(session_data.cookies)
        };

        with_retry(operation, self.retry_config.clone()).await
            .map_err(|e| GistStorageError::from(e).into())
    }

    /// Check if gist storage has valid session data
    pub async fn has_cookies(&self) -> bool {
        match self.get_cookies().await {
            Ok(cookies) => !cookies.is_empty(),
            Err(e) => {
                debug!("No valid cookies found: {}", e);
                false
            }
        }
    }

    /// Delete session data from gist (preserves GitHub authentication)
    pub async fn delete_cookies(&mut self) -> Result<()> {
        let operation = || async {
            // Ensure GitHub client is authenticated
            let client = self.authenticator.ensure_authenticated().await?;

            // Get configuration
            if let Some(config) = self.config_manager.load()? {
                // Delete the gist
                client.gists()
                    .delete(&config.gist_id)
                    .await?;

                info!("Successfully deleted gist: {}", config.gist_id);
            }

            // Remove local configuration
            self.config_manager.remove()?;

            // Note: We do NOT clear GitHub authentication here because:
            // 1. The GitHub token is used for gist storage, not platform authentication
            // 2. Clearing it would force unnecessary OAuth flows
            // 3. The token should persist across session clearing operations

            info!("Session data and configuration cleared");
            Ok(())
        };

        with_retry(operation, self.retry_config.clone()).await
            .map_err(|e| GistStorageError::from(e).into())
    }

    /// Delete session data from gist AND clear GitHub authentication (full reset)
    pub async fn delete_all_authentication(&mut self) -> Result<()> {
        let operation = || async {
            // Ensure GitHub client is authenticated
            let client = self.authenticator.ensure_authenticated().await?;

            // Get configuration
            if let Some(config) = self.config_manager.load()? {
                // Delete the gist
                client.gists()
                    .delete(&config.gist_id)
                    .await?;

                info!("Successfully deleted gist: {}", config.gist_id);
            }

            // Remove local configuration
            self.config_manager.remove()?;

            // Clear GitHub authentication (for full reset scenarios)
            self.authenticator.clear_authentication().await?;

            info!("All authentication data cleared (including GitHub)");
            Ok(())
        };

        with_retry(operation, self.retry_config.clone()).await
            .map_err(|e| GistStorageError::from(e).into())
    }

    /// Store data to GitHub gist (create or update)
    async fn store_to_gist(&mut self, client: &Octocrab, content: &str) -> GistResult<()> {
        if let Some(config) = self.config_manager.load()? {
            // Update existing gist
            self.update_existing_gist(client, &config.gist_id, content).await?;
            
            // Update config with new sync time
            let mut updated_config = config;
            updated_config.update_sync_time();
            self.config_manager.save(&updated_config)?;
            
            info!("Updated existing gist: {}", updated_config.gist_id);
        } else {
            // Create new gist
            let gist_id = self.create_new_gist(client, content).await?;
            self.save_gist_config(client, &gist_id).await?;
            
            info!("Created new gist: {}", gist_id);
        }

        Ok(())
    }

    /// Create new private gist for session storage
    async fn create_new_gist(&self, client: &Octocrab, content: &str) -> GistResult<String> {
        debug!("Creating new GitHub gist for session storage");
        debug!("Content length: {} bytes", content.len());

        let gist = client
            .gists()
            .create()
            .description(GIST_DESCRIPTION)
            .public(false) // Private gist
            .file(GIST_FILENAME, content)
            .send()
            .await?;

        debug!("Successfully created gist with ID: {}", gist.id);
        Ok(gist.id)
    }

    /// Update existing gist with new content
    async fn update_existing_gist(
        &self,
        client: &Octocrab,
        gist_id: &str,
        content: &str,
    ) -> GistResult<()> {
        client
            .gists()
            .update(gist_id)
            .file(GIST_FILENAME)
            .with_content(content)
            .send()
            .await?;

        Ok(())
    }

    /// Fetch gist content
    async fn fetch_gist_content(&self, client: &Octocrab, gist_id: &str) -> GistResult<String> {
        let gist = client
            .gists()
            .get(gist_id)
            .await?;

        let file_content = gist
            .files
            .get(GIST_FILENAME)
            .and_then(|file| file.content.as_ref())
            .ok_or_else(|| GistStorageError::GistNotFound {
                gist_id: gist_id.to_string(),
            })?;

        Ok(file_content.trim().to_string())
    }

    /// Save gist configuration locally
    async fn save_gist_config(&mut self, client: &Octocrab, gist_id: &str) -> GistResult<()> {
        // Get GitHub user info for configuration
        let user = client.current().user().await?;

        // Get token hash for validation
        let token_hash = self.get_token_hash().await?;

        let user_login = user.login.clone();
        let config = GistConfig::new(
            gist_id.to_string(),
            user_login.clone(),
            token_hash,
        );

        self.config_manager.save(&config)?;
        info!("Saved gist configuration for user: {}", user_login);
        Ok(())
    }

    /// Get token hash for configuration
    async fn get_token_hash(&self) -> GistResult<String> {
        // This is a simplified approach - in production, you might want to
        // store the token hash during authentication
        use sha2::{Digest, Sha256};
        
        // For now, generate a placeholder hash
        // In a real implementation, you'd get the actual token hash
        let placeholder = "placeholder-token-hash";
        Ok(format!("{:x}", Sha256::digest(placeholder.as_bytes())))
    }

    /// Get current retry configuration
    pub fn retry_config(&self) -> &RetryConfig {
        &self.retry_config
    }

    /// Update retry configuration
    pub fn set_retry_config(&mut self, config: RetryConfig) {
        self.retry_config = config;
    }

    /// Get configuration manager for advanced operations
    pub fn config_manager(&self) -> &GistConfigManager {
        &self.config_manager
    }

    /// Get authenticator for advanced operations
    pub fn authenticator(&self) -> &GitHubAuthenticator {
        &self.authenticator
    }
}

impl Clone for GitHubGistStorage {
    fn clone(&self) -> Self {
        Self {
            authenticator: self.authenticator.clone(),
            encryption: self.encryption.clone(),
            config_manager: GistConfigManager::new().expect("Failed to clone config manager"),
            retry_config: self.retry_config.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

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

    #[test]
    fn test_session_data_creation() {
        let mut cookies = HashMap::new();
        cookies.insert("session_id".to_string(), "test_value".to_string());
        
        let session_data = SessionData::new(cookies.clone());
        assert!(session_data.is_valid());
        assert_eq!(session_data.cookies, cookies);
    }
}