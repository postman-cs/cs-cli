//! GitHub Client Connection Pool
//!
//! Provides connection pooling and HTTP client reuse for GitHub API calls
//! to improve performance and reduce connection overhead.

use super::github_gist_errors::{GistStorageError, GistResult};
use octocrab::{Octocrab, OctocrabBuilder};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Configuration for GitHub client pool
#[derive(Debug, Clone)]
pub struct GitHubClientPoolConfig {
    /// Maximum number of connections in the pool
    pub max_connections: usize,
    /// Connection timeout
    pub connect_timeout: Duration,
    /// Request timeout
    pub request_timeout: Duration,
    /// Keep-alive timeout
    pub keep_alive_timeout: Duration,
    /// Maximum idle connections
    pub max_idle_connections: usize,
    /// User agent string
    pub user_agent: String,
}

impl Default for GitHubClientPoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 10,
            connect_timeout: Duration::from_secs(30),
            request_timeout: Duration::from_secs(60),
            keep_alive_timeout: Duration::from_secs(90),
            max_idle_connections: 5,
            user_agent: "cs-cli/1.0.0".to_string(),
        }
    }
}

/// GitHub client pool with connection reuse
pub struct GitHubClientPool {
    config: GitHubClientPoolConfig,
    client: Arc<RwLock<Option<Octocrab>>>,
    token: Arc<RwLock<Option<String>>>,
}

impl GitHubClientPool {
    /// Create new client pool with default configuration
    pub fn new() -> Self {
        Self::with_config(GitHubClientPoolConfig::default())
    }

    /// Create new client pool with custom configuration
    pub fn with_config(config: GitHubClientPoolConfig) -> Self {
        Self {
            config,
            client: Arc::new(RwLock::new(None)),
            token: Arc::new(RwLock::new(None)),
        }
    }

    /// Get or create GitHub client with connection pooling
    pub async fn get_client(&self) -> GistResult<Arc<Octocrab>> {
        // Check if we already have a valid client
        {
            let client_guard = self.client.read().await;
            if let Some(ref client) = *client_guard {
                // Test the client to ensure it's still valid
                if self.test_client(client).await.is_ok() {
                    debug!("Reusing existing GitHub client from pool");
                    return Ok(Arc::new(client.clone()));
                } else {
                    warn!("Existing GitHub client is no longer valid, creating new one");
                }
            }
        }

        // Need to create a new client
        self.create_new_client().await
    }

    /// Set authentication token for the client pool
    pub async fn set_token(&self, token: String) -> GistResult<()> {
        let mut token_guard = self.token.write().await;
        *token_guard = Some(token.clone());

        // Create new client with the token
        self.create_client_with_token(token).await?;
        
        info!("Updated GitHub client pool with new token");
        Ok(())
    }

    /// Clear authentication and client
    pub async fn clear(&self) -> GistResult<()> {
        let mut client_guard = self.client.write().await;
        let mut token_guard = self.token.write().await;
        
        *client_guard = None;
        *token_guard = None;
        
        info!("Cleared GitHub client pool");
        Ok(())
    }

    /// Test if a client is still valid
    async fn test_client(&self, client: &Octocrab) -> GistResult<()> {
        // Try to get current user info as a simple test
        client.current().user().await
            .map_err(|e| GistStorageError::ApiRequestFailed {
                operation: "client_test".to_string(),
                status: 0,
                details: Some(e.to_string()),
            })?;
        
        Ok(())
    }

    /// Create new client with current token
    async fn create_new_client(&self) -> GistResult<Arc<Octocrab>> {
        let token_guard = self.token.read().await;
        let token = token_guard.as_ref()
            .ok_or_else(|| GistStorageError::AuthenticationRequired {
                reason: "No authentication token available".to_string(),
            })?;

        self.create_client_with_token(token.clone()).await
    }

    /// Create client with specific token
    async fn create_client_with_token(&self, token: String) -> GistResult<Arc<Octocrab>> {
        let client = self.build_client_with_token(token)?;
        
        // Test the client
        self.test_client(&client).await?;
        
        // Store the client in the pool
        let mut client_guard = self.client.write().await;
        *client_guard = Some(client.clone());
        
        debug!("Created new GitHub client and added to pool");
        Ok(Arc::new(client))
    }

    /// Build GitHub client with connection pooling configuration
    fn build_client_with_token(&self, token: String) -> GistResult<Octocrab> {
        // Create HTTP client with connection pooling
        let http_client = reqwest::Client::builder()
            .timeout(self.config.request_timeout)
            .connect_timeout(self.config.connect_timeout)
            .pool_max_idle_per_host(self.config.max_idle_connections)
            .pool_idle_timeout(self.config.keep_alive_timeout)
            .user_agent(&self.config.user_agent)
            .build()
            .map_err(|e| GistStorageError::ApiRequestFailed {
                operation: "http_client_creation".to_string(),
                status: 0,
                details: Some(e.to_string()),
            })?;

        // Create Octocrab client with the HTTP client
        let client = OctocrabBuilder::new()
            .personal_token(token)
            .http_client(http_client)
            .build()
            .map_err(|e| GistStorageError::ApiRequestFailed {
                operation: "octocrab_client_creation".to_string(),
                status: 0,
                details: Some(e.to_string()),
            })?;

        Ok(client)
    }

    /// Get current configuration
    pub fn config(&self) -> &GitHubClientPoolConfig {
        &self.config
    }

    /// Update configuration (requires new client creation)
    pub async fn update_config(&mut self, config: GitHubClientPoolConfig) -> GistResult<()> {
        self.config = config;
        
        // Clear existing client to force recreation with new config
        self.clear().await?;
        
        info!("Updated GitHub client pool configuration");
        Ok(())
    }

    /// Get pool statistics
    pub async fn get_stats(&self) -> GitHubClientPoolStats {
        let client_guard = self.client.read().await;
        let token_guard = self.token.read().await;
        
        GitHubClientPoolStats {
            has_client: client_guard.is_some(),
            has_token: token_guard.is_some(),
            max_connections: self.config.max_connections,
            max_idle_connections: self.config.max_idle_connections,
        }
    }
}

impl Clone for GitHubClientPool {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            client: Arc::clone(&self.client),
            token: Arc::clone(&self.token),
        }
    }
}

/// Statistics about the client pool
#[derive(Debug, Clone)]
pub struct GitHubClientPoolStats {
    pub has_client: bool,
    pub has_token: bool,
    pub max_connections: usize,
    pub max_idle_connections: usize,
}

/// Enhanced GitHub authenticator with connection pooling
pub struct PooledGitHubAuthenticator {
    pool: GitHubClientPool,
}

impl PooledGitHubAuthenticator {
    /// Create new pooled authenticator
    pub fn new() -> Self {
        Self {
            pool: GitHubClientPool::new(),
        }
    }

    /// Create with custom pool configuration
    pub fn with_pool_config(config: GitHubClientPoolConfig) -> Self {
        Self {
            pool: GitHubClientPool::with_config(config),
        }
    }

    /// Authenticate and get pooled client
    pub async fn authenticate(&self, token: String) -> GistResult<Arc<Octocrab>> {
        self.pool.set_token(token).await?;
        self.pool.get_client().await
    }

    /// Get client from pool (must be authenticated first)
    pub async fn get_client(&self) -> GistResult<Arc<Octocrab>> {
        self.pool.get_client().await
    }

    /// Clear authentication
    pub async fn clear(&self) -> GistResult<()> {
        self.pool.clear().await
    }

    /// Get pool statistics
    pub async fn get_pool_stats(&self) -> GitHubClientPoolStats {
        self.pool.get_stats().await
    }

    /// Update pool configuration
    pub async fn update_pool_config(&mut self, config: GitHubClientPoolConfig) -> GistResult<()> {
        self.pool.update_config(config).await
    }
}

impl Clone for PooledGitHubAuthenticator {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_client_pool_creation() {
        let pool = GitHubClientPool::new();
        let stats = pool.get_stats().await;
        
        assert!(!stats.has_client);
        assert!(!stats.has_token);
        assert_eq!(stats.max_connections, 10);
    }

    #[tokio::test]
    async fn test_client_pool_with_custom_config() {
        let config = GitHubClientPoolConfig {
            max_connections: 20,
            connect_timeout: Duration::from_secs(60),
            request_timeout: Duration::from_secs(120),
            keep_alive_timeout: Duration::from_secs(180),
            max_idle_connections: 10,
            user_agent: "test-agent/1.0.0".to_string(),
        };

        let pool = GitHubClientPool::with_config(config.clone());
        let stats = pool.get_stats().await;
        
        assert_eq!(stats.max_connections, 20);
        assert_eq!(stats.max_idle_connections, 10);
    }

    #[tokio::test]
    async fn test_client_pool_token_management() {
        let pool = GitHubClientPool::new();
        
        // Set token
        let result = pool.set_token("test-token".to_string()).await;
        // This will fail because we can't actually authenticate with a fake token
        // but it should not panic
        assert!(result.is_err());
        
        let stats = pool.get_stats().await;
        assert!(stats.has_token);
    }

    #[tokio::test]
    async fn test_client_pool_clear() {
        let pool = GitHubClientPool::new();
        
        // Set token (will fail but should set internal state)
        let _ = pool.set_token("test-token".to_string()).await;
        
        // Clear
        let result = pool.clear().await;
        assert!(result.is_ok());
        
        let stats = pool.get_stats().await;
        assert!(!stats.has_client);
        assert!(!stats.has_token);
    }

    #[tokio::test]
    async fn test_pooled_authenticator() {
        let authenticator = PooledGitHubAuthenticator::new();
        let stats = authenticator.get_pool_stats().await;
        
        assert!(!stats.has_client);
        assert!(!stats.has_token);
    }

    #[tokio::test]
    async fn test_pooled_authenticator_with_config() {
        let config = GitHubClientPoolConfig {
            max_connections: 15,
            connect_timeout: Duration::from_secs(45),
            request_timeout: Duration::from_secs(90),
            keep_alive_timeout: Duration::from_secs(120),
            max_idle_connections: 8,
            user_agent: "test-authenticator/1.0.0".to_string(),
        };

        let authenticator = PooledGitHubAuthenticator::with_pool_config(config);
        let stats = authenticator.get_pool_stats().await;
        
        assert_eq!(stats.max_connections, 15);
        assert_eq!(stats.max_idle_connections, 8);
    }

    #[tokio::test]
    async fn test_pool_clone() {
        let pool1 = GitHubClientPool::new();
        let pool2 = pool1.clone();
        
        let stats1 = pool1.get_stats().await;
        let stats2 = pool2.get_stats().await;
        
        assert_eq!(stats1.max_connections, stats2.max_connections);
    }

    #[tokio::test]
    async fn test_authenticator_clone() {
        let auth1 = PooledGitHubAuthenticator::new();
        let auth2 = auth1.clone();
        
        let stats1 = auth1.get_pool_stats().await;
        let stats2 = auth2.get_pool_stats().await;
        
        assert_eq!(stats1.max_connections, stats2.max_connections);
    }
}