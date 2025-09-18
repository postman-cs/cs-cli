//! Gainsight authentication module
//!
//! Platform-specific authentication logic for Gainsight integrations.
//! Uses common auth abstractions for cookie extraction and session management.

pub mod gainsight_auth;

pub use gainsight_auth::*;

