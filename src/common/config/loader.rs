//! Common configuration loading utilities
//!
//! Provides standardized config loading from files and environment variables
//! that can be used by both Gong and Slack integrations.

use crate::{CsCliError, Result};
use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use serde::de::DeserializeOwned;

/// Load configuration from multiple sources with precedence:
/// 1. Environment variables (highest priority)
/// 2. Config file (cs-cli.toml)  
/// 3. Defaults (lowest priority)
pub fn load_config<T>() -> Result<T>
where
    T: DeserializeOwned,
{
    let config = Figment::new()
        .merge(Toml::file("cs-cli.toml")) // Optional config file
        .merge(Env::prefixed("CS_CLI_")) // Environment variables
        .extract()
        .map_err(|e| CsCliError::Configuration(format!("Configuration error: {e}")))?;

    Ok(config)
}

/// Load configuration with custom environment prefix
pub fn load_config_with_prefix<T>(env_prefix: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    let config = Figment::new()
        .merge(Toml::file("cs-cli.toml"))
        .merge(Env::prefixed(env_prefix))
        .extract()
        .map_err(|e| CsCliError::Configuration(format!("Configuration error: {e}")))?;

    Ok(config)
}

/// Load configuration from a custom TOML file
pub fn load_config_from_file<T>(file_path: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    let config = Figment::new()
        .merge(Toml::file(file_path))
        .merge(Env::prefixed("CS_CLI_"))
        .extract()
        .map_err(|e| CsCliError::Configuration(format!("Configuration error: {e}")))?;

    Ok(config)
}
