//! Common authentication modules  
//!
//! Shared authentication functionality including cookie retrieval from authenticated sessions,
//! session management, browser-based authentication patterns, and
//! cross-platform cookie synchronization via GitHub gists.

pub mod cookie_retriever;
pub mod guided_auth;
pub mod guided_auth_config;
pub mod guided_cookie_storage;
pub mod profile_detector;
pub mod session_manager;
pub mod temp_profile_manager;

// GitHub OAuth and gist storage modules
pub mod github_gist_storage;
pub mod github_gist_storage_v2;
pub mod github_gist_errors;
pub mod github_authenticator;
pub mod github_client_pool;
pub mod gist_config_manager;
pub mod session_metadata;
pub mod async_session_encryption;
pub mod github_oauth_config;
pub mod github_oauth_flow;
pub mod hybrid_cookie_storage;
pub mod session_encryption;

// Test modules
#[cfg(test)]
mod tests;

// Tests for guided auth are in integration tests

pub use cookie_retriever::*;
pub use guided_auth::*;
pub use guided_auth_config::*;
pub use guided_cookie_storage::*;
pub use profile_detector::*;
pub use session_manager::*;
pub use temp_profile_manager::*;

// GitHub OAuth and gist storage exports
pub use github_gist_storage::*;
pub use github_gist_storage_v2::*;
pub use github_gist_errors::*;
pub use github_authenticator::*;
pub use github_client_pool::*;
pub use gist_config_manager::*;
pub use session_metadata::*;
pub use async_session_encryption::*;
pub use github_oauth_config::*;
pub use github_oauth_flow::*;
pub use hybrid_cookie_storage::*;
pub use session_encryption::*;
