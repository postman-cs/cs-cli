//! Generic browser-based authentication patterns
//!
//! Common patterns for extracting authentication data from browsers
//! that can be shared between Gong and Slack integrations.

use super::cookie_extractor::{Cookie, CookieExtractor};
use crate::{CsCliError, Result};
use async_trait::async_trait;
use std::collections::HashMap;

/// Browser authentication trait for services that use browser sessions
#[async_trait]
pub trait BrowserAuth {
    type AuthData;

    /// Extract authentication data from browser
    async fn extract_browser_auth(&mut self, domain: &str) -> Result<Self::AuthData>;

    /// Validate extracted authentication data
    fn validate_auth_data(&self, auth_data: &Self::AuthData) -> bool;

    /// Convert auth data to HTTP headers
    fn auth_data_to_headers(&self, auth_data: &Self::AuthData) -> HashMap<String, String>;
}

/// Generic browser session extractor
pub struct BrowserSessionExtractor {
    pub cookie_extractor: CookieExtractor,
    pub supported_domains: Vec<String>,
}

impl BrowserSessionExtractor {
    /// Create new browser session extractor
    pub fn new(domains: Vec<String>) -> Self {
        Self {
            cookie_extractor: CookieExtractor::new(domains.clone()),
            supported_domains: domains,
        }
    }

    /// Extract cookies for a specific domain
    pub fn extract_domain_cookies(&self, domain: &str) -> Result<Vec<Cookie>> {
        if !self.supported_domains.contains(&domain.to_string()) {
            return Err(CsCliError::Authentication(format!(
                "Domain '{domain}' not supported for cookie extraction"
            )));
        }

        // Try Firefox first (most reliable for our use cases)
        if let Ok(cookies) = self.cookie_extractor.extract_firefox_cookies() {
            let domain_cookies: Vec<Cookie> = cookies
                .into_iter()
                .filter(|cookie| cookie.domain.contains(domain))
                .collect();

            if !domain_cookies.is_empty() {
                return Ok(domain_cookies);
            }
        }

        // Fallback to Chrome
        if let Ok(cookies) = self.cookie_extractor.extract_chrome_cookies() {
            let domain_cookies: Vec<Cookie> = cookies
                .into_iter()
                .filter(|cookie| cookie.domain.contains(domain))
                .collect();

            return Ok(domain_cookies);
        }

        Err(CsCliError::CookieExtraction(format!(
            "No cookies found for domain: {domain}"
        )))
    }

    /// Extract specific cookie by name
    pub fn extract_cookie_by_name(&self, domain: &str, cookie_name: &str) -> Result<Cookie> {
        let cookies = self.extract_domain_cookies(domain)?;

        cookies
            .into_iter()
            .find(|cookie| cookie.name == cookie_name)
            .ok_or_else(|| {
                CsCliError::CookieExtraction(format!(
                    "Cookie '{cookie_name}' not found for domain '{domain}'"
                ))
            })
    }
}
