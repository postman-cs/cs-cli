//! GitHub OAuth Flow Using Default Browser
//!
//! Handles GitHub OAuth for personal account access by opening the authorization
//! page in the user's default browser and listening for the callback.

use super::github_oauth_config::{GitHubOAuthConfig, OAuthState, build_oauth_url, validate_oauth_callback};
use crate::{CsCliError, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tracing::{debug, info};

pub struct GitHubOAuthFlow {
    oauth_state: OAuthState,
    config: GitHubOAuthConfig,
}

impl Default for GitHubOAuthFlow {
    fn default() -> Self {
        Self::new().expect("Failed to create GitHubOAuthFlow - check environment variables")
    }
}

impl GitHubOAuthFlow {
    /// Create new OAuth flow that opens the user's default browser
    pub fn new() -> Result<Self> {
        let config = GitHubOAuthConfig::from_env()?;
        let oauth_state = OAuthState::new()?;
        
        Ok(Self {
            oauth_state,
            config,
        })
    }

    /// Open URL in the user's default browser using platform-specific commands
    fn open_browser(&self, url: &str) -> Result<()> {
        use std::process::Command;

        let (cmd, args) = match std::env::consts::OS {
            "macos" => ("open", vec![url]),
            "linux" => ("xdg-open", vec![url]),
            "windows" => ("cmd", vec!["/c", "start", "", url]),
            _ => return Err(CsCliError::GitHubOAuth("Unsupported OS".to_string())),
        };

        Command::new(cmd)
            .args(&args)
            .spawn()
            .map_err(|e| CsCliError::GitHubOAuth(format!("Failed to open browser: {}", e)))?;

        Ok(())
    }

    /// Execute complete OAuth flow and return access token
    pub async fn authenticate(&mut self) -> Result<String> {
        info!("Starting GitHub OAuth authentication...");

        // Start local callback server
        let (listener, port) = self.start_callback_server().await?;

        // Update callback URL with the actual port
        let dynamic_callback_url = format!("http://localhost:{}/auth/github/callback", port);

        // Navigate to GitHub OAuth authorization page
        let oauth_url = build_oauth_url(&self.config, self.oauth_state.as_str(), Some(&dynamic_callback_url))?;
        info!("Opening GitHub authorization page in your browser...");
        self.open_browser(&oauth_url)?;

        // Wait for user to complete OAuth flow
        info!("Waiting for GitHub authorization...");
        let auth_code = self.handle_oauth_callback(listener).await?;

        // Exchange authorization code for access token
        info!("Exchanging authorization code for access token...");
        let access_token = self.exchange_code_for_token(&auth_code, &dynamic_callback_url).await?;

        info!("GitHub OAuth authentication completed successfully");
        Ok(access_token)
    }

    /// Start local callback server to receive OAuth response
    async fn start_callback_server(&self) -> Result<(TcpListener, u16)> {
        // Try multiple ports to avoid conflicts
        for port in 8080..8090 {
            if let Ok(listener) = TcpListener::bind(format!("127.0.0.1:{}", port)).await {
                debug!("OAuth callback server listening on http://localhost:{}", port);
                return Ok((listener, port));
            }
        }
        
        Err(CsCliError::GitHubOAuth("No available ports 8080-8089".to_string()))
    }

    /// Handle OAuth callback and extract authorization code
    async fn handle_oauth_callback(&self, listener: TcpListener) -> Result<String> {
        // Set reasonable timeout for OAuth flow (5 minutes)
        let timeout = Duration::from_secs(300);

        tokio::time::timeout(timeout, async {
            let (mut stream, _) = listener.accept().await?;

            // Read HTTP request
            let mut buffer = [0; 4096];
            let bytes_read = stream.read(&mut buffer).await?;
            let request = String::from_utf8_lossy(&buffer[..bytes_read]);

            // Send success response to browser
            let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<h1>Authorization Complete!</h1><p>You can close this window and return to cs-cli.</p>";

            stream.write_all(response.as_bytes()).await?;
            stream.flush().await?;

            // Extract authorization code from request
            self.extract_auth_code(&request)
        })
        .await
        .map_err(|_| {
            CsCliError::Authentication("OAuth flow timed out - please try again".to_string())
        })?
    }

    /// Parse authorization code from HTTP callback request
    fn extract_auth_code(&self, request: &str) -> Result<String> {
        debug!("Processing OAuth callback request");

        // Get the request line (first line)
        let request_line = request
            .lines()
            .next()
            .ok_or_else(|| CsCliError::Authentication("Invalid callback request".to_string()))?;

        // Extract URL path and query parameters
        let url_path = request_line
            .split_whitespace()
            .nth(1)
            .ok_or_else(|| CsCliError::Authentication("Invalid callback URL format".to_string()))?;

        // Parse query parameters
        if let Some(query_start) = url_path.find('?') {
            let query_string = &url_path[query_start + 1..];
            let params = self.parse_query_params(query_string);

            // Verify state parameter (CSRF protection)
            if let Some(received_state) = params.get("state") {
                if !validate_oauth_callback(self.oauth_state.as_str(), received_state)? {
                    return Err(CsCliError::Authentication(
                        "OAuth state mismatch - possible security issue".to_string(),
                    ));
                }
            } else {
                return Err(CsCliError::Authentication(
                    "Missing OAuth state parameter".to_string(),
                ));
            }

            // Check for authorization error
            if let Some(error) = params.get("error") {
                let error_desc = params
                    .get("error_description")
                    .map(|s| s.as_str())
                    .unwrap_or("No description provided");
                return Err(CsCliError::Authentication(format!(
                    "GitHub OAuth error: {error} - {error_desc}"
                )));
            }

            // Extract authorization code
            if let Some(code) = params.get("code") {
                debug!("Successfully extracted authorization code");
                return Ok(code.clone());
            }
        }

        Err(CsCliError::Authentication(
            "No authorization code found in callback".to_string(),
        ))
    }

    /// Parse URL query parameters into HashMap
    fn parse_query_params(&self, query: &str) -> HashMap<String, String> {
        query.split('&')
            .filter_map(|pair| {
                let (key, value) = pair.split_once('=')?;
                Some((key.to_string(), urlencoding::decode(value).ok()?.into_owned()))
            })
            .collect()
    }

    /// Exchange authorization code for GitHub access token
    async fn exchange_code_for_token(&self, auth_code: &str, callback_url: &str) -> Result<String> {
        #[derive(Deserialize)]
        struct TokenResponse {
            access_token: String,
            #[allow(dead_code)]
            token_type: String,
            scope: String,
        }

        let client = reqwest::Client::new();
        let response = client
            .post(&self.config.token_url)
            .header("Accept", "application/json")
            .header("User-Agent", "cs-cli/1.0.0")
            .form(&[
                ("client_id", &self.config.client_id),
                ("client_secret", &self.config.client_secret),
                ("code", &auth_code.to_string()),
                ("redirect_uri", &callback_url.to_string()),
            ])
            .send()
            .await
            .map_err(|e| {
                CsCliError::GitHubOAuth(format!("Failed to contact GitHub token endpoint: {e}"))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();
            return Err(CsCliError::Authentication(format!(
                "GitHub token exchange failed ({status}): {error_body}"
            )));
        }

        let token_data: TokenResponse = response.json().await.map_err(|e| {
            CsCliError::GitHubOAuth(format!("Failed to parse GitHub token response: {e}"))
        })?;

        debug!(
            "Successfully obtained GitHub access token with scope: {}",
            token_data.scope
        );
        Ok(token_data.access_token)
    }
}

/// Simple user prompt for enabling cross-device sync
pub fn prompt_user_for_sync() -> Result<bool> {
    use inquire::Confirm;

    let prompt = "Enable cross-device session sync?";
    let help = "Securely stores your login data in your personal GitHub account so you stay logged in across all devices. You'll only need to do this once.";

    Confirm::new(prompt)
        .with_default(true)
        .with_help_message(help)
        .prompt()
        .map_err(|e| CsCliError::GitHubOAuth(format!("Failed to get user input: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth_url_generation() {
        // Set up test environment
        std::env::set_var("GITHUB_CLIENT_ID", "test_client_id_12345");
        std::env::set_var("GITHUB_CLIENT_SECRET", "fake_test_secret_not_real_123456");
        
        let config = GitHubOAuthConfig::from_env().unwrap();
        let state = "test_state_123456789012345";
        let url = build_oauth_url(&config, state, None).unwrap();

        assert!(url.contains("client_id="));
        assert!(url.contains("scope=gist"));
        assert!(url.contains("state=test_state_123456789012345"));
        assert!(url.contains("redirect_uri="));
        
        // Clean up
        std::env::remove_var("GITHUB_CLIENT_ID");
        std::env::remove_var("GITHUB_CLIENT_SECRET");
    }

    #[test]
    fn test_query_param_parsing() {
        // Set up test environment
        std::env::set_var("GITHUB_CLIENT_ID", "test_client_id_12345");
        std::env::set_var("GITHUB_CLIENT_SECRET", "fake_test_secret_not_real_123456");
        
        let oauth_flow = GitHubOAuthFlow::new().unwrap();
        let query = "code=abc123&state=xyz789&scope=gist";
        let params = oauth_flow.parse_query_params(query);

        assert_eq!(params.get("code"), Some(&"abc123".to_string()));
        assert_eq!(params.get("state"), Some(&"xyz789".to_string()));
        assert_eq!(params.get("scope"), Some(&"gist".to_string()));
        
        // Clean up
        std::env::remove_var("GITHUB_CLIENT_ID");
        std::env::remove_var("GITHUB_CLIENT_SECRET");
    }

    #[test]
    fn test_auth_code_extraction_success() {
        // Set up test environment
        std::env::set_var("GITHUB_CLIENT_ID", "test_client_id_12345");
        std::env::set_var("GITHUB_CLIENT_SECRET", "fake_test_secret_not_real_123456");
        
        let oauth_flow = GitHubOAuthFlow::new().unwrap();
        let request = format!(
            "GET /auth/github/callback?code=test_code_123&state={} HTTP/1.1\r\nHost: localhost:8080\r\n\r\n",
            oauth_flow.oauth_state.as_str()
        );

        let result = oauth_flow.extract_auth_code(&request);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test_code_123");
        
        // Clean up
        std::env::remove_var("GITHUB_CLIENT_ID");
        std::env::remove_var("GITHUB_CLIENT_SECRET");
    }

    #[test]
    fn test_auth_code_extraction_state_mismatch() {
        // Set up test environment
        std::env::set_var("GITHUB_CLIENT_ID", "test_client_id_12345");
        std::env::set_var("GITHUB_CLIENT_SECRET", "fake_test_secret_not_real_123456");
        
        let oauth_flow = GitHubOAuthFlow::new().unwrap();
        let request = "GET /auth/github/callback?code=test_code_123&state=wrong_state HTTP/1.1\r\nHost: localhost:8080\r\n\r\n";

        let result = oauth_flow.extract_auth_code(request);
        assert!(result.is_err());
        
        // Clean up
        std::env::remove_var("GITHUB_CLIENT_ID");
        std::env::remove_var("GITHUB_CLIENT_SECRET");
    }

    #[test]
    fn test_auth_code_extraction_oauth_error() {
        // Set up test environment
        std::env::set_var("GITHUB_CLIENT_ID", "test_client_id_12345");
        std::env::set_var("GITHUB_CLIENT_SECRET", "fake_test_secret_not_real_123456");
        
        let oauth_flow = GitHubOAuthFlow::new().unwrap();
        let request = format!(
            "GET /auth/github/callback?error=access_denied&error_description=User%20denied%20access&state={} HTTP/1.1\r\nHost: localhost:8080\r\n\r\n",
            oauth_flow.oauth_state.as_str()
        );

        let result = oauth_flow.extract_auth_code(&request);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("access_denied"));
        
        // Clean up
        std::env::remove_var("GITHUB_CLIENT_ID");
        std::env::remove_var("GITHUB_CLIENT_SECRET");
    }
}
