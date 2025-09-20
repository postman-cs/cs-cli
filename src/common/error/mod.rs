//! Common error types shared across all CS-CLI modules
//!
//! This module provides standardized error handling for both Gong and Slack integrations.

pub mod conversion;
pub mod types;

pub use conversion::*;
pub use types::*;
