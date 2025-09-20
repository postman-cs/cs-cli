// Error types for the application
// To be implemented in Phase 1

use std::fmt;

#[derive(Debug, Clone)]
pub enum CsCliError {
    Authentication(String),
    ApiRequest(String),
    FileIo(String),
    Configuration(String),
    NetworkTimeout(String),
    CookieRetrieval(String),
    InvalidArguments { message: String },
    UpdateError(String),
    Generic(String),

    // GitHub OAuth and gist storage errors
    GitHubOAuth(String),
    Encryption(String),
    GistStorage(String),
    GistStorageStructured(crate::common::auth::github_gist_errors::GistStorageError),
}

impl fmt::Display for CsCliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CsCliError::Authentication(msg) => write!(f, "Authentication error: {msg}"),
            CsCliError::ApiRequest(msg) => write!(f, "API request error: {msg}"),
            CsCliError::FileIo(msg) => write!(f, "File I/O error: {msg}"),
            CsCliError::Configuration(msg) => write!(f, "Configuration error: {msg}"),
            CsCliError::NetworkTimeout(msg) => write!(f, "Network timeout error: {msg}"),
            CsCliError::CookieRetrieval(msg) => write!(f, "Cookie retrieval error: {msg}"),
            CsCliError::InvalidArguments { message } => write!(f, "Invalid arguments: {message}"),
            CsCliError::UpdateError(msg) => write!(f, "Update error: {msg}"),
            CsCliError::Generic(msg) => write!(f, "Error: {msg}"),
            CsCliError::GitHubOAuth(msg) => write!(f, "GitHub OAuth error: {msg}"),
            CsCliError::Encryption(msg) => write!(f, "Session encryption error: {msg}"),
            CsCliError::GistStorage(msg) => write!(f, "GitHub gist storage error: {msg}"),
            CsCliError::GistStorageStructured(err) => write!(f, "GitHub gist storage error: {err}"),
        }
    }
}

impl std::error::Error for CsCliError {}

impl From<std::io::Error> for CsCliError {
    fn from(err: std::io::Error) -> Self {
        CsCliError::FileIo(err.to_string())
    }
}

impl From<anyhow::Error> for CsCliError {
    fn from(err: anyhow::Error) -> Self {
        CsCliError::Generic(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, CsCliError>;
