//! Full interactive TUI that handles the complete flow
//!
//! This module keeps the entire interactive experience within ratatui,
//! providing a cohesive UI for customer selection, time period, and content type.

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::collections::HashSet;
use std::io;
use std::time::Duration;
use crate::common::cli::args::{ContentType, ParsedCommand};
use crate::common::cli::multiselect::SuggestionProvider;

/// Result type for TUI operations
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// State machine for the interactive flow
#[derive(Debug, Clone, PartialEq)]
enum InteractiveStep {
    CustomerSelection,
    TimeSelection,
    ContentSelection,
    Confirmation,
}

/// Complete interactive state
pub struct InteractiveState {
    /// Current step in the flow
    step: InteractiveStep,

    // Customer selection state
    input: String,
    cursor: usize,
    suggestions: Vec<String>,
    selected_customers: HashSet<String>,
    highlight_index: usize,
    in_dropdown: bool,

    // Time selection state
    days_input: String,
    days_cursor: usize,

    // Content type selection state
    content_selection: usize, // 0=calls, 1=emails, 2=both
}

impl Default for InteractiveState {
    fn default() -> Self {
        Self::new()
    }
}

impl InteractiveState {
    pub fn new() -> Self {
        Self {
            step: InteractiveStep::CustomerSelection,
            input: String::new(),
            cursor: 0,
            suggestions: Vec::new(),
            selected_customers: HashSet::new(),
            highlight_index: 0,
            in_dropdown: false,
            days_input: "180".to_string(),
            days_cursor: 3,
            content_selection: 2, // Default to "both"
        }
    }

    fn add_char(&mut self, c: char) {
        match self.step {
            InteractiveStep::CustomerSelection => {
                self.input.insert(self.cursor, c);
                self.cursor += 1;
                self.in_dropdown = false;
                self.highlight_index = 0;
            },
            InteractiveStep::TimeSelection => {
                if c.is_ascii_digit() {
                    self.days_input.insert(self.days_cursor, c);
                    self.days_cursor += 1;
                }
            },
            _ => {}
        }
    }

    fn backspace(&mut self) {
        match self.step {
            InteractiveStep::CustomerSelection => {
                if self.cursor > 0 {
                    self.input.remove(self.cursor - 1);
                    self.cursor -= 1;
                    self.in_dropdown = false;
                    self.highlight_index = 0;
                }
            },
            InteractiveStep::TimeSelection => {
                if self.days_cursor > 0 && !self.days_input.is_empty() {
                    self.days_input.remove(self.days_cursor - 1);
                    self.days_cursor -= 1;
                }
            },
            _ => {}
        }
    }

    fn move_cursor_left(&mut self) {
        match self.step {
            InteractiveStep::CustomerSelection => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
            },
            InteractiveStep::TimeSelection => {
                if self.days_cursor > 0 {
                    self.days_cursor -= 1;
                }
            },
            _ => {}
        }
    }

    fn move_cursor_right(&mut self) {
        match self.step {
            InteractiveStep::CustomerSelection => {
                if self.cursor < self.input.len() {
                    self.cursor += 1;
                }
            },
            InteractiveStep::TimeSelection => {
                if self.days_cursor < self.days_input.len() {
                    self.days_cursor += 1;
                }
            },
            _ => {}
        }
    }

    fn move_up(&mut self) {
        match self.step {
            InteractiveStep::CustomerSelection => {
                if !self.suggestions.is_empty() {
                    if !self.in_dropdown {
                        self.in_dropdown = true;
                        self.highlight_index = self.suggestions.len() - 1;
                    } else if self.highlight_index > 0 {
                        self.highlight_index -= 1;
                    }
                }
            },
            InteractiveStep::ContentSelection => {
                if self.content_selection > 0 {
                    self.content_selection -= 1;
                }
            },
            _ => {}
        }
    }

    fn move_down(&mut self) {
        match self.step {
            InteractiveStep::CustomerSelection => {
                if !self.suggestions.is_empty() {
                    if !self.in_dropdown {
                        self.in_dropdown = true;
                        self.highlight_index = 0;
                    } else if self.highlight_index < self.suggestions.len() - 1 {
                        self.highlight_index += 1;
                    }
                }
            },
            InteractiveStep::ContentSelection => {
                if self.content_selection < 2 {
                    self.content_selection += 1;
                }
            },
            _ => {}
        }
    }

    fn toggle_selection(&mut self) {
        if self.step == InteractiveStep::CustomerSelection
           && self.in_dropdown
           && self.highlight_index < self.suggestions.len() {
            let item = self.suggestions[self.highlight_index].clone();
            if self.selected_customers.contains(&item) {
                self.selected_customers.remove(&item);
            } else {
                self.selected_customers.insert(item);
            }
        }
    }

    fn update_suggestions(&mut self, suggestions: Vec<String>) {
        self.suggestions = suggestions;
        if self.highlight_index >= self.suggestions.len() && !self.suggestions.is_empty() {
            self.highlight_index = 0;
        }
    }

    fn next_step(&mut self) -> bool {
        match self.step {
            InteractiveStep::CustomerSelection => {
                if !self.selected_customers.is_empty() {
                    self.step = InteractiveStep::TimeSelection;
                    true
                } else {
                    false
                }
            },
            InteractiveStep::TimeSelection => {
                if !self.days_input.is_empty() {
                    self.step = InteractiveStep::ContentSelection;
                    true
                } else {
                    false
                }
            },
            InteractiveStep::ContentSelection => {
                self.step = InteractiveStep::Confirmation;
                true
            },
            InteractiveStep::Confirmation => false,
        }
    }
}

/// Run the complete interactive TUI flow
pub fn run_full_interactive(
    suggestion_provider: SuggestionProvider,
) -> Result<ParsedCommand> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize state
    let mut state = InteractiveState::new();
    let mut last_input = String::new();

    // Main loop
    loop {
        // Update suggestions if in customer selection
        if state.step == InteractiveStep::CustomerSelection {
            if state.input != last_input && state.input.len() >= 2 {
                let suggestions = suggestion_provider(&state.input);
                state.update_suggestions(suggestions);
                last_input = state.input.clone();
            } else if state.input.len() < 2 {
                state.update_suggestions(Vec::new());
            }
        }

        // Draw UI
        terminal.draw(|f| draw_full_ui(f, &state))?;

        // Handle input
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Enter => {
                            if state.step == InteractiveStep::Confirmation {
                                break;
                            } else {
                                state.next_step();
                            }
                        }
                        KeyCode::Esc => {
                            // Go back a step or cancel
                            match state.step {
                                InteractiveStep::CustomerSelection => {
                                    state.selected_customers.clear();
                                    break;
                                }
                                InteractiveStep::TimeSelection => {
                                    state.step = InteractiveStep::CustomerSelection;
                                }
                                InteractiveStep::ContentSelection => {
                                    state.step = InteractiveStep::TimeSelection;
                                }
                                InteractiveStep::Confirmation => {
                                    state.step = InteractiveStep::ContentSelection;
                                }
                            }
                        }
                        KeyCode::Tab if state.step == InteractiveStep::CustomerSelection => {
                            state.toggle_selection();
                        }
                        KeyCode::Up => state.move_up(),
                        KeyCode::Down => state.move_down(),
                        KeyCode::Left => state.move_cursor_left(),
                        KeyCode::Right => state.move_cursor_right(),
                        KeyCode::Backspace => state.backspace(),
                        KeyCode::Char(c) => state.add_char(c),
                        _ => {}
                    }
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    // Return empty result if cancelled
    if state.selected_customers.is_empty() {
        return Err("Operation cancelled".into());
    }

    // Parse days
    let days = state.days_input.parse::<u32>().unwrap_or(180);

    // Map content selection
    let content_type = match state.content_selection {
        0 => ContentType::Calls,
        1 => ContentType::Emails,
        _ => ContentType::Both,
    };

    let emails_only = content_type == ContentType::Emails;
    let fetch_email_bodies = matches!(content_type, ContentType::Emails | ContentType::Both);

    // Return appropriate command
    let customers: Vec<String> = state.selected_customers.into_iter().collect();
    if customers.len() == 1 {
        Ok(ParsedCommand::Customer {
            name: customers[0].clone(),
            days: Some(days),
            from_date: None,
            to_date: None,
            content_type,
            emails_only,
            fetch_email_bodies,
        })
    } else {
        Ok(ParsedCommand::MultipleCustomers {
            names: customers,
            days: Some(days),
            from_date: None,
            to_date: None,
            content_type,
            emails_only,
            fetch_email_bodies,
        })
    }
}

/// Draw the complete UI
fn draw_full_ui(f: &mut Frame, state: &InteractiveState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),   // Header
            Constraint::Min(10),     // Main content
            Constraint::Length(2),   // Footer
        ])
        .split(f.area());

    // Draw header
    draw_header(f, chunks[0], &state.step);

    // Draw main content based on current step
    match state.step {
        InteractiveStep::CustomerSelection => {
            draw_customer_selection(f, chunks[1], state);
        }
        InteractiveStep::TimeSelection => {
            draw_time_selection(f, chunks[1], state);
        }
        InteractiveStep::ContentSelection => {
            draw_content_selection(f, chunks[1], state);
        }
        InteractiveStep::Confirmation => {
            draw_confirmation(f, chunks[1], state);
        }
    }

    // Draw footer with help
    draw_footer(f, chunks[2], &state.step);
}

fn draw_header(f: &mut Frame, area: Rect, step: &InteractiveStep) {
    let progress = match step {
        InteractiveStep::CustomerSelection => "Step 1/3: Select Customers",
        InteractiveStep::TimeSelection => "Step 2/3: Time Period",
        InteractiveStep::ContentSelection => "Step 3/3: Content Type",
        InteractiveStep::Confirmation => "Review & Confirm",
    };

    let header = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("CS-CLI: Customer Success Deep Research Tool",
                        Style::default().fg(Color::Rgb(255, 108, 55)).add_modifier(Modifier::BOLD))
        ]),
        Line::from(vec![
            Span::styled(progress, Style::default().fg(Color::Rgb(230, 230, 230)))
        ]),
    ])
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::BOTTOM));

    f.render_widget(header, area);
}

fn draw_customer_selection(f: &mut Frame, area: Rect, state: &InteractiveState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),     // Selected customers
            Constraint::Length(3),  // Input field
            Constraint::Min(5),     // Suggestions
        ])
        .split(area);

    // Selected customers
    let selected_text = if state.selected_customers.is_empty() {
        vec![Line::from(vec![
            Span::styled("No customers selected", Style::default().fg(Color::DarkGray))
        ])]
    } else {
        let mut lines = vec![Line::from(vec![
            Span::styled("Selected customers:", Style::default().fg(Color::Green))
        ])];

        for customer in &state.selected_customers {
            lines.push(Line::from(vec![
                Span::styled(format!("  • {customer}"),
                            Style::default().fg(Color::Rgb(255, 142, 100)))
            ]));
        }
        lines
    };

    let selected_widget = Paragraph::new(selected_text)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(selected_widget, chunks[0]);

    // Input field
    let input_widget = Paragraph::new(state.input.as_str())
        .style(Style::default().fg(Color::White))
        .block(Block::default()
            .borders(Borders::ALL)
            .title("What customers are you looking for?")
            .title_style(Style::default().fg(Color::Rgb(111, 44, 186))));
    f.render_widget(input_widget, chunks[1]);

    // Set cursor
    f.set_cursor_position((
        chunks[1].x + state.cursor as u16 + 1,
        chunks[1].y + 1,
    ));

    // Suggestions
    if !state.suggestions.is_empty() {
        let items: Vec<ListItem> = state.suggestions
            .iter()
            .enumerate()
            .map(|(i, suggestion)| {
                let mut style = Style::default();

                if state.in_dropdown && i == state.highlight_index {
                    style = style.bg(Color::DarkGray);
                }

                if state.selected_customers.contains(suggestion) {
                    style = style.fg(Color::Rgb(255, 142, 100)).add_modifier(Modifier::BOLD);
                } else {
                    style = style.fg(Color::White);
                }

                ListItem::new(suggestion.as_str()).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::NONE));
        f.render_widget(list, chunks[2]);
    }
}

fn draw_time_selection(f: &mut Frame, area: Rect, state: &InteractiveState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),  // Selected customers summary
            Constraint::Length(3),  // Days input
            Constraint::Min(3),     // Help text
        ])
        .split(area);

    // Show selected customers summary
    let summary = format!("Selected: {} customer(s)", state.selected_customers.len());
    let summary_widget = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(summary, Style::default().fg(Color::Green))
        ]),
    ])
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::NONE));
    f.render_widget(summary_widget, chunks[0]);

    // Days input
    let days_widget = Paragraph::new(state.days_input.as_str())
        .style(Style::default().fg(Color::White))
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Number of days back to search")
            .title_style(Style::default().fg(Color::Rgb(111, 44, 186))));
    f.render_widget(days_widget, chunks[1]);

    // Set cursor
    f.set_cursor_position((
        chunks[1].x + state.days_cursor as u16 + 1,
        chunks[1].y + 1,
    ));

    // Help text
    let help = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Common choices:", Style::default().fg(Color::DarkGray))
        ]),
        Line::from(vec![
            Span::styled("  30 days (1 month), 90 days (3 months), 180 days (6 months)",
                        Style::default().fg(Color::Rgb(230, 230, 230)))
        ]),
    ];
    let help_widget = Paragraph::new(help)
        .alignment(Alignment::Center);
    f.render_widget(help_widget, chunks[2]);
}

fn draw_content_selection(f: &mut Frame, area: Rect, state: &InteractiveState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),  // Summary
            Constraint::Min(10),    // Options
        ])
        .split(area);

    // Show summary
    let summary = vec![
        Line::from(vec![
            Span::styled(format!("Selected: {} customer(s)", state.selected_customers.len()),
                        Style::default().fg(Color::Green))
        ]),
        Line::from(vec![
            Span::styled(format!("Time period: {} days", state.days_input),
                        Style::default().fg(Color::Green))
        ]),
    ];
    let summary_widget = Paragraph::new(summary)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(summary_widget, chunks[0]);

    // Content options
    let options = [("Calls only", 0),
        ("Emails only", 1),
        ("Both calls and emails (recommended)", 2)];

    let items: Vec<ListItem> = options
        .iter()
        .map(|(label, idx)| {
            let style = if *idx == state.content_selection {
                Style::default()
                    .fg(Color::Rgb(255, 142, 100))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let prefix = if *idx == state.content_selection {
                "▶ "
            } else {
                "  "
            };

            ListItem::new(format!("{prefix}{label}")).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("What would you like to analyze?")
            .title_style(Style::default().fg(Color::Rgb(111, 44, 186))));
    f.render_widget(list, chunks[1]);
}

fn draw_confirmation(f: &mut Frame, area: Rect, state: &InteractiveState) {
    let content_type = match state.content_selection {
        0 => "Calls only",
        1 => "Emails only",
        _ => "Calls and emails",
    };

    let customer_list: Vec<String> = state.selected_customers.iter().cloned().collect();
    let customer_display = if customer_list.len() > 3 {
        format!("{} customers", customer_list.len())
    } else {
        customer_list.join(", ")
    };

    let confirmation = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Ready to extract data with these settings:",
                        Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("✓ ", Style::default().fg(Color::Green)),
            Span::styled("Customers: ", Style::default().fg(Color::White)),
            Span::styled(&customer_display, Style::default().fg(Color::Rgb(255, 142, 100))),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("✓ ", Style::default().fg(Color::Green)),
            Span::styled("Time period: ", Style::default().fg(Color::White)),
            Span::styled(format!("Last {} days", state.days_input),
                        Style::default().fg(Color::Rgb(255, 142, 100))),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("✓ ", Style::default().fg(Color::Green)),
            Span::styled("Content: ", Style::default().fg(Color::White)),
            Span::styled(content_type, Style::default().fg(Color::Rgb(255, 142, 100))),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ENTER to begin extraction",
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        ]),
    ];

    let widget = Paragraph::new(confirmation)
        .alignment(Alignment::Center)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Confirmation")
            .title_style(Style::default().fg(Color::Rgb(111, 44, 186))));
    f.render_widget(widget, area);
}

fn draw_footer(f: &mut Frame, area: Rect, step: &InteractiveStep) {
    let help_text = match step {
        InteractiveStep::CustomerSelection => {
            "Type to search | TAB: Select/Deselect | ENTER: Next | ESC: Cancel"
        }
        InteractiveStep::TimeSelection | InteractiveStep::ContentSelection => {
            "↑/↓: Navigate | ENTER: Next | ESC: Back"
        }
        InteractiveStep::Confirmation => {
            "ENTER: Start Extraction | ESC: Back"
        }
    };

    let footer = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(footer, area);
}