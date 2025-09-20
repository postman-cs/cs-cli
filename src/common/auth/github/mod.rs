//! GitHub integration modules
//!
//! Handles GitHub OAuth flows, gist storage for cross-platform cookie sync,
//! client pool management, and GitHub API integration.

pub mod gist_config_manager;
pub mod github_authenticator;
pub mod github_client_pool;
pub mod github_gist_errors;
pub mod github_gist_storage;
pub mod github_oauth_config;
pub mod github_oauth_flow;

pub use gist_config_manager::*;
pub use github_authenticator::*;
pub use github_client_pool::*;
pub use github_gist_errors::*;
pub use github_gist_storage::*;
pub use github_oauth_config::*;
pub use github_oauth_flow::*;