//! Exponential backoff and jitter calculations
//!
//! Common backoff calculation utilities used across retry operations.

use std::time::Duration;
use tracing::debug;

/// Calculate exponential backoff delay with jitter
pub fn calculate_backoff_delay(
    attempt: u32,
    base_delay_ms: u64,
    max_delay_ms: u64,
    multiplier: f64,
    jitter_factor: f64,
) -> Duration {
    // Calculate exponential backoff
    let exponential_delay = (base_delay_ms as f64 * multiplier.powi(attempt as i32)) as u64;
    
    // Cap at maximum delay
    let capped_delay = exponential_delay.min(max_delay_ms);
    
    // Add jitter to prevent thundering herd
    let jitter_range = (capped_delay as f64 * jitter_factor) as u64;
    let jitter = if jitter_range > 0 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let mut hasher = DefaultHasher::new();
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos().hash(&mut hasher);
        hasher.finish() % (jitter_range + 1)
    } else {
        0
    };
    
    let final_delay = capped_delay + jitter;
    
    debug!(
        "Backoff calculation: attempt={}, base={}ms, exponential={}ms, capped={}ms, jitter={}ms, final={}ms",
        attempt, base_delay_ms, exponential_delay, capped_delay, jitter, final_delay
    );
    
    Duration::from_millis(final_delay)
}

/// Calculate retry delay honoring Retry-After header
pub fn calculate_retry_after_delay(
    retry_after_header: Option<&str>,
    attempt: u32,
    config: &crate::common::retry::RetryConfig,
) -> Duration {
    // Try to parse Retry-After header first
    if let Some(retry_after) = retry_after_header {
        // Try parsing as seconds
        if let Ok(seconds) = retry_after.parse::<u64>() {
            debug!("Using Retry-After header: {} seconds", seconds);
            return Duration::from_secs(seconds);
        }
        
        // Could also handle HTTP date format here if needed
        debug!("Failed to parse Retry-After header: {}", retry_after);
    }
    
    // Fall back to exponential backoff
    calculate_backoff_delay(
        attempt,
        config.base_delay_ms,
        config.max_delay_ms,
        config.backoff_multiplier,
        config.jitter_factor,
    )
}

/// Check if an error is retryable based on HTTP status code
pub fn is_http_error_retryable(status: u16) -> bool {
    match status {
        // Server errors (5xx) - always retryable
        500..=599 => true,
        // Rate limiting (429) - retryable
        429 => true,
        // Client errors (4xx) - generally not retryable
        400..=499 => false,
        // Success and redirects - not retryable
        _ => false,
    }
}

/// Check if an error is retryable based on error type
pub fn is_error_retryable(error: &crate::CsCliError) -> bool {
    match error {
        crate::CsCliError::NetworkTimeout(_) => true,
        crate::CsCliError::ApiRequest(msg) => {
            // Check for specific retryable error messages
            msg.contains("timeout") || 
            msg.contains("connection") || 
            msg.contains("temporary") ||
            msg.contains("rate limit")
        },
        crate::CsCliError::Authentication(msg) => {
            // Some auth errors might be retryable (e.g., temporary token issues)
            msg.contains("temporary") || msg.contains("expired")
        },
        _ => false,
    }
}