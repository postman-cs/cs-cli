// Error types for the application
// To be implemented in Phase 1

use std::fmt;

#[derive(Debug)]
pub enum CsCliError {
    Authentication(String),
    ApiRequest(String),
    FileIo(String),
    Configuration(String),
    NetworkTimeout(String),
    CookieExtraction(String),
    InvalidArguments { message: String },
    Generic(String),
}

impl fmt::Display for CsCliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CsCliError::Authentication(msg) => write!(f, "Authentication error: {msg}"),
            CsCliError::ApiRequest(msg) => write!(f, "API request error: {msg}"),
            CsCliError::FileIo(msg) => write!(f, "File I/O error: {msg}"),
            CsCliError::Configuration(msg) => write!(f, "Configuration error: {msg}"),
            CsCliError::NetworkTimeout(msg) => write!(f, "Network timeout error: {msg}"),
            CsCliError::CookieExtraction(msg) => write!(f, "Cookie extraction error: {msg}"),
            CsCliError::InvalidArguments { message } => write!(f, "Invalid arguments: {message}"),
            CsCliError::Generic(msg) => write!(f, "Error: {msg}"),
        }
    }
}

impl std::error::Error for CsCliError {}

pub type Result<T> = std::result::Result<T, CsCliError>;
