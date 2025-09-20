//! Common HTTP client modules
//!
//! Shared HTTP client functionality including browser compatibility,
//! client pooling, and generic traits for both Gong and Slack integrations.

pub mod browser_client;
pub mod client;
pub mod error_helpers;
pub mod platform_client_builder;
pub mod pool;
pub mod rate_limiting;

pub use browser_client::*;
pub use client::*;
pub use error_helpers::*;
pub use platform_client_builder::*;
pub use pool::*;
pub use rate_limiting::*;
