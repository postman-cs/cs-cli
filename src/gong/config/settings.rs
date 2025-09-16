use crate::{CsCliError, Result};
use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};

// HttpSettings and AuthSettings are now imported from common::config
// Re-export for backward compatibility within gong modules
pub use crate::common::config::{AuthSettings, HttpSettings};

/// Main application configuration matching Python SimplifiedPerformanceConfig
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub http: HttpSettings,
    pub auth: AuthSettings,
    pub debug: bool,
    pub max_concurrent_timeline_requests: usize,
    pub max_concurrent_email_requests: usize,
    pub max_concurrent_transcript_requests: usize,
    pub max_workers: usize,
    pub worker_idle_sleep_ms: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            http: HttpSettings::default(),
            auth: AuthSettings::default(),
            debug: false,
            max_concurrent_timeline_requests: 105,
            max_concurrent_email_requests: 135,
            max_concurrent_transcript_requests: 60,
            max_workers: 80,
            worker_idle_sleep_ms: 10,
        }
    }
}

impl AppConfig {
    /// Create default configuration matching Python create_default()
    pub fn create_default() -> Self {
        let mut http = HttpSettings {
            pool_size: 50,
            max_concurrency_per_client: 40,
            timeout_seconds: 30.0,
            max_clients: None,
            global_max_concurrency: Some(2000),
            enable_http3: false,
            force_http3: false,
            tls_version: None,
            impersonate_browser: "chrome".to_string(),
        };
        http.validate_and_fill_defaults();

        Self {
            http,
            auth: AuthSettings {
                csrf_token_ttl_minutes: 30,
                csrf_token_buffer_minutes: 5,
                retry_attempts: 3,
                retry_backoff_base: 2.0,
                retry_backoff_seconds: 1.0,
            },
            debug: false,
            max_concurrent_timeline_requests: 105,
            max_concurrent_email_requests: 135,
            max_concurrent_transcript_requests: 60,
            max_workers: 80,
            worker_idle_sleep_ms: 10,
        }
    }

    /// Load configuration from environment matching Python from_env()
    pub fn from_env() -> Result<Self> {
        let mut config = Self::create_default();

        // Check debug flag
        if let Ok(debug_val) = std::env::var("GONG_DEBUG") {
            match debug_val.to_lowercase().as_str() {
                "true" | "1" | "yes" => config.debug = true,
                _ => {}
            }
        }

        // Handle HTTP concurrency override
        if let Ok(concurrency_val) = std::env::var("GONG_HTTP_CONCURRENCY") {
            if let Ok(total_concurrency) = concurrency_val.parse::<usize>() {
                config.http.global_max_concurrency = Some(std::cmp::max(1, total_concurrency));
                let per_client = std::cmp::max(
                    1,
                    total_concurrency / std::cmp::max(1, config.http.pool_size),
                );
                config.http.max_concurrency_per_client = per_client;
            } else {
                tracing::warn!("Invalid GONG_HTTP_CONCURRENCY value: {}", concurrency_val);
            }
        }

        config.validate()?;
        Ok(config)
    }

    /// Load configuration from multiple sources with precedence:
    /// 1. Environment variables (highest priority)
    /// 2. Config file (cs-cli.toml)
    /// 3. Defaults (lowest priority)
    pub fn load() -> Result<Self> {
        let config = Figment::new()
            .merge(Toml::file("cs-cli.toml")) // Optional config file
            .merge(Env::prefixed("CS_CLI_")) // Environment variables
            .extract()
            .map_err(|e| CsCliError::Configuration(format!("Configuration error: {e}")))?;

        Ok(config)
    }

    /// Validate configuration matching Python validate()
    pub fn validate(&self) -> Result<()> {
        if self.http.pool_size == 0 {
            return Err(CsCliError::Configuration(
                "HTTP pool_size must be positive".to_string(),
            ));
        }
        if self.http.max_concurrency_per_client == 0 {
            return Err(CsCliError::Configuration(
                "HTTP max_concurrency_per_client must be positive".to_string(),
            ));
        }
        Ok(())
    }

    /// Create a configuration with custom settings
    pub fn with_settings(http: HttpSettings, auth: AuthSettings) -> Self {
        Self {
            http,
            auth,
            debug: false,
            max_concurrent_timeline_requests: 105,
            max_concurrent_email_requests: 135,
            max_concurrent_transcript_requests: 60,
            max_workers: 80,
            worker_idle_sleep_ms: 10,
        }
    }
}
