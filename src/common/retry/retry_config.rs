//! Retry configuration and settings
//!
//! Common retry configuration that can be used across all modules.

use serde::{Deserialize, Serialize};

/// Configuration for retry operations with exponential backoff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Base delay in milliseconds before first retry
    pub base_delay_ms: u64,
    /// Maximum delay in milliseconds (caps exponential backoff)
    pub max_delay_ms: u64,
    /// Multiplier for exponential backoff (typically 2.0)
    pub backoff_multiplier: f64,
    /// Add jitter to prevent thundering herd (0.0 to 1.0)
    pub jitter_factor: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 1000,
            max_delay_ms: 10000,
            backoff_multiplier: 2.0,
            jitter_factor: 0.1, // 10% jitter
        }
    }
}

impl RetryConfig {
    /// Create a new retry configuration
    pub fn new(max_retries: u32, base_delay_ms: u64) -> Self {
        Self {
            max_retries,
            base_delay_ms,
            max_delay_ms: base_delay_ms * 10, // 10x base delay as max
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
        }
    }
    
    /// Create a fast retry configuration (for quick operations)
    pub fn fast() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 100,
            max_delay_ms: 1000,
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
        }
    }
    
    /// Create a slow retry configuration (for expensive operations)
    pub fn slow() -> Self {
        Self {
            max_retries: 5,
            base_delay_ms: 2000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
            jitter_factor: 0.2,
        }
    }
    
    /// Create a configuration for HTTP requests
    pub fn http() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 500,
            max_delay_ms: 5000,
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
        }
    }
    
    /// Create a configuration for authentication operations
    pub fn auth() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 1000,
            max_delay_ms: 10000,
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
        }
    }
}

/// Platform-specific retry configurations
pub struct PlatformRetryConfigs;

impl PlatformRetryConfigs {
    /// HTTP-specific retry configuration
    pub fn http() -> RetryConfig {
        RetryConfig::http()
    }
    
    /// Authentication-specific retry configuration
    pub fn auth() -> RetryConfig {
        RetryConfig::auth()
    }
    
    /// Gong-specific retry configuration
    pub fn gong() -> RetryConfig {
        RetryConfig::http()
    }
    
    /// Slack-specific retry configuration
    pub fn slack() -> RetryConfig {
        RetryConfig::http()
    }
    
    /// Gainsight-specific retry configuration
    pub fn gainsight() -> RetryConfig {
        RetryConfig::slow() // Gainsight can be slower
    }
    
    /// GitHub API retry configuration
    pub fn github() -> RetryConfig {
        RetryConfig::auth()
    }
    
    /// Browser automation retry configuration
    pub fn browser() -> RetryConfig {
        RetryConfig::fast()
    }
}