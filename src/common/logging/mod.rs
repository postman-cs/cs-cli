//! Common logging utilities
//!
//! Provides shared logging initialization and management patterns
//! that can be used across all modules to eliminate duplication.

pub mod init;
pub mod file_utils;

pub use init::*;
pub use file_utils::*;