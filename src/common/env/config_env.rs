//! Configuration environment utilities
//!
//! Common environment variable patterns for configuration.

use crate::common::env::env_utils::*;
use std::env;

/// Common environment variable names
pub struct EnvVars;

impl EnvVars {
    /// RUST_LOG environment variable
    pub const RUST_LOG: &str = "RUST_LOG";
    
    /// RUST_BACKTRACE environment variable
    pub const RUST_BACKTRACE: &str = "RUST_BACKTRACE";
    
    /// CS_CLI_TUI_MODE environment variable
    pub const CS_CLI_TUI_MODE: &str = "CS_CLI_TUI_MODE";
    
    /// CS_CLI_LOG_FILE environment variable
    pub const CS_CLI_LOG_FILE: &str = "CS_CLI_LOG_FILE";
    
    /// TEST_CUSTOMER_NAME environment variable
    pub const TEST_CUSTOMER_NAME: &str = "TEST_CUSTOMER_NAME";
    
    /// TEST_DAYS_BACK environment variable
    pub const TEST_DAYS_BACK: &str = "TEST_DAYS_BACK";
    
    /// GONG_DEBUG environment variable
    pub const GONG_DEBUG: &str = "GONG_DEBUG";
    
    /// GONG_HTTP_CONCURRENCY environment variable
    pub const GONG_HTTP_CONCURRENCY: &str = "GONG_HTTP_CONCURRENCY";
}

/// Setup common environment variables for development
pub fn setup_dev_env() {
    set_env_if_unset(EnvVars::RUST_LOG, "cs_cli=debug");
    set_env_if_unset(EnvVars::RUST_BACKTRACE, "1");
}

/// Setup environment variables for testing
pub fn setup_test_env() {
    set_env_if_unset(EnvVars::RUST_LOG, "cs_cli=debug");
    set_env_if_unset(EnvVars::TEST_CUSTOMER_NAME, "TestCustomer");
    set_env_if_unset(EnvVars::TEST_DAYS_BACK, "30");
}

/// Setup environment variables for TUI mode
pub fn setup_tui_env() {
    env::set_var(EnvVars::CS_CLI_TUI_MODE, "1");
}

/// Check if we're in TUI mode
pub fn is_tui_mode() -> bool {
    is_env_set(EnvVars::CS_CLI_TUI_MODE)
}

/// Get current log file path (if in TUI mode)
pub fn get_current_log_file() -> Option<String> {
    get_env_optional(EnvVars::CS_CLI_LOG_FILE)
}

/// Get test customer name
pub fn get_test_customer_name() -> String {
    get_env_or_default(EnvVars::TEST_CUSTOMER_NAME, "TestCustomer")
}

/// Get test days back
pub fn get_test_days_back() -> u32 {
    get_env_int(EnvVars::TEST_DAYS_BACK, 30)
}

/// Check if Gong debug mode is enabled
pub fn is_gong_debug_enabled() -> bool {
    get_env_bool(EnvVars::GONG_DEBUG, false)
}

/// Get Gong HTTP concurrency setting
pub fn get_gong_http_concurrency() -> Option<usize> {
    get_env_int::<usize>(EnvVars::GONG_HTTP_CONCURRENCY, 0).into()
}

/// Setup logging environment
pub fn setup_logging_env() {
    if !is_env_set(EnvVars::RUST_LOG) {
        set_env_if_unset(EnvVars::RUST_LOG, "cs_cli=info");
    }
}

/// Clear all CS-CLI environment variables
pub fn clear_cs_cli_env() {
    clear_env(EnvVars::CS_CLI_TUI_MODE);
    clear_env(EnvVars::CS_CLI_LOG_FILE);
}

/// Get all CS-CLI environment variables
pub fn get_cs_cli_env() -> std::collections::HashMap<String, String> {
    get_env_with_prefix("CS_CLI_")
}