//! Retry execution utilities
//!
//! Common retry execution patterns that can be used across all modules.

use crate::common::retry::{RetryConfig, calculate_backoff_delay, is_error_retryable};
use crate::{CsCliError, Result};
use tracing::{debug, warn};

/// Execute an operation with retry logic
pub async fn execute_with_retry<F, Fut, T>(
    operation: F,
    config: RetryConfig,
    operation_name: &str,
) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut attempt = 0;
    let mut _last_error: Option<CsCliError> = None;

    loop {
        match operation().await {
            Ok(result) => {
                if attempt > 0 {
                    debug!("Operation '{}' succeeded on attempt {}", operation_name, attempt + 1);
                }
                return Ok(result);
            }
            Err(error) => {
                _last_error = Some(error.clone());
                
                // Check if error is retryable
                if !is_error_retryable(&error) || attempt >= config.max_retries {
                    if attempt >= config.max_retries {
                        warn!(
                            "Operation '{}' failed after {} attempts: {}",
                            operation_name, config.max_retries, error
                        );
                    } else {
                        debug!(
                            "Operation '{}' failed with non-retryable error: {}",
                            operation_name, error
                        );
                    }
                    return Err(error);
                }
                
                attempt += 1;
                let delay = calculate_backoff_delay(
                    attempt - 1,
                    config.base_delay_ms,
                    config.max_delay_ms,
                    config.backoff_multiplier,
                    config.jitter_factor,
                );
                
                warn!(
                    "Operation '{}' failed (attempt {}/{}): {}. Retrying in {:.1}s",
                    operation_name,
                    attempt,
                    config.max_retries,
                    error,
                    delay.as_secs_f64()
                );
                
                tokio::time::sleep(delay).await;
            }
        }
    }
}

/// Execute an HTTP operation with retry logic and status code handling
pub async fn execute_http_with_retry<F, Fut>(
    operation: F,
    config: RetryConfig,
    operation_name: &str,
) -> Result<reqwest::Response>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<reqwest::Response>>,
{
    let mut attempt = 0;
    let mut _last_error: Option<CsCliError> = None;

    loop {
        match operation().await {
            Ok(response) => {
                let status = response.status().as_u16();
                
                // Check if we should retry based on status code
                if crate::common::retry::is_http_error_retryable(status) && attempt < config.max_retries {
                    attempt += 1;
                    
                    // Handle rate limiting (429) specially
                    if status == 429 {
                        // Try to get Retry-After header
                        let retry_after = response.headers()
                            .get("retry-after")
                            .and_then(|h| h.to_str().ok());
                        
                        let delay = crate::common::retry::calculate_retry_after_delay(
                            retry_after,
                            attempt - 1,
                            &config,
                        );
                        
                        warn!(
                            "HTTP operation '{}' rate limited (attempt {}/{}). Retrying in {:.1}s",
                            operation_name,
                            attempt,
                            config.max_retries,
                            delay.as_secs_f64()
                        );
                        
                        tokio::time::sleep(delay).await;
                        continue;
                    }
                    
                    // Handle server errors (5xx)
                    if status >= 500 {
                        let delay = calculate_backoff_delay(
                            attempt - 1,
                            config.base_delay_ms,
                            config.max_delay_ms,
                            config.backoff_multiplier,
                            config.jitter_factor,
                        );
                        
                        warn!(
                            "HTTP operation '{}' server error {} (attempt {}/{}). Retrying in {:.1}s",
                            operation_name,
                            status,
                            attempt,
                            config.max_retries,
                            delay.as_secs_f64()
                        );
                        
                        tokio::time::sleep(delay).await;
                        continue;
                    }
                }
                
                // Success or non-retryable error
                if attempt > 0 {
                    debug!("HTTP operation '{}' succeeded on attempt {}", operation_name, attempt + 1);
                }
                return Ok(response);
            }
            Err(error) => {
                _last_error = Some(error.clone());
                
                if !is_error_retryable(&error) || attempt >= config.max_retries {
                    if attempt >= config.max_retries {
                        warn!(
                            "HTTP operation '{}' failed after {} attempts: {}",
                            operation_name, config.max_retries, error
                        );
                    } else {
                        debug!(
                            "HTTP operation '{}' failed with non-retryable error: {}",
                            operation_name, error
                        );
                    }
                    return Err(error);
                }
                
                attempt += 1;
                let delay = calculate_backoff_delay(
                    attempt - 1,
                    config.base_delay_ms,
                    config.max_delay_ms,
                    config.backoff_multiplier,
                    config.jitter_factor,
                );
                
                warn!(
                    "HTTP operation '{}' failed (attempt {}/{}): {}. Retrying in {:.1}s",
                    operation_name,
                    attempt,
                    config.max_retries,
                    error,
                    delay.as_secs_f64()
                );
                
                tokio::time::sleep(delay).await;
            }
        }
    }
}

/// Execute an operation with custom retry condition
pub async fn execute_with_custom_retry<F, Fut, T, C>(
    operation: F,
    config: RetryConfig,
    operation_name: &str,
    retry_condition: C,
) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
    C: Fn(&CsCliError) -> bool,
{
    let mut attempt = 0;
    let mut _last_error: Option<CsCliError> = None;

    loop {
        match operation().await {
            Ok(result) => {
                if attempt > 0 {
                    debug!("Operation '{}' succeeded on attempt {}", operation_name, attempt + 1);
                }
                return Ok(result);
            }
            Err(error) => {
                _last_error = Some(error.clone());
                
                // Check custom retry condition
                if !retry_condition(&error) || attempt >= config.max_retries {
                    if attempt >= config.max_retries {
                        warn!(
                            "Operation '{}' failed after {} attempts: {}",
                            operation_name, config.max_retries, error
                        );
                    } else {
                        debug!(
                            "Operation '{}' failed with non-retryable error: {}",
                            operation_name, error
                        );
                    }
                    return Err(error);
                }
                
                attempt += 1;
                let delay = calculate_backoff_delay(
                    attempt - 1,
                    config.base_delay_ms,
                    config.max_delay_ms,
                    config.backoff_multiplier,
                    config.jitter_factor,
                );
                
                warn!(
                    "Operation '{}' failed (attempt {}/{}): {}. Retrying in {:.1}s",
                    operation_name,
                    attempt,
                    config.max_retries,
                    error,
                    delay.as_secs_f64()
                );
                
                tokio::time::sleep(delay).await;
            }
        }
    }
}