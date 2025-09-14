//! CS-CLI: Customer Success Communication Extraction Tool
//!
//! A Rust implementation of the customer success CLI for extracting
//! customer communications from Gong and formatting them as markdown.

pub mod common;
pub mod gong;
pub mod slack;

// Re-export core types for convenience
pub use common::{CsCliError, Result};
