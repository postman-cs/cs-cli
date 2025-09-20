//! GitHub Gist Storage Backend for Session Data
//!
//! Manages encrypted session data storage in user's private GitHub gists.
//! Each user gets their own private gist for complete data isolation.

use super::github_oauth_config::{GIST_DESCRIPTION, GIST_FILENAME};
use super::github_oauth_flow::GitHubOAuthFlow;
use super::session_encryption::SessionEncryption;
use crate::{CsCliError, Result};
use octocrab::Octocrab;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::process::Command;
use tracing::{debug, error, info};

/// Session data metadata for gist storage
#[derive(Serialize, Deserialize, Debug)]
struct SessionMetadata {
    version: u32,
    created_at: String,
    updated_at: String,
    platforms: Vec<String>,
}

/// Configuration stored locally to track gist
#[derive(Serialize, Deserialize, Debug)]
struct GistConfig {
    gist_id: String,
    github_username: String,
    token_hash: String, // SHA-256 hash of token for validation
    last_sync: String,
}

pub struct GitHubGistStorage {
    github_client: Option<Octocrab>,
    encryption: SessionEncryption,
    config: Option<GistConfig>,
}

impl GitHubGistStorage {
    /// Initialize gist storage with OAuth authentication if needed
    pub async fn new() -> Result<Self> {
        let encryption = SessionEncryption::new()?;

        // Try to load existing configuration
        if let Ok(config) = Self::load_config() {
            // Try to create GitHub client with stored token
            if let Ok(github_client) = Self::create_github_client().await {
                info!("Using existing GitHub gist storage configuration");
                return Ok(Self {
                    github_client: Some(github_client),
                    encryption,
                    config: Some(config),
                });
            } else {
                info!("Stored GitHub token expired, will re-authenticate if user enables sync");
            }
        }

        // No config file, but check if we have a stored token from recent OAuth
        if let Ok(github_client) = Self::create_github_client().await {
            info!("Found GitHub token in keychain, initializing gist storage");
            return Ok(Self {
                github_client: Some(github_client),
                encryption,
                config: None, // Config will be created on first gist save
            });
        }

        // No valid configuration or token - will authenticate on first use
        Ok(Self {
            github_client: None,
            encryption,
            config: None,
        })
    }

    /// Store session cookies with GitHub gist sync
    pub async fn store_cookies(&mut self, cookies: &HashMap<String, String>) -> Result<()> {
        info!("GitHubGistStorage::store_cookies called with {} cookies", cookies.len());

        // Ensure GitHub client is available
        if self.github_client.is_none() {
            info!("GitHub client not initialized, setting up storage...");
            self.setup_github_storage().await?;
        } else {
            info!("GitHub client already initialized");
        }

        let client = self
            .github_client
            .as_ref()
            .ok_or_else(|| CsCliError::GistStorage("GitHub client not initialized".to_string()))?;

        // Prepare session data with metadata
        let metadata = SessionMetadata {
            version: 1,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            platforms: cookies.keys().cloned().collect(),
        };

        #[derive(Serialize)]
        struct SessionData {
            metadata: SessionMetadata,
            cookies: HashMap<String, String>,
        }

        let session_data = SessionData {
            metadata,
            cookies: cookies.clone(),
        };

        // Serialize and encrypt
        let json_data = serde_json::to_vec(&session_data)
            .map_err(|e| CsCliError::GistStorage(format!("Serialization failed: {e}")))?;

        let encrypted_data = self.encryption.encrypt(&json_data)?;
        let encoded_data = base64_simd::STANDARD.encode_to_string(&encrypted_data);

        // Update or create gist
        if let Some(config) = &self.config {
            self.update_existing_gist(client, &config.gist_id, &encoded_data)
                .await?;
            info!("Session data synced to existing gist");
        } else {
            let gist_id = self.create_new_gist(client, &encoded_data).await?;
            self.save_gist_config(client.clone(), &gist_id).await?;
            info!("Session data synced to new gist");
        }

        Ok(())
    }

    /// Retrieve session cookies from GitHub gist
    pub async fn get_cookies(&self) -> Result<HashMap<String, String>> {
        let client = self
            .github_client
            .as_ref()
            .ok_or_else(|| CsCliError::GistStorage("GitHub client not available".to_string()))?;

        let config = self
            .config
            .as_ref()
            .ok_or_else(|| CsCliError::GistStorage("Gist configuration not found".to_string()))?;

        // Fetch gist content
        let gist = client
            .gists()
            .get(&config.gist_id)
            .await
            .map_err(|e| CsCliError::GistStorage(format!("Failed to fetch gist: {e}")))?;

        // Extract encrypted content
        let file_content = gist
            .files
            .get(GIST_FILENAME)
            .and_then(|file| file.content.as_ref())
            .ok_or_else(|| CsCliError::GistStorage("Gist file content not found".to_string()))?;

        // Decode and decrypt
        let encrypted_data = base64_simd::STANDARD
            .decode_to_vec(file_content.trim())
            .map_err(|_| CsCliError::GistStorage("Invalid base64 in gist content".to_string()))?;

        let decrypted_data = self.encryption.decrypt(&encrypted_data)?;

        // Deserialize session data
        #[derive(Deserialize)]
        struct SessionData {
            metadata: SessionMetadata,
            cookies: HashMap<String, String>,
        }

        let session_data: SessionData = serde_json::from_slice(&decrypted_data).map_err(|e| {
            CsCliError::GistStorage(format!("Failed to deserialize session data: {e}"))
        })?;

        debug!(
            "Retrieved session data for platforms: {:?}",
            session_data.metadata.platforms
        );
        Ok(session_data.cookies)
    }

    /// Check if gist storage has session data
    pub async fn has_cookies(&self) -> bool {
        match self.get_cookies().await {
            Ok(cookies) => !cookies.is_empty(),
            Err(_) => false,
        }
    }

    /// Delete session data from gist
    pub async fn delete_cookies(&self) -> Result<()> {
        if let (Some(client), Some(config)) = (&self.github_client, &self.config) {
            client
                .gists()
                .delete(&config.gist_id)
                .await
                .map_err(|e| CsCliError::GistStorage(format!("Failed to delete gist: {e}")))?;

            // Remove local configuration
            Self::remove_config()?;
            info!("Session data gist deleted successfully");
        }
        Ok(())
    }

    /// Setup GitHub storage with OAuth authentication
    async fn setup_github_storage(&mut self) -> Result<()> {
        info!("Setting up GitHub gist storage...");

        // Perform OAuth authentication
        let mut oauth_flow = GitHubOAuthFlow::new();
        let access_token = oauth_flow.authenticate().await?;

        // Create GitHub client
        let github_client = Octocrab::builder()
            .personal_token(access_token.clone())
            .build()
            .map_err(|e| CsCliError::GistStorage(format!("Failed to create GitHub client: {e}")))?;

        // Store token securely in keychain
        Self::store_github_token(&access_token)?;

        self.github_client = Some(github_client);
        info!("GitHub gist storage setup completed");
        Ok(())
    }

    /// Create new private gist for session storage
    async fn create_new_gist(&self, client: &Octocrab, content: &str) -> Result<String> {
        info!("Creating new GitHub gist for session storage...");
        info!("Gist description: {}", GIST_DESCRIPTION);
        info!("Gist filename: {}", GIST_FILENAME);
        info!("Content length: {} bytes", content.len());

        let gist = client
            .gists()
            .create()
            .description(GIST_DESCRIPTION)
            .public(false) // Private gist
            .file(GIST_FILENAME, content)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to create gist: {}", e);
                CsCliError::GistStorage(format!("Failed to create gist: {e}"))
            })?;

        info!("Successfully created gist with ID: {}", gist.id);
        Ok(gist.id)
    }

    /// Update existing gist with new content
    async fn update_existing_gist(
        &self,
        client: &Octocrab,
        gist_id: &str,
        content: &str,
    ) -> Result<()> {
        // The octocrab API for updating gists requires using the file builder pattern
        client
            .gists()
            .update(gist_id)
            .file(GIST_FILENAME)
            .with_content(content)
            .send()
            .await
            .map_err(|e| CsCliError::GistStorage(format!("Failed to update gist: {e}")))?;

        Ok(())
    }

    /// Save gist configuration locally
    async fn save_gist_config(&mut self, client: Octocrab, gist_id: &str) -> Result<()> {
        // Get GitHub user info for configuration
        let user =
            client.current().user().await.map_err(|e| {
                CsCliError::GistStorage(format!("Failed to get GitHub user info: {e}"))
            })?;

        let token = Self::get_stored_github_token()?;
        let token_hash = format!("{:x}", Sha256::digest(token.as_bytes()));

        let config = GistConfig {
            gist_id: gist_id.to_string(),
            github_username: user.login,
            token_hash,
            last_sync: chrono::Utc::now().to_rfc3339(),
        };

        Self::save_config(&config)?;
        self.config = Some(config);
        Ok(())
    }

    /// Create GitHub client from stored token
    async fn create_github_client() -> Result<Octocrab> {
        let token = Self::get_stored_github_token()?;
        Octocrab::builder()
            .personal_token(token)
            .build()
            .map_err(|e| CsCliError::GistStorage(format!("Failed to create GitHub client: {e}")))
    }

    /// Store GitHub token in keychain
    fn store_github_token(token: &str) -> Result<()> {
        let output = Command::new("security")
            .args([
                "add-generic-password",
                "-s",
                "com.postman.cs-cli.github-token",
                "-a",
                "oauth-access-token",
                "-w",
                token,
                "-U", // Update if exists
            ])
            .output()
            .map_err(|e| CsCliError::GistStorage(format!("Failed to store GitHub token: {e}")))?;

        if output.status.success() {
            debug!("GitHub token stored securely in keychain");
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(CsCliError::GistStorage(format!(
                "Keychain storage failed: {stderr}"
            )))
        }
    }

    /// Retrieve GitHub token from keychain
    fn get_stored_github_token() -> Result<String> {
        let output = Command::new("security")
            .args([
                "find-generic-password",
                "-s",
                "com.postman.cs-cli.github-token",
                "-a",
                "oauth-access-token",
                "-w", // Return password only
            ])
            .output()
            .map_err(|e| CsCliError::GistStorage(format!("Failed to access keychain: {e}")))?;

        if output.status.success() {
            let token = String::from_utf8(output.stdout)
                .map_err(|_| CsCliError::GistStorage("Invalid token data in keychain".to_string()))?
                .trim()
                .to_string();
            Ok(token)
        } else {
            Err(CsCliError::GistStorage(
                "GitHub token not found in keychain".to_string(),
            ))
        }
    }

    /// Load gist configuration from local storage
    fn load_config() -> Result<GistConfig> {
        let config_path = Self::get_config_path()?;
        let config_data = std::fs::read_to_string(&config_path)
            .map_err(|_| CsCliError::GistStorage("Configuration file not found".to_string()))?;

        let config: GistConfig = serde_json::from_str(&config_data)
            .map_err(|e| CsCliError::GistStorage(format!("Invalid configuration format: {e}")))?;

        Ok(config)
    }

    /// Save gist configuration to local storage
    fn save_config(config: &GistConfig) -> Result<()> {
        let config_path = Self::get_config_path()?;

        // Create config directory if needed
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                CsCliError::GistStorage(format!("Failed to create config directory: {e}"))
            })?;
        }

        let config_data = serde_json::to_string_pretty(config)
            .map_err(|e| CsCliError::GistStorage(format!("Failed to serialize config: {e}")))?;

        std::fs::write(&config_path, config_data)
            .map_err(|e| CsCliError::GistStorage(format!("Failed to save config: {e}")))?;

        Ok(())
    }

    /// Remove gist configuration
    fn remove_config() -> Result<()> {
        let config_path = Self::get_config_path()?;
        if config_path.exists() {
            std::fs::remove_file(&config_path)
                .map_err(|e| CsCliError::GistStorage(format!("Failed to remove config: {e}")))?;
        }
        Ok(())
    }

    /// Get configuration file path
    fn get_config_path() -> Result<std::path::PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| {
                CsCliError::GistStorage("Unable to determine config directory".to_string())
            })?
            .join("cs-cli");

        Ok(config_dir.join("github-gist-config.json"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_metadata_serialization() {
        let metadata = SessionMetadata {
            version: 1,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
            platforms: vec!["gong".to_string(), "slack".to_string()],
        };

        let serialized = serde_json::to_string(&metadata).unwrap();
        let deserialized: SessionMetadata = serde_json::from_str(&serialized).unwrap();

        assert_eq!(metadata.version, deserialized.version);
        assert_eq!(metadata.platforms, deserialized.platforms);
    }

    #[test]
    fn test_gist_config_serialization() {
        let config = GistConfig {
            gist_id: "abc123def456".to_string(),
            github_username: "testuser".to_string(),
            token_hash: "sha256hash".to_string(),
            last_sync: "2024-01-01T00:00:00Z".to_string(),
        };

        let serialized = serde_json::to_string_pretty(&config).unwrap();
        let deserialized: GistConfig = serde_json::from_str(&serialized).unwrap();

        assert_eq!(config.gist_id, deserialized.gist_id);
        assert_eq!(config.github_username, deserialized.github_username);
    }

    #[tokio::test]
    async fn test_gist_storage_initialization() {
        // Test that GitHubGistStorage can be created without authentication
        let result = GitHubGistStorage::new().await;
        assert!(result.is_ok());

        let storage = result.unwrap();
        assert!(storage.github_client.is_none()); // Should start without client
    }
}
