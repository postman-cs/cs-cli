//! Guided authentication module for automating Okta SSO login
//!
//! This module provides browser automation for guided authentication through Okta SSO.
//! It uses headless Chrome to automate the login process and extract session cookies.

use crate::common::auth::guided_auth_config::{GuidedAuthConfig, PlatformDomains};
use crate::common::auth::guided_cookie_storage::{store_cookies, get_stored_cookies, has_stored_cookies, are_cookies_expired, validate_cookies};
use anyhow::{anyhow, Context, Result};
use headless_chrome::{Browser, LaunchOptions, Tab};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Main guided authentication struct
pub struct GuidedAuth {
    browser: Option<Browser>,
    active_tab: Option<Arc<Tab>>,
    config: GuidedAuthConfig,
    platform_domains: PlatformDomains,
}

impl GuidedAuth {
    /// Create new guided authentication instance
    pub fn new() -> Self {
        Self {
            browser: None,
            active_tab: None,
            config: GuidedAuthConfig::default(),
            platform_domains: PlatformDomains::default(),
        }
    }

    /// Check if browser is initialized (for testing)
    pub fn has_browser(&self) -> bool {
        self.browser.is_some()
    }

    /// Check if Okta Verify app is running on macOS
    #[cfg(target_os = "macos")]
    pub async fn check_okta_verify(&self) -> Result<bool> {
        use std::process::Command;

        info!("Checking if Okta Verify is running...");

        let output = Command::new("pgrep")
            .arg("-f")
            .arg("Okta Verify")
            .output()
            .context("Failed to check for Okta Verify process")?;

        let is_running = output.status.success() && !output.stdout.is_empty();

        if is_running {
            info!("Okta Verify is running");
        } else {
            info!("Okta Verify is not running");
        }

        Ok(is_running)
    }

    #[cfg(not(target_os = "macos"))]
    pub async fn check_okta_verify(&self) -> Result<bool> {
        info!("Okta Verify check not implemented for this platform");
        Ok(false)
    }

    /// Force fresh authentication (ignoring stored cookies)
    pub async fn authenticate_fresh(&mut self) -> Result<HashMap<String, String>> {
        info!("Performing forced fresh authentication...");
        self.perform_fresh_authentication().await
    }

    /// Start browser and navigate to Okta login
    pub async fn authenticate(&mut self) -> Result<HashMap<String, String>> {
        info!("Starting guided authentication flow...");

        // Check if we have valid stored cookies first
        if let Ok(stored_cookies) = self.check_stored_cookies().await {
            info!("Using existing valid cookies from keychain");
            return Ok(stored_cookies);
        }

        // No valid cookies found, perform fresh authentication
        info!("No valid stored cookies found, performing fresh authentication...");
        self.perform_fresh_authentication().await
    }

    /// Perform fresh authentication flow
    async fn perform_fresh_authentication(&mut self) -> Result<HashMap<String, String>> {
        info!("Starting fresh authentication flow...");

        // Navigate to Okta OAuth authorization endpoint
        info!("Navigating to Okta login page...");
        let okta_url = self.config.okta_oauth_url.clone();
        self.navigate_to_okta(&okta_url).await?;

        // Check if we're already logged in (landed on dashboard)
        if self.check_already_authenticated().await? {
            info!("Already authenticated! Proceeding directly to platform collection...");
        } else {
            // Wait for authentication options to load
            info!("Waiting for authentication options to load...");
            tokio::time::sleep(std::time::Duration::from_secs(self.config.timing.auth_options_load_wait)).await;

            // Click on Okta Verify authentication option
            info!("Selecting Okta Verify authentication...");
            match self.click_okta_verify_option().await {
                Ok(()) => info!("Successfully selected Okta Verify"),
                Err(e) => {
                    warn!("Failed to click Okta Verify, trying alternative selector: {}", e);
                    // Try alternative selector
                    let fallback_selector = self.config.selectors.okta_verify_fallback.clone();
                    self.click_element(&fallback_selector).await?;
                }
            }

            // Wait for Okta Verify to process and detect authentication success
            info!("Waiting for Okta Verify authentication...");
            self.wait_for_authentication_success().await?;
        }


        // Navigate to platform apps and collect cookies
        let all_cookies = self.navigate_to_platforms().await?;

        // Store the fresh cookies for future use (with timeout protection)
        let cookies_to_store = all_cookies.clone();
        match tokio::task::spawn_blocking(move || store_cookies(&cookies_to_store)).await {
            Ok(Ok(())) => {
                info!("Successfully stored {} cookies in keychain for future use", all_cookies.len());
            }
            Ok(Err(e)) => {
                warn!("Failed to store cookies in keychain: {}", e);
                // Non-fatal - we still have the cookies for this session
            }
            Err(e) => {
                warn!("Cookie storage task failed: {}", e);
                // Non-fatal - we still have the cookies for this session
            }
        }

        Ok(all_cookies)
    }


    /// Check for valid stored cookies
    async fn check_stored_cookies(&self) -> Result<HashMap<String, String>> {
        info!("Checking for stored cookies in keychain...");
        
        // Check if we have stored cookies (with timeout protection)
        let has_cookies = tokio::task::spawn_blocking(|| has_stored_cookies())
            .await
            .unwrap_or(false);
            
        if !has_cookies {
            debug!("No stored cookies found in keychain");
            return Err(anyhow!("No stored cookies found"));
        }

        // Retrieve stored cookies (with timeout protection)
        let stored_cookies = tokio::task::spawn_blocking(|| get_stored_cookies())
            .await
            .map_err(|e| anyhow!("Task failed: {}", e))?
            .map_err(|e| anyhow!("Failed to retrieve stored cookies: {}", e))?;

        // Check if cookies are expired
        if are_cookies_expired(&stored_cookies) {
            info!("Stored cookies are expired, will perform fresh authentication");
            return Err(anyhow!("Stored cookies are expired"));
        }

        // Validate cookies by testing them
        match validate_cookies(&stored_cookies).await {
            Ok(true) => {
                info!("Stored cookies are valid and ready to use");
                Ok(stored_cookies)
            }
            Ok(false) => {
                info!("Stored cookies failed validation, will perform fresh authentication");
                Err(anyhow!("Stored cookies failed validation"))
            }
            Err(e) => {
                warn!("Cookie validation failed: {}, will perform fresh authentication", e);
                Err(anyhow!("Cookie validation failed: {}", e))
            }
        }
    }

    /// Check if we're already authenticated by detecting dashboard page
    async fn check_already_authenticated(&self) -> Result<bool> {
        info!("Checking if already authenticated...");
        
        // Check for dashboard-specific elements that indicate we're already logged in
        let current_url = self.get_current_url().await?;
        info!("Current URL after navigation: {}", current_url);
        
        // Method 1: Check for window._oktaEnduser object (most reliable)
        let dashboard_script_check = r#"
            try {
                return typeof window._oktaEnduser !== 'undefined' && 
                       window._oktaEnduser !== null;
            } catch(e) {
                return false;
            }
        "#;
        
        match self.execute_js(dashboard_script_check).await {
            Ok(result) => {
                if result.contains("true") {
                    info!("Dashboard detected via window._oktaEnduser object");
                    return Ok(true);
                }
            }
            Err(e) => {
                debug!("JavaScript check failed: {}", e);
            }
        }
        
        // Method 2: Check for specific meta tag
        let meta_check = self.element_exists(r#"meta[name="ui-service"][content="iris"]"#).await;
        if meta_check {
            info!("Dashboard detected via iris meta tag");
            return Ok(true);
        }
        
        // Method 3: Check URL patterns
        if current_url.contains("enduser/dashboard") || 
           current_url.contains("app/UserHome") ||
           (current_url.contains("postman.okta.com") && current_url.ends_with("/")) {
            info!("Dashboard detected via URL pattern: {}", current_url);
            return Ok(true);
        }
        
        // Method 4: Check for root div that mounts dashboard React app
        let root_div_check = self.element_exists("#root").await;
        if root_div_check {
            info!("Dashboard detected via #root element");
            return Ok(true);
        }
        
        info!("Not authenticated - need to proceed with login flow");
        Ok(false)
    }

    /// Click on Okta Verify authentication option
    async fn click_okta_verify_option(&self) -> Result<()> {
        // Try to click using configured primary selector
        let primary_selector = &self.config.selectors.okta_verify_primary;
        self.click_element(primary_selector).await
    }

    /// Wait for authentication success by monitoring URL changes
    async fn wait_for_authentication_success(&self) -> Result<()> {
        info!("Waiting for authentication to complete...");
        
        let max_attempts = self.config.timing.auth_success_timeout;
        let mut attempts = 0;
        
        while attempts < max_attempts {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            
            let current_url = self.get_current_url().await?;
            
            // Check if URL or page content indicates successful authentication
            if self.config.is_success_url(&current_url) || self.check_already_authenticated().await? {
                info!("Authentication successful, redirected to: {}", current_url);
                return Ok(());
            }
            
            attempts += 1;
        }
        
        let current_url = self.get_current_url().await?;
        warn!("Authentication timeout after {}s, current URL: {}", max_attempts, current_url);
        
        // Even if we timeout, continue anyway as the user might have authenticated
        Ok(())
    }

    /// Navigate to each platform app and collect cookies
    async fn navigate_to_platforms(&mut self) -> Result<HashMap<String, String>> {
        let mut all_cookies = HashMap::new();

        // Clone the platform URLs to avoid borrowing issues
        let platform_urls = self.config.platform_urls.clone();
        let navigation_delay = self.config.timing.platform_navigation_delay;

        for (platform_name, platform_url) in platform_urls {
            info!("Navigating to {} platform...", platform_name);
            
            match self.navigate_to_platform(&platform_name, &platform_url).await {
                Ok(platform_cookies) => {
                    info!("Successfully collected {} cookies from {}", platform_cookies.len(), platform_name);
                    all_cookies.extend(platform_cookies);
                }
                Err(e) => {
                    warn!("Failed to collect cookies from {}: {}", platform_name, e);
                    // Continue with other platforms even if one fails
                }
            }
            
            // Wait between platform navigations using configured delay
            tokio::time::sleep(std::time::Duration::from_secs(navigation_delay)).await;
        }

        info!("Total cookies collected from all platforms: {}", all_cookies.len());
        Ok(all_cookies)
    }

    /// Navigate to a specific platform and extract cookies
    async fn navigate_to_platform(&mut self, platform_name: &str, platform_url: &str) -> Result<HashMap<String, String>> {
        info!("Loading {} app from Okta: {}", platform_name, platform_url);
        
        // Navigate to the platform URL
        self.navigate_to_url(platform_url).await?;
        
        // Wait for SSO redirection to complete
        info!("Waiting for {} SSO login to complete...", platform_name);
        let sso_wait = self.config.timing.sso_redirect_wait;
        tokio::time::sleep(std::time::Duration::from_secs(sso_wait)).await;
        
        // Get the current URL to see where we ended up
        let current_url = self.get_current_url().await?;
        info!("{} redirected to: {}", platform_name, current_url);
        
        // Extract cookies from the platform domain
        let cookies = self.extract_cookies().await?;
        
        // Filter cookies relevant to this platform
        let platform_cookies = self.filter_platform_cookies(&cookies, platform_name);
        
        Ok(platform_cookies)
    }

    /// Filter cookies relevant to the specific platform
    fn filter_platform_cookies(&self, cookies: &HashMap<String, String>, platform_name: &str) -> HashMap<String, String> {
        let _platform_domains = self.platform_domains.get_domains(platform_name);
        
        let mut platform_cookies = HashMap::new();
        
        // For now, include all cookies as we may not know exact domain matching
        // This will be refined as we test with actual platform redirections
        for (name, value) in cookies {
            // Add platform prefix to cookie names to avoid conflicts
            let prefixed_name = format!("{}_{}", platform_name, name);
            platform_cookies.insert(prefixed_name, value.clone());
        }
        
        platform_cookies
    }

    /// Launch Chrome browser with appropriate options
    fn launch_browser(&self) -> Result<Browser> {
        info!("Launching Chrome browser...");

        // Try to find Chrome in common locations
        let chrome_paths = self.find_chrome_paths();

        // Configure launch options
        let mut launch_options = LaunchOptions::default_builder();

        // Try each Chrome path until one works
        for path in &chrome_paths {
            if std::path::Path::new(path).exists() {
                info!("Found Chrome at: {}", path);
                launch_options.path(Some(std::path::PathBuf::from(path)));
                break;
            }
        }

        // Create persistent user data directory for session reuse
        let user_data_dir = dirs::data_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("cs-cli")
            .join("chrome-profile");

        // Ensure the directory exists
        std::fs::create_dir_all(&user_data_dir)
            .context("Failed to create Chrome profile directory")?;

        info!("Using Chrome profile at: {}", user_data_dir.display());

        // Set browser options
        let user_data_arg = format!("--user-data-dir={}", user_data_dir.display());
        launch_options
            .headless(false)  // Run in visible mode for debugging authentication issues
            .window_size(Some((1280, 1024)))
            .args(vec![
                std::ffi::OsStr::new("--disable-blink-features=AutomationControlled"),
                std::ffi::OsStr::new("--user-agent=Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36"),
                std::ffi::OsStr::new(&user_data_arg),
            ]);

        // Launch browser
        let browser =
            Browser::new(launch_options.build()?).context("Failed to launch Chrome browser")?;

        info!("Chrome browser launched successfully");
        Ok(browser)
    }

    /// Find Chrome executable paths based on OS
    fn find_chrome_paths(&self) -> Vec<String> {
        #[cfg(target_os = "macos")]
        {
            vec![
                "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome".to_string(),
                "/Applications/Chromium.app/Contents/MacOS/Chromium".to_string(),
                "/Applications/Google Chrome Canary.app/Contents/MacOS/Google Chrome Canary"
                    .to_string(),
                "/Applications/Brave Browser.app/Contents/MacOS/Brave Browser".to_string(),
                "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge".to_string(),
            ]
        }

        #[cfg(target_os = "linux")]
        {
            vec![
                "/usr/bin/google-chrome".to_string(),
                "/usr/bin/chromium".to_string(),
                "/usr/bin/chromium-browser".to_string(),
                "/usr/bin/brave-browser".to_string(),
                "/usr/bin/microsoft-edge".to_string(),
                "/snap/bin/chromium".to_string(),
            ]
        }

        #[cfg(target_os = "windows")]
        {
            vec![
                r"C:\Program Files\Google\Chrome\Application\chrome.exe".to_string(),
                r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe".to_string(),
                r"C:\Program Files\Chromium\Application\chrome.exe".to_string(),
                r"C:\Program Files\Microsoft\Edge\Application\msedge.exe".to_string(),
            ]
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            vec![]
        }
    }

    /// Navigate to any URL
    pub async fn navigate_to_url(&mut self, url: &str) -> Result<()> {
        // Launch browser if not already initialized
        if self.browser.is_none() {
            let browser = self.launch_browser()?;
            self.browser = Some(browser);
        }

        let browser = self
            .browser
            .as_ref()
            .ok_or_else(|| anyhow!("Browser not initialized"))?;

        // Use existing tab or create new one
        let tab = if let Some(ref tab) = self.active_tab {
            tab.clone()
        } else {
            // Get the first tab (the default blank one) or create a new one
            let tabs = browser.get_tabs().lock().unwrap();
            let tab = if let Some(first_tab) = tabs.first() {
                first_tab.clone()
            } else {
                drop(tabs); // Release lock before creating new tab
                browser
                    .new_tab()
                    .context("Failed to create new browser tab")?
            };
            self.active_tab = Some(tab.clone());
            tab
        };

        info!("Navigating to: {}", url);
        tab.navigate_to(url).context("Failed to navigate to URL")?;

        tab.wait_until_navigated()
            .context("Failed to wait for navigation")?;

        Ok(())
    }

    /// Navigate to Okta and perform authentication  
    pub async fn navigate_to_okta(&mut self, url: &str) -> Result<()> {
        self.navigate_to_url(url).await
    }

    /// Extract cookies from authenticated session
    pub async fn extract_cookies(&self) -> Result<HashMap<String, String>> {
        let tab = self
            .active_tab
            .as_ref()
            .ok_or_else(|| anyhow!("No active tab"))?;

        // Get cookies from the page
        let cookies = tab.get_cookies().context("Failed to get cookies")?;

        let mut cookie_map = HashMap::new();

        // Convert cookies to map
        for cookie in cookies {
            debug!("Cookie: {} = {}", cookie.name, cookie.value);
            cookie_map.insert(cookie.name, cookie.value);
        }

        info!("Extracted {} cookies", cookie_map.len());
        Ok(cookie_map)
    }

    /// Click on element by selector
    pub async fn click_element(&self, selector: &str) -> Result<()> {
        let tab = self
            .active_tab
            .as_ref()
            .ok_or_else(|| anyhow!("No active tab"))?;

        info!("Clicking element: {}", selector);

        // Find and click the element
        let element = tab
            .find_element(selector)
            .context("Failed to find element")?;

        element.click().context("Failed to click element")?;

        Ok(())
    }

    /// Wait for element to appear
    pub async fn wait_for_element(&self, selector: &str, timeout_secs: u64) -> Result<()> {
        let tab = self
            .active_tab
            .as_ref()
            .ok_or_else(|| anyhow!("No active tab"))?;

        info!(
            "Waiting for element: {} (timeout: {}s)",
            selector, timeout_secs
        );

        // Use built-in wait functionality
        tab.wait_for_element(selector)
            .context("Element not found within timeout")?;

        info!("Element found: {}", selector);
        Ok(())
    }

    /// Check if element exists
    pub async fn element_exists(&self, selector: &str) -> bool {
        let tab = match self.active_tab.as_ref() {
            Some(t) => t,
            None => return false,
        };

        tab.find_element(selector).is_ok()
    }

    /// Get current URL
    pub async fn get_current_url(&self) -> Result<String> {
        let tab = self
            .active_tab
            .as_ref()
            .ok_or_else(|| anyhow!("No active tab"))?;

        let url = tab.get_url();
        Ok(url)
    }

    /// Execute JavaScript in the page
    pub async fn execute_js(&self, script: &str) -> Result<String> {
        let tab = self
            .active_tab
            .as_ref()
            .ok_or_else(|| anyhow!("No active tab"))?;

        let result = tab
            .evaluate(script, false)
            .context("Failed to execute JavaScript")?;

        Ok(format!("{:?}", result.value))
    }

    /// Close browser
    pub fn close(&mut self) -> Result<()> {
        self.active_tab = None;
        if let Some(browser) = self.browser.take() {
            // Browser will be closed when dropped
            drop(browser);
            info!("Browser closed");
        }
        Ok(())
    }
}

impl Default for GuidedAuth {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for GuidedAuth {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_guided_auth_creation() {
        let auth = GuidedAuth::new();
        assert!(!auth.has_browser());
    }

    #[test]
    fn test_chrome_paths() {
        let auth = GuidedAuth::new();
        let paths = auth.find_chrome_paths();
        assert!(!paths.is_empty());
    }
}
