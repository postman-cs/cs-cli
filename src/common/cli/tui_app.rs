//! Complete TUI application that handles the entire workflow
//!
//! This module provides a full-featured TUI that manages:
//! - Customer selection with autocomplete
//! - Time period and content type selection
//! - Extraction progress with loading bars
//! - Results summary

use crate::common::cli::args::{ContentType, ParsedCommand};
use crossterm::event::{KeyCode, MouseButton, MouseEvent, MouseEventKind};
use crossterm::terminal;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
};
use std::collections::HashSet;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::Instant;
use tokio::sync::mpsc;

/// Result type for TUI operations
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Debug logging function that writes to a file without interfering with TUI
fn debug_log(message: &str) {
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("mouse_debug.log")
    {
        let _ = writeln!(file, "{message}");
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
}

/// Complete application state
pub struct TuiApp {
    /// Current state in the workflow
    pub state: AppState,

    // Authentication
    pub auth_progress: f64,
    pub auth_status: String,
    pub auth_error: Option<String>,

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

    // Layout coordinates (for better mouse handling)
    pub last_suggestion_area: Option<ratatui::layout::Rect>,
    pub last_time_input_area: Option<ratatui::layout::Rect>,
    pub last_content_area: Option<ratatui::layout::Rect>,

    // Store the actual rendered area of the suggestions list
    pub suggestions_render_area: Option<ratatui::layout::Rect>,
}

impl Default for TuiApp {
    fn default() -> Self {
        Self::new()
    }
}

impl TuiApp {
    pub fn new() -> Self {
        // Clear debug log file and start new session
        let _ = std::fs::write("mouse_debug.log", "=== NEW TUI SESSION ===\n");

        Self {
            state: AppState::Authenticating,
            auth_progress: 0.0,
            auth_status: "Starting authentication...".to_string(),
            auth_error: None,
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
            last_suggestion_area: None,
            last_time_input_area: None,
            last_content_area: None,
            suggestions_render_area: None,
        }
    }

    pub fn set_extraction_channel(&mut self, rx: mpsc::UnboundedReceiver<ExtractionMessage>) {
        self.extraction_rx = Some(rx);
    }

    pub fn handle_extraction_message(&mut self, msg: ExtractionMessage) {
        match msg {
            // Authentication messages
            ExtractionMessage::AuthProgress(progress, status) => {
                self.auth_progress = progress;
                self.auth_status = status;
            }
            ExtractionMessage::AuthSuccess => {
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
        match self.state {
            AppState::Authenticating => false, // No input during auth
            AppState::AuthenticationFailed(_) => {
                match key {
                    KeyCode::Char('r') | KeyCode::Enter => {
                        // Retry authentication
                        self.state = AppState::Authenticating;
                        self.auth_progress = 0.0;
                        self.auth_status = "Retrying authentication...".to_string();
                        self.auth_error = None;
                        false
                    }
                    KeyCode::Esc => true, // Exit
                    _ => false,
                }
            }
            AppState::CustomerSelection => self.handle_customer_input(key),
            AppState::TimeSelection => self.handle_time_input(key),
            AppState::ContentSelection => self.handle_content_input(key),
            AppState::Confirmation => self.handle_confirmation_input(key),
            AppState::Complete | AppState::Error(_) => {
                matches!(key, KeyCode::Enter | KeyCode::Esc)
            }
            _ => false,
        }
    }

    /// Handle mouse events
    pub fn handle_mouse(&mut self, mouse: MouseEvent) -> bool {
        match self.state {
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
            KeyCode::Esc => {
                self.selected_customers.clear();
                true // Exit
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
            KeyCode::Esc => {
                self.state = AppState::CustomerSelection;
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
            KeyCode::Esc => {
                self.state = AppState::TimeSelection;
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
            KeyCode::Esc => {
                self.state = AppState::ContentSelection;
                false
            }
            _ => false,
        }
    }

    fn move_up(&mut self) {
        if !self.suggestions.is_empty() && !self.suggestions.is_empty() {
            if !self.in_dropdown {
                self.in_dropdown = true;
                self.highlight_index = self.suggestions.len().saturating_sub(1);
            } else if self.highlight_index > 0 {
                self.highlight_index = self.highlight_index.saturating_sub(1);
            }
            // Safety check to prevent out of bounds
            if self.highlight_index >= self.suggestions.len() {
                self.highlight_index = self.suggestions.len().saturating_sub(1);
            }
        }
    }

    fn move_down(&mut self) {
        if !self.suggestions.is_empty() && !self.suggestions.is_empty() {
            if !self.in_dropdown {
                self.in_dropdown = true;
                self.highlight_index = 0;
            } else if self.highlight_index < self.suggestions.len().saturating_sub(1) {
                self.highlight_index =
                    (self.highlight_index + 1).min(self.suggestions.len().saturating_sub(1));
            }
            // Safety check to prevent out of bounds
            if self.highlight_index >= self.suggestions.len() {
                self.highlight_index = self.suggestions.len().saturating_sub(1);
            }
        }
    }

    fn toggle_selection(&mut self) {
        // Clean implementation - no debug logging for normal operation
        if self.in_dropdown
            && !self.suggestions.is_empty()
            && self.highlight_index < self.suggestions.len()
            && !self.suggestions.is_empty()
        {
            // Double-check bounds before array access
            if let Some(item) = self.suggestions.get(self.highlight_index) {
                let item = item.clone();
                if self.selected_customers.contains(&item) {
                    self.selected_customers.remove(&item);
                } else {
                    self.selected_customers.insert(item);
                }
            }
        }
    }

    pub fn update_suggestions(&mut self, suggestions: Vec<String>) {
        self.suggestions = suggestions;
        if self.highlight_index >= self.suggestions.len() && !self.suggestions.is_empty() {
            self.highlight_index = 0;
        }
    }

    fn handle_dropdown_click(&mut self, y: usize, is_click: bool) -> bool {
        // Multiple safety checks to prevent crashes
        if self.suggestions.is_empty() || self.suggestions.is_empty() {
            return false;
        }

        // Ensure highlight_index is always valid
        if self.highlight_index >= self.suggestions.len() {
            self.highlight_index = 0;
        }

        // If dropdown isn't open but we have suggestions, open it first
        if !self.in_dropdown {
            self.in_dropdown = true;
        }

        // Use the stored render area if available (set during draw_customer_selection)
        if let Some(area) = self.suggestions_render_area {
            // Check if mouse y coordinate is within the suggestions area
            if y >= area.y as usize && y < (area.y + area.height) as usize {
                // Convert to widget-relative coordinates
                let relative_y = y.saturating_sub(area.y as usize);

                // Direct mapping: each row in the List corresponds to one suggestion
                let suggestion_index = relative_y;

                // Debug logging for clicks
                if is_click {
                    let terminal_size = terminal::size().unwrap_or((80, 24));
                    debug_log(&format!(
                        "CLICK: Terminal {}x{}, y={}, area.y={}, relative_y={}, index={}",
                        terminal_size.0, terminal_size.1, y, area.y, relative_y, suggestion_index
                    ));
                    if suggestion_index < self.suggestions.len() {
                        debug_log(&format!(
                            "  -> Mapped to item {} ('{}')\n",
                            suggestion_index, self.suggestions[suggestion_index]
                        ));
                    }
                }

                // Bounds check
                if suggestion_index < self.suggestions.len() {
                    if is_click {
                        self.highlight_index = suggestion_index;
                        self.toggle_selection();
                        self.error_message = None;
                        self.in_dropdown = true;
                        return false; // Don't trigger redraw to avoid crash
                    } else {
                        // Hover effect
                        if suggestion_index != self.highlight_index {
                            self.highlight_index = suggestion_index;
                            return false; // Don't trigger redraw to avoid crash
                        }
                    }
                }
            }
        }

        false
    }

    fn handle_customer_mouse(&mut self, mouse: MouseEvent) -> bool {
        let row = mouse.row as usize;
        let col = mouse.column as usize;

        // Comprehensive coordinate validation
        if row > 200 || col > 500 || row == 0 {
            return false;
        }

        // Additional safety for suggestions state
        if self.highlight_index >= self.suggestions.len() && !self.suggestions.is_empty() {
            self.highlight_index = 0;
        }

        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                // Simple click logging
                let terminal_size = terminal::size().unwrap_or((80, 24));
                debug_log(&format!(
                    "Click: {}x{} at row {} -> ",
                    terminal_size.0, terminal_size.1, row
                ));

                // Always handle clicks
                self.handle_dropdown_click(row, true)
            }
            MouseEventKind::Moved => {
                // Process hover if we have suggestions (no logging for moves)
                if !self.suggestions.is_empty() && !self.suggestions.is_empty() {
                    if !self.in_dropdown {
                        self.in_dropdown = true;
                    }
                    self.handle_dropdown_click(row, false)
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

/// Draw the complete TUI
pub fn draw_tui(f: &mut Frame, app: &mut TuiApp) {
    match app.state {
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

    let header = Paragraph::new(vec![
        Line::from(vec![Span::styled(
            "CS-CLI: Customer Success Deep Research Tool",
            Style::default()
                .fg(Color::Rgb(255, 108, 55))
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            progress,
            Style::default().fg(Color::Rgb(230, 230, 230)),
        )]),
    ])
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(header, chunks[0]);

    // Draw main content based on state
    match app.state {
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
            Style::default().fg(Color::DarkGray),
        )])]
    } else {
        let mut lines = vec![Line::from(vec![Span::styled(
            "Selected customers:",
            Style::default().fg(Color::Rgb(255, 108, 55)),
        )])];
        for customer in &app.selected_customers {
            lines.push(Line::from(vec![Span::styled(
                format!("  • {customer}"),
                Style::default().fg(Color::Rgb(255, 142, 100)),
            )]));
        }
        lines
    };

    let selected_widget = Paragraph::new(selected_text);
    f.render_widget(selected_widget, chunks[0]);

    // Input field
    let input_widget = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("What customers are you looking for?")
                .title_style(Style::default().fg(Color::Rgb(111, 44, 186))),
        );
    f.render_widget(input_widget, chunks[1]);
    f.set_cursor_position((chunks[1].x + app.cursor as u16 + 1, chunks[1].y + 1));

    // Error message if present
    if let Some(ref error_msg) = app.error_message {
        let error_widget = Paragraph::new(vec![Line::from(vec![
            Span::styled(
                "⚠ ",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::styled(error_msg, Style::default().fg(Color::Red)),
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
                        style = style.bg(Color::DarkGray);
                    }
                    if app.selected_customers.contains(s) {
                        style = style
                            .fg(Color::Rgb(255, 142, 100))
                            .add_modifier(Modifier::BOLD);
                    } else {
                        style = style.fg(Color::White);
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
                        style = style.bg(Color::DarkGray);
                    }
                    if app.selected_customers.contains(s) {
                        style = style
                            .fg(Color::Rgb(255, 142, 100))
                            .add_modifier(Modifier::BOLD);
                    } else {
                        style = style.fg(Color::White);
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
        format!("Selected: {} customer(s)", app.selected_customers.len()),
        Style::default().fg(Color::Green),
    )])])
    .alignment(Alignment::Center);
    f.render_widget(summary, chunks[0]);

    // Days input
    let days_widget = Paragraph::new(app.days_input.as_str())
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Number of days back")
                .title_style(Style::default().fg(Color::Rgb(111, 44, 186))),
        );
    f.render_widget(days_widget, chunks[1]);
    f.set_cursor_position((chunks[1].x + app.days_cursor as u16 + 1, chunks[1].y + 1));

    // Help
    let help = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Common: 30 (1 month), 90 (3 months), 180 (6 months)",
            Style::default().fg(Color::DarkGray),
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
            "Selected: {} customer(s), {} days",
            app.selected_customers.len(),
            app.days_input
        ),
        Style::default().fg(Color::Green),
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
                    .fg(Color::Rgb(255, 142, 100))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
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
            .title_style(Style::default().fg(Color::Rgb(111, 44, 186))),
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
        format!("{} customers", customers.len())
    } else {
        customers.join(", ")
    };

    let confirmation = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Ready to extract:",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ✓ ", Style::default().fg(Color::Green)),
            Span::styled("Customers: ", Style::default().fg(Color::White)),
            Span::styled(
                &customer_display,
                Style::default().fg(Color::Rgb(255, 142, 100)),
            ),
        ]),
        Line::from(vec![
            Span::styled("  ✓ ", Style::default().fg(Color::Green)),
            Span::styled("Period: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("{} days", app.days_input),
                Style::default().fg(Color::Rgb(255, 142, 100)),
            ),
        ]),
        Line::from(vec![
            Span::styled("  ✓ ", Style::default().fg(Color::Green)),
            Span::styled("Content: ", Style::default().fg(Color::White)),
            Span::styled(content_type, Style::default().fg(Color::Rgb(255, 142, 100))),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Press ENTER to begin",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )]),
    ];

    let widget = Paragraph::new(confirmation)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Confirmation")
                .title_style(Style::default().fg(Color::Rgb(111, 44, 186))),
        );
    f.render_widget(widget, area);
}

fn draw_extraction_ui(f: &mut Frame, app: &TuiApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(5), // Progress
            Constraint::Min(10),   // Log
            Constraint::Length(2), // Footer
        ])
        .split(f.area());

    // Header
    let header = Paragraph::new(vec![
        Line::from(vec![Span::styled(
            "CS-CLI: Extraction in Progress",
            Style::default()
                .fg(Color::Rgb(255, 108, 55))
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            app.current_phase.as_str(),
            Style::default().fg(Color::Yellow),
        )]),
    ])
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(header, chunks[0]);

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
    let elapsed_str = format!("Elapsed: {}m {}s", elapsed / 60, elapsed % 60);

    let progress_label = Paragraph::new(elapsed_str)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(progress_label, progress_chunks[0]);

    let progress = Gauge::default()
        .block(Block::default().borders(Borders::NONE))
        .gauge_style(Style::default().fg(Color::Rgb(255, 142, 100)))
        .percent((app.current_progress * 100.0) as u16)
        .label(format!("{}%", (app.current_progress * 100.0) as u16));
    f.render_widget(progress, progress_chunks[1]);

    let subtask = Paragraph::new(app.current_subtask.as_str())
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Cyan));
    f.render_widget(subtask, progress_chunks[2]);

    // Activity log
    let log_height = chunks[2].height as usize;
    let start = app.extraction_log.len().saturating_sub(log_height - 2);
    let visible_log = &app.extraction_log[start..];

    let log_items: Vec<ListItem> = visible_log
        .iter()
        .map(|line| {
            let style = if line.starts_with("▶") {
                Style::default().fg(Color::Yellow)
            } else if line.starts_with("  ✓") {
                Style::default().fg(Color::Green)
            } else if line.starts_with("❌") {
                Style::default().fg(Color::Red)
            } else if line.starts_with("  📄") {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(line.as_str()).style(style)
        })
        .collect();

    let log = List::new(log_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Activity Log")
            .title_style(Style::default().fg(Color::Rgb(111, 44, 186))),
    );
    f.render_widget(log, chunks[2]);

    // Footer
    let footer = Paragraph::new("Extraction in progress... Please wait")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(footer, chunks[3]);
}

fn draw_results_ui(f: &mut Frame, app: &TuiApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Results
            Constraint::Length(2), // Footer
        ])
        .split(f.area());

    // Header
    let header = Paragraph::new(vec![Line::from(vec![Span::styled(
        "CS-CLI: Extraction Complete!",
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    )])])
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(header, chunks[0]);

    // Results
    if let Some(ref results) = app.results {
        let elapsed = app.start_time.map(|t| t.elapsed().as_secs()).unwrap_or(0);

        let content = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "✓ Extraction Complete",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                format!("Total time: {}m {}s", elapsed / 60, elapsed % 60),
                Style::default().fg(Color::DarkGray),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Results Summary:",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    format!("• Calls extracted: {}", results.total_calls),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    format!("• Emails extracted: {}", results.total_emails),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    format!("• Files saved: {}", results.files_saved),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Output directory:",
                Style::default().fg(Color::White),
            )]),
            Line::from(vec![Span::styled(
                &results.output_directory,
                Style::default()
                    .fg(Color::Rgb(255, 142, 100))
                    .add_modifier(Modifier::BOLD),
            )]),
        ];

        let results_widget = Paragraph::new(content).alignment(Alignment::Center).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Extraction Results")
                .title_style(Style::default().fg(Color::Rgb(111, 44, 186))),
        );
        f.render_widget(results_widget, chunks[1]);
    }

    // Footer
    let footer = Paragraph::new("Press ENTER to exit")
        .style(Style::default().fg(Color::Green))
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
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
    )])])
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(header, chunks[0]);

    let error_widget = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "❌ An error occurred:",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(error, Style::default().fg(Color::White))]),
    ])
    .alignment(Alignment::Center)
    .wrap(ratatui::widgets::Wrap { trim: true })
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red)),
    );
    f.render_widget(error_widget, chunks[1]);

    let footer = Paragraph::new("Press ESC to exit")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(footer, chunks[2]);
}

fn draw_footer(f: &mut Frame, area: Rect, state: &AppState) {
    let help = match state {
        AppState::CustomerSelection => {
            "TAB/Click: Select | ENTER: Next | ESC: Cancel | Mouse: Navigate"
        }
        AppState::TimeSelection => "ENTER: Next | ESC: Back | Mouse: Set cursor",
        AppState::ContentSelection => "↑/↓/Click or 1-3: Select | ENTER: Next | ESC: Back",
        AppState::Confirmation => "ENTER/Click: Start | ESC: Back",
        _ => "",
    };

    let footer = Paragraph::new(help)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(footer, area);
}

/// Draw authentication UI with progress
fn draw_authentication_ui(f: &mut Frame, app: &TuiApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Auth content
            Constraint::Length(2), // Footer
        ])
        .split(f.area());

    // Header
    let header = Paragraph::new(vec![
        Line::from(vec![Span::styled(
            "CS-CLI: Authenticating",
            Style::default()
                .fg(Color::Rgb(255, 108, 55))
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            "Connecting to Gong...",
            Style::default().fg(Color::Rgb(230, 230, 230)),
        )]),
    ])
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(header, chunks[0]);

    // Content area
    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8), // Auth status
            Constraint::Length(3), // Progress bar
        ])
        .split(chunks[1]);

    // Authentication status
    let auth_content = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "🔐 Authentication Progress",
            Style::default()
                .fg(Color::Rgb(111, 44, 186))
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            &app.auth_status,
            Style::default().fg(Color::White),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Please ensure you're logged into Gong in your browser",
            Style::default().fg(Color::DarkGray),
        )]),
    ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(111, 44, 186))),
    );
    f.render_widget(auth_content, content_chunks[0]);

    // Progress bar
    let progress_percent = (app.auth_progress * 100.0) as u16;
    let progress_bar = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("Progress"))
        .gauge_style(Style::default().fg(Color::Rgb(111, 44, 186)))
        .percent(progress_percent)
        .label(format!("{progress_percent}%"));
    f.render_widget(progress_bar, content_chunks[1]);

    // Footer
    let footer = Paragraph::new("Please wait... Authentication in progress")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(footer, chunks[2]);
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
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
    )])])
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(header, chunks[0]);

    // Error content
    let error_content = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "❌ Authentication Error",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(error, Style::default().fg(Color::White))]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "💡 Troubleshooting:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            "• Make sure you're logged into Gong in your browser",
            Style::default().fg(Color::Rgb(230, 230, 230)),
        )]),
        Line::from(vec![Span::styled(
            "• Try logging out and back in to Gong",
            Style::default().fg(Color::Rgb(230, 230, 230)),
        )]),
        Line::from(vec![Span::styled(
            "• Check your network connection",
            Style::default().fg(Color::Rgb(230, 230, 230)),
        )]),
    ])
    .alignment(Alignment::Center)
    .wrap(ratatui::widgets::Wrap { trim: true })
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red)),
    );
    f.render_widget(error_content, chunks[1]);

    // Footer
    let footer = Paragraph::new("Press R or ENTER to retry | ESC to exit")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(footer, chunks[2]);
}
