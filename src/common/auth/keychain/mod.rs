//! Keychain and security modules
//!
//! Handles macOS keychain operations, password storage/retrieval,
//! and secure credential management.

pub mod cli_unlock;
pub mod smart_keychain;

pub use cli_unlock::*;
pub use smart_keychain::*;