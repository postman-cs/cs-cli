//! Common CLI functionality shared across all platform integrations
//!
//! This module provides platform-agnostic CLI components including:
//! - Argument parsing structures
//! - Interactive mode with multi-select customer functionality
//! - Custom TUI components for rich terminal interactions

pub mod args;
pub mod interactive;
pub mod multiselect;
pub mod full_interactive;
pub mod tui_app;

pub use args::*;
pub use interactive::*;
pub use multiselect::{SuggestionProvider, MultiSelectState, run_multiselect};
pub use full_interactive::run_full_interactive;
pub use tui_app::{TuiApp, ExtractionMessage, ExtractionResults, draw_tui, AppState};