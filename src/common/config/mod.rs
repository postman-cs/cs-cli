//! Common configuration modules
//!
//! Shared configuration types and loading utilities for both Gong and Slack integrations.

pub mod auth;
pub mod http;
pub mod loader;

pub use auth::*;
pub use http::*;
pub use loader::*;
