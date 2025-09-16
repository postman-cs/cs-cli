//! Common HTTP client traits and interfaces
//!
//! Generic HTTP client abstractions that can be implemented by both
//! Gong and Slack API clients.

use crate::Result;
use async_trait::async_trait;
use reqwest::Response;
use std::collections::HashMap;

/// Generic HTTP client trait for API interactions
#[async_trait]
pub trait HttpClient: Send + Sync {
    /// Make an HTTP GET request
    async fn get(&self, url: &str) -> Result<Response>;
    
    /// Make an HTTP POST request with JSON body
    async fn post(&self, url: &str, body: Option<&str>) -> Result<Response>;
    
    /// Make an HTTP PUT request with JSON body
    async fn put(&self, url: &str, body: Option<&str>) -> Result<Response>;
    
    /// Make an HTTP DELETE request
    async fn delete(&self, url: &str) -> Result<Response>;
    
    /// Set cookies for all subsequent requests
    async fn set_cookies(&self, cookies: HashMap<String, String>) -> Result<()>;
    
    /// Set headers for all subsequent requests
    async fn set_headers(&self, headers: HashMap<String, String>) -> Result<()>;
    
    /// Get current request headers
    async fn get_headers(&self) -> HashMap<String, String>;
    
    /// Check if the client is healthy/available
    async fn health_check(&self) -> bool;
}

/// HTTP client pool trait for managing multiple clients
#[async_trait]
pub trait HttpClientPool<T: HttpClient>: Send + Sync {
    /// Get the next available client from the pool
    async fn get_client(&self) -> Result<&T>;
    
    /// Get pool statistics
    fn get_pool_stats(&self) -> PoolStats;
    
    /// Check pool health
    async fn health_check(&self) -> bool;
}

/// Statistics for HTTP client pool
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub total_clients: usize,
    pub active_clients: usize,
    pub requests_completed: u64,
    pub requests_failed: u64,
    pub average_response_time_ms: f64,
}

impl Default for PoolStats {
    fn default() -> Self {
        Self {
            total_clients: 0,
            active_clients: 0,
            requests_completed: 0,
            requests_failed: 0,
            average_response_time_ms: 0.0,
        }
    }
}

/// Request options for HTTP operations
#[derive(Debug, Clone, Default)]
pub struct RequestOptions {
    pub timeout_seconds: Option<f64>,
    pub max_retries: Option<u32>,
    pub user_agent: Option<String>,
    pub custom_headers: HashMap<String, String>,
}

/// HTTP request methods
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
}

impl HttpMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST", 
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Head => "HEAD",
            HttpMethod::Options => "OPTIONS",
        }
    }
}
