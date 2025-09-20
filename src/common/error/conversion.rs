//! Error conversion utilities
//!
//! Common error handling and conversion patterns that can be used across all modules.

use crate::CsCliError;

/// Convert external errors to CsCliError with context
pub struct ErrorConverter;

impl ErrorConverter {
    /// Convert std::io::Error to CsCliError
    pub fn io_error(error: std::io::Error, context: &str) -> CsCliError {
        CsCliError::FileIo(format!("{}: {}", context, error))
    }
    
    /// Convert reqwest::Error to CsCliError
    pub fn reqwest_error(error: reqwest::Error, context: &str) -> CsCliError {
        if error.is_timeout() {
            CsCliError::NetworkTimeout(format!("{}: {}", context, error))
        } else if error.is_connect() {
            CsCliError::ApiRequest(format!("Connection failed in {}: {}", context, error))
        } else {
            CsCliError::ApiRequest(format!("{}: {}", context, error))
        }
    }
    
    /// Convert serde_json::Error to CsCliError
    pub fn json_error(error: serde_json::Error, context: &str) -> CsCliError {
        CsCliError::ApiRequest(format!("JSON parsing failed in {}: {}", context, error))
    }
    
    /// Convert tokio::task::JoinError to CsCliError
    pub fn join_error(error: tokio::task::JoinError, context: &str) -> CsCliError {
        CsCliError::Generic(format!("Task join failed in {}: {}", context, error))
    }
    
    /// Convert anyhow::Error to CsCliError
    pub fn anyhow_error(error: anyhow::Error, context: &str) -> CsCliError {
        CsCliError::Generic(format!("{}: {}", context, error))
    }
    
    /// Convert string to CsCliError with context
    pub fn string_error(message: String, context: &str) -> CsCliError {
        CsCliError::Generic(format!("{}: {}", context, message))
    }
    
    /// Convert authentication error with platform context
    pub fn auth_error(error: CsCliError, platform: &str) -> CsCliError {
        match error {
            CsCliError::Authentication(msg) => {
                CsCliError::Authentication(format!("{} authentication failed: {}", platform, msg))
            }
            CsCliError::CookieRetrieval(msg) => {
                CsCliError::Authentication(format!("Failed to retrieve {} cookies: {}. Please ensure you're logged into {} in your browser.", platform, msg, platform))
            }
            other => other,
        }
    }
    
    /// Convert API error with endpoint context
    pub fn api_error(error: CsCliError, endpoint: &str) -> CsCliError {
        match error {
            CsCliError::ApiRequest(msg) => {
                CsCliError::ApiRequest(format!("API request to {} failed: {}", endpoint, msg))
            }
            CsCliError::NetworkTimeout(msg) => {
                CsCliError::NetworkTimeout(format!("Timeout calling {}: {}", endpoint, msg))
            }
            other => other,
        }
    }
}

/// Trait for converting errors to CsCliError
pub trait ToCsCliError<T> {
    /// Convert to CsCliError with context
    fn with_context(self, context: &str) -> Result<T, CsCliError>;
}

impl<T> ToCsCliError<T> for Result<T, std::io::Error> {
    fn with_context(self, context: &str) -> Result<T, CsCliError> {
        self.map_err(|e| ErrorConverter::io_error(e, context))
    }
}

impl<T> ToCsCliError<T> for Result<T, reqwest::Error> {
    fn with_context(self, context: &str) -> Result<T, CsCliError> {
        self.map_err(|e| ErrorConverter::reqwest_error(e, context))
    }
}

impl<T> ToCsCliError<T> for Result<T, serde_json::Error> {
    fn with_context(self, context: &str) -> Result<T, CsCliError> {
        self.map_err(|e| ErrorConverter::json_error(e, context))
    }
}

impl<T> ToCsCliError<T> for Result<T, tokio::task::JoinError> {
    fn with_context(self, context: &str) -> Result<T, CsCliError> {
        self.map_err(|e| ErrorConverter::join_error(e, context))
    }
}

impl<T> ToCsCliError<T> for Result<T, anyhow::Error> {
    fn with_context(self, context: &str) -> Result<T, CsCliError> {
        self.map_err(|e| ErrorConverter::anyhow_error(e, context))
    }
}

/// Macro for easy error conversion with context
#[macro_export]
macro_rules! with_context {
    ($result:expr, $context:expr) => {
        $result.map_err(|e| crate::common::error::conversion::ErrorConverter::string_error(e.to_string(), $context))
    };
}

/// Macro for authentication error conversion
#[macro_export]
macro_rules! auth_error {
    ($error:expr, $platform:expr) => {
        crate::common::error::conversion::ErrorConverter::auth_error($error, $platform)
    };
}

/// Macro for API error conversion
#[macro_export]
macro_rules! api_error {
    ($error:expr, $endpoint:expr) => {
        crate::common::error::conversion::ErrorConverter::api_error($error, $endpoint)
    };
}