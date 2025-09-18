//! Guided Authentication Configuration
//!
//! Centralized configuration for Okta SSO URLs and platform endpoints.
//! This allows easy modification of authentication parameters without changing core logic.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for guided authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuidedAuthConfig {
    /// Okta OAuth authorization endpoint with all required parameters
    pub okta_oauth_url: String,
    
    /// Platform application URLs for SSO login
    pub platform_urls: HashMap<String, String>,
    
    /// Authentication selectors for UI automation
    pub selectors: AuthSelectors,
    
    /// Timing configurations for waits and timeouts
    pub timing: TimingConfig,
}

/// CSS selectors for authentication UI elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSelectors {
    /// Primary Okta Verify selector
    pub okta_verify_primary: String,
    
    /// Fallback authentication selector
    pub okta_verify_fallback: String,
    
    /// Selectors to detect successful authentication
    pub success_indicators: Vec<String>,
}

/// Timing configuration for authentication flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingConfig {
    /// Wait time for auth options to load (seconds)
    pub auth_options_load_wait: u64,
    
    /// Maximum wait time for authentication success (seconds)
    pub auth_success_timeout: u64,
    
    /// Wait time for SSO redirection (seconds)
    pub sso_redirect_wait: u64,
    
    /// Wait time between platform navigations (seconds)
    pub platform_navigation_delay: u64,
}

impl Default for GuidedAuthConfig {
    fn default() -> Self {
        let mut platform_urls = HashMap::new();
        
        // Postman Okta platform URLs
        platform_urls.insert(
            "gong".to_string(),
            "https://postman.okta.com/home/gong/0oa2498tgvv07sY4u5d7/aln18z05fityF2rra1d8".to_string()
        );
        platform_urls.insert(
            "slack".to_string(),
            "https://postman.okta.com/home/slack/0oah7rbz24ePsdEUb5d7/19411".to_string()
        );
        platform_urls.insert(
            "gainsight".to_string(),
            "https://postman.okta.com/home/postman_gainsight_1/0oamoyvq23HxTiYXn5d7/alnmoz76mc9cunIgV5d7".to_string()
        );
        platform_urls.insert(
            "salesforce".to_string(),
            "https://postman.okta.com/home/salesforce/0oa278r39cGoxK3TW5d7/46".to_string()
        );

        Self {
            okta_oauth_url: Self::build_okta_oauth_url(),
            platform_urls,
            selectors: AuthSelectors {
                okta_verify_primary: r#"a[aria-label="Select Okta Verify."]"#.to_string(),
                okta_verify_fallback: "a.select-factor:first-of-type".to_string(),
                success_indicators: vec![
                    "/enduser/callback".to_string(),
                    "/app/UserHome".to_string(), 
                    "/enduser/dashboard".to_string(),
                    "window._oktaEnduser".to_string(), // Dashboard-specific JS object
                    "ui-service".to_string(), // Meta tag indicating dashboard
                    "sid=".to_string(),
                ],
            },
            timing: TimingConfig {
                auth_options_load_wait: 3,
                auth_success_timeout: 30,
                sso_redirect_wait: 5,
                platform_navigation_delay: 2,
            },
        }
    }
}

impl GuidedAuthConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Load configuration (just returns default for Postman)
    pub fn from_env() -> Self {
        Self::default()
    }

    /// Build the Okta URL for Postman - start at main page and follow redirects naturally
    fn build_okta_oauth_url() -> String {
        // Simply start at postman.okta.com and let Okta handle the natural redirect flow
        // This avoids OAuth parameter complexity and expired tokens
        "https://postman.okta.com".to_string()
    }

    /// Get platform URL by name
    pub fn get_platform_url(&self, platform_name: &str) -> Option<&String> {
        self.platform_urls.get(platform_name)
    }


    /// Check if a URL indicates successful authentication
    pub fn is_success_url(&self, url: &str) -> bool {
        self.selectors.success_indicators.iter().any(|indicator| url.contains(indicator))
    }
}

/// Platform-specific cookie filtering configuration
#[derive(Debug, Clone)]
pub struct PlatformDomains {
    domains: HashMap<String, Vec<String>>,
}

impl Default for PlatformDomains {
    fn default() -> Self {
        let mut domains = HashMap::new();
        
        domains.insert(
            "gong".to_string(),
            vec!["gong.io".to_string(), "postman-success.gong.io".to_string()]
        );
        domains.insert(
            "slack".to_string(),
            vec!["slack.com".to_string(), "postman.slack.com".to_string()]
        );
        domains.insert(
            "gainsight".to_string(),
            vec!["gainsightcloud.com".to_string(), "postman.gainsightcloud.com".to_string()]
        );
        domains.insert(
            "salesforce".to_string(),
            vec![
                "salesforce.com".to_string(),
                "postman.my.salesforce.com".to_string(),
                "postman.lightning.force.com".to_string()
            ]
        );
        
        Self { domains }
    }
}

impl PlatformDomains {
    /// Get domains for a platform (for future cookie domain filtering)
    pub fn get_domains(&self, platform_name: &str) -> Vec<&String> {
        self.domains.get(platform_name).map(|v| v.iter().collect()).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = GuidedAuthConfig::default();
        
        assert!(config.okta_oauth_url.contains("postman.okta.com"));
        assert!(config.platform_urls.contains_key("gong"));
        assert!(config.platform_urls.contains_key("slack"));
        assert!(config.platform_urls.contains_key("gainsight"));
        assert!(config.platform_urls.contains_key("salesforce"));
    }

    #[test]
    fn test_success_url_detection() {
        let config = GuidedAuthConfig::default();
        
        assert!(config.is_success_url("https://postman.okta.com/enduser/callback?code=123"));
        assert!(config.is_success_url("https://postman.okta.com/app/UserHome"));
        assert!(!config.is_success_url("https://postman.okta.com/signin"));
    }

    #[test]
    fn test_platform_domains() {
        let domains = PlatformDomains::default();
        
        let gong_domains = domains.get_domains("gong");
        assert!(!gong_domains.is_empty());
        assert!(gong_domains.iter().any(|d| d.contains("gong.io")));
    }
}