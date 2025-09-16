//! Gong-related functionality for CS-CLI
//! 
//! This module contains all the core functionality for interacting with 
//! Gong API and managing customer success operations.

pub mod api;
pub mod auth;
pub mod cli;
pub mod config;
pub mod error;
pub mod models;
pub mod output;

// Re-export commonly used types for convenience (from common module)
pub use crate::common::{CsCliError, Result};
