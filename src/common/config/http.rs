//! Common HTTP configuration settings
//!
//! Shared HTTP client configuration that can be used by both Gong and Slack integrations.

use serde::{Deserialize, Serialize};

/// HTTP client configuration settings matching Python HTTPConfig
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpSettings {
    pub pool_size: usize,
    pub max_concurrency_per_client: usize,
    pub timeout_seconds: f64,
    pub max_clients: Option<usize>,
    pub global_max_concurrency: Option<usize>,

    // HTTP version configuration
    pub enable_http3: bool,
    pub force_http3: bool,

    // TLS configuration
    pub tls_version: Option<String>,
    pub impersonate_browser: String,
}

impl Default for HttpSettings {
    fn default() -> Self {
        Self {
            pool_size: 50,
            max_concurrency_per_client: 40,
            timeout_seconds: 30.0,
            max_clients: None,
            global_max_concurrency: None,
            enable_http3: true,   // Enable HTTP/3 with fallback to HTTP/2
            force_http3: false,   // Allow fallback to HTTP/2 if HTTP/3 fails
            tls_version: None,
            impersonate_browser: "chrome".to_string(),
        }
    }
}

impl HttpSettings {
    /// Post-initialization logic matching Python __post_init__
    pub fn validate_and_fill_defaults(&mut self) {
        if self.max_clients.is_none() {
            self.max_clients = Some(self.pool_size * self.max_concurrency_per_client);
        }
        if self.global_max_concurrency.is_none() {
            self.global_max_concurrency = self.max_clients;
        }
    }
}
