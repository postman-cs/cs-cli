use crate::gong::api::client::GongHttpClient;
use crate::common::config::AuthSettings;
use crate::{CsCliError, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use tracing::{debug, error, info};

/// CSRF token management for Gong API with production-grade async handling
///
/// Manages CSRF token lifecycle with:
/// - Async-safe token caching with tokio::sync::Mutex
/// - Configurable expiry and buffer time
/// - Exponential backoff retry logic
/// - Rich diagnostic error reporting
pub struct CSRFManager {
    /// HTTP client for token requests
    http_client: Arc<GongHttpClient>,

    /// Authentication configuration settings
    config: AuthSettings,

    /// Current CSRF token (protected by async mutex)
    token_state: Mutex<CSRFTokenState>,

    /// Maximum number of refresh attempts before giving up
    max_refresh_attempts: u32,
}

/// Internal state for CSRF token management
#[derive(Debug)]
struct CSRFTokenState {
    /// Current CSRF token value
    csrf_token: Option<String>,

    /// When the token expires (Unix timestamp)
    token_expires_at: f64,

    /// Number of consecutive refresh attempts
    refresh_attempts: u32,
}

impl CSRFManager {
    /// Create a new CSRF manager with production configuration
    pub fn new(config: AuthSettings, http_client: Arc<GongHttpClient>) -> Self {
        let max_refresh_attempts = 3; // From Python config: self.config.retry_attempts

        Self {
            http_client,
            config,
            token_state: Mutex::new(CSRFTokenState {
                csrf_token: None,
                token_expires_at: 0.0,
                refresh_attempts: 0,
            }),
            max_refresh_attempts,
        }
    }

    /// Get valid CSRF token, refreshing if needed
    ///
    /// # Arguments
    /// * `cell` - Gong cell identifier (e.g., "us-14496")
    /// * `force_refresh` - Force token refresh regardless of expiry
    ///
    /// # Returns
    /// Valid CSRF token or None if all refresh attempts fail
    pub async fn get_csrf_token(&self, cell: &str, force_refresh: bool) -> Result<Option<String>> {
        // Acquire async lock for thread-safe token management
        let mut state = self.token_state.lock().await;

        // Check if we need to refresh the token
        if force_refresh || !self.is_token_valid(&state) {
            // Drop the lock temporarily to allow _refresh_csrf_token to acquire it
            drop(state);
            self._refresh_csrf_token(cell).await?;

            // Re-acquire lock to return the token
            state = self.token_state.lock().await;
        }

        Ok(state.csrf_token.clone())
    }

    /// Check if current token is still valid with configurable buffer
    fn is_token_valid(&self, state: &CSRFTokenState) -> bool {
        if state.csrf_token.is_none() {
            return false;
        }

        let buffer_seconds = (self.config.csrf_token_buffer_minutes * 60) as f64;
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        current_time < (state.token_expires_at - buffer_seconds)
    }

    /// Invalidate current token to force refresh on next request
    pub async fn invalidate_token(&self) {
        let mut state = self.token_state.lock().await;
        state.csrf_token = None;
        state.token_expires_at = 0.0;

        debug!("CSRF token invalidated");
    }

    /// Fetch new CSRF token from Gong API with exponential backoff retry logic
    async fn _refresh_csrf_token(&self, cell: &str) -> Result<()> {
        let url = format!("https://{cell}.app.gong.io/ajax/common/rtkn");

        // TODO: Replace with actual HTTP client in Phase 2
        // For now, simulate the token refresh process

        for attempt in 0..self.max_refresh_attempts {
            match self.attempt_token_refresh(&url, cell, attempt).await {
                Ok(token) => {
                    let mut state = self.token_state.lock().await;
                    state.csrf_token = Some(token.clone());

                    // Token expires based on configuration
                    let token_ttl_seconds = (self.config.csrf_token_ttl_minutes * 60) as f64;
                    state.token_expires_at = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs_f64()
                        + token_ttl_seconds;

                    state.refresh_attempts = 0;

                    info!(
                        cell = %cell,
                        expires_in_minutes = self.config.csrf_token_ttl_minutes,
                        token_length = token.len(),
                        "CSRF token refreshed successfully"
                    );

                    return Ok(());
                }
                Err(e) => {
                    error!(
                        error = %e,
                        attempt = attempt + 1,
                        max_attempts = self.max_refresh_attempts,
                        "CSRF token fetch attempt failed"
                    );

                    // Wait before retry with exponential backoff
                    if attempt < self.max_refresh_attempts - 1 {
                        let backoff_base: f64 = 2.0; // From Python: self.config.retry_backoff_base
                        let base_delay: f64 = 1.0; // From Python: self.config.retry_backoff_seconds
                        let backoff_delay = base_delay * backoff_base.powi(attempt as i32);

                        debug!(
                            delay_seconds = backoff_delay,
                            attempt = attempt + 1,
                            "Waiting before CSRF token retry"
                        );

                        tokio::time::sleep(tokio::time::Duration::from_secs_f64(backoff_delay))
                            .await;
                    }
                }
            }
        }

        // All attempts failed - clear token state
        let mut state = self.token_state.lock().await;
        state.csrf_token = None;
        state.token_expires_at = 0.0;

        error!(
            cell = %cell,
            max_attempts = self.max_refresh_attempts,
            "CSRF token refresh failed after all attempts"
        );

        Err(CsCliError::Authentication(
            "Failed to refresh CSRF token after all retry attempts".to_string(),
        ))
    }

    /// Attempt a single token refresh with proper error handling
    async fn attempt_token_refresh(&self, url: &str, cell: &str, attempt: u32) -> Result<String> {
        debug!(
            url = %url,
            attempt = attempt + 1,
            "Attempting CSRF token refresh"
        );

        // Set required headers for CSRF token request (matching Python exactly)
        let mut headers = HashMap::new();
        headers.insert("X-Requested-With".to_string(), "XMLHttpRequest".to_string());
        headers.insert(
            "Referer".to_string(),
            format!("https://{cell}.app.gong.io/"),
        );
        headers.insert(
            "Accept".to_string(),
            "application/json, text/javascript, */*; q=0.01".to_string(),
        );

        // Update headers on the HTTP client
        self.http_client.update_headers(headers).await?;

        // Make the actual HTTP request
        let response = self.http_client.get(url).await?;

        // Parse the JSON response
        let json_text = response
            .text()
            .await
            .map_err(|e| CsCliError::ApiRequest(format!("Failed to read CSRF response: {e}")))?;

        // Use serde_json for JSON parsing
        let data: Value = serde_json::from_str(&json_text)
            .map_err(|e| CsCliError::ApiRequest(format!("Failed to parse CSRF response: {e}")))?;

        let token = data.get("token").and_then(|v| v.as_str()).ok_or_else(|| {
            CsCliError::Authentication("CSRF response missing token field".to_string())
        })?;

        Ok(token.to_string())
    }

    /// Get current token without refresh (for testing/debugging)
    pub async fn get_current_token(&self) -> Option<String> {
        let state = self.token_state.lock().await;
        state.csrf_token.clone()
    }

    /// Check if token is currently valid (for health checks)
    pub async fn is_token_currently_valid(&self) -> bool {
        let state = self.token_state.lock().await;
        self.is_token_valid(&state)
    }
}
