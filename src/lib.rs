//! CS-CLI: Customer Success Communication Extraction Tool
//!
//! A Rust implementation of the customer success CLI for extracting
//! customer communications from Gong and formatting them as markdown.

pub mod api;
pub mod auth;
pub mod cli;
pub mod config;
pub mod error;
pub mod models;
pub mod output;

pub use error::{CsCliError, Result};
