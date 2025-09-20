//! Browser-compatible HTTP client implementation
//!
//! Generic HTTP client with TLS fingerprinting and browser compatibility
//! that can be used by both Gong and Slack integrations.

use super::client::HttpClient;
use super::error_helpers::{HttpErrorMapper, DefaultHttpErrorMapper};
use super::pool::GenericHttpClientPool;
use crate::common::config::HttpSettings;
use crate::Result;
use async_trait::async_trait;
use futures;
use impit::cookie::Jar;
use impit::emulation::Browser;
use impit::impit::Impit;
use reqwest::Response;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, Semaphore};
use tracing::{debug, info, warn};

/// Browser-compatible HTTP client with TLS fingerprinting
pub struct BrowserHttpClient {
    client: Impit<Jar>,
    config: HttpSettings,
    semaphore: Arc<Semaphore>,
    cookies: Arc<Mutex<HashMap<String, String>>>,
    headers: Arc<Mutex<HashMap<String, String>>>,
}

impl BrowserHttpClient {
    /// Create a new browser HTTP client with the specified configuration
    pub async fn new(config: HttpSettings) -> Result<Self> {
        // Parse browser type - impit supports Chrome and Firefox directly
        // Other browsers (Brave, Edge, Safari) are Chromium-based so use Chrome fingerprinting
        let browser = match config.browser_type.to_lowercase().as_str() {
            "firefox" => Browser::Firefox,
            "chrome" => Browser::Chrome,
            "brave" | "edge" | "safari" | "chromium" | "arc" | "opera" | "vivaldi" => {
                info!(
                    "Browser '{}' is Chromium-based, using Chrome fingerprinting",
                    config.browser_type
                );
                Browser::Chrome
            }
            _ => {
                warn!(
                    "Unknown browser '{}', defaulting to Chrome",
                    config.browser_type
                );
                Browser::Chrome
            }
        };

        let mut builder = Impit::builder().with_browser(browser);

        // Always enable HTTP3 support for intelligent fallback
        // The client will attempt HTTP3 first and automatically fall back to HTTP2
        builder = builder.with_http3();
        debug!("HTTP client configured with intelligent HTTP/3 -> HTTP/2 fallback");

        // Enhanced fingerprinting to match browser exactly
        // impit automatically handles:
        // - TLS fingerprinting (JA3/JA4)
        // - HTTP/2 fingerprinting (frame ordering, settings, priorities)
        // - HTTP/3 fingerprinting (QUIC parameters, frame patterns)
        // - Header ordering and casing (browser-specific)
        // - Connection patterns and timing behaviors
        // - Browser-specific protocol negotiations

        debug!(
            "Building HTTP client with enhanced {} fingerprinting",
            config.browser_type
        );
        debug!("Features: TLS(JA3/JA4) + HTTP/2 + HTTP/3(QUIC) + Headers + Timing");

        let client = builder.build();
        let semaphore = Arc::new(Semaphore::new(config.max_concurrency_per_client));

        debug!(
            "Browser HTTP client initialized: browser={}, concurrency={}, timeout={:.1}s",
            config.browser_type, config.max_concurrency_per_client, config.timeout_seconds
        );

        Ok(Self {
            client,
            config,
            semaphore,
            cookies: Arc::new(Mutex::new(HashMap::new())),
            headers: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Make HTTP request with advanced retry logic and rate limiting (from Gong client)
    async fn make_request_with_retries(
        &self,
        method: &str,
        url: &str,
        body: Option<&str>,
    ) -> Result<Response> {
        // Acquire semaphore permit for rate limiting
        let mapper = DefaultHttpErrorMapper;
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|e| mapper.map_semaphore_error(e))?;

<<<<<<< HEAD
        // Use common retry utilities
        let config = crate::common::retry::PlatformRetryConfigs::http();
        let operation_name = format!("{} {}", method, url);
        
        crate::common::retry::execute_http_with_retry(
            || self.make_request(method, url, body),
            config,
            &operation_name,
        ).await
=======
        for attempt in 0..MAX_RETRIES {
            info!("Making HTTP request: {} {} (attempt {})", method, url, attempt + 1);
            match self.make_request(method, url, body).await {
                Ok(response) => {
                    let status = response.status().as_u16();
                    info!("HTTP request completed: {} {} -> {}", method, url, status);

                    // Handle rate limiting (429) - honor Retry-After header
                    if status == 429 {
                        let sleep_duration = self.calculate_retry_delay(&response, attempt).await;
                        warn!(
                            "Rate limited: url={}, attempt={}, retry_after={:.1}s",
                            url,
                            attempt,
                            sleep_duration.as_secs_f64()
                        );
                        tokio::time::sleep(sleep_duration).await;
                        continue;
                    }

                    // Handle server errors (5xx)
                    if status >= 500 {
                        if attempt < MAX_RETRIES - 1 {
                            let sleep_duration = self.exponential_backoff_delay(attempt);
                            warn!(
                                "Server error - will retry: url={}, status={}, attempt={}",
                                url, status, attempt
                            );
                            tokio::time::sleep(sleep_duration).await;
                            continue;
                        } else {
                            warn!(
                                "Server error - not retrying: url={}, status={}, attempt={}",
                                url, status, attempt
                            );
                        }
                    }

                    // Handle client errors (4xx) - don't retry except for 429
                    if (400..500).contains(&status) && status != 429 {
                        warn!(
                            "Client error - not retrying: url={}, status={}, attempt={}",
                            url, status, attempt
                        );
                    }

                    return Ok(response);
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < MAX_RETRIES - 1 {
                        let sleep_duration = self.exponential_backoff_delay(attempt);
                        tokio::time::sleep(sleep_duration).await;
                    }
                }
            }
        }

        Err(last_error
            .unwrap_or_else(|| CsCliError::ApiRequest("Request failed after retries".to_string())))
>>>>>>> 30887b9 (github auth improvements)
    }


    /// Make the actual HTTP request using impit API
    async fn make_request(&self, method: &str, url: &str, body: Option<&str>) -> Result<Response> {
        use reqwest::header::{HeaderMap, HeaderName, HeaderValue, COOKIE};
        use std::str::FromStr;

        // Build headers
        let mut header_map = HeaderMap::new();

        // Add custom headers
        let headers = self.headers.lock().await;
        for (name, value) in headers.iter() {
            if let (Ok(header_name), Ok(header_value)) =
                (HeaderName::from_str(name), HeaderValue::from_str(value))
            {
                header_map.insert(header_name, header_value);
            }
        }
        drop(headers);

        // Add cookies
        let cookies = self.cookies.lock().await;
        if !cookies.is_empty() {
            let cookie_string: String = cookies
                .iter()
                .map(|(name, value)| format!("{name}={value}"))
                .collect::<Vec<_>>()
                .join("; ");

            if let Ok(cookie_value) = HeaderValue::from_str(&cookie_string) {
                header_map.insert(COOKIE, cookie_value);
            }
        }
        drop(cookies);

        // Convert HeaderMap to Vec<(String, String)> as required by impit
        let headers_vec = header_map
            .iter()
            .map(|(name, value)| (name.to_string(), value.to_str().unwrap_or("").to_string()))
            .collect::<Vec<(String, String)>>();

        let request_options = Some(impit::request::RequestOptions {
            headers: headers_vec,
            timeout: Some(Duration::from_secs_f64(self.config.timeout_seconds)),
            // Set to false for intelligent fallback: try HTTP3, fall back to HTTP2 automatically
            http3_prior_knowledge: false,
        });

        // Make the request using impit API (url: String, body: Option<Vec<u8>>, options: Option<RequestOptions>)
        let mapper = DefaultHttpErrorMapper;
        let response = match method.to_uppercase().as_str() {
            "GET" => self
                .client
                .get(url.to_string(), None, request_options)
                .await
                .map_err(|e| mapper.map_api_error("GET", e)),
            "POST" => {
                // For POST requests, convert body to Vec<u8>
                let body_bytes = body.map(|b| b.as_bytes().to_vec());
                self.client
                    .post(url.to_string(), body_bytes, request_options)
                    .await
                    .map_err(|e| mapper.map_api_error("POST", e))
            }
            "PUT" => {
                // For PUT requests, convert body to Vec<u8>
                let body_bytes = body.map(|b| b.as_bytes().to_vec());
                self.client
                    .put(url.to_string(), body_bytes, request_options)
                    .await
                    .map_err(|e| mapper.map_api_error("PUT", e))
            }
            "DELETE" => self
                .client
                .delete(url.to_string(), None, request_options)
                .await
                .map_err(|e| mapper.map_api_error("DELETE", e)),
            _ => {
                return Err(mapper.map_api_error(method, "Unsupported HTTP method"))
            }
        };

        response
    }

    /// Calculate exponential backoff delay with jitter (from Gong client)
    #[allow(dead_code)]
    fn exponential_backoff_delay(&self, attempt: u32) -> Duration {
        use rand::Rng;

        let base_delay = 2_u64.pow(attempt);
        let jitter = rand::thread_rng().gen_range(0.0..=0.5);
        let total_seconds = base_delay as f64 + (jitter * base_delay as f64);

        Duration::from_secs_f64(total_seconds)
    }

    /// Perform multiple GET requests concurrently (from Gong client)
    pub async fn batch_get(&self, urls: Vec<String>) -> Vec<Result<Response>> {
        let tasks = urls.into_iter().map(|url| {
            let client = self;
            async move { client.get(&url).await }
        });
        futures::future::join_all(tasks).await
    }

    /// Perform multiple POST requests concurrently (from Gong client)
    pub async fn batch_post(
        &self,
        requests: Vec<(String, Option<String>)>,
    ) -> Vec<Result<Response>> {
        let tasks = requests.into_iter().map(|(url, body)| {
            let client = self;
            async move { client.post(&url, body.as_deref()).await }
        });
        futures::future::join_all(tasks).await
    }
}

#[async_trait]
impl HttpClient for BrowserHttpClient {
    async fn get(&self, url: &str) -> Result<Response> {
        debug!("Making GET request to: {}", url);
        self.make_request_with_retries("GET", url, None).await
    }

    async fn post(&self, url: &str, body: Option<&str>) -> Result<Response> {
        debug!("Making POST request to: {}", url);
        self.make_request_with_retries("POST", url, body).await
    }

    async fn put(&self, url: &str, body: Option<&str>) -> Result<Response> {
        debug!("Making PUT request to: {}", url);
        self.make_request_with_retries("PUT", url, body).await
    }

    async fn delete(&self, url: &str) -> Result<Response> {
        debug!("Making DELETE request to: {}", url);
        self.make_request_with_retries("DELETE", url, None).await
    }

    async fn set_cookies(&self, cookies: HashMap<String, String>) -> Result<()> {
        debug!("Setting {} cookies", cookies.len());
        let mut client_cookies = self.cookies.lock().await;
        *client_cookies = cookies;
        Ok(())
    }

    async fn set_headers(&self, headers: HashMap<String, String>) -> Result<()> {
        debug!("Setting {} headers", headers.len());
        let mut client_headers = self.headers.lock().await;
        *client_headers = headers;
        Ok(())
    }

    async fn get_headers(&self) -> HashMap<String, String> {
        let headers = self.headers.lock().await;
        headers.clone()
    }

    async fn health_check(&self) -> bool {
        // Simple health check - try to acquire semaphore permit
        match self.semaphore.try_acquire() {
            Ok(_permit) => true,
            Err(_) => false,
        }
    }
}

/// Type alias for a pool of browser HTTP clients
pub type BrowserHttpClientPool = GenericHttpClientPool<BrowserHttpClient>;

impl BrowserHttpClientPool {
    /// Create a new pool of browser HTTP clients
    pub async fn new_browser_pool(config: HttpSettings) -> Result<Self> {
        GenericHttpClientPool::<BrowserHttpClient>::new(config, |config| async move {
            BrowserHttpClient::new(config).await
        })
        .await
    }
}
