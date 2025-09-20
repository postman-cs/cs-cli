//! Environment variable utilities
//!
//! Common environment variable handling patterns.

use std::env;

/// Get environment variable with default value
pub fn get_env_or_default(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

/// Get environment variable as boolean
pub fn get_env_bool(key: &str, default: bool) -> bool {
    match env::var(key) {
        Ok(value) => match value.to_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => true,
            "false" | "0" | "no" | "off" => false,
            _ => default,
        },
        Err(_) => default,
    }
}

/// Get environment variable as integer
pub fn get_env_int<T>(key: &str, default: T) -> T
where
    T: std::str::FromStr + Copy,
{
    match env::var(key) {
        Ok(value) => value.parse().unwrap_or(default),
        Err(_) => default,
    }
}

/// Get environment variable as float
pub fn get_env_float<T>(key: &str, default: T) -> T
where
    T: std::str::FromStr + Copy,
{
    match env::var(key) {
        Ok(value) => value.parse().unwrap_or(default),
        Err(_) => default,
    }
}

/// Set environment variable if not already set
pub fn set_env_if_unset(key: &str, value: &str) {
    if env::var(key).is_err() {
        env::set_var(key, value);
    }
}

/// Check if environment variable is set
pub fn is_env_set(key: &str) -> bool {
    env::var(key).is_ok()
}

/// Get environment variable or return None
pub fn get_env_optional(key: &str) -> Option<String> {
    env::var(key).ok()
}

/// Get environment variable with validation
pub fn get_env_validated<F>(key: &str, validator: F) -> Option<String>
where
    F: Fn(&str) -> bool,
{
    env::var(key).ok().filter(|value| validator(value))
}

/// Get environment variable with custom error handling
pub fn get_env_or_error(key: &str) -> Result<String, String> {
    env::var(key).map_err(|_| format!("Environment variable {} is required but not set", key))
}

/// Parse comma-separated environment variable
pub fn get_env_list(key: &str, default: &[String]) -> Vec<String> {
    match env::var(key) {
        Ok(value) => {
            if value.is_empty() {
                default.to_vec()
            } else {
                value.split(',').map(|s| s.trim().to_string()).collect()
            }
        }
        Err(_) => default.to_vec(),
    }
}

/// Set multiple environment variables from a map
pub fn set_env_vars(vars: &std::collections::HashMap<String, String>) {
    for (key, value) in vars {
        env::set_var(key, value);
    }
}

/// Clear environment variable
pub fn clear_env(key: &str) {
    env::remove_var(key);
}

/// Get all environment variables with a prefix
pub fn get_env_with_prefix(prefix: &str) -> std::collections::HashMap<String, String> {
    env::vars()
        .filter(|(key, _)| key.starts_with(prefix))
        .collect()
}