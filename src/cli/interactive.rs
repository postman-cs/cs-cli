//! Interactive mode functionality for CS-CLI
//!
//! This module provides a user-friendly interactive interface for users who
//! don't provide command-line arguments, matching the Python version's experience.

use console::{style, Term};
use dialoguer::Input;
use std::io::Write;

use crate::cli::args::{ContentType, ParsedCommand};
use crate::Result;

/// Interactive mode for users who don't provide command-line arguments
///
/// Returns a ParsedCommand::Customer with all the collected information
pub fn interactive_mode() -> Result<ParsedCommand> {
    let term = Term::stdout();

    // Clear screen and show banner
    term.clear_screen().ok();

    // Welcome banner - matching Python version styling
    println!(
        "\n{}",
        style("CS-CLI: Customer Success Deep Research Tool")
            .bold()
            .cyan()
    );
    println!(
        "{}",
        style("Let's find insights from your customer conversations").dim()
    );
    println!();

    // Step 1: Get customer name
    let customer_name = get_customer_name()?;

    // Step 2: Get time period
    let days = get_time_period()?;

    // Step 3: Get content type
    let content_type = get_content_type()?;

    // Step 4: Show confirmation summary
    show_confirmation_summary(&customer_name, days, content_type);

    // Convert content type to individual flags
    let emails_only = content_type == ContentType::Emails;
    let fetch_email_bodies = matches!(content_type, ContentType::Emails | ContentType::Both);

    Ok(ParsedCommand::Customer {
        name: customer_name,
        days: Some(days),
        from_date: None,
        to_date: None,
        content_type,
        emails_only,
        fetch_email_bodies,
    })
}

/// Get customer name with validation
fn get_customer_name() -> Result<String> {
    loop {
        let customer: String = Input::new()
            .with_prompt(
                style("What customer are you looking for?")
                    .cyan()
                    .to_string(),
            )
            .interact_text()
            .map_err(|e| crate::error::CsCliError::Generic(format!("Input error: {e}")))?;

        let customer = customer.trim();
        if customer.is_empty() {
            println!(
                "{}",
                style("Customer name cannot be empty. Please try again.").red()
            );
            continue;
        }

        if customer.len() < 2 {
            println!(
                "{}",
                style("Customer name must be at least 2 characters. Please try again.").red()
            );
            continue;
        }

        return Ok(customer.to_string());
    }
}

/// Get time period with sensible defaults and validation (matching Python version)
fn get_time_period() -> Result<u32> {
    println!();
    println!("{}", style("How far back should I look?").cyan());
    println!(
        "{}",
        style("Common choices: 30 days (1 month), 90 days (3 months), 180 days (6 months)").dim()
    );

    loop {
        let days_input: String = Input::new()
            .with_prompt("Number of days")
            .default("90".to_string())
            .show_default(true)
            .interact_text()
            .map_err(|e| crate::error::CsCliError::Generic(format!("Input error: {e}")))?;

        match days_input.trim().parse::<u32>() {
            Ok(days) if days > 0 && days <= 3650 => return Ok(days), // Max ~10 years
            Ok(_) => {
                println!(
                    "{}",
                    style("Please enter a number between 1 and 3650 days.").yellow()
                );
                continue;
            }
            Err(_) => {
                println!("{}", style("Using default: 90 days").yellow());
                return Ok(90);
            }
        }
    }
}

/// Get content type selection with menu (matching Python table style)
fn get_content_type() -> Result<ContentType> {
    println!();
    println!("{}", style("What would you like to analyze?").cyan());
    println!();

    // Display options in table format like Python version
    println!("{}  Calls only", style("1.").cyan().bold());
    println!("{}  Emails only", style("2.").cyan().bold());
    println!(
        "{}  Both calls and emails (recommended)",
        style("3.").cyan().bold()
    );
    println!();

    let choice: String = Input::new()
        .with_prompt("Type a number and press Enter")
        .default("3".to_string())
        .show_default(true)
        .interact_text()
        .map_err(|e| crate::error::CsCliError::Generic(format!("Input error: {e}")))?;

    let content_type = match choice.trim() {
        "1" => ContentType::Calls,
        "2" => ContentType::Emails,
        "3" => ContentType::Both,
        _ => ContentType::Both, // Default to both for invalid input
    };

    Ok(content_type)
}

/// Show confirmation summary before proceeding (matching Python version)
fn show_confirmation_summary(customer: &str, days: u32, content_type: ContentType) {
    println!();
    println!("{} {}", style("✓ Looking for:").green(), customer);
    println!("{} Last {} days", style("✓ Time period:").green(), days);

    let content_display = match content_type {
        ContentType::Calls => "Calls",
        ContentType::Emails => "Emails",
        ContentType::Both => "Calls and emails",
    };
    println!("{} {}", style("✓ Content:").green(), content_display);
    println!();
}

/// Interactive team mode for stream ID configuration
///
/// This handles team extraction mode when no stream ID is provided
pub fn interactive_team_mode(saved_stream_id: Option<String>) -> Result<ParsedCommand> {
    let _term = Term::stdout();

    println!();
    println!("{}", style("Team Calls Extraction").bold().cyan());
    println!(
        "{}",
        style("Extract calls from a specific call stream in your Gong library").dim()
    );
    println!();

    let stream_id = if let Some(saved_id) = saved_stream_id {
        // Show reuse option
        println!(
            "{} {}",
            style("Previously used stream ID:").green(),
            saved_id
        );
        println!();

        let reuse_prompt = format!("Enter call stream ID (leave blank to reuse {saved_id})");

        let input: String = Input::new()
            .with_prompt(style(reuse_prompt).cyan().to_string())
            .allow_empty(true)
            .interact_text()
            .map_err(|e| crate::error::CsCliError::Generic(format!("Input error: {e}")))?;

        if input.trim().is_empty() {
            println!(
                "{} {}",
                style("Reusing previous stream ID:").green(),
                saved_id
            );
            saved_id
        } else {
            println!("{}", style("Saved new stream ID for future use").green());
            input.trim().to_string()
        }
    } else {
        // First time - show instructions
        show_team_instructions();

        let stream_id: String = Input::new()
            .with_prompt(style("Enter your call stream ID").cyan().to_string())
            .interact_text()
            .map_err(|e| crate::error::CsCliError::Generic(format!("Input error: {e}")))?;

        let stream_id = stream_id.trim().to_string();
        if stream_id.is_empty() {
            return Err(crate::error::CsCliError::InvalidArguments {
                message: "Call stream ID is required for team extraction".to_string(),
            });
        }

        println!("{}", style("Stream ID saved for future use").green());
        stream_id
    };

    Ok(ParsedCommand::Team {
        stream_id: Some(stream_id),
        days: Some(7), // Default to 7 days for team
        from_date: None,
        to_date: None,
    })
}

/// Show instructions for finding team call stream ID
fn show_team_instructions() {
    println!("{}", style("To find your call stream ID:").yellow());
    println!(
        "1. In Gong, go to {} > {}",
        style("Conversations").bold(),
        style("Your Library").bold()
    );
    println!("2. Create a call stream to filter by your team members");
    println!("3. The stream ID will be at the end of the URL as folder-id:");
    println!("   {}",
             style("https://xxxxxx.app.gong.io/library/private?workspace-id=xxxxxxxxxxxxxxx&folder-id=[your-stream-id]").dim());
    println!();
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
        let input: String = Input::new()
            .with_prompt(&prompt)
            .allow_empty(true)
            .interact_text()
            .map_err(|e| crate::error::CsCliError::Generic(format!("Input error: {e}")))?;

        let input = input.trim().to_lowercase();

        if input.is_empty() {
            return Ok(default);
        }

        match input.as_str() {
            "y" | "yes" | "true" | "1" => return Ok(true),
            "n" | "no" | "false" | "0" => return Ok(false),
            _ => {
                println!(
                    "{}",
                    style("Please answer 'y' for yes or 'n' for no.").yellow()
                );
                continue;
            }
        }
    }
}

/// Display a progress indicator while waiting
#[allow(dead_code)]
pub fn show_progress_message(message: &str) {
    print!("{} ", style(message).cyan());
    std::io::stdout().flush().ok();
}

/// Clear the current line (useful after progress messages)
#[allow(dead_code)]
pub fn clear_line() {
    let term = Term::stdout();
    term.clear_line().ok();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_type_display() {
        // Test that content types display correctly
        assert_eq!(format!("{}", ContentType::Calls), "calls");
        assert_eq!(format!("{}", ContentType::Emails), "emails");
        assert_eq!(format!("{}", ContentType::Both), "calls and emails");
    }

    // Note: Interactive functions are difficult to unit test due to stdin interaction
    // Integration tests would be better for the full interactive flow
}
