//! Common authentication modules  
//!
//! Shared authentication functionality including cookie retrieval from authenticated sessions,
//! session management, browser-based authentication patterns, and
//! cross-platform cookie synchronization via GitHub gists.

// Sub-modules
pub mod auth_flow;
pub mod browser;
pub mod cookies;
pub mod github;
pub mod keychain;
pub mod platform_authenticator;
pub mod session;

// Test modules
#[cfg(test)]
mod tests;

// Re-export all modules for backward compatibility
pub use auth_flow::*;
pub use browser::*;
pub use cookies::*;
pub use github::*;
pub use keychain::*;
pub use platform_authenticator::*;
pub use session::*;