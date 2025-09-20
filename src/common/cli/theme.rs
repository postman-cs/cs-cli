//! Theme constants for the TUI application
//!
//! This module provides consistent color definitions and styling
//! for all UI components in the application.

use ratatui::style::Color;

/// Primary application theme colors
pub struct Theme {
    // Brand colors
    pub primary: Color,           // Main Postman purple
    pub postman_orange: Color,    // Official Postman Orange #FF6C37
    pub accent: Color,            // Lighter orange accent
    
    // Text colors
    pub text_primary: Color,      // White text
    pub text_secondary: Color,    // Light gray text
    pub text_muted: Color,        // Dark gray text
    pub text_disabled: Color,     // Disabled text
    
    // Status colors
    pub success: Color,           // Green for success states
    pub warning: Color,           // Yellow for warnings
    pub error: Color,             // Red for errors
    pub info: Color,              // Cyan for information
    
    // UI element colors
    pub background: Color,        // Dark background
    pub surface: Color,           // Card/surface background
    pub border: Color,            // Border color
    pub border_focused: Color,    // Focused border color
    pub highlight: Color,         // Selection highlight
    
    // Interactive states
    pub button_bg: Color,         // Button background
    pub button_hover: Color,      // Button hover state
    pub text_hover: Color,        // Text hover state
    pub highlight_hover: Color,   // Light hover highlight
}

/// Default application theme
pub const THEME: Theme = Theme {
    // Brand colors
    primary: Color::Rgb(111, 44, 186),      // Postman purple
    postman_orange: Color::Rgb(255, 108, 55), // Official Postman Orange #FF6C37
    accent: Color::Rgb(255, 142, 100),      // Lighter orange accent
    
    // Text colors
    text_primary: Color::White,
    text_secondary: Color::Rgb(230, 230, 230),
    text_muted: Color::Rgb(150, 150, 150),
    text_disabled: Color::Rgb(120, 120, 120),
    
    // Status colors
    success: Color::Green,
    warning: Color::Yellow,
    error: Color::Red,
    info: Color::Cyan,
    
    // UI element colors
    background: Color::Black,                // Default terminal background
    surface: Color::Rgb(50, 50, 50),        // Card background
    border: Color::Rgb(100, 100, 100),      // Default border
    border_focused: Color::Rgb(111, 44, 186), // Focused border (primary)
    highlight: Color::DarkGray,              // Selection highlight
    
    // Interactive states
    button_bg: Color::Rgb(200, 200, 200),   // Button background
    button_hover: Color::Rgb(140, 70, 220), // Button hover (lighter purple)
    text_hover: Color::Rgb(160, 160, 160),  // Text hover
    highlight_hover: Color::Rgb(240, 240, 240), // Light hover highlight
};

/// Convenience functions for common styling patterns
impl Theme {
    /// Get style for primary headings
    pub const fn heading_primary(&self) -> Color {
        self.primary
    }
    
    /// Get style for secondary headings
    pub const fn heading_secondary(&self) -> Color {
        self.text_secondary
    }
    
    /// Get style for selected items
    pub const fn selected(&self) -> Color {
        self.accent
    }
    
    /// Get style for interactive elements
    pub const fn interactive(&self) -> Color {
        self.primary
    }
    
    /// Get style for success messages
    pub const fn success_text(&self) -> Color {
        self.success
    }
    
    /// Get style for error messages
    pub const fn error_text(&self) -> Color {
        self.error
    }
    
    /// Get style for warning messages
    pub const fn warning_text(&self) -> Color {
        self.warning
    }
    
    /// Get style for info messages
    pub const fn info_text(&self) -> Color {
        self.info
    }
}
