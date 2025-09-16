//! Generic HTTP client pool implementation
//!
//! Provides client pooling and load balancing that can be used by both
//! Gong and Slack API integrations.

use super::client::{HttpClient, HttpClientPool, PoolStats};
use crate::common::config::HttpSettings;
use crate::{CsCliError, Result};
use async_trait::async_trait;
use reqwest::Response;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};
use tracing::info;

/// Generic HTTP client pool that can work with any HttpClient implementation
pub struct GenericHttpClientPool<T: HttpClient + 'static> {
    clients: Vec<Arc<T>>,
    current_client: Arc<Mutex<usize>>,
    pool_size: usize,
    global_semaphore: Option<Arc<Semaphore>>,
    config: HttpSettings,
    stats: Arc<Mutex<PoolStats>>,
}

impl<T: HttpClient + 'static> GenericHttpClientPool<T> {
    /// Create a new HTTP client pool with the given client factory function
    pub async fn new<F, Fut>(config: HttpSettings, client_factory: F) -> Result<Self>
    where
        F: Fn(HttpSettings) -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let pool_size = config.pool_size;
        let mut clients = Vec::with_capacity(pool_size);

        // Create pool of clients
        for i in 0..pool_size {
            let client = client_factory(config.clone())
                .await
                .map_err(|e| CsCliError::ApiRequest(format!("Failed to create HTTP client {}: {}", i, e)))?;
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

        let initial_stats = PoolStats {
            total_clients: pool_size,
            active_clients: pool_size,
            requests_completed: 0,
            requests_failed: 0,
            average_response_time_ms: 0.0,
        };

        Ok(Self {
            clients,
            current_client: Arc::new(Mutex::new(0)),
            pool_size,
            global_semaphore,
            config,
            stats: Arc::new(Mutex::new(initial_stats)),
        })
    }

    /// Get next client using round-robin selection
    async fn get_next_client(&self) -> Arc<T> {
        let mut current = self.current_client.lock().await;
        let client = self.clients[*current].clone();
        *current = (*current + 1) % self.pool_size;
        client
    }

    /// Set cookies on all clients in the pool
    pub async fn set_cookies(&self, cookies: HashMap<String, String>) -> Result<()> {
        let tasks = self
            .clients
            .iter()
            .map(|client| client.set_cookies(cookies.clone()));
        
        let results = futures::future::join_all(tasks).await;
        
        // Check if any failed
        for result in results {
            result?;
        }
        
        Ok(())
    }

    /// Set headers on all clients in the pool
    pub async fn set_headers(&self, headers: HashMap<String, String>) -> Result<()> {
        let tasks = self
            .clients
            .iter()
            .map(|client| client.set_headers(headers.clone()));
            
        let results = futures::future::join_all(tasks).await;
        
        // Check if any failed
        for result in results {
            result?;
        }
        
        Ok(())
    }

    /// Perform HTTP request with global rate limiting
    async fn make_pooled_request<F, Fut>(&self, request_fn: F) -> Result<Response>
    where
        F: FnOnce(Arc<T>) -> Fut,
        Fut: std::future::Future<Output = Result<Response>>,
    {
        let start_time = std::time::Instant::now();
        let client = self.get_next_client().await;

        let result = if let Some(global_sem) = &self.global_semaphore {
            let _global_permit = global_sem.acquire().await.map_err(|e| {
                CsCliError::ApiRequest(format!("Failed to acquire global semaphore: {}", e))
            })?;
            request_fn(client).await
        } else {
            request_fn(client).await
        };

        // Update statistics
        let elapsed_ms = start_time.elapsed().as_millis() as f64;
        let mut stats = self.stats.lock().await;
        match &result {
            Ok(_) => {
                stats.requests_completed += 1;
                // Simple running average
                let total_requests = stats.requests_completed + stats.requests_failed;
                stats.average_response_time_ms = 
                    (stats.average_response_time_ms * (total_requests - 1) as f64 + elapsed_ms) / total_requests as f64;
            }
            Err(_) => {
                stats.requests_failed += 1;
            }
        }

        result
    }

    /// Perform GET request using the pool
    pub async fn get(&self, url: &str) -> Result<Response> {
        self.make_pooled_request(|client| async move {
            client.get(url).await
        }).await
    }

    /// Perform POST request using the pool
    pub async fn post(&self, url: &str, body: Option<&str>) -> Result<Response> {
        self.make_pooled_request(|client| async move {
            client.post(url, body).await
        }).await
    }

    /// Perform PUT request using the pool
    pub async fn put(&self, url: &str, body: Option<&str>) -> Result<Response> {
        self.make_pooled_request(|client| async move {
            client.put(url, body).await
        }).await
    }

    /// Perform DELETE request using the pool
    pub async fn delete(&self, url: &str) -> Result<Response> {
        self.make_pooled_request(|client| async move {
            client.delete(url).await
        }).await
    }

    /// Get pool configuration
    pub fn get_config(&self) -> &HttpSettings {
        &self.config
    }

    /// Execute multiple requests in parallel across the pool
    pub async fn batch_requests(&self, requests: Vec<(String, String, Option<String>)>) -> Vec<Result<Response>> {
        if requests.is_empty() {
            return Vec::new();
        }

        let mut tasks = Vec::new();

        for (i, (method, url, body)) in requests.into_iter().enumerate() {
            let client_idx = i % self.pool_size;
            let client = self.clients[client_idx].clone();
            let global_sem = self.global_semaphore.clone();

            let task = async move {
                let result = if let Some(global_sem) = global_sem {
                    let _global_permit = global_sem.acquire().await.map_err(|e| {
                        CsCliError::ApiRequest(format!("Failed to acquire global semaphore: {}", e))
                    })?;
                    
                    match method.to_uppercase().as_str() {
                        "GET" => client.get(&url).await,
                        "POST" => client.post(&url, body.as_deref()).await,
                        "PUT" => client.put(&url, body.as_deref()).await,
                        "DELETE" => client.delete(&url).await,
                        _ => Err(CsCliError::ApiRequest(format!("Unsupported HTTP method: {}", method))),
                    }
                } else {
                    match method.to_uppercase().as_str() {
                        "GET" => client.get(&url).await,
                        "POST" => client.post(&url, body.as_deref()).await,
                        "PUT" => client.put(&url, body.as_deref()).await,
                        "DELETE" => client.delete(&url).await,
                        _ => Err(CsCliError::ApiRequest(format!("Unsupported HTTP method: {}", method))),
                    }
                };
                result
            };

            tasks.push(task);
        }

        futures::future::join_all(tasks).await
    }
}

#[async_trait]
impl<T: HttpClient + 'static> HttpClientPool<T> for GenericHttpClientPool<T> {
    async fn get_client(&self) -> Result<&T> {
        // Note: This is a bit tricky with Arc<T>. In practice, most users will use the
        // direct methods like get(), post(), etc. rather than accessing individual clients.
        // For now, we'll return a reference to the first client for API compatibility.
        Ok(&*self.clients[0])
    }

    fn get_pool_stats(&self) -> PoolStats {
        // This blocks briefly, but it's for monitoring so it should be acceptable
        match self.stats.try_lock() {
            Ok(stats) => stats.clone(),
            Err(_) => PoolStats::default(), // Return default if lock is contended
        }
    }

    async fn health_check(&self) -> bool {
        // Check if all clients are healthy
        let health_checks = self.clients.iter().map(|client| client.health_check());
        let results = futures::future::join_all(health_checks).await;
        
        let healthy_count = results.iter().filter(|&&healthy| healthy).count();
        let health_ratio = healthy_count as f64 / self.clients.len() as f64;
        
        // Consider the pool healthy if at least 50% of clients are healthy
        health_ratio >= 0.5
    }
}
