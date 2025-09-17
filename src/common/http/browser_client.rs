//! Browser-impersonating HTTP client implementation
//!
//! Generic HTTP client with TLS fingerprinting and browser impersonation
//! that can be used by both Gong and Slack integrations.

use super::client::HttpClient;
use crate::common::config::HttpSettings;
use crate::{CsCliError, Result};
use async_trait::async_trait;
use impit::cookie::Jar;
use impit::emulation::Browser;
use impit::impit::Impit;
use reqwest::Response;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, Semaphore};
use tracing::{debug, info, warn};

/// Browser-impersonating HTTP client with TLS fingerprinting
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
        let browser = match config.impersonate_browser.to_lowercase().as_str() {
            "firefox" => Browser::Firefox,
            "chrome" => Browser::Chrome,
            "brave" | "edge" | "safari" | "chromium" | "arc" | "opera" | "vivaldi" => {
                info!(
                    "Browser '{}' is Chromium-based, using Chrome fingerprinting",
                    config.impersonate_browser
                );
                Browser::Chrome
            }
            _ => {
                warn!(
                    "Unknown browser '{}', defaulting to Chrome",
                    config.impersonate_browser
                );
                Browser::Chrome
            }
        };

        let mut builder = Impit::builder().with_browser(browser);

        // Configure HTTP version - prefer HTTP/3 for better performance and modern browser matching
        if config.enable_http3 || config.force_http3 {
            builder = builder.with_http3();
            if config.force_http3 {
                info!("HTTP client configured for HTTP/3 (forced, no fallback)");
            } else {
                info!("HTTP client configured for HTTP/3 with HTTP/2 fallback");
            }
        } else {
            info!("HTTP client configured for HTTP/2 only");
        }

        // Enhanced fingerprinting to match browser exactly
        // impit automatically handles:
        // - TLS fingerprinting (JA3/JA4)
        // - HTTP/2 fingerprinting (frame ordering, settings, priorities)
        // - HTTP/3 fingerprinting (QUIC parameters, frame patterns)
        // - Header ordering and casing (browser-specific)
        // - Connection patterns and timing behaviors
        // - Browser-specific protocol negotiations

        info!(
            "Building HTTP client with enhanced {} fingerprinting",
            config.impersonate_browser
        );
        info!("Features: TLS(JA3/JA4) + HTTP/2 + HTTP/3(QUIC) + Headers + Timing");

        let client = builder.build();
        let semaphore = Arc::new(Semaphore::new(config.max_concurrency_per_client));

        info!(
            "Browser HTTP client initialized: browser={}, concurrency={}, timeout={:.1}s",
            config.impersonate_browser, config.max_concurrency_per_client, config.timeout_seconds
        );

        Ok(Self {
            client,
            config,
            semaphore,
            cookies: Arc::new(Mutex::new(HashMap::new())),
            headers: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Make HTTP request with retry logic and rate limiting
    async fn make_request_with_retries(
        &self,
        method: &str,
        url: &str,
        body: Option<&str>,
    ) -> Result<Response> {
        const MAX_RETRIES: usize = 3;
        let mut last_error: Option<CsCliError> = None;

        // Acquire semaphore permit for rate limiting
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|e| CsCliError::Generic(format!("Failed to acquire semaphore: {e}")))?;

        for attempt in 0..MAX_RETRIES {
            match self.make_request(method, url, body).await {
                Ok(response) => {
                    let status = response.status().as_u16();

                    // Handle rate limiting (429)
                    if status == 429 {
                        if attempt < MAX_RETRIES - 1 {
                            let sleep_duration = self.exponential_backoff_delay(attempt);
                            warn!(
                                "Rate limited - will retry: url={}, attempt={}, delay={:.1}s",
                                url,
                                attempt,
                                sleep_duration.as_secs_f64()
                            );
                            tokio::time::sleep(sleep_duration).await;
                            continue;
                        }
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
                .map(|(name, value)| format!("{}={}", name, value))
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
            http3_prior_knowledge: self.config.enable_http3 || self.config.force_http3,
        });

        // Make the request using impit API (url: String, body: Option<Vec<u8>>, options: Option<RequestOptions>)
        let response = match method.to_uppercase().as_str() {
            "GET" => self
                .client
                .get(url.to_string(), None, request_options)
                .await
                .map_err(|e| CsCliError::ApiRequest(format!("GET request failed: {e}"))),
            "POST" => {
                // For POST requests, convert body to Vec<u8>
                let body_bytes = body.map(|b| b.as_bytes().to_vec());
                self.client
                    .post(url.to_string(), body_bytes, request_options)
                    .await
                    .map_err(|e| CsCliError::ApiRequest(format!("POST request failed: {e}")))
            }
            "PUT" => {
                // For PUT requests, convert body to Vec<u8>
                let body_bytes = body.map(|b| b.as_bytes().to_vec());
                self.client
                    .put(url.to_string(), body_bytes, request_options)
                    .await
                    .map_err(|e| CsCliError::ApiRequest(format!("PUT request failed: {e}")))
            }
            "DELETE" => self
                .client
                .delete(url.to_string(), None, request_options)
                .await
                .map_err(|e| CsCliError::ApiRequest(format!("DELETE request failed: {e}"))),
            _ => {
                return Err(CsCliError::ApiRequest(format!(
                    "Unsupported HTTP method: {}",
                    method
                )))
            }
        };

        response
    }

    /// Calculate exponential backoff delay for retries
    fn exponential_backoff_delay(&self, attempt: usize) -> Duration {
        let base_delay = Duration::from_millis(1000); // 1 second base
        let delay_ms = base_delay.as_millis() * (2_u128.pow(attempt as u32));
        let max_delay = Duration::from_secs(30); // Cap at 30 seconds

        Duration::from_millis(std::cmp::min(delay_ms as u64, max_delay.as_millis() as u64))
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
