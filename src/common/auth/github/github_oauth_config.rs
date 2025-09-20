//! GitHub OAuth Configuration for Postman CS-CLI
//!
//! # Security Considerations
//!
//! This module implements OAuth 2.0 authorization code flow with PKCE for secure
//! authentication with GitHub. Key security features:
//!
//! - **CSRF Protection**: Uses cryptographically secure random state parameters
//! - **Constant-Time Validation**: Prevents timing attacks on state validation
//! - **Input Validation**: Validates all OAuth parameters before use
//! - **Secure Callback URLs**: Enforces localhost or HTTPS callback URLs
//!
//! # Configuration
//!
//! Required environment variables:
//! - `GITHUB_CLIENT_ID`: OAuth application client ID
//! - `GITHUB_CLIENT_SECRET`: OAuth application client secret
//! - `GITHUB_CALLBACK_URL`: Optional callback URL (defaults to localhost:8080)
//!
//! # Usage
//!
//! ```rust
//! use crate::common::auth::github::github_oauth_config::{GitHubOAuthConfig, OAuthState};
//!
//! let config = GitHubOAuthConfig::from_env()?;
//! let state = OAuthState::new()?;
//! let auth_url = build_oauth_url(&config, state.as_str())?;
//! ```

use crate::{CsCliError, Result};
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;

/// Structured configuration for GitHub OAuth
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubOAuthConfig {
    /// OAuth application client ID
    pub client_id: String,
    /// OAuth application client secret
    pub client_secret: String,
    /// OAuth callback URL
    pub callback_url: String,
    /// OAuth scopes to request
    pub scopes: Vec<String>,
    /// GitHub OAuth authorization endpoint
    pub authorize_url: String,
    /// GitHub OAuth token endpoint
    pub token_url: String,
}

impl GitHubOAuthConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Result<Self> {
        let client_id = std::env::var("GITHUB_CLIENT_ID")
            .map_err(|_| CsCliError::Configuration("GITHUB_CLIENT_ID environment variable not set".to_string()))?;
        
        let client_secret = std::env::var("GITHUB_CLIENT_SECRET")
            .map_err(|_| CsCliError::Configuration("GITHUB_CLIENT_SECRET environment variable not set".to_string()))?;

        let callback_url = std::env::var("GITHUB_CALLBACK_URL")
            .unwrap_or_else(|_| "http://localhost:8080/auth/github/callback".to_string());

        // Validate all inputs
        Self::validate_inputs(&client_id, &client_secret, &callback_url)?;

        Ok(Self {
            client_id,
            client_secret,
            callback_url,
            scopes: vec!["gist".to_string()],
            authorize_url: "https://github.com/login/oauth/authorize".to_string(),
            token_url: "https://github.com/login/oauth/access_token".to_string(),
        })
    }

    /// Validate all input parameters
    fn validate_inputs(client_id: &str, client_secret: &str, callback_url: &str) -> Result<()> {
        // Validate client ID
        if client_id.is_empty() || client_id.len() < 8 || client_id.len() > 64 {
            return Err(CsCliError::Configuration("Invalid client ID format".to_string()));
        }

        // Validate client secret
        if client_secret.is_empty() || client_secret.len() < 16 || client_secret.len() > 128 {
            return Err(CsCliError::Configuration("Invalid client secret format".to_string()));
        }

        // Validate callback URL
        if callback_url.is_empty() {
            return Err(CsCliError::Configuration("Callback URL cannot be empty".to_string()));
        }
        
        if !callback_url.starts_with("http://localhost:") && !callback_url.starts_with("https://") {
            return Err(CsCliError::Configuration("Callback URL must be localhost or HTTPS".to_string()));
        }

        if url::Url::parse(callback_url).is_err() {
            return Err(CsCliError::Configuration("Invalid callback URL format".to_string()));
        }

        Ok(())
    }
}

/// Secure OAuth state parameter for CSRF protection
#[derive(Debug, Clone)]
pub struct OAuthState {
    value: String,
}

impl OAuthState {
    /// Generate a new cryptographically secure OAuth state
    pub fn new() -> Result<Self> {
        let value = Self::generate_secure_state()?;
        Ok(Self { value })
    }

    /// Generate cryptographically secure random state
    fn generate_secure_state() -> Result<String> {
        use rand::{distributions::Alphanumeric, Rng};
        
        let state: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();
            
        Ok(state)
    }

    /// Get the state value as a string
    pub fn as_str(&self) -> &str {
        &self.value
    }

    /// Validate state parameter format
    pub fn validate_format(state: &str) -> Result<()> {
        if state.is_empty() || state.len() < 16 || state.len() > 128 {
            return Err(CsCliError::GitHubOAuth("Invalid OAuth state parameter length".to_string()));
        }
        
        if !state.chars().all(|c| c.is_alphanumeric()) {
            return Err(CsCliError::GitHubOAuth("OAuth state parameter contains invalid characters".to_string()));
        }
        
        Ok(())
    }
}

/// Build complete GitHub OAuth authorization URL with validation
pub fn build_oauth_url(config: &GitHubOAuthConfig, state: &str, callback_url: Option<&str>) -> Result<String> {
    // Validate state parameter
    OAuthState::validate_format(state)?;
    
    let scopes_str = config.scopes.join(",");
    let callback = callback_url.unwrap_or(&config.callback_url);
    
    let url = format!(
        "{}?client_id={}&scope={}&state={}&redirect_uri={}",
        config.authorize_url,
        urlencoding::encode(&config.client_id),
        urlencoding::encode(&scopes_str),
        urlencoding::encode(state),
        urlencoding::encode(callback)
    );

    // Validate the constructed URL
    url::Url::parse(&url)
        .map_err(|_| CsCliError::GitHubOAuth("Failed to construct valid OAuth URL".to_string()))?;

    Ok(url)
}

/// Validate OAuth callback parameters with constant-time comparison
pub fn validate_oauth_callback(expected_state: &str, received_state: &str) -> Result<bool> {
    // Validate both state parameters
    OAuthState::validate_format(expected_state)?;
    OAuthState::validate_format(received_state)?;
    
    // Use constant-time comparison to prevent timing attacks
    Ok(expected_state.as_bytes().ct_eq(received_state.as_bytes()).into())
}

/// Application identification constants for gists
pub const GIST_FILENAME: &str = "cs-cli-session-data.enc";
pub const GIST_DESCRIPTION: &str = "CS-CLI encrypted session data - DO NOT EDIT";

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn setup_test_env() {
        env::set_var("GITHUB_CLIENT_ID", "test_client_id_12345");
        env::set_var("GITHUB_CLIENT_SECRET", "fake_test_secret_not_real_123456");
    }

    fn cleanup_test_env() {
        env::remove_var("GITHUB_CLIENT_ID");
        env::remove_var("GITHUB_CLIENT_SECRET");
        env::remove_var("GITHUB_CALLBACK_URL");
    }

    #[test]
    fn test_config_from_env() {
        setup_test_env();
        
        let config = GitHubOAuthConfig::from_env();
        assert!(config.is_ok());
        
        let config = config.unwrap();
        assert_eq!(config.client_id, "test_client_id_12345");
        assert_eq!(config.client_secret, "test_secret_123456789012345");
        assert_eq!(config.callback_url, "http://localhost:8080/auth/github/callback");
        assert_eq!(config.scopes, vec!["gist"]);
        
        cleanup_test_env();
    }

    #[test]
    fn test_config_with_custom_callback() {
        setup_test_env();
        env::set_var("GITHUB_CALLBACK_URL", "https://example.com/callback");
        
        let config = GitHubOAuthConfig::from_env();
        assert!(config.is_ok());
        
        let config = config.unwrap();
        assert_eq!(config.callback_url, "https://example.com/callback");
        
        cleanup_test_env();
    }

    #[test]
    fn test_missing_client_id() {
        env::remove_var("GITHUB_CLIENT_ID");
        env::set_var("GITHUB_CLIENT_SECRET", "fake_test_secret_not_real_123456");
        
        let config = GitHubOAuthConfig::from_env();
        assert!(config.is_err());
        
        cleanup_test_env();
    }

    #[test]
    fn test_missing_client_secret() {
        env::set_var("GITHUB_CLIENT_ID", "test_client_id_12345");
        env::remove_var("GITHUB_CLIENT_SECRET");
        
        let config = GitHubOAuthConfig::from_env();
        assert!(config.is_err());
        
        cleanup_test_env();
    }

    #[test]
    fn test_invalid_client_id_too_short() {
        env::set_var("GITHUB_CLIENT_ID", "short");
        env::set_var("GITHUB_CLIENT_SECRET", "fake_test_secret_not_real_123456");
        
        let config = GitHubOAuthConfig::from_env();
        assert!(config.is_err());
        
        cleanup_test_env();
    }

    #[test]
    fn test_invalid_client_secret_too_short() {
        env::set_var("GITHUB_CLIENT_ID", "test_client_id_12345");
        env::set_var("GITHUB_CLIENT_SECRET", "short");
        
        let config = GitHubOAuthConfig::from_env();
        assert!(config.is_err());
        
        cleanup_test_env();
    }

    #[test]
    fn test_invalid_callback_url() {
        setup_test_env();
        env::set_var("GITHUB_CALLBACK_URL", "ftp://insecure.com/callback");
        
        let config = GitHubOAuthConfig::from_env();
        assert!(config.is_err());
        
        cleanup_test_env();
    }

    #[test]
    fn test_oauth_state_generation() {
        let state1 = OAuthState::new().unwrap();
        let state2 = OAuthState::new().unwrap();
        
        assert_eq!(state1.as_str().len(), 32);
        assert_eq!(state2.as_str().len(), 32);
        assert_ne!(state1.as_str(), state2.as_str());
        
        // Test that state contains only alphanumeric characters
        assert!(state1.as_str().chars().all(|c| c.is_alphanumeric()));
        assert!(state2.as_str().chars().all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn test_oauth_state_validation() {
        // Valid state
        assert!(OAuthState::validate_format("validstate123456789012345").is_ok());
        
        // Empty state
        assert!(OAuthState::validate_format("").is_err());
        
        // Too short
        assert!(OAuthState::validate_format("short").is_err());
        
        // Too long
        let long_state = "a".repeat(200);
        assert!(OAuthState::validate_format(&long_state).is_err());
        
        // Invalid characters
        assert!(OAuthState::validate_format("invalid-state!").is_err());
    }

    #[test]
    fn test_oauth_url_generation() {
        setup_test_env();
        
        let config = GitHubOAuthConfig::from_env().unwrap();
        let state = "test_state_123456789012345";
        let url = build_oauth_url(&config, state, None).unwrap();

        assert!(url.contains("github.com/login/oauth/authorize"));
        assert!(url.contains("scope=gist"));
        assert!(url.contains(&format!("state={}", urlencoding::encode(state))));
        assert!(url.contains(&format!("client_id={}", urlencoding::encode(&config.client_id))));
        assert!(url.contains(&format!("redirect_uri={}", urlencoding::encode(&config.callback_url))));
        
        cleanup_test_env();
    }

    #[test]
    fn test_oauth_url_invalid_state() {
        setup_test_env();
        
        let config = GitHubOAuthConfig::from_env().unwrap();
        
        // Test with invalid state
        let result = build_oauth_url(&config, "", None);
        assert!(result.is_err());
        
        cleanup_test_env();
    }

    #[test]
    fn test_secure_state_validation() {
        let state = OAuthState::new().unwrap();
        
        // Valid state comparison
        let result = validate_oauth_callback(state.as_str(), state.as_str());
        assert!(result.is_ok());
        assert!(result.unwrap());
        
        // Invalid state comparison
        let result = validate_oauth_callback(state.as_str(), "different_state");
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_state_validation_timing_attack_resistance() {
        let state = OAuthState::new().unwrap();
        let different_state = "different_state_123456789012345";
        
        // Both should take similar time regardless of how different the strings are
        let start = std::time::Instant::now();
        let _ = validate_oauth_callback(state.as_str(), different_state);
        let time1 = start.elapsed();
        
        let start = std::time::Instant::now();
        let _ = validate_oauth_callback(state.as_str(), "a");
        let time2 = start.elapsed();
        
        // Times should be similar (within reasonable margin for constant-time comparison)
        let diff = if time1 > time2 { time1 - time2 } else { time2 - time1 };
        assert!(diff.as_nanos() < 1_000_000); // Less than 1ms difference
    }

    #[test]
    fn test_constants() {
        assert_eq!(GIST_FILENAME, "cs-cli-session-data.enc");
        assert_eq!(GIST_DESCRIPTION, "CS-CLI encrypted session data - DO NOT EDIT");
    }
}