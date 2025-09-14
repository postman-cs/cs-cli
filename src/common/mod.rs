
//! Common functionality shared across CS-CLI modules
//!
//! Provides shared abstractions for authentication, HTTP clients, configuration,
//! error handling, and data models used by both Gong and Slack integrations.

pub mod auth;
pub mod config;
pub mod error;
pub mod http;
pub mod models;

// Re-export commonly used types for convenience
pub use error::{CsCliError, Result};
