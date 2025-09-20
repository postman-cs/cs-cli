//! Logging initialization utilities
//!
//! Common logging setup patterns for different modes and contexts.

use std::fs::OpenOptions;
use std::path::PathBuf;
use tracing_subscriber::fmt;

/// Initialize logging for TUI mode (logs to file)
pub fn init_tui_logging() -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Set environment variable to indicate TUI mode
    std::env::set_var("CS_CLI_TUI_MODE", "1");

    // Create log file path
    let log_file_path = create_log_file_path("cs-cli")?;
    
    // Open log file
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file_path)?;

    // Initialize tracing to log file if RUST_LOG is set
    if std::env::var("RUST_LOG").is_ok() {
        fmt()
            .with_writer(file)
            .with_ansi(false) // No ANSI colors in log file
            .try_init()
            .map_err(|e| format!("Failed to initialize tracing: {}", e))?;
    }

    // Store log file path in environment for potential debugging
    std::env::set_var("CS_CLI_LOG_FILE", &*log_file_path.to_string_lossy());
    
    Ok(log_file_path)
}

/// Initialize logging for non-interactive mode (logs to stderr)
pub fn init_console_logging() -> Result<(), Box<dyn std::error::Error>> {
    // For non-interactive mode, log to stderr as usual
    if std::env::var("RUST_LOG").is_ok() {
        fmt().try_init().map_err(|e| format!("Failed to initialize tracing: {}", e))?;
    }
    Ok(())
}

/// Initialize logging for tests
pub fn init_test_logging() -> Result<(), Box<dyn std::error::Error>> {
    // Set up test environment variables if needed
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "cs_cli=debug");
    }

    // Initialize tracing for tests (ignore if already initialized)
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let _ = fmt::try_init();
    });
    
    Ok(())
}

/// Initialize logging for cookie validation (creates dedicated log file)
pub fn init_cookie_validation_logging() -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Create logs directory if it doesn't exist
    let logs_dir = std::path::Path::new("logs");
    if !logs_dir.exists() {
        std::fs::create_dir_all(logs_dir)?;
    }
    
    // Generate timestamp for log file
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();
    let log_filename = format!("logs/cookie_validation_{}.log", now);
    
    Ok(PathBuf::from(log_filename))
}

/// Create a log file path with timestamp
pub fn create_log_file_path(prefix: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Use project directory for logs
    let log_dir = PathBuf::from("logs");

    // Create the log directory if it doesn't exist
    let _ = std::fs::create_dir_all(&log_dir);

    // Use a timestamp for the log file name
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    Ok(log_dir.join(format!("{}_{}.txt", prefix, timestamp)))
}

/// Check if we're in TUI mode
pub fn is_tui_mode() -> bool {
    std::env::var("CS_CLI_TUI_MODE").is_ok()
}

/// Get the current log file path (if in TUI mode)
pub fn get_current_log_file() -> Option<PathBuf> {
    std::env::var("CS_CLI_LOG_FILE")
        .ok()
        .map(PathBuf::from)
}