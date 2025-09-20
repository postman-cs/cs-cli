//! Platform-specific HTTP client configuration builder
//!
//! Eliminates duplicated HTTP client configuration patterns across platforms
//! by providing a unified builder pattern for platform-specific HTTP clients.

use crate::common::config::HttpSettings;
use crate::Result;
use tracing::{info, warn};

use super::{BrowserHttpClient, HttpClient};

/// Builder for platform-specific HTTP clients
pub struct PlatformHttpClientBuilder {
    browser_type: String,
    pool_size: usize,
    max_concurrency_per_client: usize,
    timeout_seconds: f64,
    enable_http3: bool,
    force_http3: bool,
}

impl PlatformHttpClientBuilder {
    /// Create new builder with detected browser type
    pub fn new(browser_type: Option<&str>) -> Self {
        Self {
            browser_type: browser_type.unwrap_or("chrome").to_string(),
            pool_size: 1,
            max_concurrency_per_client: 5,
            timeout_seconds: 30.0,
            enable_http3: true,
            force_http3: false,
        }
    }
    
    /// Set pool size
    pub fn with_pool_size(mut self, size: usize) -> Self {
        self.pool_size = size;
        self
    }
    
    /// Set concurrency per client
    pub fn with_concurrency(mut self, concurrency: usize) -> Self {
        self.max_concurrency_per_client = concurrency;
        self
    }
    
    /// Set timeout in seconds
    pub fn with_timeout(mut self, timeout: f64) -> Self {
        self.timeout_seconds = timeout;
        self
    }
    
    /// Enable/disable HTTP/3
    pub fn with_http3(mut self, enable: bool) -> Self {
        self.enable_http3 = enable;
        self
    }
    
    /// Force HTTP/3 (disable fallback)
    pub fn force_http3(mut self, force: bool) -> Self {
        self.force_http3 = force;
        self
    }
    
    /// Build HttpSettings
    pub fn build_settings(self) -> HttpSettings {
        HttpSettings {
            pool_size: self.pool_size,
            max_concurrency_per_client: self.max_concurrency_per_client,
            timeout_seconds: self.timeout_seconds,
            max_clients: Some(self.pool_size * self.max_concurrency_per_client),
            global_max_concurrency: Some(self.pool_size * self.max_concurrency_per_client),
            tls_version: None,
            browser_type: self.browser_type,
        }
    }
    
    /// Build and create BrowserHttpClient
    pub async fn build_client(self) -> Result<BrowserHttpClient> {
        let settings = self.build_settings();
        info!("Creating HTTP client with browser type: {}", settings.browser_type);
        BrowserHttpClient::new(settings).await
    }
}

/// Platform-specific HTTP client configurations
pub struct PlatformConfigs;

impl PlatformConfigs {
    /// Gong HTTP client configuration
    pub fn gong(browser_type: Option<&str>) -> PlatformHttpClientBuilder {
        PlatformHttpClientBuilder::new(browser_type)
            .with_pool_size(50)
            .with_concurrency(40)
            .with_timeout(30.0)
            .with_http3(true)
            .force_http3(false)
    }
    
    /// Slack HTTP client configuration
    pub fn slack(browser_type: Option<&str>) -> PlatformHttpClientBuilder {
        PlatformHttpClientBuilder::new(browser_type)
            .with_pool_size(1)
            .with_concurrency(5)
            .with_timeout(30.0)
            .with_http3(true)
            .force_http3(false)
    }
    
    /// Gainsight HTTP client configuration
    pub fn gainsight(browser_type: Option<&str>) -> PlatformHttpClientBuilder {
        PlatformHttpClientBuilder::new(browser_type)
            .with_pool_size(1)
            .with_concurrency(3)
            .with_timeout(45.0) // Gainsight can be slower
            .with_http3(true)
            .force_http3(false)
    }
}

/// Common HTTP client setup patterns
pub struct CommonHttpSetup;

impl CommonHttpSetup {
    /// Setup HTTP client with authentication cookies and headers
    pub async fn setup_authenticated_client(
        client: &mut BrowserHttpClient,
        cookies: &std::collections::HashMap<String, String>,
        headers: &std::collections::HashMap<String, String>,
    ) -> Result<()> {
        // Set cookies
        client.set_cookies(cookies.clone()).await?;
        
        // Set headers
        client.set_headers(headers.clone()).await?;
        
        info!("HTTP client configured with {} cookies and {} headers", 
              cookies.len(), headers.len());
        
        Ok(())
    }
    
    /// Validate HTTP client health
    pub async fn validate_client_health(client: &BrowserHttpClient) -> Result<bool> {
        let is_healthy = client.health_check().await;
        if !is_healthy {
            warn!("HTTP client health check failed");
        }
        Ok(is_healthy)
    }
}