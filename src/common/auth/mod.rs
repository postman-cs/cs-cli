//! Common authentication modules  
//!
//! Shared authentication functionality including cookie extraction,
//! session management, browser-based authentication patterns, and
//! cross-platform cookie synchronization via GitHub gists.

pub mod browser_auth;
pub mod cookie_extractor;
pub mod guided_auth;
pub mod guided_auth_config;
pub mod guided_cookie_storage;
pub mod profile_detector;
pub mod session_manager;

// GitHub OAuth and gist storage modules
pub mod github_gist_storage;
pub mod github_oauth_config;
pub mod github_oauth_flow;
pub mod hybrid_cookie_storage;
pub mod session_encryption;

// Tests for guided auth are in integration tests

pub use browser_auth::*;
pub use cookie_extractor::*;
pub use guided_auth::*;
pub use guided_auth_config::*;
pub use guided_cookie_storage::*;
pub use profile_detector::*;
pub use session_manager::*;

// GitHub OAuth and gist storage exports
pub use github_gist_storage::*;
pub use github_oauth_config::*;
pub use github_oauth_flow::*;
pub use hybrid_cookie_storage::*;
pub use session_encryption::*;
