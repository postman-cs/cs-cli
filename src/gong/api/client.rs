use crate::common::config::HttpSettings;
use crate::common::http::{BrowserHttpClient, BrowserHttpClientPool};
use crate::gong::config::settings::AppConfig;
use crate::Result;

// Type alias for Gong HTTP client - now uses common browser client
pub type GongHttpClient = BrowserHttpClient;

impl GongHttpClient {
    /// Update headers (compatibility method for existing Gong code)
    pub async fn update_headers(
        &self,
        headers: std::collections::HashMap<String, String>,
    ) -> Result<()> {
        use crate::common::http::HttpClient;
        self.set_headers(headers).await
    }
}

// Type alias for Gong HTTP client pool - now uses common browser client pool
pub type HttpClientPool = BrowserHttpClientPool;

/// Implementation of compatibility methods for Gong's HttpClientPool
impl BrowserHttpClientPool {
    /// Create a new HTTP client pool (compatibility method for existing Gong code)
    pub async fn new_gong_pool(config: Option<HttpSettings>) -> Result<Self> {
        let config = config.unwrap_or_else(|| {
            let app_config = AppConfig::create_default();
            app_config.http
        });

        BrowserHttpClientPool::new_browser_pool(config).await
    }

    /// Update headers on all clients (compatibility method for existing Gong code)
    pub async fn update_headers(
        &self,
        headers: std::collections::HashMap<String, String>,
    ) -> Result<()> {
        self.set_headers(headers).await
    }
}

// Make HttpClientPool the default export matching Python HTTPClientPool
pub type HTTPClientPool = HttpClientPool;
