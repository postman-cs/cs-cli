//! Common authentication configuration settings
//!
//! Shared authentication configuration that can be used by both Gong and Slack integrations.

use serde::{Deserialize, Serialize};

/// Authentication configuration settings matching Python AuthConfig
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSettings {
    pub csrf_token_ttl_minutes: u64,
    pub csrf_token_buffer_minutes: u64,
    pub retry_attempts: u32,
    pub retry_backoff_base: f64,
    pub retry_backoff_seconds: f64,
}

impl Default for AuthSettings {
    fn default() -> Self {
        Self {
            csrf_token_ttl_minutes: 30,
            csrf_token_buffer_minutes: 5,
            retry_attempts: 3,
            retry_backoff_base: 2.0,
            retry_backoff_seconds: 1.0,
        }
    }
}
