//! Platform-agnostic interactive mode functionality
//!
//! This module provides common interactive interface components that can be
//! used across all platform integrations.

use console::Term;
use inquire::Text;
use owo_colors::OwoColorize;
use std::io::Write;

use crate::common::cli::args::ContentType;
use crate::Result;

/// Get time period with sensible defaults and validation
pub fn get_time_period() -> Result<u32> {
    println!();
    println!("{}", "How far back should I look?".truecolor(111, 44, 186));
    println!(
        "{}",
        "Common choices: 30 days (1 month), 90 days (3 months), 180 days (6 months)"
            .truecolor(230, 230, 230)
    );

    loop {
        let days_input: String = Text::new("Number of days")
            .with_default("180")
            .prompt()
            .map_err(|e| crate::CsCliError::Generic(format!("Input error: {e}")))?;

        match days_input.trim().parse::<u32>() {
            Ok(days) if days > 0 && days <= 3650 => return Ok(days), // Max ~10 years
            Ok(_) => {
                println!(
                    "{}",
                    "Please enter a number between 1 and 3650 days.".yellow()
                );
                continue;
            }
            Err(_) => {
                println!("{}", "Using default: 180 days".yellow());
                return Ok(180);
            }
        }
    }
}

/// Get content type selection with menu
pub fn get_content_type() -> Result<ContentType> {
    println!();
    println!(
        "{}",
        "What would you like to analyze?".truecolor(111, 44, 186)
    );
    println!();

    // Display options in table format
    println!("{}  Calls only", "1.".truecolor(111, 44, 186).bold());
    println!("{}  Emails only", "2.".truecolor(111, 44, 186).bold());
    println!(
        "{}  Both calls and emails (recommended)",
        "3.".truecolor(111, 44, 186).bold()
    );
    println!();

    let choice: String = Text::new("Type a number and press Enter")
        .with_default("3")
        .prompt()
        .map_err(|e| crate::CsCliError::Generic(format!("Input error: {e}")))?;

    let content_type = match choice.trim() {
        "1" => ContentType::Calls,
        "2" => ContentType::Emails,
        "3" => ContentType::Both,
        _ => ContentType::Both, // Default to both for invalid input
    };

    Ok(content_type)
}

/// Show confirmation summary before proceeding
pub fn show_confirmation_summary(customer: &str, days: u32, content_type: ContentType) {
    println!();
    println!("{} {}", "✓ Looking for:".truecolor(255, 255, 255), customer);
    println!(
        "{} Last {} days",
        "✓ Time period:".truecolor(255, 255, 255),
        days
    );

    let content_display = match content_type {
        ContentType::Calls => "Calls",
        ContentType::Emails => "Emails",
        ContentType::Both => "Calls and emails",
    };
    println!(
        "{} {}",
        "✓ Content:".truecolor(255, 255, 255),
        content_display
    );
    println!();
}

/// Basic interactive mode without autocomplete (fallback)
pub fn interactive_mode() -> Result<crate::common::cli::args::ParsedCommand> {
    let term = Term::stdout();

    // Clear screen and show banner
    term.clear_screen().ok();

    // Welcome banner
    println!(
        "\n{}",
        "CS-CLI: Customer Success Deep Research Tool"
            .bold()
            .truecolor(255, 108, 55)
    );
    println!(
        "{}",
        "Let's find insights from your customer conversations".truecolor(230, 230, 230)
    );
    println!();

    // Get customer name
    let customer_name = get_customer_name_basic()?;

    // Get time period
    let days = get_time_period()?;

    // Get content type
    let content_type = get_content_type()?;

    // Show confirmation summary
    show_confirmation_summary(&customer_name, days, content_type);

    // Convert content type to individual flags
    let emails_only = content_type == ContentType::Emails;
    let fetch_email_bodies = matches!(content_type, ContentType::Emails | ContentType::Both);

    Ok(crate::common::cli::args::ParsedCommand::Customer {
        name: customer_name,
        days: Some(days),
        from_date: None,
        to_date: None,
        content_type,
        emails_only,
        fetch_email_bodies,
    })
}

/// Get customer name with basic validation (fallback without autocomplete)
fn get_customer_name_basic() -> Result<String> {
    loop {
        let customer: String = Text::new(
            "What customer are you looking for?"
                .truecolor(111, 44, 186)
                .to_string()
                .as_str(),
        )
        .prompt()
        .map_err(|e| crate::CsCliError::Generic(format!("Input error: {e}")))?;

        let customer = customer.trim();
        if customer.is_empty() {
            println!(
                "{}",
                "Customer name cannot be empty. Please try again.".red()
            );
            continue;
        }

        if customer.len() < 2 {
            println!(
                "{}",
                "Customer name must be at least 2 characters. Please try again.".red()
            );
            continue;
        }

        return Ok(customer.to_string());
    }
}

/// Utility function to ask for confirmation (yes/no)
#[allow(dead_code)]
pub fn confirm(message: &str, default: bool) -> Result<bool> {
    let prompt = if default {
        format!("{message} [Y/n]")
    } else {
        format!("{message} [y/N]")
    };

    loop {
        let input: String = Text::new(&prompt)
            .with_default("")
            .prompt()
            .map_err(|e| crate::CsCliError::Generic(format!("Input error: {e}")))?;

        let input = input.trim().to_lowercase();

        if input.is_empty() {
            return Ok(default);
        }

        match input.as_str() {
            "y" | "yes" | "true" | "1" => return Ok(true),
            "n" | "no" | "false" | "0" => return Ok(false),
            _ => {
                println!("{}", "Please answer 'y' for yes or 'n' for no.".yellow());
                continue;
            }
        }
    }
}

/// Display a progress indicator while waiting
#[allow(dead_code)]
pub fn show_progress_message(message: &str) {
    print!("{} ", message.truecolor(111, 44, 186));
    std::io::stdout().flush().ok();
}

/// Clear the current line (useful after progress messages)
#[allow(dead_code)]
pub fn clear_line() {
    let term = Term::stdout();
    term.clear_line().ok();
}

/// Interactive team mode for stream ID configuration
///
/// This handles team extraction mode when no stream ID is provided
pub fn interactive_team_mode(saved_stream_id: Option<String>) -> Result<crate::common::cli::args::ParsedCommand> {
    let _term = Term::stdout();

    println!();
    println!("{}", "Team Calls Extraction".bold().truecolor(255, 108, 55));
    println!(
        "{}",
        "Extract calls from a specific call stream in your library".truecolor(230, 230, 230)
    );
    println!();

    let stream_id = if let Some(saved_id) = saved_stream_id {
        // Show reuse option
        println!("{} {}", "Previously used stream ID:".green(), saved_id);
        println!();

        let reuse_prompt = format!("Enter call stream ID (leave blank to reuse {saved_id})");

        let input: String = Text::new(reuse_prompt.truecolor(111, 44, 186).to_string().as_str())
            .with_default("")
            .prompt()
            .map_err(|e| crate::CsCliError::Generic(format!("Input error: {e}")))?;

        if input.trim().is_empty() {
            println!("{} {}", "Reusing previous stream ID:".green(), saved_id);
            saved_id
        } else {
            println!("{}", "Saved new stream ID for future use".green());
            input.trim().to_string()
        }
    } else {
        // First time - show instructions
        show_team_instructions();

        let stream_id: String = Text::new(
            "Enter your call stream ID"
                .truecolor(111, 44, 186)
                .to_string()
                .as_str(),
        )
        .prompt()
        .map_err(|e| crate::CsCliError::Generic(format!("Input error: {e}")))?;

        let stream_id = stream_id.trim().to_string();
        if stream_id.is_empty() {
            return Err(crate::CsCliError::InvalidArguments {
                message: "Call stream ID is required for team extraction".to_string(),
            });
        }

        println!("{}", "Stream ID saved for future use".green());
        stream_id
    };

    Ok(crate::common::cli::args::ParsedCommand::Team {
        stream_id: Some(stream_id),
        days: Some(7), // Default to 7 days for team
        from_date: None,
        to_date: None,
    })
}

/// Show instructions for finding team call stream ID
fn show_team_instructions() {
    println!("{}", "To find your call stream ID:".yellow());
    println!(
        "1. In your platform, go to {} > {}",
        "Conversations".bold(),
        "Your Library".bold()
    );
    println!("2. Create a call stream to filter by your team members");
    println!("3. The stream ID will be at the end of the URL as folder-id:");
    println!("   {}",
             "https://xxxxxx.app.platform.io/library/private?workspace-id=xxxxxxxxxxxxxxx&folder-id=[your-stream-id]".dimmed());
    println!();
}