//! Custom multi-select autocomplete widget using ratatui
//!
//! This module provides a rich terminal UI for selecting multiple customers
//! with autocomplete functionality, TAB to select/deselect, and visual feedback.

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::collections::HashSet;
use std::io;
use std::time::Duration;

/// Result type for multi-select operations
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Multi-select autocomplete state
pub struct MultiSelectState {
    /// Current input text
    pub input: String,
    /// Cursor position in input
    pub cursor: usize,
    /// Currently available suggestions
    pub suggestions: Vec<String>,
    /// Selected items (via TAB)
    pub selected: HashSet<String>,
    /// Current highlighted suggestion index
    pub highlight_index: usize,
    /// Whether we're in the dropdown
    pub in_dropdown: bool,
}

impl Default for MultiSelectState {
    fn default() -> Self {
        Self::new()
    }
}

impl MultiSelectState {
    /// Create a new multi-select state
    pub fn new() -> Self {
        Self {
            input: String::new(),
            cursor: 0,
            suggestions: Vec::new(),
            selected: HashSet::new(),
            highlight_index: 0,
            in_dropdown: false,
        }
    }

    /// Add character to input at cursor position
    pub fn add_char(&mut self, c: char) {
        self.input.insert(self.cursor, c);
        self.cursor += 1;
        self.in_dropdown = false;
        self.highlight_index = 0;
    }

    /// Remove character before cursor
    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            self.input.remove(self.cursor - 1);
            self.cursor -= 1;
            self.in_dropdown = false;
            self.highlight_index = 0;
        }
    }

    /// Move cursor left
    pub fn move_cursor_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    /// Move cursor right
    pub fn move_cursor_right(&mut self) {
        if self.cursor < self.input.len() {
            self.cursor += 1;
        }
    }

    /// Move highlight up in suggestions
    pub fn move_up(&mut self) {
        if !self.suggestions.is_empty() {
            if !self.in_dropdown {
                // Enter dropdown mode, start at bottom
                self.in_dropdown = true;
                self.highlight_index = self.suggestions.len() - 1;
            } else if self.highlight_index > 0 {
                self.highlight_index -= 1;
            }
        }
    }

    /// Move highlight down in suggestions
    pub fn move_down(&mut self) {
        if !self.suggestions.is_empty() {
            if !self.in_dropdown {
                // Enter dropdown mode, start at top
                self.in_dropdown = true;
                self.highlight_index = 0;
            } else if self.highlight_index < self.suggestions.len() - 1 {
                self.highlight_index += 1;
            }
        }
    }

    /// Toggle selection of highlighted item
    pub fn toggle_selection(&mut self) {
        if self.in_dropdown && self.highlight_index < self.suggestions.len() {
            let item = self.suggestions[self.highlight_index].clone();
            if self.selected.contains(&item) {
                self.selected.remove(&item);
            } else {
                self.selected.insert(item);
            }
        }
    }

    /// Update suggestions based on current input
    pub fn update_suggestions(&mut self, suggestions: Vec<String>) {
        self.suggestions = suggestions;
        // Reset highlight if suggestions changed
        if self.highlight_index >= self.suggestions.len() && !self.suggestions.is_empty() {
            self.highlight_index = 0;
        }
    }

    /// Get the currently highlighted suggestion
    pub fn get_highlighted(&self) -> Option<&String> {
        if self.in_dropdown && self.highlight_index < self.suggestions.len() {
            Some(&self.suggestions[self.highlight_index])
        } else {
            None
        }
    }
}

/// Function to get customer suggestions (callback type)
pub type SuggestionProvider = Box<dyn Fn(&str) -> Vec<String> + Send>;

/// Run the multi-select autocomplete UI
pub fn run_multiselect(
    prompt: &str,
    suggestion_provider: SuggestionProvider,
) -> Result<Vec<String>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize state
    let mut state = MultiSelectState::new();
    let mut last_input = String::new();

    // Main loop
    loop {
        // Update suggestions if input changed
        if state.input != last_input && state.input.len() >= 2 {
            let suggestions = suggestion_provider(&state.input);
            state.update_suggestions(suggestions);
            last_input = state.input.clone();
        } else if state.input.len() < 2 {
            state.update_suggestions(Vec::new());
        }

        // Draw UI
        terminal.draw(|f| draw_ui(f, &state, prompt))?;

        // Handle input with timeout for async operations
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Enter => {
                            // Done selecting
                            break;
                        }
                        KeyCode::Esc => {
                            // Cancel
                            state.selected.clear();
                            break;
                        }
                        KeyCode::Tab => {
                            // Toggle selection
                            state.toggle_selection();
                        }
                        KeyCode::Up => {
                            state.move_up();
                        }
                        KeyCode::Down => {
                            state.move_down();
                        }
                        KeyCode::Left => {
                            state.move_cursor_left();
                        }
                        KeyCode::Right => {
                            state.move_cursor_right();
                        }
                        KeyCode::Backspace => {
                            state.backspace();
                        }
                        KeyCode::Char(c) => {
                            state.add_char(c);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    // Return selected items as vector
    Ok(state.selected.into_iter().collect())
}

/// Draw the UI
fn draw_ui(f: &mut Frame, state: &MultiSelectState, prompt: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),     // Selected items
            Constraint::Length(3),  // Input field
            Constraint::Min(1),     // Suggestions
        ])
        .split(f.area());

    // Draw selected items at top
    draw_selected_items(f, chunks[0], &state.selected);

    // Draw input field
    draw_input_field(f, chunks[1], state, prompt);

    // Draw suggestions dropdown
    if !state.suggestions.is_empty() {
        draw_suggestions(f, chunks[2], state);
    }
}

/// Draw the selected items section
fn draw_selected_items(f: &mut Frame, area: Rect, selected: &HashSet<String>) {
    let items: Vec<Line> = if selected.is_empty() {
        vec![Line::from(vec![
            Span::styled("No customers selected", Style::default().fg(Color::DarkGray))
        ])]
    } else {
        vec![
            Line::from(vec![
                Span::styled("Selected customers:", Style::default().fg(Color::Rgb(255, 108, 55)))
            ]),
            Line::from(
                selected.iter()
                    .map(|item| {
                        Span::styled(
                            format!("  • {item} "),
                            Style::default().fg(Color::Rgb(255, 142, 100))
                        )
                    })
                    .collect::<Vec<_>>()
            )
        ]
    };

    let block = Block::default()
        .borders(Borders::NONE);

    let paragraph = Paragraph::new(items)
        .block(block);

    f.render_widget(paragraph, area);
}

/// Draw the input field
fn draw_input_field(f: &mut Frame, area: Rect, state: &MultiSelectState, prompt: &str) {
    let input = Paragraph::new(state.input.as_str())
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(prompt)
                .title_style(Style::default().fg(Color::Rgb(111, 44, 186)))
        );

    f.render_widget(input, area);

    // Show cursor
    f.set_cursor_position((
        area.x + state.cursor as u16 + 1,
        area.y + 1,
    ));
}

/// Draw the suggestions dropdown
fn draw_suggestions(f: &mut Frame, area: Rect, state: &MultiSelectState) {
    let items: Vec<ListItem> = state.suggestions
        .iter()
        .enumerate()
        .map(|(i, suggestion)| {
            let mut style = Style::default();

            // Highlight if this is the current selection
            if state.in_dropdown && i == state.highlight_index {
                style = style.bg(Color::DarkGray);
            }

            // Orange if selected via TAB
            if state.selected.contains(suggestion) {
                style = style.fg(Color::Rgb(255, 142, 100)).add_modifier(Modifier::BOLD);
            } else {
                style = style.fg(Color::White);
            }

            ListItem::new(suggestion.as_str()).style(style)
        })
        .collect();

    let suggestions_list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::NONE)
        );

    f.render_widget(suggestions_list, area);

    // Show help text at bottom
    let help_text = if state.in_dropdown {
        "Navigate: ↑/↓ | Select: TAB | Submit: ENTER | Cancel: ESC"
    } else {
        "Start typing and press ↓ for suggestions"
    };

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::NONE));

    // Render help at the bottom of the area
    if area.height > state.suggestions.len() as u16 + 2 {
        let help_area = Rect {
            y: area.y + area.height - 1,
            height: 1,
            ..area
        };
        f.render_widget(help, help_area);
    }
}