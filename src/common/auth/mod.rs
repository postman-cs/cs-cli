//! Common authentication modules  
//!
//! Shared authentication functionality including cookie retrieval from authenticated sessions,
//! session management, browser-based authentication patterns, and
//! cross-platform cookie synchronization via GitHub gists.

// Core modules
pub mod auth_flow;
pub mod browser;
pub mod cookies;
pub mod keychain;
pub mod session;
pub mod github;

// Top-level modules
pub mod cookie_retriever;
pub mod guided_auth;
pub mod profile_detector;
pub mod temp_profile_manager;
pub mod platform_authenticator;

// GitHub OAuth and gist storage modules (top-level)
pub mod github_gist_storage;
pub mod github_gist_storage_v2;
pub mod github_gist_errors;
pub mod github_authenticator;
pub mod github_client_pool;
pub mod gist_config_manager;
pub mod session_metadata;
pub mod async_session_encryption;
pub mod github_oauth_config;

// CDP browser automation
pub mod cdp_client;
pub mod cli_unlock;
pub mod smart_keychain;

// Test modules
#[cfg(test)]
mod tests;

// Re-export key items from submodules
pub use auth_flow::*;
pub use browser::*;
pub use cookies::*;
pub use keychain::*;
pub use session::*;
pub use github::*;

// Re-export top-level modules
pub use cookie_retriever::*;
pub use guided_auth::*;
pub use profile_detector::*;
pub use temp_profile_manager::*;
pub use platform_authenticator::*;

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

// CDP and other exports
pub use cdp_client::*;
pub use cli_unlock::*;
pub use smart_keychain::*;
