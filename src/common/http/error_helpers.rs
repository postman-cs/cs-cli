//! HTTP error handling helpers to eliminate DRY violations
//!
//! Provides common error mapping patterns for HTTP operations
//! to reduce repetitive error handling code.

use crate::CsCliError;
use std::fmt::Display;

/// Helper trait for mapping HTTP-related errors consistently
pub trait HttpErrorMapper {
    /// Map a generic error to an API request error with method context
    fn map_api_error(&self, method: &str, error: impl Display) -> CsCliError;
    
    /// Map a semaphore acquisition error
    fn map_semaphore_error(&self, error: impl Display) -> CsCliError;
    
    /// Map a request timeout error
    fn map_timeout_error(&self, method: &str, error: impl Display) -> CsCliError;
    
    /// Map a connection error
    fn map_connection_error(&self, method: &str, error: impl Display) -> CsCliError;
}

/// Default implementation for HTTP error mapping
pub struct DefaultHttpErrorMapper;

impl HttpErrorMapper for DefaultHttpErrorMapper {
    fn map_api_error(&self, method: &str, error: impl Display) -> CsCliError {
        CsCliError::ApiRequest(format!("{} request failed: {}", method.to_uppercase(), error))
    }
    
    fn map_semaphore_error(&self, error: impl Display) -> CsCliError {
        CsCliError::ApiRequest(format!("Failed to acquire semaphore: {}", error))
    }
    
    fn map_timeout_error(&self, method: &str, error: impl Display) -> CsCliError {
        CsCliError::NetworkTimeout(format!("{} request timed out: {}", method.to_uppercase(), error))
    }
    
    fn map_connection_error(&self, method: &str, error: impl Display) -> CsCliError {
        CsCliError::ApiRequest(format!("Connection failed for {} request: {}", method.to_uppercase(), error))
    }
}

/// Convenience functions for common HTTP error patterns
pub struct HttpErrorHelpers;

impl HttpErrorHelpers {
    /// Map a result with API error context
    pub fn map_result<T, E: Display>(
        result: std::result::Result<T, E>,
        method: &str,
    ) -> std::result::Result<T, CsCliError> {
        let mapper = DefaultHttpErrorMapper;
        result.map_err(|e| mapper.map_api_error(method, e))
    }
    
    /// Map a result with semaphore error context
    pub fn map_semaphore_result<T, E: Display>(
        result: std::result::Result<T, E>,
    ) -> std::result::Result<T, CsCliError> {
        let mapper = DefaultHttpErrorMapper;
        result.map_err(|e| mapper.map_semaphore_error(e))
    }
    
    /// Map a result with timeout error context
    pub fn map_timeout_result<T, E: Display>(
        result: std::result::Result<T, E>,
        method: &str,
    ) -> std::result::Result<T, CsCliError> {
        let mapper = DefaultHttpErrorMapper;
        result.map_err(|e| mapper.map_timeout_error(method, e))
    }
    
    /// Map a result with connection error context
    pub fn map_connection_result<T, E: Display>(
        result: std::result::Result<T, E>,
        method: &str,
    ) -> std::result::Result<T, CsCliError> {
        let mapper = DefaultHttpErrorMapper;
        result.map_err(|e| mapper.map_connection_error(method, e))
    }
}

/// Extension trait for Result types to add HTTP error mapping
pub trait HttpResultExt<T, E: Display> {
    /// Map error to API request error with method context
    fn map_api_error(self, method: &str) -> std::result::Result<T, CsCliError>;
    
    /// Map error to semaphore error
    fn map_semaphore_error(self) -> std::result::Result<T, CsCliError>;
    
    /// Map error to timeout error with method context
    fn map_timeout_error(self, method: &str) -> std::result::Result<T, CsCliError>;
    
    /// Map error to connection error with method context
    fn map_connection_error(self, method: &str) -> std::result::Result<T, CsCliError>;
}

impl<T, E: Display> HttpResultExt<T, E> for std::result::Result<T, E> {
    fn map_api_error(self, method: &str) -> std::result::Result<T, CsCliError> {
        HttpErrorHelpers::map_result(self, method)
    }
    
    fn map_semaphore_error(self) -> std::result::Result<T, CsCliError> {
        HttpErrorHelpers::map_semaphore_result(self)
    }
    
    fn map_timeout_error(self, method: &str) -> std::result::Result<T, CsCliError> {
        HttpErrorHelpers::map_timeout_result(self, method)
    }
    
    fn map_connection_error(self, method: &str) -> std::result::Result<T, CsCliError> {
        HttpErrorHelpers::map_connection_result(self, method)
    }
}