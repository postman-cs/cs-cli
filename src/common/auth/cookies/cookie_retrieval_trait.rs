//! Common trait for cookie retrieval across different browsers and storage methods
//!
//! This trait eliminates DRY violations by providing a unified interface
//! for cookie retrieval from various sources (browser databases, hybrid storage, etc.)

use crate::Result;
use std::collections::HashMap;

/// Generic cookie structure used across all retrieval methods
#[derive(Debug, Clone)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub secure: bool,
    pub http_only: bool,
    pub expires: Option<u64>,
}

/// Common trait for retrieving cookies from various sources
#[async_trait::async_trait]
pub trait CookieRetrieval {
    /// Retrieve cookies from the source
    async fn retrieve_cookies(&self) -> Result<Vec<Cookie>>;
    
    /// Filter cookies by target domains
    fn filter_by_domains(&self, cookies: Vec<Cookie>, target_domains: &[String]) -> Vec<Cookie> {
        cookies
            .into_iter()
            .filter(|cookie| {
                // Check if cookie domain matches any of our target domains
                target_domains.iter().any(|domain| {
                    cookie.domain.ends_with(domain)
                        || cookie.domain == format!(".{domain}")
                        || cookie.domain == domain.trim_start_matches('.')
                })
            })
            .collect()
    }
    
    /// Convert cookies to HTTP Cookie header format
    fn cookies_to_header(&self, cookies: &[Cookie]) -> String {
        cookies
            .iter()
            .map(|c| format!("{}={}", c.name, c.value))
            .collect::<Vec<_>>()
            .join("; ")
    }
    
    /// Convert cookies to HTTP headers HashMap
    fn cookies_to_headers(&self, cookies: &[Cookie]) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        let cookie_header = self.cookies_to_header(cookies);
        
        if !cookie_header.is_empty() {
            headers.insert("Cookie".to_string(), cookie_header);
        }
        
        headers
    }
}

/// Common cookie retrieval result with metadata
#[derive(Debug)]
pub struct CookieRetrievalResult {
    pub cookies: Vec<Cookie>,
    pub source: String,
    pub retrieved_at: f64,
}

impl CookieRetrievalResult {
    pub fn new(cookies: Vec<Cookie>, source: String) -> Self {
        Self {
            retrieved_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
            cookies,
            source,
        }
    }
    
    /// Filter cookies by domains and return new result
    pub fn filter_by_domains(mut self, target_domains: &[String]) -> Self {
        self.cookies = self.cookies
            .into_iter()
            .filter(|cookie| {
                target_domains.iter().any(|domain| {
                    cookie.domain.ends_with(domain)
                        || cookie.domain == format!(".{domain}")
                        || cookie.domain == domain.trim_start_matches('.')
                })
            })
            .collect();
        self
    }
    
    /// Check if result contains valid cookies
    pub fn is_valid(&self) -> bool {
        !self.cookies.is_empty()
    }
    
    /// Get count of cookies
    pub fn count(&self) -> usize {
        self.cookies.len()
    }
}