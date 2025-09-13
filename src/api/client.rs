use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use impit::cookie::Jar;
use impit::emulation::Browser;
use impit::impit::Impit;
use impit::request::RequestOptions;
use reqwest::Response;
use tokio::sync::{Mutex, Semaphore};
use tracing::{debug, info, warn};

use crate::config::settings::{AppConfig, HttpSettings};
use crate::{CsCliError, Result};

/// HTTP client with TLS fingerprinting and browser impersonation
pub struct GongHttpClient {
    client: Impit<Jar>,
    config: HttpSettings,
    semaphore: Arc<Semaphore>,
    cookies: Arc<Mutex<HashMap<String, String>>>,
    headers: Arc<Mutex<HashMap<String, String>>>,
}

impl GongHttpClient {
    /// Create a new HTTP client with browser impersonation
    pub async fn new(config: HttpSettings) -> Result<Self> {
        let mut builder = Impit::builder().with_browser(Browser::Chrome); // Default browser impersonation

        // Configure HTTP version based on settings
        if config.enable_http3 || config.force_http3 {
            builder = builder.with_http3();
            info!("HTTP client configured for HTTP/3");
        } else {
            info!("HTTP client configured for HTTP/2");
        }

        let client = builder.build();

        let semaphore = Arc::new(Semaphore::new(config.max_concurrency_per_client));

        info!(
            "HTTP client initialized: concurrency={}, timeout={:.1}s, browser={}",
            config.max_concurrency_per_client, config.timeout_seconds, config.impersonate_browser
        );

        Ok(Self {
            client,
            config,
            semaphore,
            cookies: Arc::new(Mutex::new(HashMap::new())),
            headers: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Set cookies for all requests
    pub async fn set_cookies(&self, cookies: HashMap<String, String>) -> Result<()> {
        let mut client_cookies = self.cookies.lock().await;
        client_cookies.extend(cookies.clone());
        debug!("Cookies updated: count={}", cookies.len());
        Ok(())
    }

    /// Update default headers
    pub async fn update_headers(&self, headers: HashMap<String, String>) -> Result<()> {
        let mut client_headers = self.headers.lock().await;
        client_headers.extend(headers.clone());
        debug!(
            "Headers updated: new_headers={:?}",
            headers.keys().collect::<Vec<_>>()
        );
        Ok(())
    }

    /// Perform GET request with concurrency control and retry logic
    pub async fn get(&self, url: &str) -> Result<Response> {
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|e| CsCliError::ApiRequest(format!("Failed to acquire semaphore: {e}")))?;

        self.request_with_retry("GET", url, None).await
    }

    /// Perform POST request with concurrency control and retry logic
    pub async fn post(&self, url: &str, body: Option<String>) -> Result<Response> {
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|e| CsCliError::ApiRequest(format!("Failed to acquire semaphore: {e}")))?;

        self.request_with_retry("POST", url, body).await
    }

    /// Internal request method with exponential backoff retry logic
    async fn request_with_retry(
        &self,
        method: &str,
        url: &str,
        body: Option<String>,
    ) -> Result<Response> {
        const MAX_RETRIES: u32 = 3;
        let mut last_error = None;

        for attempt in 0..MAX_RETRIES {
            match self.make_request(method, url, body.as_deref()).await {
                Ok(response) => {
                    let status = response.status().as_u16();

                    // Handle rate limiting (429)
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

                    // Handle client errors (4xx) - don't retry
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

        // Add cookies as a single Cookie header
        let cookies = self.cookies.lock().await;
        if !cookies.is_empty() {
            let cookie_string: String = cookies
                .iter()
                .map(|(name, value)| format!("{name}={value}"))
                .collect::<Vec<_>>()
                .join("; ");

            println!(
                "🔍 DEBUG: Adding {} cookies to request: {}",
                cookies.len(),
                &cookie_string[..std::cmp::min(100, cookie_string.len())]
            );

            if let Ok(cookie_value) = HeaderValue::from_str(&cookie_string) {
                header_map.insert(COOKIE, cookie_value);
                println!("🔍 DEBUG: Cookie header added successfully");
            } else {
                println!("🔍 DEBUG: Failed to create Cookie header value");
            }
        } else {
            println!("🔍 DEBUG: NO COOKIES TO ADD - this is the problem!");
        }
        drop(cookies);

        // Create RequestOptions with headers
        let headers_vec = if header_map.is_empty() {
            Vec::new()
        } else {
            header_map
                .iter()
                .map(|(name, value)| (name.to_string(), value.to_str().unwrap_or("").to_string()))
                .collect::<Vec<(String, String)>>()
        };

        let request_options = Some(RequestOptions {
            headers: headers_vec,
            timeout: Some(Duration::from_secs_f64(self.config.timeout_seconds)),
            http3_prior_knowledge: self.config.enable_http3 || self.config.force_http3,
        });

        // Make the request using impit API (url, body, options)
        match method.to_uppercase().as_str() {
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
            _ => Err(CsCliError::ApiRequest(format!(
                "Unsupported HTTP method: {method}"
            ))),
        }
    }

    /// Calculate retry delay for rate limiting (honor Retry-After header)
    async fn calculate_retry_delay(&self, response: &Response, attempt: u32) -> Duration {
        // Try to get Retry-After header
        if let Some(retry_after) = response.headers().get("retry-after") {
            if let Ok(retry_after_str) = retry_after.to_str() {
                // Try parsing as seconds
                if let Ok(seconds) = retry_after_str.parse::<u64>() {
                    return Duration::from_secs(seconds);
                }

                // Could also handle HTTP date format here if needed
                // For now, fall back to exponential backoff
            }
        }

        // Exponential backoff with jitter
        self.exponential_backoff_delay(attempt)
    }

    /// Calculate exponential backoff delay with jitter
    fn exponential_backoff_delay(&self, attempt: u32) -> Duration {
        use rand::Rng;

        let base_delay = 2_u64.pow(attempt);
        let jitter = rand::thread_rng().gen_range(0.0..=0.5);
        let total_seconds = base_delay as f64 + (jitter * base_delay as f64);

        Duration::from_secs_f64(total_seconds)
    }

    /// Perform multiple GET requests concurrently
    pub async fn batch_get(&self, urls: Vec<String>) -> Vec<Result<Response>> {
        let tasks = urls.into_iter().map(|url| {
            let client = &self;
            async move { client.get(&url).await }
        });
        futures::future::join_all(tasks).await
    }

    /// Perform multiple POST requests concurrently
    pub async fn batch_post(
        &self,
        requests: Vec<(String, Option<String>)>,
    ) -> Vec<Result<Response>> {
        let tasks = requests.into_iter().map(|(url, body)| {
            let client = &self;
            async move { client.post(&url, body).await }
        });
        futures::future::join_all(tasks).await
    }
}

/// Pool of HTTP clients for maximum concurrency
pub struct HttpClientPool {
    clients: Vec<Arc<GongHttpClient>>,
    current_client: Arc<Mutex<usize>>,
    pool_size: usize,
    global_semaphore: Option<Arc<Semaphore>>,
    config: HttpSettings,
}

impl HttpClientPool {
    /// Create a new HTTP client pool
    pub async fn new(config: Option<HttpSettings>) -> Result<Self> {
        let config = config.unwrap_or_else(|| {
            let app_config = AppConfig::create_default();
            app_config.http
        });

        let pool_size = config.pool_size;
        let mut clients = Vec::with_capacity(pool_size);

        // Create pool of clients
        for i in 0..pool_size {
            let client = GongHttpClient::new(config.clone())
                .await
                .with_context(|| format!("Failed to create HTTP client {i}"))
                .map_err(|e| CsCliError::ApiRequest(format!("{e}")))?;
            clients.push(Arc::new(client));
        }

        // Global concurrency semaphore
        let global_semaphore = config
            .global_max_concurrency
            .map(|max_concurrent| Arc::new(Semaphore::new(max_concurrent)));

        let total_concurrency = pool_size * config.max_concurrency_per_client;
        info!(
            "HTTP client pool initialized: pool_size={}, concurrency_per_client={}, total_concurrency={}, global_cap={:?}",
            pool_size,
            config.max_concurrency_per_client,
            total_concurrency,
            config.global_max_concurrency
        );

        Ok(Self {
            clients,
            current_client: Arc::new(Mutex::new(0)),
            pool_size,
            global_semaphore,
            config,
        })
    }

    /// Set cookies on all clients
    pub async fn set_cookies(&self, cookies: HashMap<String, String>) -> Result<()> {
        let tasks = self
            .clients
            .iter()
            .map(|client| client.set_cookies(cookies.clone()));
        futures::future::try_join_all(tasks).await?;
        Ok(())
    }

    /// Update headers on all clients
    pub async fn update_headers(&self, headers: HashMap<String, String>) -> Result<()> {
        let tasks = self
            .clients
            .iter()
            .map(|client| client.update_headers(headers.clone()));
        futures::future::try_join_all(tasks).await?;
        Ok(())
    }

    /// Get next client using round-robin selection
    async fn get_client(&self) -> Arc<GongHttpClient> {
        let mut current = self.current_client.lock().await;
        let client = self.clients[*current].clone();
        *current = (*current + 1) % self.pool_size;
        client
    }

    /// Perform GET request using round-robin client selection with global rate limiting
    pub async fn get(&self, url: &str) -> Result<Response> {
        let client = self.get_client().await;

        if let Some(global_sem) = &self.global_semaphore {
            let _global_permit = global_sem.acquire().await.map_err(|e| {
                CsCliError::ApiRequest(format!("Failed to acquire global semaphore: {e}"))
            })?;
            client.get(url).await
        } else {
            client.get(url).await
        }
    }

    /// Perform POST request using round-robin client selection with global rate limiting
    pub async fn post(&self, url: &str, body: Option<String>) -> Result<Response> {
        let client = self.get_client().await;

        if let Some(global_sem) = &self.global_semaphore {
            let _global_permit = global_sem.acquire().await.map_err(|e| {
                CsCliError::ApiRequest(format!("Failed to acquire global semaphore: {e}"))
            })?;
            client.post(url, body).await
        } else {
            client.post(url, body).await
        }
    }

    /// Get pool configuration for monitoring
    pub fn get_config(&self) -> &HttpSettings {
        &self.config
    }

    /// Distribute batch requests across all clients
    pub async fn batch_requests(
        &self,
        requests: Vec<(String, String, Option<String>)>,
    ) -> Vec<Result<Response>> {
        // Group requests by client using round-robin distribution
        let mut client_requests: Vec<Vec<(String, String, Option<String>)>> =
            vec![Vec::new(); self.pool_size];

        for (i, request) in requests.into_iter().enumerate() {
            let client_idx = i % self.pool_size;
            client_requests[client_idx].push(request);
        }

        // Execute requests on each client
        let mut tasks = Vec::new();

        for (client_idx, client_reqs) in client_requests.into_iter().enumerate() {
            if client_reqs.is_empty() {
                continue;
            }

            let client = self.clients[client_idx].clone();
            let global_sem = self.global_semaphore.clone();

            for (method, url, body) in client_reqs {
                let client = client.clone();
                let global_sem = global_sem.clone();

                let task = async move {
                    let result = if let Some(global_sem) = global_sem {
                        let _global_permit = global_sem.acquire().await.map_err(|e| {
                            CsCliError::ApiRequest(format!(
                                "Failed to acquire global semaphore: {e}"
                            ))
                        })?;

                        match method.to_uppercase().as_str() {
                            "GET" => client.get(&url).await,
                            "POST" => client.post(&url, body).await,
                            _ => Err(CsCliError::ApiRequest(format!(
                                "Unsupported HTTP method: {method}"
                            ))),
                        }
                    } else {
                        match method.to_uppercase().as_str() {
                            "GET" => client.get(&url).await,
                            "POST" => client.post(&url, body).await,
                            _ => Err(CsCliError::ApiRequest(format!(
                                "Unsupported HTTP method: {method}"
                            ))),
                        }
                    };
                    result
                };

                tasks.push(task);
            }
        }

        if tasks.is_empty() {
            return Vec::new();
        }

        futures::future::join_all(tasks).await
    }
}

// Make HttpClientPool the default export matching Python HTTPClientPool
pub type HTTPClientPool = HttpClientPool;
