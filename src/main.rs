mod launcher;

use clap::Parser;
use cs_cli::common::cli::args::{CliArgs, ParsedCommand};
use cs_cli::gong::cli::run_cli;
use cs_cli::Result;
use owo_colors::OwoColorize;
use std::sync::Arc;

/// Initialize logging based on mode (TUI vs non-interactive)
fn init_logging(is_tui_mode: bool) {
    if is_tui_mode {
        // Use common logging utilities for TUI mode
        if let Err(e) = cs_cli::common::logging::init_tui_logging() {
            eprintln!("Failed to initialize TUI logging: {}", e);
        }
    } else {
        // Use common logging utilities for console mode
        if let Err(e) = cs_cli::common::logging::init_console_logging() {
            eprintln!("Failed to initialize console logging: {}", e);
        }
    }
}


#[tokio::main]
async fn main() -> Result<()> {
    // Set up global CDP cleanup for browser processes
    // The CDP browser manager will handle its own cleanup registration
    let cleanup_manager = Arc::new(());
    let manager_for_panic = Arc::clone(&cleanup_manager);
    let _manager_for_signal = Arc::clone(&cleanup_manager);

    // Set up panic handler to cleanup browser processes
    std::panic::set_hook(Box::new(move |_| {
        // Use thread-based cleanup to avoid nested runtime issues
        let _manager = Arc::clone(&manager_for_panic);
        let handle = std::thread::spawn(move || {
            // Browser cleanup is handled by individual CdpBrowserManager instances
        });
        // Give cleanup thread a moment to complete
        let _ = handle.join();
    }));

    // Set up signal handler for graceful shutdown
    tokio::spawn(async move {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                // Browser cleanup is handled by individual CdpBrowserManager instances
                std::process::exit(0);
            }
            Err(err) => {
                // Only print error if not in TUI mode
                if std::env::var("CS_CLI_TUI_MODE").is_err() {
                    eprintln!("Unable to listen for shutdown signal: {err}");
                }
            }
        }
    });

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
    if !is_tui_mode {
        println!("{}", "CS-CLI - Initializing...".truecolor(255, 108, 55));
    }
    let result = run_cli().await;

    // Browser cleanup is handled by individual CdpBrowserManager instances

    result
}
