//! Gong CLI orchestration
//!
//! This module provides the CLI entry point and command routing for Gong.
//! All retrieval logic has been moved to the retriever module.

pub mod tui_runner;

// Re-export common CLI components
pub use crate::common::cli::args::*;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::gong::config::AppConfig;
use crate::gong::retriever::TeamCallsRetriever;
use crate::Result;
use clap::{CommandFactory, Parser};
use clap_complete::{generate, Shell};

/// CLI configuration file structure
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CliConfig {
    pub team_call_stream_id: Option<String>,
}

/// Load CLI configuration
pub fn load_config() -> CliConfig {
    let config_path = config_file_path();
    if config_path.exists() {
        if let Ok(content) = fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str(&content) {
                return config;
            }
        }
    }
    CliConfig::default()
}

/// Save CLI configuration
pub fn save_config(config: &CliConfig) -> Result<()> {
    let config_path = config_file_path();
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| crate::common::error::types::CsCliError::Generic(e.to_string()))?;
    fs::write(config_path, content)?;
    Ok(())
}

/// Get the path to the configuration file
fn config_file_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".config").join("cs-cli").join("config.json")
}

/// Main entry point for the Gong CLI
pub async fn run_cli() -> Result<()> {
    let args = CliArgs::parse();

    // Handle reset flag first
    if args.reset_sync {
        println!("Clearing all stored authentication data...");

        // Delete cookies from both keychain and gist storage
        match crate::common::auth::hybrid_cookie_storage::delete_cookies_hybrid().await {
            Ok(()) => {
                println!("All authentication data has been cleared successfully");
                println!("You will need to authenticate again on next run");
                return Ok(());
            }
            Err(e) => {
                eprintln!("Failed to clear authentication: {e}");
                return Err(e);
            }
        }
    }

    let command = args.parse_command()?;
    let app_config = AppConfig::from_env()?;

    // Route to appropriate handler based on command
    match command {
        ParsedCommand::Interactive => {
            // Use the new full TUI experience
            match tui_runner::run_gong_tui(app_config).await {
                Ok(_) => Ok(()), // TUI handles everything
                Err(e) => {
                    eprintln!("TUI failed: {e}");
                    Err(e)
                }
            }
        }
        ParsedCommand::Customer { .. }
        | ParsedCommand::MultipleCustomers { .. }
        | ParsedCommand::Team { .. } => {
            // Execute command directly without TUI
            execute_command_direct(command, app_config).await
        }
        ParsedCommand::Completion { shell } => {
            generate_completion(shell);
            Ok(())
        }
    }
}

/// Generate shell completion script
fn generate_completion(shell: Shell) {
    let mut app = CliArgs::command();
    let bin_name = app.get_name().to_string();
    generate(shell, &mut app, bin_name, &mut std::io::stdout());
}

/// Execute command directly without TUI
async fn execute_command_direct(command: ParsedCommand, app_config: AppConfig) -> Result<()> {
    // Initialize retriever (with console output enabled)
    let mut retriever = TeamCallsRetriever::new(app_config);

    // Setup components
    retriever.setup().await?;

    match command {
        ParsedCommand::Team {
            stream_id, days, ..
        } => {
            let stream_id = stream_id.ok_or_else(|| {
                crate::common::error::types::CsCliError::Generic(
                    "No stream ID provided".to_string(),
                )
            })?;

            let days = days.unwrap_or(7);
            let calls = retriever
                .retrieve_team_calls(&stream_id, Some(days), None, None)
                .await?;

            if !calls.is_empty() {
                let saved = retriever.save_calls_as_markdown_with_resolved_name(
                    &calls,
                    Some("Team"),
                    Some("Team"),
                )?;

                // Save config
                let mut cli_config = load_config();
                cli_config.team_call_stream_id = Some(stream_id);
                save_config(&cli_config)?;

                // Show output directory
                if let Some(first_file) = saved.first() {
                    if let Some(parent) = first_file.parent() {
                        println!("\n📂 Output saved to: {}", parent.display());
                    }
                }
            }
        }
        ParsedCommand::Customer {
            ref name,
            days,
            content_type,
            emails_only,
            fetch_email_bodies,
            ..
        } => {
            let days = days.unwrap_or(90);

            let (calls, emails, resolved_name) = retriever
                .retrieve_customer_communications(
                    name,
                    days,
                    matches!(content_type, ContentType::Both | ContentType::Emails),
                    emails_only,
                    fetch_email_bodies,
                )
                .await?;

            let mut saved_files: Vec<PathBuf> = Vec::new();

            if !calls.is_empty() && !emails_only {
                let files = retriever.save_calls_as_markdown_with_resolved_name(
                    &calls,
                    Some(name),
                    Some(&resolved_name),
                )?;
                saved_files.extend(files);
            }

            if !emails.is_empty() {
                let files = retriever.save_emails_as_markdown(&emails, name)?;
                saved_files.extend(files);
            }

            // Show output directory
            if let Some(first_file) = saved_files.first() {
                if let Some(parent) = first_file.parent() {
                    println!("\n📂 Output saved to: {}", parent.display());
                }
            }
        }
        ParsedCommand::MultipleCustomers {
            ref names,
            days,
            content_type,
            emails_only,
            fetch_email_bodies,
            ..
        } => {
            let days = days.unwrap_or(90);

            for customer_name in names {
                println!("\nProcessing customer: {customer_name}");

                let (calls, emails, resolved_name) = retriever
                    .retrieve_customer_communications(
                        customer_name,
                        days,
                        matches!(content_type, ContentType::Both | ContentType::Emails),
                        emails_only,
                        fetch_email_bodies,
                    )
                    .await?;

                if !calls.is_empty() && !emails_only {
                    retriever.save_calls_as_markdown_with_resolved_name(
                        &calls,
                        Some(customer_name),
                        Some(&resolved_name),
                    )?;
                }

                if !emails.is_empty() {
                    retriever.save_emails_as_markdown(&emails, customer_name)?;
                }
            }
        }
        _ => unreachable!("Other commands should be handled before this point"),
    }

    // Cleanup
    retriever.cleanup().await;

    Ok(())
}
