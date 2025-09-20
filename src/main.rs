mod launcher;

use clap::Parser;
use cs_cli::common::cli::args::{CliArgs, ParsedCommand};
use cs_cli::common::drivers::DriverManager;
use cs_cli::gong::cli::run_cli;
use cs_cli::Result;
use owo_colors::OwoColorize;
<<<<<<< HEAD
use std::sync::Arc;
=======
use std::fs::{self, OpenOptions};
use std::path::PathBuf;
use std::sync::Arc;

/// Get the log file path for TUI mode
fn get_log_file_path() -> PathBuf {
    // Use project directory for logs
    let log_dir = PathBuf::from("logs");

    // Create the log directory if it doesn't exist
    let _ = fs::create_dir_all(&log_dir);

    // Use a timestamp for the log file name
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    log_dir.join(format!("cs-cli_{timestamp}.txt"))
}
>>>>>>> 30887b9 (github auth improvements)

/// Initialize logging based on mode (TUI vs non-interactive)
fn init_logging(is_tui_mode: bool) {
    if is_tui_mode {
<<<<<<< HEAD
        // Use common logging utilities for TUI mode
        if let Err(e) = cs_cli::common::logging::init_tui_logging() {
            eprintln!("Failed to initialize TUI logging: {}", e);
        }
    } else {
        // Use common logging utilities for console mode
        if let Err(e) = cs_cli::common::logging::init_console_logging() {
            eprintln!("Failed to initialize console logging: {}", e);
=======
        // Set environment variable to indicate TUI mode for other components
        std::env::set_var("CS_CLI_TUI_MODE", "1");

        // For TUI mode, log to file to avoid interfering with the terminal UI
        let log_file_path = get_log_file_path();
        let file = match OpenOptions::new()
            .create(true)
            
            .append(true)
            .open(&log_file_path)
        {
            Ok(file) => file,
            Err(_) => return, // Silently fail if we can't open log file
        };

        // Initialize tracing to log file if RUST_LOG is set
        if std::env::var("RUST_LOG").is_ok() {
            let _ = tracing_subscriber::fmt()
                .with_writer(file)
                .with_ansi(false) // No ANSI colors in log file
                .try_init();
        }

        // Store log file path in environment for potential debugging
        std::env::set_var("CS_CLI_LOG_FILE", &log_file_path);
    } else {
        // For non-interactive mode, log to stderr as usual
        if std::env::var("RUST_LOG").is_ok() {
            let _ = tracing_subscriber::fmt::try_init();
>>>>>>> 30887b9 (github auth improvements)
        }
    }
}

<<<<<<< HEAD

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

=======
/// Run CLI without driver cleanup (fallback when DriverManager can't be created)
async fn run_cli_without_cleanup() -> Result<()> {
>>>>>>> 30887b9 (github auth improvements)
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
<<<<<<< HEAD
    let result = run_cli().await;

    // Browser cleanup is handled by individual CdpBrowserManager instances

    result
=======
    run_cli().await
>>>>>>> 30887b9 (github auth improvements)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Set up global driver manager for cleanup
    let driver_manager = match DriverManager::new() {
        Ok(manager) => Arc::new(manager),
        Err(_) => {
            // If we can't create driver manager, proceed without it
            // (drivers might not be available on this system)
            return run_cli_without_cleanup().await;
        }
    };
    let manager_for_panic = Arc::clone(&driver_manager);
    let manager_for_signal = Arc::clone(&driver_manager);

    // Set up panic handler to cleanup drivers
    std::panic::set_hook(Box::new(move |_| {
        // Use thread-based cleanup to avoid nested runtime issues
        let manager = Arc::clone(&manager_for_panic);
        let handle = std::thread::spawn(move || {
            // Try to use current runtime if available, otherwise create new one
            if let Ok(handle) = tokio::runtime::Handle::try_current() {
                let _ = handle.block_on(manager.cleanup_all_drivers());
            } else {
                // Only create new runtime if no current runtime exists
                if let Ok(rt) = tokio::runtime::Runtime::new() {
                    let _ = rt.block_on(manager.cleanup_all_drivers());
                }
            }
        });
        // Give cleanup thread a moment to complete
        let _ = handle.join();
    }));

    // Set up signal handler for graceful shutdown
    tokio::spawn(async move {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                let _ = manager_for_signal.cleanup_all_drivers().await;
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

    // Cleanup drivers on normal exit
    let _ = driver_manager.cleanup_all_drivers().await;

    result
}
