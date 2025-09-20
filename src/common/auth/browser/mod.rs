//! Browser management modules
//!
//! Handles browser automation, Chrome DevTools Protocol (CDP) integration,
//! profile detection, and temporary browser management for authentication flows.

pub mod cdp_browser_manager;
pub mod profile_detector;
pub mod temp_browser_manager;

pub use cdp_browser_manager::*;
pub use profile_detector::*;
pub use temp_browser_manager::*;