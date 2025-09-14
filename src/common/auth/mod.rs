//! Common authentication modules  
//!
//! Shared authentication functionality including cookie extraction,
//! session management, and browser-based authentication patterns.

pub mod browser_auth;
pub mod cli_unlock;
pub mod cookie_extractor;
pub mod session_manager;

pub use browser_auth::*;
pub use cli_unlock::*;
pub use cookie_extractor::*;
pub use session_manager::*;
