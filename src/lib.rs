//! CS-CLI: Customer Success Communication Retrieval Tool
//!
//! A Rust implementation of the customer success CLI for retrieving
//! customer communications from Gong and formatting them as markdown.

pub mod common;
pub mod gong;
pub mod updater;

// Slack module is excluded from customer-facing builds
#[cfg(feature = "slack")]
pub mod slack;

// Re-export core types for convenience
pub use common::{CsCliError, Result};
