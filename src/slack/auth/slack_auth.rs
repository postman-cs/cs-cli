//! Slack workspace authentication using common abstractions
//!
//! Demonstrates how the common auth framework works with Slack workspaces

use crate::common::auth::{Cookie, CookieRetriever, SessionManager};
use crate::{CsCliError, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Slack workspace session data
#[derive(Debug, Clone)]
pub struct SlackSession {
    pub workspace_url: String,
    pub workspace_id: String,
    pub team_name: String,
    pub user_id: String,
    pub xoxc_token: String,
    pub cookies: HashMap<String, String>,
}

/// Slack authentication manager using common abstractions
pub struct SlackAuth {
    workspace_domain: String,
    cookie_retriever: CookieRetriever,
    current_session: Option<SlackSession>,
    pub detected_browser: Option<String>,
}

impl SlackAuth {
    /// Create new Slack authenticator for a workspace domain
    pub fn new(workspace_domain: String) -> Self {
        let domains = vec![
            workspace_domain.clone(),
            "app.slack.com".to_string(),
            "slack.com".to_string(),
        ];
        let cookie_retriever = CookieRetriever::new(domains);

        Self {
            workspace_domain,
            cookie_retriever,
            current_session: None,
            detected_browser: None,
        }
    }

    /// Retrieve Slack authentication from browser using auth API
    pub async fn retrieve_browser_auth(&mut self) -> Result<SlackSession> {
        info!(
            "Retrieving Slack authentication for {}",
            self.workspace_domain
        );

        // Test browsers sequentially to find one with valid authentication
        let (all_cookies, browser_source) = self.find_browser_with_valid_auth().await?;

        info!(
            "Found {} working cookies from {}",
            all_cookies.len(),
            browser_source
        );

        // Store the browser source for consistent fingerprinting
        self.detected_browser = Some(browser_source.to_lowercase());

        // Filter for Slack-related cookies
        let slack_cookies: Vec<Cookie> = all_cookies
            .into_iter()
            .filter(|cookie| {
                cookie.domain.contains("slack") || cookie.domain.contains(&self.workspace_domain)
            })
            .collect();

        debug!("Found {} Slack-related cookies", slack_cookies.len());

        // Check for required 'd' session cookie
        let _d_cookie = slack_cookies
            .iter()
            .find(|c| c.name == "d")
            .ok_or_else(|| {
                CsCliError::Authentication(
                    "Required 'd' session cookie not found in your browser. Please log into Slack in your browser first."
                        .to_string(),
                )
            })?;

        debug!("Found required 'd' session cookie");

        // Convert cookies to HashMap for HTTP requests
        let cookie_map: HashMap<String, String> = slack_cookies
            .iter()
            .map(|c| (c.name.clone(), c.value.clone()))
            .collect();

        // Get xoxc token via auth API endpoint
        let xoxc_token = self.fetch_xoxc_token(&cookie_map).await?;

        let session = SlackSession {
            workspace_url: format!("https://{}", self.workspace_domain),
            workspace_id: "unknown".to_string(), // We'll get this from auth.test
            team_name: "unknown".to_string(),    // We'll get this from auth.test
            user_id: "unknown".to_string(),      // We'll get this from auth.test
            xoxc_token,
            cookies: cookie_map,
        };

        info!(
            "Successfully retrieved Slack session for workspace: {}",
            session.workspace_url
        );
        self.current_session = Some(session.clone());

        Ok(session)
    }

    /// Fetch xoxc token from Slack auth API endpoint using cookies
    async fn fetch_xoxc_token(&self, cookies: &HashMap<String, String>) -> Result<String> {
        use crate::common::config::HttpSettings;
        use crate::common::http::{BrowserHttpClient, HttpClient};

        info!("Fetching xoxc token from Slack auth API...");

        // Create HTTP client using common platform configuration
        let browser_type = self.detected_browser.as_deref();
        let client = crate::common::http::PlatformConfigs::slack(browser_type)
            .with_concurrency(1)
            .build_client()
            .await?;

        // Set cookies for the request
        client.set_cookies(cookies.clone()).await?;

        // Make request to auth endpoint (simplified version of your curl)
        let auth_url = "https://app.slack.com/auth?app=client&teams=&iframe=1";

        debug!("Making auth request to: {}", auth_url);
        let response = client.get(auth_url).await?;

        let response_text = response
            .text()
            .await
            .map_err(|e| CsCliError::ApiRequest(format!("Failed to read auth response: {}", e)))?;

        debug!("Auth response length: {} chars", response_text.len());

        // Parse JavaScript response to extract xoxc token
        self.parse_xoxc_from_response(&response_text)
    }

    /// Parse xoxc token from Slack auth response JavaScript
    fn parse_xoxc_from_response(&self, response: &str) -> Result<String> {
        use regex::Regex;

        // Look for the sessionStorage.incomingConfig JSON
        let config_regex =
            Regex::new(r#"sessionStorage\.incomingConfig\s*=\s*JSON\.stringify\((\{.*?\})\);"#)
                .map_err(|e| CsCliError::Generic(format!("Failed to compile regex: {}", e)))?;

        let config_json = config_regex
            .captures(response)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str())
            .ok_or_else(|| {
                CsCliError::Authentication(
                    "Could not find sessionStorage.incomingConfig in auth response".to_string(),
                )
            })?;

        debug!(
            "Found config JSON: {}...",
            &config_json[..std::cmp::min(100, config_json.len())]
        );

        // Parse JSON to extract token
        let config: serde_json::Value = serde_json::from_str(config_json).map_err(|e| {
            CsCliError::Authentication(format!("Failed to parse config JSON: {}", e))
        })?;

        // Look for token in teams structure
        if let Some(teams) = config.get("teams").and_then(|t| t.as_object()) {
            for (team_id, team_data) in teams {
                if let Some(token) = team_data.get("token").and_then(|t| t.as_str()) {
                    if token.starts_with("xoxc-") {
                        info!("Found xoxc token for team: {}", team_id);
                        return Ok(token.to_string());
                    }
                }
                // Also check enterprise_api_token
                if let Some(token) = team_data
                    .get("enterprise_api_token")
                    .and_then(|t| t.as_str())
                {
                    if token.starts_with("xoxc-") {
                        info!("Found enterprise xoxc token for team: {}", team_id);
                        return Ok(token.to_string());
                    }
                }
            }
        }

        Err(CsCliError::Authentication(
            "No xoxc token found in auth response teams data".to_string(),
        ))
    }

    /// Test if cookies can generate a valid xoxc token (for browser validation)
    async fn test_cookies_validity(&self, cookies: Vec<Cookie>) -> Result<bool> {
        use crate::common::config::HttpSettings;
        use crate::common::http::{BrowserHttpClient, HttpClient};

        // Check for required 'd' session cookie first
        let has_d_cookie = cookies.iter().any(|c| c.name == "d");
        if !has_d_cookie {
            return Ok(false);
        }

        // Convert cookies to HashMap
        let cookie_map: HashMap<String, String> = cookies
            .iter()
            .map(|c| (c.name.clone(), c.value.clone()))
            .collect();

        // Try to get xoxc token with these cookies
        let http_config = HttpSettings {
            pool_size: 1,
            max_concurrency_per_client: 1,
            timeout_seconds: 10.0, // Shorter timeout for validation
            max_clients: Some(1),
            global_max_concurrency: Some(1),
            enable_http3: true,
            force_http3: false,
            tls_version: None,
            impersonate_browser: "firefox".to_string(), // Default for testing
        };

        let client = BrowserHttpClient::new(http_config).await?;
        client.set_cookies(cookie_map).await?;

        // Test auth endpoint
        let auth_url = "https://app.slack.com/auth?app=client&teams=&iframe=1";

        match client.get(auth_url).await {
            Ok(response) => {
                match response.text().await {
                    Ok(response_text) => {
                        // Check if response contains valid token data
                        Ok(response_text.contains("sessionStorage.incomingConfig")
                            && response_text.contains("xoxc-"))
                    }
                    Err(_) => Ok(false),
                }
            }
            Err(_) => Ok(false),
        }
    }

    /// Get current session
    pub fn get_session(&self) -> Option<&SlackSession> {
        self.current_session.as_ref()
    }

    /// Find browser with valid Slack authentication by testing each one
    async fn find_browser_with_valid_auth(&self) -> Result<(Vec<Cookie>, String)> {
        info!("Testing browsers for valid Slack authentication...");

        // Note: Keychain should be unlocked at CLI startup
        // No need to unlock here - avoids multiple password prompts

        // Test Firefox
        if let Ok(cookies) = self.cookie_retriever.retrieve_firefox_cookies() {
            if self.has_valid_slack_session(&cookies, "Firefox").await? {
                info!("Firefox has valid Slack authentication");
                return Ok((cookies, "Firefox".to_string()));
            }
        }

        // Test Chrome
        if let Ok(cookies) = self.cookie_retriever.retrieve_chrome_cookies() {
            if self.has_valid_slack_session(&cookies, "Chrome").await? {
                info!("Chrome has valid Slack authentication");
                return Ok((cookies, "Chrome".to_string()));
            }
        }

        // Test Brave
        if let Ok(cookies) = self.cookie_retriever.retrieve_brave_cookies() {
            if self.has_valid_slack_session(&cookies, "Brave").await? {
                info!("Brave has valid Slack authentication");
                return Ok((cookies, "Brave".to_string()));
            }
        }

        // Test Edge
        if let Ok(cookies) = self.cookie_retriever.retrieve_edge_cookies() {
            if self.has_valid_slack_session(&cookies, "Edge").await? {
                info!("Edge has valid Slack authentication");
                return Ok((cookies, "Edge".to_string()));
            }
        }

        // Test Arc
        if let Ok(cookies) = self.cookie_retriever.retrieve_arc_cookies() {
            if self.has_valid_slack_session(&cookies, "Arc").await? {
                info!("Arc has valid Slack authentication");
                return Ok((cookies, "Arc".to_string()));
            }
        }

        Err(CsCliError::Authentication(
            "No browser has valid Slack authentication. Please log into Slack in a supported browser.".to_string()
        ))
    }

    /// Check if cookies from a specific browser have valid Slack session
    async fn has_valid_slack_session(
        &self,
        cookies: &[Cookie],
        browser_name: &str,
    ) -> Result<bool> {
        debug!("Testing {} cookies...", browser_name);

        if cookies.is_empty() {
            debug!("No cookies found in {}", browser_name);
            return Ok(false);
        }

        // Check for Slack cookies
        let slack_cookies: Vec<_> = cookies
            .iter()
            .filter(|c| c.domain.contains("slack"))
            .collect();

        if slack_cookies.is_empty() {
            debug!("No Slack cookies in {}", browser_name);
            return Ok(false);
        }

        // Check for 'd' session cookie
        if !slack_cookies.iter().any(|c| c.name == "d") {
            debug!("No 'd' session cookie in {}", browser_name);
            return Ok(false);
        }

        info!(
            "Testing {} cookies for valid authentication...",
            browser_name
        );

        // Test if cookies can generate valid xoxc token
        match self.test_cookies_validity(cookies.to_vec()).await {
            Ok(is_valid) => {
                if !is_valid {
                    warn!("{} cookies are invalid/expired", browser_name);
                }
                Ok(is_valid)
            }
            Err(e) => {
                warn!("{} authentication test failed: {}", browser_name, e);
                Ok(false)
            }
        }
    }

    /// Get cookies for debugging
    pub fn get_domain_cookies(&self, domain: &str) -> Result<Vec<Cookie>> {
        // Get ALL Firefox cookies and filter for Slack-related ones
        let all_cookies = self.cookie_retriever.retrieve_firefox_cookies()?;

        let slack_cookies: Vec<Cookie> = all_cookies
            .into_iter()
            .filter(|cookie| {
                cookie.domain.contains("slack")
                    || cookie.domain.contains(domain)
                    || domain.contains(&cookie.domain)
            })
            .collect();

        Ok(slack_cookies)
    }
}

#[async_trait]
impl SessionManager for SlackAuth {
    type SessionData = SlackSession;

    async fn initialize_session(&mut self) -> Result<()> {
        self.retrieve_browser_auth().await?;
        Ok(())
    }

    async fn authenticate(&mut self) -> Result<bool> {
        match self.retrieve_browser_auth().await {
            Ok(_) => Ok(true),
            Err(e) => {
                warn!("Slack authentication failed: {}", e);
                Ok(false)
            }
        }
    }

    async fn is_authenticated(&self) -> bool {
        self.current_session.is_some()
    }

    async fn refresh_session(&mut self) -> Result<bool> {
        self.authenticate().await
    }

    fn get_session_data(&self) -> Option<&Self::SessionData> {
        self.current_session.as_ref()
    }

    fn get_auth_headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();

        if let Some(session) = &self.current_session {
            // Add authorization header with xoxc token
            headers.insert(
                "Authorization".to_string(),
                format!("Bearer {}", session.xoxc_token),
            );

            // Add common Slack headers
            headers.insert("User-Agent".to_string(), 
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:122.0) Gecko/20100101 Firefox/122.0".to_string());
            headers.insert("Accept".to_string(), "application/json".to_string());
            headers.insert(
                "Content-Type".to_string(),
                "application/x-www-form-urlencoded".to_string(),
            );
        }

        headers
    }
}
