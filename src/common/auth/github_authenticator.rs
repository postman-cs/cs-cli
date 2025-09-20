//! GitHub Authentication Manager
//!
//! Handles GitHub OAuth authentication and client management with
//! connection pooling and retry logic.

use super::github_oauth_flow::GitHubOAuthFlow;
use super::github_gist_errors::{GistStorageError, GistResult, RetryConfig, with_retry};
use octocrab::Octocrab;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// GitHub authentication manager with connection pooling
pub struct GitHubAuthenticator {
    client: Arc<RwLock<Option<Octocrab>>>,
    retry_config: RetryConfig,
}

impl GitHubAuthenticator {
    /// Create new GitHub authenticator
    pub fn new() -> Self {
        Self {
            client: Arc::new(RwLock::new(None)),
            retry_config: RetryConfig::default(),
        }
    }

    /// Create authenticator with custom retry configuration
    pub fn with_retry_config(retry_config: RetryConfig) -> Self {
        Self {
            client: Arc::new(RwLock::new(None)),
            retry_config,
        }
    }

    /// Ensure GitHub client is authenticated and ready
    pub async fn ensure_authenticated(&self) -> GistResult<Arc<Octocrab>> {
        // Check if we already have a client
        {
            let client_guard = self.client.read().await;
            if let Some(ref client) = *client_guard {
                // Test the client with a simple API call
                if self.test_client(client).await.is_ok() {
                    debug!("Using existing authenticated GitHub client");
                    return Ok(Arc::new(client.clone()));
                } else {
                    warn!("Existing GitHub client is no longer valid, re-authenticating");
                }
            }
        }

        // Need to authenticate
        info!("Authenticating with GitHub...");
        self.authenticate().await?;

        // Return the newly authenticated client
        let client_guard = self.client.read().await;
        Ok(Arc::new(client_guard.as_ref().unwrap().clone()))
    }

    /// Perform GitHub OAuth authentication
    async fn authenticate(&self) -> GistResult<()> {
        let operation = || async {
            let mut oauth_flow = GitHubOAuthFlow::new()
                .map_err(|e| GistStorageError::AuthenticationRequired {
                    reason: e.to_string(),
                })?;
            let access_token = oauth_flow.authenticate().await
                .map_err(|e| GistStorageError::AuthenticationRequired {
                    reason: e.to_string(),
                })?;

            // Create GitHub client
            let github_client = Octocrab::builder()
                .personal_token(access_token.clone())
                .build()
                .map_err(|e| GistStorageError::ApiRequestFailed {
                    operation: "GitHub client creation".to_string(),
                    status: 0,
                    details: Some(e.to_string()),
                })?;

            // Store token securely in keychain
            self.store_github_token(&access_token)?;

            // Store the client
            let mut client_guard = self.client.write().await;
            *client_guard = Some(github_client);

            info!("GitHub authentication completed successfully");
            Ok(())
        };

        with_retry(operation, self.retry_config.clone()).await
    }

    /// Test if a GitHub client is still valid
    async fn test_client(&self, client: &Octocrab) -> GistResult<()> {
        // Try to get current user info as a simple test
        client.current().user().await
            .map_err(|e| GistStorageError::ApiRequestFailed {
                operation: "GitHub client test".to_string(),
                status: 0,
                details: Some(e.to_string()),
            })?;
        
        Ok(())
    }

    /// Store GitHub token in keychain
    fn store_github_token(&self, token: &str) -> GistResult<()> {
        use std::process::Command;
        
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
            .map_err(|e| GistStorageError::ConfigError {
                field: "keychain".to_string(),
                reason: format!("Failed to store GitHub token: {}", e),
            })?;

        if output.status.success() {
            debug!("GitHub token stored securely in keychain");
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(GistStorageError::ConfigError {
                field: "keychain".to_string(),
                reason: format!("Keychain storage failed: {}", stderr),
            })
        }
    }

    /// Retrieve GitHub token from keychain
    fn get_stored_github_token(&self) -> GistResult<String> {
        use std::process::Command;
        
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
            .map_err(|e| GistStorageError::ConfigError {
                field: "keychain".to_string(),
                reason: format!("Failed to access keychain: {}", e),
            })?;

        if output.status.success() {
            let token = String::from_utf8(output.stdout)
                .map_err(|_| GistStorageError::ConfigError {
                    field: "keychain".to_string(),
                    reason: "Invalid token data in keychain".to_string(),
                })?
                .trim()
                .to_string();
            Ok(token)
        } else {
            Err(GistStorageError::AuthenticationRequired {
                reason: "GitHub token not found in keychain".to_string(),
            })
        }
    }

    /// Try to create client from stored token
    pub async fn try_from_stored_token(&self) -> GistResult<()> {
        let token = self.get_stored_github_token()?;
        
        let github_client = Octocrab::builder()
            .personal_token(token)
            .build()
            .map_err(|e| GistStorageError::ApiRequestFailed {
                operation: "GitHub client creation from stored token".to_string(),
                status: 0,
                details: Some(e.to_string()),
            })?;

        // Test the client
        self.test_client(&github_client).await?;

        // Store the client
        let mut client_guard = self.client.write().await;
        *client_guard = Some(github_client);

        info!("Successfully created GitHub client from stored token");
        Ok(())
    }

    /// Clear stored authentication
    pub async fn clear_authentication(&self) -> GistResult<()> {
        use std::process::Command;
        
        // Clear from memory
        let mut client_guard = self.client.write().await;
        *client_guard = None;

        // Clear from keychain
        let output = Command::new("security")
            .args([
                "delete-generic-password",
                "-s",
                "com.postman.cs-cli.github-token",
                "-a",
                "oauth-access-token",
            ])
            .output()
            .map_err(|e| GistStorageError::ConfigError {
                field: "keychain".to_string(),
                reason: format!("Failed to clear keychain: {}", e),
            })?;

        // Don't error if token doesn't exist (already clean)
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.contains("could not be found") {
                return Err(GistStorageError::ConfigError {
                    field: "keychain".to_string(),
                    reason: format!("Failed to clear keychain: {}", stderr),
                });
            }
        }

        info!("GitHub authentication cleared");
        Ok(())
    }

    /// Get current retry configuration
    pub fn retry_config(&self) -> &RetryConfig {
        &self.retry_config
    }

    /// Update retry configuration
    pub fn set_retry_config(&mut self, config: RetryConfig) {
        self.retry_config = config;
    }
}

impl Clone for GitHubAuthenticator {
    fn clone(&self) -> Self {
        Self {
            client: Arc::clone(&self.client),
            retry_config: self.retry_config.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_authenticator_creation() {
        let authenticator = GitHubAuthenticator::new();
        assert_eq!(authenticator.retry_config().max_retries, 3);
    }

    #[tokio::test]
    async fn test_authenticator_clone() {
        let authenticator = GitHubAuthenticator::new();
        let cloned = authenticator.clone();
        
        // Both should have the same retry config
        assert_eq!(
            authenticator.retry_config().max_retries,
            cloned.retry_config().max_retries
        );
    }

    #[test]
    fn test_retry_config_customization() {
        let custom_config = RetryConfig {
            max_retries: 5,
            base_delay_ms: 2000,
            max_delay_ms: 20000,
            backoff_multiplier: 1.5,
        };
        
        let authenticator = GitHubAuthenticator::with_retry_config(custom_config.clone());
        assert_eq!(authenticator.retry_config().max_retries, 5);
        assert_eq!(authenticator.retry_config().base_delay_ms, 2000);
    }
}