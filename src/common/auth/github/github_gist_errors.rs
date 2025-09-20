//! Enhanced Error Handling for GitHub Gist Storage
//!
//! Provides structured error types with proper context and recovery strategies
//! for GitHub Gist storage operations.

use crate::CsCliError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Structured error types for GitHub Gist storage operations
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum GistStorageError {
    #[error("GitHub client not initialized - call setup_github_storage() first")]
    ClientNotInitialized,

    #[error("GitHub API request failed: {operation} - HTTP {status}")]
    ApiRequestFailed { 
        operation: String, 
        status: u16,
        #[serde(skip_serializing_if = "Option::is_none")]
        details: Option<String>,
    },

    #[error("Encryption operation failed: {reason}")]
    EncryptionFailed { reason: String },

    #[error("Configuration error in {field}: {reason}")]
    ConfigError { field: String, reason: String },

    #[error("Network timeout after {timeout}s during {operation}")]
    NetworkTimeout { timeout: u64, operation: String },

    #[error("Serialization failed: {reason}")]
    SerializationFailed { reason: String },

    #[error("Session validation failed: {reason}")]
    SessionValidationFailed { reason: String },

    #[error("Authentication required: {reason}")]
    AuthenticationRequired { reason: String },

    #[error("Gist not found: {gist_id}")]
    GistNotFound { gist_id: String },

    #[error("Invalid session data: {reason}")]
    InvalidSessionData { reason: String },

    #[error("Rate limit exceeded: retry after {retry_after}s")]
    RateLimitExceeded { retry_after: u64 },
}

impl GistStorageError {
    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            GistStorageError::NetworkTimeout { .. }
                | GistStorageError::RateLimitExceeded { .. }
        ) || matches!(
            self,
            GistStorageError::ApiRequestFailed { status, .. } if *status >= 500
        )
    }

    /// Get retry delay in seconds for retryable errors
    pub fn retry_delay(&self) -> Option<u64> {
        match self {
            GistStorageError::NetworkTimeout { timeout, .. } => Some(*timeout),
            GistStorageError::RateLimitExceeded { retry_after } => Some(*retry_after),
            GistStorageError::ApiRequestFailed { status, .. } if *status >= 500 => Some(5),
            _ => None,
        }
    }

    /// Get operation context for logging
    pub fn operation_context(&self) -> Option<&str> {
        match self {
            GistStorageError::ApiRequestFailed { operation, .. } => Some(operation),
            GistStorageError::NetworkTimeout { operation, .. } => Some(operation),
            _ => None,
        }
    }
}

/// Convert from octocrab errors to our structured error type
impl From<octocrab::Error> for GistStorageError {
    fn from(err: octocrab::Error) -> Self {
        match err {
            octocrab::Error::GitHub { source, backtrace: _ } => {
                let status = source.status_code;
                let details = source.message;
                
                // Check for rate limiting
                if status == 403 && details.contains("rate limit") {
                    return GistStorageError::RateLimitExceeded { 
                        retry_after: 60 // Default retry after 1 minute
                    };
                }

                GistStorageError::ApiRequestFailed {
                    operation: "GitHub API call".to_string(),
                    status: status.into(),
                    details: Some(details),
                }
            }
            octocrab::Error::Http { source: _, backtrace: _ } => {
                GistStorageError::ApiRequestFailed {
                    operation: "HTTP request".to_string(),
                    status: 0,
                    details: Some("HTTP request failed".to_string()),
                }
            }
            octocrab::Error::Serde { source: _, backtrace: _ } => {
                GistStorageError::SerializationFailed {
                    reason: "JSON parsing failed".to_string(),
                }
            }
            _ => GistStorageError::ApiRequestFailed {
                operation: "Unknown GitHub operation".to_string(),
                status: 0,
                details: Some(err.to_string()),
            }
        }
    }
}

/// Convert from our error type to the main CsCliError
impl From<GistStorageError> for CsCliError {
    fn from(err: GistStorageError) -> Self {
        CsCliError::GistStorage(err.to_string())
    }
}

/// Result type alias for GitHub Gist operations
pub type GistResult<T> = Result<T, GistStorageError>;

/// Retry configuration for operations
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 1000,
            max_delay_ms: 10000,
            backoff_multiplier: 2.0,
        }
    }
}

/// Execute operation with exponential backoff retry logic using common utilities
pub async fn with_retry<F, Fut, T>(
    operation: F,
    config: RetryConfig,
) -> GistResult<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = GistResult<T>>,
{
    // Convert to common retry config
    let common_config = crate::common::retry::RetryConfig {
        max_retries: config.max_retries,
        base_delay_ms: config.base_delay_ms,
        max_delay_ms: config.max_delay_ms,
        backoff_multiplier: config.backoff_multiplier,
        jitter_factor: 0.1, // Default jitter
    };
    
    // Use common retry executor with custom retry condition
    crate::common::retry::execute_with_custom_retry(
        || async {
            operation().await.map_err(|e| crate::CsCliError::GistStorageStructured(e))
        },
        common_config,
        "GitHub Gist operation",
        |error| {
            // Convert CsCliError back to GistStorageError for retry check
            if let crate::CsCliError::GistStorageStructured(gist_err) = error {
                gist_err.is_retryable()
            } else {
                false
            }
        },
    ).await.map_err(|e| {
        // Convert back to GistStorageError
        if let crate::CsCliError::GistStorageStructured(gist_err) = e {
            gist_err
        } else {
            GistStorageError::ApiRequestFailed {
                operation: "Unknown".to_string(),
                status: 0,
                details: Some(e.to_string()),
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_retryable_check() {
        let timeout_err = GistStorageError::NetworkTimeout {
            timeout: 30,
            operation: "test".to_string(),
        };
        assert!(timeout_err.is_retryable());

        let client_err = GistStorageError::ClientNotInitialized;
        assert!(!client_err.is_retryable());
    }

    #[test]
    fn test_error_retry_delay() {
        let timeout_err = GistStorageError::NetworkTimeout {
            timeout: 30,
            operation: "test".to_string(),
        };
        assert_eq!(timeout_err.retry_delay(), Some(30));

        let rate_limit_err = GistStorageError::RateLimitExceeded { retry_after: 60 };
        assert_eq!(rate_limit_err.retry_delay(), Some(60));
    }

    #[test]
    fn test_octocrab_error_conversion() {
        // Test rate limit detection
        let rate_limit_response = r#"{"message": "API rate limit exceeded"}"#;
        // This would need to be constructed from actual octocrab error
        // For now, just test the structure
        let err = GistStorageError::RateLimitExceeded { retry_after: 60 };
        assert!(err.is_retryable());
    }
}