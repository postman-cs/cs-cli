//! Common environment variable utilities
//!
//! Provides shared environment variable handling and configuration patterns
//! that can be used across all modules to eliminate duplication.

pub mod env_utils;
pub mod config_env;

pub use env_utils::*;
pub use config_env::*;