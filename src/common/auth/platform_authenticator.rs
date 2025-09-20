//! Platform authentication traits and common implementations
//!
//! Provides unified authentication patterns that can be implemented by all platforms
//! (Gong, Slack, Gainsight) to eliminate code duplication.

use crate::{CsCliError, Result};
use crate::gong::error::ErrorConverter;
use async_trait::async_trait;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info};

use super::{Cookie, CookieRetriever};

/// Platform-specific authentication session data
pub trait PlatformSession {
    /// Get the platform identifier (e.g., "gong", "slack", "gainsight")
    fn platform_name(&self) -> &str;
    
    /// Get the workspace/instance identifier
    fn workspace_id(&self) -> &str;
    
    /// Get the base URL for API calls
    fn base_url(&self) -> &str;
    
    /// Get session cookies
    fn cookies(&self) -> &HashMap<String, String>;
    
    /// Get authentication headers
    fn auth_headers(&self) -> HashMap<String, String>;
    
    /// Check if session is valid
    fn is_valid(&self) -> bool;
    
    /// Get when session was created (Unix timestamp)
    fn created_at(&self) -> f64;
    
    /// Get browser used for authentication
    fn browser_source(&self) -> Option<&str>;
}

/// Common authentication patterns for all platforms
#[async_trait]
pub trait PlatformAuthenticator {
    type Session: PlatformSession + Clone + Send + Sync;
    
    /// Initialize the authenticator with platform-specific configuration
    async fn initialize(&mut self) -> Result<()>;
    
    /// Perform complete authentication flow
    async fn authenticate(&mut self) -> Result<bool>;
    
    /// Check if currently authenticated
    async fn is_authenticated(&self) -> bool;
    
    /// Get current session data
    fn get_session(&self) -> Option<&Self::Session>;
    
    /// Refresh authentication if needed
    async fn refresh_auth(&mut self) -> Result<bool>;
    
    /// Get platform-specific domains for cookie retrieval
    fn get_cookie_domains(&self) -> Vec<String>;
    
    /// Validate platform-specific cookies
    fn validate_cookies(&self, cookies: &[Cookie]) -> Result<Vec<Cookie>>;
    
    /// Extract platform-specific authentication tokens
    async fn extract_auth_tokens(&self, cookies: &HashMap<String, String>) -> Result<HashMap<String, String>>;
    
    /// Get platform-specific authentication headers
    fn get_auth_headers(&self) -> HashMap<String, String>;
}

/// Common browser authentication flow implementation
pub struct CommonBrowserAuth {
    cookie_retriever: CookieRetriever,
    detected_browser: Option<String>,
}

impl CommonBrowserAuth {
    /// Create new common browser auth with platform domains
    pub fn new(domains: Vec<String>) -> Self {
        Self {
            cookie_retriever: CookieRetriever::new(domains),
            detected_browser: None,
        }
    }
    
    /// Find browser with valid authentication (common pattern)
    pub async fn find_browser_with_valid_auth(&mut self) -> Result<(Vec<Cookie>, String)> {
        info!("Searching for browser with valid authentication...");
        
        // Try browsers in order of preference
        let browsers = [
            ("firefox", "Firefox"),
            ("chrome", "Chrome"), 
            ("safari", "Safari"),
            ("edge", "Edge"),
            ("brave", "Brave"),
        ];
        
        for (browser_key, browser_name) in browsers.iter() {
            debug!("Trying {} authentication...", browser_name);
            
            let cookies = match *browser_key {
                "firefox" => {
                    tokio::task::spawn_blocking({
                        let retriever = self.cookie_retriever.clone();
                        move || retriever.retrieve_firefox_cookies_ephemeral()
                    }).await.map_err(|e| CsCliError::CookieRetrieval(format!("Firefox cookie task failed: {}", e)))?
                },
                "chrome" => {
                    // TODO: Implement Chrome cookie retrieval
                    continue;
                },
                "safari" => {
                    tokio::task::spawn_blocking({
                        let retriever = self.cookie_retriever.clone();
                        move || retriever.retrieve_safari_cookies_ephemeral()
                    }).await.map_err(|e| CsCliError::CookieRetrieval(format!("Safari cookie task failed: {}", e)))?
                },
                "edge" => {
                    // TODO: Implement Edge cookie retrieval
                    continue;
                },
                "brave" => {
                    // TODO: Implement Brave cookie retrieval
                    continue;
                },
                _ => continue,
            };
            
            if let Ok(cookies) = cookies {
                if !cookies.is_empty() {
                    info!("Found {} cookies from {}", cookies.len(), browser_name);
                    self.detected_browser = Some(browser_key.to_string());
                    return Ok((cookies, browser_name.to_string()));
                }
            }
        }
        
        Err(ErrorConverter::auth_error(
            CsCliError::Authentication("No valid authentication found in any browser. Please log into the platform in your browser first.".to_string()),
            "platform"
        ))
    }
    
    /// Get detected browser for consistent HTTP client configuration
    pub fn detected_browser(&self) -> Option<&str> {
        self.detected_browser.as_deref()
    }
    
    /// Convert cookies to HashMap format
    pub fn cookies_to_map(cookies: &[Cookie]) -> HashMap<String, String> {
        cookies.iter()
            .map(|c| (c.name.clone(), c.value.clone()))
            .collect()
    }
    
    /// Get current timestamp
    pub fn current_timestamp() -> f64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64()
    }
}

/// Platform-specific HTTP client configuration builder
pub struct PlatformHttpConfig {
    browser_type: String,
    pool_size: usize,
    max_concurrency_per_client: usize,
    timeout_seconds: f64,
}

impl PlatformHttpConfig {
    /// Create HTTP config for platform with detected browser
    pub fn new(browser_type: Option<&str>) -> Self {
        Self {
            browser_type: browser_type.unwrap_or("chrome").to_string(),
            pool_size: 1,
            max_concurrency_per_client: 5,
            timeout_seconds: 30.0,
        }
    }
    
    /// Set pool size
    pub fn with_pool_size(mut self, size: usize) -> Self {
        self.pool_size = size;
        self
    }
    
    /// Set concurrency
    pub fn with_concurrency(mut self, concurrency: usize) -> Self {
        self.max_concurrency_per_client = concurrency;
        self
    }
    
    /// Set timeout
    pub fn with_timeout(mut self, timeout: f64) -> Self {
        self.timeout_seconds = timeout;
        self
    }
    
    /// Build HttpSettings
    pub fn build(self) -> crate::common::config::HttpSettings {
        crate::common::config::HttpSettings {
            pool_size: self.pool_size,
            max_concurrency_per_client: self.max_concurrency_per_client,
            timeout_seconds: self.timeout_seconds,
            max_clients: Some(self.pool_size * self.max_concurrency_per_client),
            global_max_concurrency: Some(self.pool_size * self.max_concurrency_per_client),
            tls_version: None,
            browser_type: self.browser_type,
        }
    }
}

/// Common authentication error handling
pub struct AuthErrorHandler;

impl AuthErrorHandler {
    /// Handle authentication errors with platform-specific context
    pub fn handle_auth_error(error: CsCliError, platform: &str) -> CsCliError {
        match error {
            CsCliError::CookieRetrieval(msg) => {
                CsCliError::Authentication(format!(
                    "Failed to retrieve {} cookies: {}. Please ensure you're logged into {} in your browser.",
                    platform, msg, platform
                ))
            },
            CsCliError::Authentication(msg) => {
                CsCliError::Authentication(format!("{} authentication failed: {}", platform, msg))
            },
            other => other,
        }
    }
    
    /// Create authentication success message
    pub fn auth_success_message(platform: &str, browser: &str, cookie_count: usize) -> String {
        format!("Successfully authenticated with {} using {} ({} cookies)", platform, browser, cookie_count)
    }
}