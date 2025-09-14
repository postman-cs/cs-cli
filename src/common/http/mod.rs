//! Common HTTP client modules
//!
//! Shared HTTP client functionality including browser impersonation,
//! client pooling, and generic traits for both Gong and Slack integrations.

pub mod browser_client;
pub mod client;
pub mod pool;
pub mod rate_limiting;

pub use browser_client::*;
pub use client::*;
pub use pool::*;
pub use rate_limiting::*;
