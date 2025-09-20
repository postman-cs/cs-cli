//! Common CLI functionality shared across all platform integrations
//!
//! This module provides platform-agnostic CLI components including:
//! - Argument parsing structures
//! - Interactive mode with multi-select customer functionality
//! - Custom TUI components for rich terminal interactions

pub mod args;
pub mod full_interactive;
pub mod interactive;
pub mod multiselect;
pub mod theme;
pub mod tui_app;

pub use args::*;
pub use full_interactive::run_full_interactive;
pub use interactive::*;
pub use multiselect::{run_multiselect, MultiSelectState, SuggestionProvider};
pub use theme::THEME;
pub use tui_app::{draw_tui, AppState, ExtractionMessage, ExtractionResults, TuiApp};
