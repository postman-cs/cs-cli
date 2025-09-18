//! Common authentication modules  
//!
//! Shared authentication functionality including cookie extraction,
//! session management, and browser-based authentication patterns.
//! macOS-only build.

pub mod browser_auth;
pub mod cdp_client;
pub mod cli_unlock;
pub mod cookie_extractor;
pub mod guided_auth;
pub mod session_manager;
pub mod smart_keychain;

#[cfg(test)]
mod guided_auth_test;

pub use browser_auth::*;
pub use cli_unlock::*;
pub use cookie_extractor::*;
pub use guided_auth::*;
pub use session_manager::*;
pub use smart_keychain::*;
