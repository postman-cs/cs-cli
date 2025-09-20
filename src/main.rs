mod launcher;

use clap::Parser;
use cs_cli::common::cli::args::{CliArgs, ParsedCommand};
use cs_cli::gong::cli::run_cli;
use cs_cli::Result;
use owo_colors::OwoColorize;
use std::fs::{self, OpenOptions};
use std::path::PathBuf;

/// Get the log file path for TUI mode
fn get_log_file_path() -> PathBuf {
    // Use project directory for logs
    let log_dir = PathBuf::from("logs");

    // Create the log directory if it doesn't exist
    let _ = fs::create_dir_all(&log_dir);

    // Use a timestamp for the log file name
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    log_dir.join(format!("cs-cli_{}.log", timestamp))
}

/// Initialize logging based on mode (TUI vs non-interactive)
fn init_logging(is_tui_mode: bool) {
    // Only initialize if RUST_LOG is set
    if std::env::var("RUST_LOG").is_err() {
        return;
    }

    if is_tui_mode {
        // For TUI mode, log to file to avoid interfering with the terminal UI
        let log_file_path = get_log_file_path();
        let file = match OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&log_file_path)
        {
            Ok(file) => file,
            Err(_) => return, // Silently fail if we can't open log file
        };

        let _ = tracing_subscriber::fmt()
            .with_writer(file)
            .with_ansi(false) // No ANSI colors in log file
            .try_init();

        // Store log file path in environment for potential debugging
        std::env::set_var("CS_CLI_LOG_FILE", &log_file_path);
    } else {
        // For non-interactive mode, log to stderr as usual
        let _ = tracing_subscriber::fmt::try_init();
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Check for updates first (silent fail if network issues)
    let _ = cs_cli::updater::check_and_update().await;

    // Check if we're in a terminal, self-launch if not
    launcher::ensure_terminal();

    // Parse args to determine if we're in TUI mode
    let parsed_args = CliArgs::parse();
    let is_tui_mode = matches!(
        parsed_args.parse_command(),
        Ok(ParsedCommand::Interactive)
    );

    // Initialize logging based on mode
    init_logging(is_tui_mode);

    // Run Gong CLI (handles its own argument parsing)
    println!("{}", "CS-CLI - Initializing...".truecolor(255, 108, 55));
    run_cli().await
}
