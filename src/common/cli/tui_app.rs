//! Complete TUI application that handles the entire workflow
//!
//! This module provides a full-featured TUI that manages:
//! - Customer selection with autocomplete
//! - Time period and content type selection
//! - Extraction progress with loading bars
//! - Results summary

use crate::common::cli::args::{ContentType, ParsedCommand};
use crate::common::cli::theme::THEME;
use crossterm::event::{KeyCode, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
};
use std::collections::HashSet;
use std::time::Instant;
use tokio::sync::mpsc;

/// Easing functions for smooth animations
mod easing {
    /// Ease out cubic for smooth deceleration
    pub fn ease_out_cubic(t: f64) -> f64 {
        let t = t - 1.0;
        t * t * t + 1.0
    }
    
    /// Ease in out cubic for smooth acceleration and deceleration
    pub fn ease_in_out_cubic(t: f64) -> f64 {
        if t < 0.5 {
            4.0 * t * t * t
        } else {
            let t = 2.0 * t - 2.0;
            1.0 + t * t * t / 2.0
        }
    }
}

/// Result type for TUI operations
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Helper functions for mouse coordinate validation and hit testing
impl TuiApp {
    /// Check if mouse coordinates are within a given area
    fn is_mouse_in_area(&self, mouse: &MouseEvent, area: Rect) -> bool {
        let x = mouse.column;
        let y = mouse.row;
        x >= area.x && x < area.x + area.width && y >= area.y && y < area.y + area.height
    }

    /// Validate mouse coordinates are within reasonable bounds
    fn is_mouse_coordinate_valid(&self, mouse: &MouseEvent) -> bool {
        let row = mouse.row as usize;
        let col = mouse.column as usize;
        row <= 200 && col <= 500 && row > 0
    }

    /// Ensure suggestions state is consistent and highlight_index is valid
    fn ensure_suggestions_state_valid(&mut self) {
        if !self.suggestions.is_empty() && self.highlight_index >= self.suggestions.len() {
            self.highlight_index = 0;
        }
    }

    /// Handle suggestion selection with bounds checking
    fn handle_suggestion_interaction(&mut self, suggestion_index: usize, is_click: bool) -> bool {
        if suggestion_index >= self.suggestions.len() {
            return false;
        }

        if is_click {
            self.highlight_index = suggestion_index;
            self.toggle_selection();
            self.error_message = None;
            self.in_dropdown = true;
        } else if suggestion_index != self.highlight_index {
            // Hover effect
            self.highlight_index = suggestion_index;
        }
        false
    }
}


/// Messages from extraction thread to TUI
#[derive(Debug, Clone)]
pub enum ExtractionMessage {
    // Authentication messages
    AuthProgress(f64, String), // Progress and status message
    AuthSuccess,
    AuthFailed(String),

    // Extraction messages
    Phase(String),
    Progress(f64), // 0.0 to 1.0
    SubTask(String),
    CallsFound(usize),
    EmailsFound(usize),
    FileSaved(String),
    Complete(ExtractionResults),
    Error(String),
}

/// Final extraction results
#[derive(Debug, Clone)]
pub struct ExtractionResults {
    pub total_calls: usize,
    pub total_emails: usize,
    pub files_saved: usize,
    pub output_directory: String,
}

/// State machine for the complete workflow
#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    // Sync choice phase (shown on first run)
    SyncChoice,

    // Authentication phase
    Authenticating,
    AuthenticationFailed(String),

    // Selection phases
    CustomerSelection,
    TimeSelection,
    ContentSelection,
    Confirmation,

    // Extraction phases
    Initializing,
    Extracting,
    Complete,
    Error(String),

    // Graceful shutdown
    Exiting(f64), // Progress of fade-out animation (0.0 to 1.0)
}

/// Complete application state
pub struct TuiApp {
    /// Current state in the workflow
    pub state: AppState,

    // Authentication
    pub auth_progress: f64,
    pub auth_status: String,
    pub auth_error: Option<String>,
    pub auth_steps_completed: Vec<String>,
    pub auth_current_step: usize,
    pub auth_total_steps: usize,
    pub auth_start_time: Option<Instant>,

    // Customer selection
    pub input: String,
    pub cursor: usize,
    pub suggestions: Vec<String>,
    pub selected_customers: HashSet<String>,
    pub highlight_index: usize,
    pub in_dropdown: bool,
    pub dropdown_render_area: Option<ratatui::layout::Rect>, // Store actual render area for mouse handling

    // Time selection
    pub days_input: String,
    pub days_cursor: usize,

    // Content type selection
    pub content_selection: usize, // 0=calls, 1=emails, 2=both

    // Extraction progress
    pub current_phase: String,
    pub current_progress: f64,
    pub current_subtask: String,
    pub extraction_log: Vec<String>,
    pub start_time: Option<Instant>,

    // Results
    pub results: Option<ExtractionResults>,

    // Communication channel
    pub extraction_rx: Option<mpsc::UnboundedReceiver<ExtractionMessage>>,

    // UI feedback
    pub error_message: Option<String>,

    // Cross-device sync status and choice
    pub sync_enabled: bool,
    pub sync_choice_made: bool,
    pub sync_choice_selection: usize, // 0=GitHub OAuth, 1=Local only

    // Authentication mode toggle
    pub auth_mode_button_area: Option<Rect>, // Store button click area

    // Authentication choice overlay areas
    pub github_button_area: Option<Rect>,
    pub local_text_area: Option<Rect>,
    pub github_button_hovered: bool,
    pub local_text_hovered: bool,

    // Layout coordinates (for better mouse handling)
    pub last_suggestion_area: Option<ratatui::layout::Rect>,
    pub last_time_input_area: Option<ratatui::layout::Rect>,
    pub last_content_area: Option<ratatui::layout::Rect>,

    // Store the actual rendered area of the suggestions list
    pub suggestions_render_area: Option<ratatui::layout::Rect>,
    
    // Animation state management for smooth rendering
    pub last_animation_progress: f64,
    pub animation_dirty: bool, // Flag to track if animation needs redraw
    pub last_frame_time: Option<Instant>,
}

impl Default for TuiApp {
    fn default() -> Self {
        Self::new()
    }
}

impl TuiApp {
    pub fn new() -> Self {
        Self {
            state: AppState::SyncChoice, // Start with sync choice - TUI runner will determine if needed
            auth_progress: 0.0,
            auth_status: "Starting authentication...".to_string(),
            auth_error: None,
            auth_steps_completed: Vec::new(),
            auth_current_step: 0,
            auth_total_steps: 7, // Total auth steps: Check credentials, Initialize session, Connect Okta, Select Verify, Verify auth, Connect platforms, Save auth
            auth_start_time: Some(Instant::now()),
            input: String::new(),
            cursor: 0,
            suggestions: Vec::new(),
            selected_customers: HashSet::new(),
            highlight_index: 0,
            in_dropdown: false,
            dropdown_render_area: None,
            days_input: "180".to_string(),
            days_cursor: 3,
            content_selection: 2,
            current_phase: String::new(),
            current_progress: 0.0,
            current_subtask: String::new(),
            extraction_log: Vec::new(),
            start_time: None,
            results: None,
            extraction_rx: None,
            error_message: None,
            sync_enabled: false, // Will be updated when we determine sync status
            sync_choice_made: false,
            sync_choice_selection: 0, // Default to GitHub OAuth
            auth_mode_button_area: None,
            github_button_area: None,
            local_text_area: None,
            github_button_hovered: false,
            local_text_hovered: false,
            last_suggestion_area: None,
            last_time_input_area: None,
            last_content_area: None,
            suggestions_render_area: None,
            last_animation_progress: 0.0,
            animation_dirty: true, // Start dirty to force initial draw
            last_frame_time: None,
        }
    }

    pub fn set_extraction_channel(&mut self, rx: mpsc::UnboundedReceiver<ExtractionMessage>) {
        self.extraction_rx = Some(rx);
    }

    /// Set cross-device sync status
    pub fn with_sync_status(mut self, sync_enabled: bool) -> Self {
        self.sync_enabled = sync_enabled;
        self
    }

    /// Get sync status indicator text
    pub fn get_sync_indicator(&self) -> &'static str {
        if self.sync_enabled {
            " [✓ Synced]"
        } else {
            " [Local]"
        }
    }

    /// Get authentication mode button text
    pub fn get_auth_mode_button_text(&self) -> String {
        if self.sync_enabled {
            "Authentication Mode: Cloud | Local".to_string()
        } else {
            "Authentication Mode: Local | Cloud".to_string()
        }
    }

    /// Toggle authentication mode
    pub fn toggle_auth_mode(&mut self) {
        self.sync_enabled = !self.sync_enabled;
        // Update sync choice selection to match
        self.sync_choice_selection = if self.sync_enabled { 0 } else { 1 };
    }

    /// Handle sync choice input
    fn handle_sync_choice_input(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Up => {
                if self.sync_choice_selection > 0 {
                    self.sync_choice_selection -= 1;
                }
                false
            }
            KeyCode::Down => {
                if self.sync_choice_selection < 1 {
                    self.sync_choice_selection += 1;
                }
                false
            }
            KeyCode::Enter => {
                self.sync_choice_made = true;
                self.sync_enabled = self.sync_choice_selection == 0; // 0 = GitHub OAuth, 1 = Local only
                self.state = AppState::Authenticating;
                false
            }
            _ => false,
        }
    }

    /// Handle authentication storage choice input
    fn handle_auth_choice_input(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Char('g') | KeyCode::Char('G') => {
                // GitHub storage (cross-device sync)
                self.sync_enabled = true;
                self.sync_choice_made = true;
                // Authentication will continue automatically with GitHub storage
                false
            }
            KeyCode::Char('l') | KeyCode::Char('L') => {
                // Local keychain storage only
                self.sync_enabled = false;
                self.sync_choice_made = true;
                // Authentication will continue automatically with local storage
                false
            }
            _ => false,
        }
    }

    /// Handle sync choice mouse events
    fn handle_sync_choice_mouse(&mut self, mouse: MouseEvent) -> bool {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                // Simple click handling - could be enhanced with precise button detection
                self.sync_choice_made = true;
                self.sync_enabled = self.sync_choice_selection == 0;
                self.state = AppState::Authenticating;
                false
            }
            _ => false,
        }
    }

    /// Handle authentication storage choice mouse events
    fn handle_auth_choice_mouse(&mut self, mouse: MouseEvent) -> bool {
        // Reset hover states
        self.github_button_hovered = false;
        self.local_text_hovered = false;

        // Check GitHub button interaction
        if let Some(button_area) = self.github_button_area {
            if self.is_mouse_in_area(&mouse, button_area) {
                self.github_button_hovered = true;

                if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) {
                    self.sync_enabled = true;
                    self.sync_choice_made = true;
                }
                return false;
            }
        }

        // Check local text interaction
        if let Some(text_area) = self.local_text_area {
            if self.is_mouse_in_area(&mouse, text_area) {
                self.local_text_hovered = true;

                if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) {
                    self.sync_enabled = false;
                    self.sync_choice_made = true;
                }
                return false;
            }
        }

        false
    }

    pub fn handle_extraction_message(&mut self, msg: ExtractionMessage) {
        match msg {
            // Authentication messages
            ExtractionMessage::AuthProgress(_progress, status) => {
                // Parse status to determine which step we're on
                let step_progress = self.determine_auth_step(&status);

                // Update current step and add to completed if it's a new step
                if step_progress > self.auth_current_step {
                    if !status.starts_with("Failed") && !status.starts_with("Error") {
                        self.auth_steps_completed.push(self.auth_status.clone());
                    }
                    self.auth_current_step = step_progress;
                }

                // Calculate overall progress based on steps
                self.auth_progress =
                    (self.auth_current_step as f64) / (self.auth_total_steps as f64);
                self.auth_status = status;
            }
            ExtractionMessage::AuthSuccess => {
                self.auth_progress = 1.0;
                self.auth_current_step = self.auth_total_steps;
                self.state = AppState::CustomerSelection;
            }
            ExtractionMessage::AuthFailed(error) => {
                self.state = AppState::AuthenticationFailed(error.clone());
                self.auth_error = Some(error);
            }

            // Extraction messages
            ExtractionMessage::Phase(phase) => {
                self.current_phase = phase.clone();
                self.current_progress = 0.0;
                self.extraction_log.push(format!("▶ {phase}"));
            }
            ExtractionMessage::Progress(p) => {
                self.current_progress = p;
            }
            ExtractionMessage::SubTask(task) => {
                self.current_subtask = task.clone();
                self.extraction_log.push(format!("  • {task}"));
            }
            ExtractionMessage::CallsFound(n) => {
                self.extraction_log.push(format!("  ✓ Found {n} calls"));
            }
            ExtractionMessage::EmailsFound(n) => {
                self.extraction_log.push(format!("  ✓ Found {n} emails"));
            }
            ExtractionMessage::FileSaved(path) => {
                self.extraction_log.push(format!("  📄 {path}"));
            }
            ExtractionMessage::Complete(results) => {
                self.results = Some(results);
                self.state = AppState::Complete;
            }
            ExtractionMessage::Error(e) => {
                self.state = AppState::Error(e.clone());
                self.extraction_log.push(format!("❌ Error: {e}"));
            }
        }

        // Keep log size manageable
        if self.extraction_log.len() > 100 {
            self.extraction_log.drain(0..50);
        }
    }

    pub fn handle_input(&mut self, key: KeyCode) -> bool {
        // Always allow Escape to exit immediately
        if matches!(key, KeyCode::Esc) {
            return true; // Exit immediately
        }

        match self.state {
            AppState::SyncChoice => self.handle_sync_choice_input(key),
            AppState::Authenticating => self.handle_auth_choice_input(key),
            AppState::AuthenticationFailed(_) => {
                match key {
                    KeyCode::Char('r') | KeyCode::Enter => {
                        // Retry authentication - reset all auth state
                        self.state = AppState::Authenticating;
                        self.auth_progress = 0.0;
                        self.auth_status = "Starting authentication...".to_string();
                        self.auth_error = None;
                        self.auth_current_step = 0;
                        self.auth_steps_completed.clear();
                        self.auth_start_time = Some(Instant::now());
                        self.sync_choice_made = false; // Force user to choose storage again
                        false
                    }
                    _ => false,
                }
            }
            AppState::CustomerSelection => self.handle_customer_input(key),
            AppState::TimeSelection => self.handle_time_input(key),
            AppState::ContentSelection => self.handle_content_input(key),
            AppState::Confirmation => self.handle_confirmation_input(key),
            AppState::Initializing | AppState::Extracting => false, // No input during extraction (Escape already handled)
            AppState::Complete | AppState::Error(_) => {
                matches!(key, KeyCode::Enter) // Enter also exits on complete/error screens
            }
            AppState::Exiting(_) => false, // No input during exit animation
        }
    }

    /// Handle mouse events
    pub fn handle_mouse(&mut self, mouse: MouseEvent) -> bool {
        // Check for auth mode button clicks first (available on most screens)
        if let Some(button_area) = self.auth_mode_button_area {
            if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) {
                if self.is_mouse_in_area(&mouse, button_area) {
                    self.toggle_auth_mode();
                    return false; // Don't exit, just toggle mode
                }
            }
        }

        match self.state {
            AppState::SyncChoice => self.handle_sync_choice_mouse(mouse),
            AppState::Authenticating => self.handle_auth_choice_mouse(mouse),
            AppState::CustomerSelection => self.handle_customer_mouse(mouse),
            AppState::TimeSelection => self.handle_time_mouse(mouse),
            AppState::ContentSelection => self.handle_content_mouse(mouse),
            AppState::Confirmation => self.handle_confirmation_mouse(mouse),
            AppState::Complete | AppState::Error(_) => {
                // Click to exit on complete/error screens
                matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left))
            }
            _ => false,
        }
    }

    fn handle_customer_input(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Enter => {
                if !self.selected_customers.is_empty() {
                    self.state = AppState::TimeSelection;
                    self.error_message = None;
                } else {
                    self.error_message =
                        Some("Please select at least one customer (use TAB to select)".to_string());
                }
                false
            }
            KeyCode::Tab => {
                self.toggle_selection();
                self.error_message = None; // Clear error when selecting
                false
            }
            KeyCode::Up => {
                self.move_up();
                self.error_message = None; // Clear error when navigating
                false
            }
            KeyCode::Down => {
                self.move_down();
                self.error_message = None; // Clear error when navigating
                false
            }
            KeyCode::Left => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
                false
            }
            KeyCode::Right => {
                if self.cursor < self.input.len() {
                    self.cursor += 1;
                }
                false
            }
            KeyCode::Backspace => {
                if self.cursor > 0 {
                    self.input.remove(self.cursor - 1);
                    self.cursor -= 1;
                    self.in_dropdown = false;
                    self.highlight_index = 0;
                    self.error_message = None; // Clear error when typing
                }
                false
            }
            KeyCode::Char(c) => {
                self.input.insert(self.cursor, c);
                self.cursor += 1;
                self.in_dropdown = false;
                self.highlight_index = 0;
                self.error_message = None; // Clear error when typing
                false
            }
            _ => false,
        }
    }

    fn handle_time_input(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Enter => {
                if !self.days_input.is_empty() {
                    self.state = AppState::ContentSelection;
                }
                false
            }
            KeyCode::Left => {
                if self.days_cursor > 0 {
                    self.days_cursor -= 1;
                }
                false
            }
            KeyCode::Right => {
                if self.days_cursor < self.days_input.len() {
                    self.days_cursor += 1;
                }
                false
            }
            KeyCode::Backspace => {
                if self.days_cursor > 0 && !self.days_input.is_empty() {
                    self.days_input.remove(self.days_cursor - 1);
                    self.days_cursor -= 1;
                }
                false
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                self.days_input.insert(self.days_cursor, c);
                self.days_cursor += 1;
                false
            }
            _ => false,
        }
    }

    fn handle_content_input(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Enter => {
                self.state = AppState::Confirmation;
                false
            }
            KeyCode::Up => {
                if self.content_selection > 0 {
                    self.content_selection -= 1;
                }
                false
            }
            KeyCode::Down => {
                if self.content_selection < 2 {
                    self.content_selection += 1;
                }
                false
            }
            KeyCode::Char('1') => {
                self.content_selection = 0;
                false
            }
            KeyCode::Char('2') => {
                self.content_selection = 1;
                false
            }
            KeyCode::Char('3') => {
                self.content_selection = 2;
                false
            }
            _ => false,
        }
    }

    fn handle_confirmation_input(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Enter => {
                self.state = AppState::Initializing;
                self.start_time = Some(Instant::now());
                false
            }
            _ => false,
        }
    }

    fn move_up(&mut self) {
        if self.suggestions.is_empty() {
            return;
        }

        if !self.in_dropdown {
            self.in_dropdown = true;
            self.highlight_index = self.suggestions.len().saturating_sub(1);
        } else if self.highlight_index > 0 {
            self.highlight_index = self.highlight_index.saturating_sub(1);
        }
        
        self.ensure_suggestions_state_valid();
    }

    fn move_down(&mut self) {
        if self.suggestions.is_empty() {
            return;
        }

        if !self.in_dropdown {
            self.in_dropdown = true;
            self.highlight_index = 0;
        } else if self.highlight_index < self.suggestions.len().saturating_sub(1) {
            self.highlight_index = self.highlight_index + 1;
        }
        
        self.ensure_suggestions_state_valid();
    }

    fn toggle_selection(&mut self) {
        if !self.in_dropdown || self.suggestions.is_empty() {
            return;
        }

        if let Some(item) = self.suggestions.get(self.highlight_index) {
            let item = item.clone();
            if self.selected_customers.contains(&item) {
                self.selected_customers.remove(&item);
            } else {
                self.selected_customers.insert(item);
            }
        }
    }

    pub fn update_suggestions(&mut self, suggestions: Vec<String>) {
        self.suggestions = suggestions;
        self.ensure_suggestions_state_valid();
    }

    fn handle_dropdown_click(&mut self, y: usize, is_click: bool) -> bool {
        // Early return if no suggestions
        if self.suggestions.is_empty() {
            return false;
        }

        self.ensure_suggestions_state_valid();

        // Open dropdown if not already open
        if !self.in_dropdown {
            self.in_dropdown = true;
        }

        // Check if click/hover is within suggestions area
        if let Some(area) = self.suggestions_render_area {
            if y >= area.y as usize && y < (area.y + area.height) as usize {
                let relative_y = y.saturating_sub(area.y as usize);
                self.handle_suggestion_interaction(relative_y, is_click)
            } else {
                false
            }
        } else {
            false
        }
    }

    fn handle_customer_mouse(&mut self, mouse: MouseEvent) -> bool {
        // Validate mouse coordinates
        if !self.is_mouse_coordinate_valid(&mouse) {
            return false;
        }

        self.ensure_suggestions_state_valid();

        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                self.handle_dropdown_click(mouse.row as usize, true)
            }
            MouseEventKind::Moved => {
                // Process hover if we have suggestions
                if !self.suggestions.is_empty() {
                    self.handle_dropdown_click(mouse.row as usize, false)
                } else {
                    false
                }
            }
            MouseEventKind::ScrollUp => {
                self.move_up();
                false
            }
            MouseEventKind::ScrollDown => {
                self.move_down();
                false
            }
            _ => false,
        }
    }

    fn handle_time_mouse(&mut self, mouse: MouseEvent) -> bool {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                let x = mouse.column as usize;
                let y = mouse.row as usize;

                // Check if clicking in the input field (approximate area)
                if (5..=7).contains(&y) && x >= 2 {
                    // Approximate time input field area
                    // Set cursor position in the time input
                    let field_x = x.saturating_sub(2); // Account for border
                    self.days_cursor = field_x.min(self.days_input.len());
                }
                false
            }
            _ => false,
        }
    }

    fn handle_content_mouse(&mut self, mouse: MouseEvent) -> bool {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                let y = mouse.row as usize;

                // Check if clicking on content options (approximate area)
                if (6..=8).contains(&y) {
                    // Approximate content selection area
                    let option_index = (y - 6).min(2);
                    self.content_selection = option_index;
                }
                false
            }
            MouseEventKind::ScrollUp => {
                if self.content_selection > 0 {
                    self.content_selection -= 1;
                }
                false
            }
            MouseEventKind::ScrollDown => {
                if self.content_selection < 2 {
                    self.content_selection += 1;
                }
                false
            }
            _ => false,
        }
    }

    fn handle_confirmation_mouse(&mut self, mouse: MouseEvent) -> bool {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                // Any click starts the extraction
                self.state = AppState::Initializing;
                self.start_time = Some(std::time::Instant::now());
                false
            }
            _ => false,
        }
    }

    fn determine_auth_step(&self, status: &str) -> usize {
        // Lookup table for auth step patterns
        const STEP_PATTERNS: &[(&[&str], usize)] = &[
            (&["Checking", "stored", "saved"], 1),
            (&["Launching", "browser", "Initializ"], 2),
            (&["Navigating", "Okta", "Connect"], 3),
            (&["Selecting", "Verify"], 4),
            (&["Waiting", "authentication", "Verif"], 5),
            (&["platform", "Loading"], 6),
            (&["Storing", "Successfully", "Sav"], 7),
        ];
        
        STEP_PATTERNS
            .iter()
            .find(|(patterns, _)| patterns.iter().any(|pattern| status.contains(pattern)))
            .map(|(_, step)| *step)
            .unwrap_or(self.auth_current_step) // Keep current step if no pattern matches
    }

    /// Update animations with smooth timing
    pub fn update_animations(&mut self) -> bool {
        let now = Instant::now();
        let dt = if let Some(last_time) = self.last_frame_time {
            now.duration_since(last_time).as_secs_f64()
        } else {
            0.016 // First frame, assume 60 FPS
        };
        self.last_frame_time = Some(now);
        
        let mut animation_complete = false;
        
        // Update exit animation
        if let AppState::Exiting(ref mut progress) = self.state {
            // Animate over 500ms (0.5 seconds) with smooth timing
            *progress += dt * 2.0; // 2.0 = 1.0 / 0.5 seconds
            if *progress >= 1.0 {
                *progress = 1.0;
                animation_complete = true;
            }
            self.animation_dirty = true;
        }
        
        // Update authentication animation progress
        if matches!(self.state, AppState::Authenticating) {
            let current_progress = if let Some(start_time) = self.auth_start_time {
                let elapsed = start_time.elapsed().as_millis() as f64;
                (elapsed / 3000.0).min(1.0) // 3 second animation
            } else {
                0.0
            };
            
            // Only mark dirty if progress changed significantly (avoid micro-updates)
            if (current_progress - self.last_animation_progress).abs() > 0.001 {
                self.last_animation_progress = current_progress;
                self.animation_dirty = true;
            }
        }
        
        animation_complete
    }

    pub fn update_exit_animation(&mut self) -> bool {
        // Legacy method for compatibility - use update_animations instead
        self.update_animations()
    }

    pub fn get_parsed_command(&self) -> ParsedCommand {
        let days = self.days_input.parse::<u32>().unwrap_or(180);
        let content_type = match self.content_selection {
            0 => ContentType::Calls,
            1 => ContentType::Emails,
            _ => ContentType::Both,
        };

        let emails_only = content_type == ContentType::Emails;
        let fetch_email_bodies = matches!(content_type, ContentType::Emails | ContentType::Both);

        let customers: Vec<String> = self.selected_customers.iter().cloned().collect();
        if customers.len() == 1 {
            ParsedCommand::Customer {
                name: customers[0].clone(),
                days: Some(days),
                from_date: None,
                to_date: None,
                content_type,
                emails_only,
                fetch_email_bodies,
            }
        } else {
            ParsedCommand::MultipleCustomers {
                names: customers,
                days: Some(days),
                from_date: None,
                to_date: None,
                content_type,
                emails_only,
                fetch_email_bodies,
            }
        }
    }
}

/// Helper function to draw auth mode button (used across multiple UI screens)
fn draw_auth_mode_button(f: &mut Frame, app: &mut TuiApp, area: Rect) {
    let button_text = app.get_auth_mode_button_text();
    let button_widget = Paragraph::new(button_text)
        .style(
            Style::default()
                .fg(THEME.button_bg)
                .bg(THEME.surface),
        )
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(THEME.border)),
        );

    // Store button area for mouse click detection
    app.auth_mode_button_area = Some(area);
    f.render_widget(button_widget, area);
}

/// Helper function to create standard header layout with auth button
fn create_header_layout(area: Rect) -> std::rc::Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),     // Main title area
            Constraint::Length(35), // Auth mode button area
        ])
        .split(area)
}

/// Helper functions for consistent string formatting with reduced allocations
fn format_header_title(title: &str, sync_indicator: &str) -> String {
    // Pre-allocate capacity to avoid reallocation
    let mut result = String::with_capacity(title.len() + sync_indicator.len());
    result.push_str(title);
    result.push_str(sync_indicator);
    result
}

fn format_elapsed_time(elapsed_secs: u64) -> String {
    let minutes = elapsed_secs / 60;
    let seconds = elapsed_secs % 60;
    
    // Pre-allocate reasonable capacity (max "999m 59s" = 8 chars)
    let mut result = String::with_capacity(8);
    result.push_str(&minutes.to_string());
    result.push('m');
    result.push(' ');
    result.push_str(&seconds.to_string());
    result.push('s');
    result
}


// Add a separate function for when we need the exact count
fn format_customer_count_exact(count: usize) -> String {
    if count == 1 {
        String::from("1 customer")
    } else {
        format!("{} customers", count)
    }
}




/// Draw the complete TUI
pub fn draw_tui(f: &mut Frame, app: &mut TuiApp) {
    // Check state without borrowing to avoid conflicts
    let state = app.state.clone();

    match state {
        AppState::SyncChoice => {
            // Draw sync choice UI - this would need a proper UI function
            // For now, just draw authentication UI as placeholder
            draw_authentication_ui(f, app);
        }
        AppState::Exiting(_) => {
            // Should not reach here with immediate exit, but just in case
        }
        AppState::Authenticating => {
            draw_authentication_ui(f, app);
        }
        AppState::AuthenticationFailed(ref error) => {
            draw_auth_failed_ui(f, error);
        }
        AppState::CustomerSelection
        | AppState::TimeSelection
        | AppState::ContentSelection
        | AppState::Confirmation => {
            draw_selection_ui(f, app);
        }
        AppState::Initializing | AppState::Extracting => {
            draw_extraction_ui(f, app);
        }
        AppState::Complete => {
            draw_results_ui(f, app);
        }
        AppState::Error(ref e) => {
            draw_error_ui(f, e);
        }
    }
}

fn draw_selection_ui(f: &mut Frame, app: &mut TuiApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Main content
            Constraint::Length(2), // Footer
        ])
        .split(f.area());

    // Draw header with progress indicator
    let progress = match app.state {
        AppState::CustomerSelection => "Step 1/3: Select Customers",
        AppState::TimeSelection => "Step 2/3: Time Period",
        AppState::ContentSelection => "Step 3/3: Content Type",
        AppState::Confirmation => "Review & Confirm",
        _ => "",
    };

    // Create header with authentication mode button
    let header_chunks = create_header_layout(chunks[0]);

    // Main title
    let header_title = format_header_title(
        "CS-CLI: Customer Success Deep Research Tool",
        app.get_sync_indicator()
    );
    let header = Paragraph::new(vec![
        Line::from(vec![Span::styled(
            header_title,
            Style::default()
                .fg(THEME.primary)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            progress,
            Style::default().fg(THEME.text_secondary),
        )]),
    ])
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(header, header_chunks[0]);

    // Authentication mode button
    draw_auth_mode_button(f, app, header_chunks[1]);

    // Draw main content based on state
    match app.state {
        AppState::SyncChoice => draw_sync_choice(f, chunks[1], app),
        AppState::CustomerSelection => draw_customer_selection(f, chunks[1], app),
        AppState::TimeSelection => draw_time_selection(f, chunks[1], app),
        AppState::ContentSelection => draw_content_selection(f, chunks[1], app),
        AppState::Confirmation => draw_confirmation(f, chunks[1], app),
        _ => {}
    }

    // Draw footer
    draw_footer(f, chunks[2], &app.state);
}

fn draw_customer_selection(f: &mut Frame, area: Rect, app: &mut TuiApp) {
    // Create layout based on whether we have an error message
    let chunks = if app.error_message.is_some() {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),    // Selected customers
                Constraint::Length(3), // Input field
                Constraint::Length(2), // Error message
                Constraint::Min(5),    // Suggestions
            ])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),    // Selected customers
                Constraint::Length(3), // Input field
                Constraint::Min(5),    // Suggestions
            ])
            .split(area)
    };

    // Selected customers
    let selected_text = if app.selected_customers.is_empty() {
        vec![Line::from(vec![Span::styled(
            "No customers selected - use TAB to select from dropdown",
            Style::default().fg(THEME.text_muted),
        )])]
    } else {
        let mut lines = vec![Line::from(vec![Span::styled(
            "Selected customers:",
            Style::default().fg(THEME.primary),
        )])];
        for customer in &app.selected_customers {
            lines.push(Line::from(vec![Span::styled(
                format!("  • {customer}"),
                Style::default().fg(THEME.accent),
            )]));
        }
        lines
    };

    let selected_widget = Paragraph::new(selected_text);
    f.render_widget(selected_widget, chunks[0]);

    // Input field
    let input_widget = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(THEME.text_primary))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("What customers are you looking for?")
                .title_style(Style::default().fg(THEME.primary)),
        );
    f.render_widget(input_widget, chunks[1]);
    f.set_cursor_position((chunks[1].x + app.cursor as u16 + 1, chunks[1].y + 1));

    // Error message if present
    if let Some(ref error_msg) = app.error_message {
        let error_widget = Paragraph::new(vec![Line::from(vec![
            Span::styled(
                "⚠ ",
                Style::default().fg(THEME.error).add_modifier(Modifier::BOLD),
            ),
            Span::styled(error_msg, Style::default().fg(THEME.error)),
        ])])
        .alignment(Alignment::Center);
        f.render_widget(error_widget, chunks[2]);

        // Suggestions in chunks[3] when error is present
        if !app.suggestions.is_empty() {
            let items: Vec<ListItem> = app
                .suggestions
                .iter()
                .enumerate()
                .map(|(i, s)| {
                    let mut style = Style::default();
                    if app.in_dropdown
                        && i == app.highlight_index
                        && app.highlight_index < app.suggestions.len()
                    {
                        style = style.bg(THEME.text_muted);
                    }
                    if app.selected_customers.contains(s) {
                        style = style
                            .fg(THEME.accent)
                            .add_modifier(Modifier::BOLD);
                    } else {
                        style = style.fg(THEME.text_primary);
                    }
                    ListItem::new(s.as_str()).style(style)
                })
                .collect();
            let list = List::new(items);
            // Store the actual render area for mouse hit testing
            app.suggestions_render_area = Some(chunks[3]);
            f.render_widget(list, chunks[3]);
        }
    } else {
        // No error message, suggestions in chunks[2]
        if !app.suggestions.is_empty() {
            let items: Vec<ListItem> = app
                .suggestions
                .iter()
                .enumerate()
                .map(|(i, s)| {
                    let mut style = Style::default();
                    if app.in_dropdown
                        && i == app.highlight_index
                        && app.highlight_index < app.suggestions.len()
                    {
                        style = style.bg(THEME.text_muted);
                    }
                    if app.selected_customers.contains(s) {
                        style = style
                            .fg(THEME.accent)
                            .add_modifier(Modifier::BOLD);
                    } else {
                        style = style.fg(THEME.text_primary);
                    }
                    ListItem::new(s.as_str()).style(style)
                })
                .collect();
            let list = List::new(items);
            // Store the actual render area for mouse hit testing
            app.suggestions_render_area = Some(chunks[2]);
            f.render_widget(list, chunks[2]);
        }
    }
}

fn draw_time_selection(f: &mut Frame, area: Rect, app: &TuiApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Summary
            Constraint::Length(3), // Input
            Constraint::Min(3),    // Help
        ])
        .split(area);

    // Summary
    let summary = Paragraph::new(vec![Line::from(vec![Span::styled(
        format!("Selected: {}", format_customer_count_exact(app.selected_customers.len())),
        Style::default().fg(THEME.success),
    )])])
    .alignment(Alignment::Center);
    f.render_widget(summary, chunks[0]);

    // Days input
    let days_widget = Paragraph::new(app.days_input.as_str())
        .style(Style::default().fg(THEME.text_primary))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Number of days back")
                .title_style(Style::default().fg(THEME.primary)),
        );
    f.render_widget(days_widget, chunks[1]);
    f.set_cursor_position((chunks[1].x + app.days_cursor as u16 + 1, chunks[1].y + 1));

    // Help
    let help = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Common: 30 (1 month), 90 (3 months), 180 (6 months)",
            Style::default().fg(THEME.text_muted),
        )]),
    ])
    .alignment(Alignment::Center);
    f.render_widget(help, chunks[2]);
}

fn draw_content_selection(f: &mut Frame, area: Rect, app: &TuiApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Summary
            Constraint::Min(10),   // Options
        ])
        .split(area);

    // Summary
    let summary = Paragraph::new(vec![Line::from(vec![Span::styled(
        format!(
            "Selected: {}, {} days",
            format_customer_count_exact(app.selected_customers.len()),
            app.days_input
        ),
        Style::default().fg(THEME.success),
    )])])
    .alignment(Alignment::Center);
    f.render_widget(summary, chunks[0]);

    // Options
    let options = [
        ("1. Calls only", 0),
        ("2. Emails only", 1),
        ("3. Both calls and emails (recommended)", 2),
    ];

    let items: Vec<ListItem> = options
        .iter()
        .map(|(label, idx)| {
            let style = if *idx == app.content_selection {
                Style::default()
                    .fg(THEME.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(THEME.text_primary)
            };
            let prefix = if *idx == app.content_selection {
                "▶ "
            } else {
                "  "
            };
            ListItem::new(format!("{prefix}{label}")).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("What would you like to analyze?")
            .title_style(Style::default().fg(THEME.primary)),
    );
    f.render_widget(list, chunks[1]);
}

fn draw_confirmation(f: &mut Frame, area: Rect, app: &TuiApp) {
    let content_type = match app.content_selection {
        0 => "Calls only",
        1 => "Emails only",
        _ => "Calls and emails",
    };

    let customers: Vec<String> = app.selected_customers.iter().cloned().collect();
    let customer_display = if customers.len() > 3 {
        format_customer_count_exact(customers.len())
    } else {
        customers.join(", ")
    };

    let confirmation = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Ready to extract:",
            Style::default()
                .fg(THEME.text_primary)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ✓ ", Style::default().fg(THEME.success)),
            Span::styled("Customers: ", Style::default().fg(THEME.text_primary)),
            Span::styled(
                &customer_display,
                Style::default().fg(THEME.accent),
            ),
        ]),
        Line::from(vec![
            Span::styled("  ✓ ", Style::default().fg(THEME.success)),
            Span::styled("Period: ", Style::default().fg(THEME.text_primary)),
            Span::styled(
                format!("{} days", app.days_input),
                Style::default().fg(THEME.accent),
            ),
        ]),
        Line::from(vec![
            Span::styled("  ✓ ", Style::default().fg(THEME.success)),
            Span::styled("Content: ", Style::default().fg(THEME.text_primary)),
            Span::styled(content_type, Style::default().fg(THEME.accent)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Press ENTER to begin",
            Style::default()
                .fg(THEME.success)
                .add_modifier(Modifier::BOLD),
        )]),
    ];

    let widget = Paragraph::new(confirmation)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Confirmation")
                .title_style(Style::default().fg(THEME.primary)),
        );
    f.render_widget(widget, area);
}

fn draw_extraction_ui(f: &mut Frame, app: &mut TuiApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(5), // Progress
            Constraint::Min(10),   // Log
            Constraint::Length(2), // Footer
        ])
        .split(f.area());

    // Header with authentication mode button
    let header_chunks = create_header_layout(chunks[0]);

    // Main title
    let extraction_title = format_header_title("CS-CLI: Extraction in Progress", app.get_sync_indicator());
    let header = Paragraph::new(vec![
        Line::from(vec![Span::styled(
            extraction_title,
            Style::default()
                .fg(THEME.primary)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            app.current_phase.as_str(),
            Style::default().fg(THEME.warning),
        )]),
    ])
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(header, header_chunks[0]);

    // Authentication mode button
    draw_auth_mode_button(f, app, header_chunks[1]);

    // Progress bar
    let progress_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(1),
        ])
        .split(chunks[1]);

    let elapsed = app.start_time.map(|t| t.elapsed().as_secs()).unwrap_or(0);
    let elapsed_str = format!("Elapsed: {}", format_elapsed_time(elapsed));

    let progress_label = Paragraph::new(elapsed_str)
        .alignment(Alignment::Center)
        .style(Style::default().fg(THEME.text_muted));
    f.render_widget(progress_label, progress_chunks[0]);

    // Apply smooth easing to progress animation
    let smooth_progress = easing::ease_in_out_cubic(app.current_progress);
    let progress = Gauge::default()
        .block(Block::default().borders(Borders::NONE))
        .gauge_style(Style::default().fg(THEME.accent))
        .percent((smooth_progress * 100.0) as u16)
        .label(format!("{}%", (smooth_progress * 100.0) as u16));
    f.render_widget(progress, progress_chunks[1]);

    let subtask = Paragraph::new(app.current_subtask.as_str())
        .alignment(Alignment::Center)
        .style(Style::default().fg(THEME.info));
    f.render_widget(subtask, progress_chunks[2]);

    // Activity log
    let log_height = chunks[2].height as usize;
    let start = app.extraction_log.len().saturating_sub(log_height - 2);
    let visible_log = &app.extraction_log[start..];

    let log_items: Vec<ListItem> = visible_log
        .iter()
        .map(|line| {
            let style = if line.starts_with("▶") {
                Style::default().fg(THEME.warning)
            } else if line.starts_with("  ✓") {
                Style::default().fg(THEME.success)
            } else if line.starts_with("❌") {
                Style::default().fg(THEME.error)
            } else if line.starts_with("  📄") {
                Style::default().fg(THEME.info)
            } else {
                Style::default().fg(THEME.text_primary)
            };
            ListItem::new(line.as_str()).style(style)
        })
        .collect();

    let log = List::new(log_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Activity Log")
            .title_style(Style::default().fg(THEME.primary)),
    );
    f.render_widget(log, chunks[2]);

    // Footer
    let footer = Paragraph::new("Extraction in progress... Please wait | ESC: Exit")
        .style(Style::default().fg(THEME.text_muted))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(footer, chunks[3]);
}

fn draw_results_ui(f: &mut Frame, app: &mut TuiApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Results
            Constraint::Length(2), // Footer
        ])
        .split(f.area());

    // Header with authentication mode button
    let header_chunks = create_header_layout(chunks[0]);

    // Main title
    let complete_title = format_header_title("CS-CLI: Extraction Complete!", app.get_sync_indicator());
    let header = Paragraph::new(vec![Line::from(vec![Span::styled(
        complete_title,
        Style::default()
            .fg(THEME.success)
            .add_modifier(Modifier::BOLD),
    )])])
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(header, header_chunks[0]);

    // Authentication mode button
    draw_auth_mode_button(f, app, header_chunks[1]);

    // Results
    if let Some(ref results) = app.results {
        let elapsed = app.start_time.map(|t| t.elapsed().as_secs()).unwrap_or(0);

        let content = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "✓ Extraction Complete",
                Style::default()
                    .fg(THEME.success)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                format!("Total time: {}", format_elapsed_time(elapsed)),
                Style::default().fg(THEME.text_muted),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Results Summary:",
                Style::default()
                    .fg(THEME.text_primary)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    format!("• Calls extracted: {}", results.total_calls),
                    Style::default().fg(THEME.info),
                ),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    format!("• Emails extracted: {}", results.total_emails),
                    Style::default().fg(THEME.info),
                ),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    format!("• Files saved: {}", results.files_saved),
                    Style::default().fg(THEME.info),
                ),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Output directory:",
                Style::default().fg(THEME.text_primary),
            )]),
            Line::from(vec![Span::styled(
                &results.output_directory,
                Style::default()
                    .fg(THEME.accent)
                    .add_modifier(Modifier::BOLD),
            )]),
        ];

        let results_widget = Paragraph::new(content).alignment(Alignment::Center).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Extraction Results")
                .title_style(Style::default().fg(THEME.primary)),
        );
        f.render_widget(results_widget, chunks[1]);
    }

    // Footer
    let footer = Paragraph::new("Press ENTER to exit")
        .style(Style::default().fg(THEME.success))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(footer, chunks[2]);
}

fn draw_error_ui(f: &mut Frame, error: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(2),
        ])
        .split(f.area());

    let header = Paragraph::new(vec![Line::from(vec![Span::styled(
        "CS-CLI: Error",
        Style::default().fg(THEME.error).add_modifier(Modifier::BOLD),
    )])])
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(header, chunks[0]);

    let error_widget = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "❌ An error occurred:",
            Style::default().fg(THEME.error).add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(error, Style::default().fg(THEME.text_primary))]),
    ])
    .alignment(Alignment::Center)
    .wrap(ratatui::widgets::Wrap { trim: true })
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(THEME.error)),
    );
    f.render_widget(error_widget, chunks[1]);

    let footer = Paragraph::new("Press ESC to exit")
        .style(Style::default().fg(THEME.text_muted))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(footer, chunks[2]);
}

fn draw_footer(f: &mut Frame, area: Rect, state: &AppState) {
    let help = match state {
        AppState::SyncChoice => "↑↓: Select option | ENTER: Confirm choice | ESC: Exit",
        AppState::CustomerSelection => {
            "TAB/Click: Select | ENTER: Next | ESC: Exit | Mouse: Navigate"
        }
        AppState::TimeSelection => "ENTER: Next | ESC: Exit | Mouse: Set cursor",
        AppState::ContentSelection => "↑/↓/Click or 1-3: Select | ENTER: Next | ESC: Exit",
        AppState::Confirmation => "ENTER/Click: Start | ESC: Exit",
        _ => "ESC: Exit",
    };

    let footer = Paragraph::new(help)
        .style(Style::default().fg(THEME.text_muted))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(footer, area);
}

// Postman logo ASCII art
const POSTMAN_LOGO_RAW: &str = r#"
                                     #*##***#*************#**#                                      
                                 *#**########################**#***#                                
                            ***#*##################################****#                            
                         **############################################****                         
                      **###################################################*#+                      
                   ***########################################################*#=                   
                 **###############################################################*                 
               *####################################################################*               
             *#######################################################################*#             
            ##########################################################################*#            
          ###########################################################=:   ::#-#########*#*          
         **########################################################.        ###############         
        *#########################################################        ##:  ###########**        
      *##########################################################       ###    :############*       
     ############################################################     :##    #  #############*      
    #*###########################################################:    :##    : :##############*     
    #*#######################################################*:-##      *##   :################*    
   *######################################################:     :###      =#:-#################*#   
  #####################################################:     :#:   :#############################*  
  ###################################################      :#:      ##############################  
 ##################################################       ##      :###############################+ 
 *###############################################       ##      :#: ############################### 
#*#############################################       ##      :#:  #################################
#*###########################################       ##      ##:   #################################*
*##########################################       :#:     ##:   :##################################*
*########################################       :#:     ##:    ####################################*
*######################################:##:    #=     ##:    :#####################################*
*#####################################    ##:##     ##:     #######################################*
*#################################*=  ::-#####   :##      ##########################################
*#################################=########. :####      +###########################################
*################################################:    :############################################*
*#######################################: :      ##::#############################################*#
######################################            ################################################*#
 *##################################            ##################################################* 
 **###############################:          :####################################################* 
  ##############################:          ######################################################*  
  *###########################=         :########################################################*  
   ##########################        :##########################################################*   
    #######################       :############################################################*    
    #*###################:    :#################################################################    
     *#################:#*   #################################################################*     
      #*#############:   ##: :#############################################################*#*      
       ###########*#:      ##:############################################################*##       
         ##########=*######=-#############################################################*         
          **#############################################################################*          
            #*########################################################################*#*           
             #*########################################################################             
               *##################################################################**#               
                 *################################################################*                 
                   =#*#######################################################****                   
                      ***##################################################***                      
                         ***############################################***                         
                            =**####################################***#*                            
                                 ****#########################***#*                                 
                                       **#**#**********#***#**#                                     
"#;
/// Draw authentication UI with progress and overlay auth choice buttons
fn draw_authentication_ui(f: &mut Frame, app: &mut TuiApp) {
    // Calculate logo animation progress with smooth easing (3 second animation)
    let logo_progress = if let Some(start_time) = app.auth_start_time {
        let elapsed = start_time.elapsed().as_millis() as f64;
        let raw_progress = (elapsed / 3000.0).min(1.0); // 3 second animation
        easing::ease_out_cubic(raw_progress) // Apply smooth easing
    } else {
        0.0
    };

    // First, draw the Postman logo background with time-based reveal effect
    draw_postman_logo_background(f, logo_progress);

    // Create compact layout centered on screen
    let center_y = f.area().height / 2;
    let compact_height = 15; // Total height for header + progress + footer + spacers
    let start_y = center_y.saturating_sub(compact_height / 2) + 2; // Shifted down by 2 rows

    let compact_area = Rect {
        x: f.area().x,
        y: start_y,
        width: f.area().width,
        height: compact_height,
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6), // Top spacer (move header down)
            Constraint::Length(1), // Header (single line)
            Constraint::Length(0), // Spacer
            Constraint::Length(1), // Progress bar (single row)
            Constraint::Length(0), // Spacer
            Constraint::Length(1), // Footer (single line)
            Constraint::Length(6), // Bottom spacer (move footer up)
        ])
        .split(compact_area);

    // Header with full-width orange background - single row with all info
    let sync_status = if app.sync_enabled {
        "Synced"
    } else {
        "Not Synced"
    };

    // Combine all header text into one line with separators
    let header_text = format!(
        "{} | CS-CLI: Authenticating | Step {}/{}: {}",
        sync_status, app.auth_current_step, app.auth_total_steps, app.auth_status
    );

    // Create full-width span that fills the entire header area
    let header_width = chunks[1].width as usize;

    let header = Paragraph::new(vec![Line::from(vec![Span::styled(
        format!("{:^width$}", header_text, width = header_width),
        Style::default()
            .fg(THEME.background)
            .bg(THEME.text_primary)
            .add_modifier(Modifier::BOLD),
    )])])
    .block(
        Block::default()
            .borders(Borders::NONE)
            .style(Style::default().bg(THEME.text_primary)), // White background for entire block
    );
    f.render_widget(header, chunks[1]); // Header is now at index 1

    // Custom progress bar that preserves logo colors underneath
    let progress_area = chunks[3];
    let progress_width = progress_area.width as f64;
    let filled_width = (progress_width * app.auth_progress) as u16;

    // Only draw white blocks over the filled portion
    if filled_width > 0 {
        let filled_text = "█".repeat(filled_width as usize);
        let progress_widget = Paragraph::new(filled_text).style(Style::default().fg(THEME.text_primary));

        let filled_area = Rect {
            x: progress_area.x,
            y: progress_area.y,
            width: filled_width.min(progress_area.width),
            height: 1,
        };
        f.render_widget(progress_widget, filled_area);
    }
    // The unfilled portion will show the logo colors underneath

    // Overlay button and text (positioning relative to overall frame)
    let overlay_center_x = f.area().width / 2;
    let overlay_center_y = (f.area().height as f32 * 0.8) as u16; // 80% down

    let button_text = "Auth via GitHub";
    let button_width = button_text.len() as u16 + 8;
    let button_height = 3;

    let button_x = overlay_center_x.saturating_sub(button_width / 2);
    let button_y = overlay_center_y.saturating_sub(button_height / 2);

    let button_area = Rect {
        x: button_x,
        y: button_y,
        width: button_width,
        height: button_height,
    };

    // Store button area for precise mouse detection
    app.github_button_area = Some(button_area);

    // Apply hover effect
    let bg_color = if app.github_button_hovered {
        THEME.accent // Lighter orange on hover
    } else {
        THEME.postman_orange // Official Postman Orange background
    };

    let padded_button_text = format!("{:^width$}", button_text, width = button_width as usize);

    let button_widget = Paragraph::new(vec![
        Line::from(vec![Span::styled(
            format!("{:^width$}", "", width = button_width as usize),
            Style::default().bg(bg_color),
        )]),
        Line::from(vec![Span::styled(
            padded_button_text,
            Style::default()
                .fg(THEME.background)
                .bg(bg_color)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            format!("{:^width$}", "", width = button_width as usize),
            Style::default().bg(bg_color),
        )]),
    ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::NONE)
            .style(Style::default().bg(bg_color)),
    );
    f.render_widget(button_widget, button_area);

    let local_text = " or local (won't sync) ";
    let local_text_width = local_text.len() as u16;
    let local_text_x = overlay_center_x.saturating_sub(local_text_width / 2);
    let local_text_y = button_y + button_height;

    let local_text_area = Rect {
        x: local_text_x,
        y: local_text_y,
        width: local_text_width,
        height: 1,
    };

    // Store local text area for precise mouse detection
    app.local_text_area = Some(local_text_area);

    let text_color = if app.local_text_hovered {
        THEME.text_hover
    } else {
        THEME.text_disabled
    };

    let local_text_widget = Paragraph::new(local_text)
        .style(Style::default().fg(text_color))
        .alignment(Alignment::Center);
    f.render_widget(local_text_widget, local_text_area);

    // Footer with white background to match header
    let footer = Paragraph::new(vec![Line::from(vec![Span::styled(
        format!(
            "{:^width$}",
            "G: GitHub | L: Local | ESC: Exit",
            width = chunks[5].width as usize
        ),
        Style::default()
            .fg(THEME.background)
            .bg(THEME.text_primary)
            .add_modifier(Modifier::BOLD),
    )])])
    .block(
        Block::default()
            .borders(Borders::NONE)
            .style(Style::default().bg(THEME.text_primary)), // White background for entire block
    );
    f.render_widget(footer, chunks[5]); // Footer is now at index 5
}

/// Pre-computed logo data for optimal performance
struct LogoCache {
    lines: Vec<String>,
    max_width: usize,
    gradient_colors: Vec<(u8, u8, u8)>, // Pre-computed gradient colors
}

impl LogoCache {
    fn new() -> Self {
        let logo_lines: Vec<&str> = POSTMAN_LOGO_RAW.lines().skip(1).collect();
        let max_width = logo_lines.iter().map(|line| line.len()).max().unwrap_or(0);
        
        // Pre-compute gradient colors for each line
        let gradient_colors: Vec<(u8, u8, u8)> = logo_lines
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let vertical_progress = i as f64 / logo_lines.len() as f64;
                let t = vertical_progress.min(1.0);
                // Gradient from #FF6C37 (255, 108, 55) to #FFB400 (255, 180, 0)
                let r = 255u8;
                let g = (108.0 + t * 72.0) as u8;
                let b = (55.0 - t * 55.0) as u8;
                (r, g, b)
            })
            .collect();
        
        Self {
            lines: logo_lines.iter().map(|s| s.to_string()).collect(),
            max_width,
            gradient_colors,
        }
    }
}

/// Draw Postman logo background with optimized left-to-right reveal effect
fn draw_postman_logo_background(f: &mut Frame, progress: f64) {
    let area = f.area();

    // Use cached logo data
    static LOGO_CACHE: std::sync::OnceLock<LogoCache> = std::sync::OnceLock::new();
    let cache = LOGO_CACHE.get_or_init(LogoCache::new);

    let reveal_width = if progress >= 1.0 {
        cache.max_width // Show full logo when animation complete
    } else {
        (cache.max_width as f64 * progress) as usize
    };

    // Calculate logo dimensions and center position
    let logo_height = cache.lines.len() as u16;
    let logo_width = cache.max_width as u16;

    // Calculate the offset to center the logo (can be negative)
    // This positions the logo's center at the screen's center
    let vertical_offset = (area.height as i16 - logo_height as i16) / 2;
    let horizontal_offset = (area.width as i16 - logo_width as i16) / 2;

    // Draw each line of the logo
    for (i, logo_line) in cache.lines.iter().enumerate() {
        // Calculate the y position for this line (could be negative)
        let y_pos = vertical_offset + i as i16;

        // Skip lines that are completely outside the visible area
        if y_pos < 0 || y_pos >= area.height as i16 {
            continue;
        }

        // Create the revealed portion of the line
        let mut revealed_line = String::new();
        for (idx, ch) in logo_line.chars().enumerate() {
            if idx < reveal_width {
                revealed_line.push(ch);
            } else {
                revealed_line.push(' ');
            }
        }

        // Calculate what portion of the line to show based on horizontal centering
        let line_start_x = horizontal_offset;
        let line_chars: Vec<char> = revealed_line.chars().collect();

        // Determine which characters are visible
        let mut visible_line = String::new();
        let mut render_x = 0u16;

        for (char_idx, ch) in line_chars.iter().enumerate() {
            let char_x = line_start_x + char_idx as i16;

            // Only include characters that are within screen bounds
            if char_x >= 0 && char_x < area.width as i16 {
                if visible_line.is_empty() {
                    // First visible character determines render position
                    render_x = char_x as u16;
                }
                visible_line.push(*ch);
            }
        }

        // Skip if no visible characters
        if visible_line.is_empty() {
            continue;
        }

        // Optimized: Use pre-computed gradient colors
        let mut spans = Vec::new();
        let has_logo_chars = visible_line.chars().any(|ch| ch != ' ');
        let visible_line_len = visible_line.len();

        if has_logo_chars {
            // Use pre-computed gradient color for this line
            let (r, g, b) = cache.gradient_colors[i];

            spans.push(Span::styled(
                visible_line,
                Style::default().fg(Color::Rgb(r, g, b)),
            ));
        } else {
            // Empty line - just render as-is
            spans.push(Span::raw(visible_line));
        }

        // Render the line at the calculated position
        let logo_widget = Paragraph::new(Line::from(spans));
        let line_area = Rect {
            x: render_x,
            y: y_pos as u16,
            width: visible_line_len as u16,
            height: 1,
        };
        f.render_widget(logo_widget, line_area);
    }
}

/// Draw authentication failed UI with retry option
fn draw_auth_failed_ui(f: &mut Frame, error: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Error content
            Constraint::Length(2), // Footer
        ])
        .split(f.area());

    // Header
    let header = Paragraph::new(vec![Line::from(vec![Span::styled(
        "CS-CLI: Authentication Failed",
        Style::default().fg(THEME.error).add_modifier(Modifier::BOLD),
    )])])
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(header, chunks[0]);

    // Error content
    let error_content = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "❌ Authentication Error",
            Style::default().fg(THEME.error).add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(error, Style::default().fg(THEME.text_primary))]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "💡 Troubleshooting:",
            Style::default()
                .fg(THEME.warning)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            "• Ensure Okta Verify is installed and configured",
            Style::default().fg(THEME.text_secondary),
        )]),
        Line::from(vec![Span::styled(
            "• Check that you have the proper application permissions in Okta",
            Style::default().fg(THEME.text_secondary),
        )]),
        Line::from(vec![Span::styled(
            "• Verify your network connection",
            Style::default().fg(THEME.text_secondary),
        )]),
    ])
    .alignment(Alignment::Center)
    .wrap(ratatui::widgets::Wrap { trim: true })
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(THEME.error)),
    );
    f.render_widget(error_content, chunks[1]);

    // Footer
    let footer = Paragraph::new("Press R or ENTER to retry | ESC: Exit")
        .style(Style::default().fg(THEME.text_muted))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(footer, chunks[2]);
}

/// Draw sync choice screen
fn draw_sync_choice(f: &mut Frame, area: Rect, app: &TuiApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8), // Title and description
            Constraint::Length(6), // Options
            Constraint::Min(0),    // Spacer
        ])
        .split(area);

    // Title and description
    let title_content = vec![
        Line::from(vec![Span::styled(
            "Session Sync",
            Style::default()
                .fg(THEME.primary)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Keep your login sessions synchronized across all your devices",
            Style::default().fg(THEME.text_secondary),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Choose your preferred sync method:",
            Style::default().fg(THEME.warning),
        )]),
    ];

    let title_widget = Paragraph::new(title_content)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(title_widget, chunks[0]);

    // Sync options
    let github_option = if app.sync_choice_selection == 0 {
        vec![
            Line::from(vec![Span::styled(
                "🔗 GitHub OAuth Sync",
                Style::default()
                    .fg(THEME.primary)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                "   Encrypted storage in your personal GitHub gist",
                Style::default().fg(THEME.button_bg),
            )]),
            Line::from(vec![Span::styled(
                "   Works across all devices, secure and private",
                Style::default().fg(THEME.button_bg),
            )]),
        ]
    } else {
        vec![
            Line::from(vec![Span::styled(
                "GitHub OAuth (sync)",
                Style::default().fg(THEME.text_muted),
            )]),
            Line::from(vec![Span::styled(
                "   Encrypted storage in your personal GitHub gist",
                Style::default().fg(THEME.text_disabled),
            )]),
            Line::from(vec![Span::styled(
                "   Works across all devices, secure and private",
                Style::default().fg(THEME.text_disabled),
            )]),
        ]
    };

    let local_option = if app.sync_choice_selection == 1 {
        vec![
            Line::from(vec![Span::styled(
                "Local Only (nosync)",
                Style::default()
                    .fg(THEME.primary)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                "   Store sessions locally in macOS keychain",
                Style::default().fg(THEME.button_bg),
            )]),
            Line::from(vec![Span::styled(
                "   No external dependencies, completely offline",
                Style::default().fg(THEME.button_bg),
            )]),
        ]
    } else {
        vec![
            Line::from(vec![Span::styled(
                "🔒 Local Keychain Only",
                Style::default().fg(THEME.text_muted),
            )]),
            Line::from(vec![Span::styled(
                "   Store sessions locally in macOS keychain",
                Style::default().fg(THEME.text_disabled),
            )]),
            Line::from(vec![Span::styled(
                "   No external dependencies, completely offline",
                Style::default().fg(THEME.text_disabled),
            )]),
        ]
    };

    let mut options_content = github_option;
    options_content.push(Line::from(""));
    options_content.extend(local_option);

    let options_widget = Paragraph::new(options_content)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(options_widget, chunks[1]);

    // Instructions
    let instructions = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Use ↑↓ to select, ENTER to confirm",
            Style::default().fg(THEME.text_muted),
        )]),
    ])
    .alignment(Alignment::Center);
    f.render_widget(instructions, chunks[2]);
}
