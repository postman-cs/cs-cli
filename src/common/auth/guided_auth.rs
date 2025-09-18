//! Guided authentication module for automating Okta SSO login
//!
//! This module provides browser automation for guided authentication through Okta SSO.
//! It uses headless Chrome to automate the login process and extract session cookies.

use std::collections::HashMap;
use std::sync::Arc;
use anyhow::{Result, Context, anyhow};
use headless_chrome::{Browser, LaunchOptions, Tab};
use tracing::{info, debug, warn};

/// Main guided authentication struct
pub struct GuidedAuth {
    browser: Option<Browser>,
    active_tab: Option<Arc<Tab>>,
}

impl GuidedAuth {
    /// Create new guided authentication instance
    pub fn new() -> Self {
        Self {
            browser: None,
            active_tab: None,
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

    /// Start browser and navigate to Okta login
    pub async fn authenticate(&mut self) -> Result<HashMap<String, String>> {
        info!("Starting guided authentication flow...");

        // Navigate to Okta OAuth authorization endpoint
        let okta_url = "https://postman.okta.com/oauth2/v1/authorize?client_id=okta.2b1959c8-bcc0-56eb-a589-cfcfb7422f26&code_challenge=QqOla_j2ieDvzX7ebLtkAvwddcCQrhgFmqW0OgXEkTE&code_challenge_method=S256&nonce=oCAMKaZsIi9TTTkla80f4MnJfMn8kOlx0uWRhtL2AG1IcN5XVZF9UF83vjxgX8dg&redirect_uri=https%3A%2F%2Fpostman.okta.com%2Fenduser%2Fcallback&response_type=code&state=X9clhfyFhEE90WBMcHIaBtS2EtXzbYZEe4eN4XTF1PTqawEr3A4TGvD6UFTO6gV0&scope=openid%20profile%20email%20okta.users.read.self%20okta.users.manage.self%20okta.internal.enduser.read%20okta.internal.enduser.manage%20okta.enduser.dashboard.read%20okta.enduser.dashboard.manage%20okta.myAccount.sessions.manage";

        info!("Navigating to Okta login page...");
        self.navigate_to_okta(okta_url).await?;

        // Wait for authentication options to load
        info!("Waiting for authentication options to load...");
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;

        // Click on Okta Verify authentication option
        info!("Selecting Okta Verify authentication...");
        match self.click_okta_verify_option().await {
            Ok(()) => info!("Successfully selected Okta Verify"),
            Err(e) => {
                warn!("Failed to click Okta Verify, trying alternative selector: {}", e);
                // Try alternative selector
                self.click_element("a.select-factor:first-of-type").await?;
            }
        }

        // Wait for Okta Verify to process
        info!("Waiting for Okta Verify authentication...");
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;

        // Check if we've been redirected after successful auth
        let current_url = self.get_current_url().await?;
        info!("Current URL after auth: {}", current_url);

        // Extract cookies after authentication
        let cookies = self.extract_cookies().await?;

        Ok(cookies)
    }

    /// Click on Okta Verify authentication option
    async fn click_okta_verify_option(&self) -> Result<()> {
        // Try to click using aria-label
        self.click_element(r#"a[aria-label="Select Okta Verify."]"#).await
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

        // Set browser options
        launch_options
            .headless(true)  // Run in headless mode for automated authentication
            .window_size(Some((1280, 1024)))
            .args(vec![
                std::ffi::OsStr::new("--disable-blink-features=AutomationControlled"),
                std::ffi::OsStr::new("--user-agent=Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36"),
            ]);

        // Launch browser
        let browser = Browser::new(launch_options.build()?)
            .context("Failed to launch Chrome browser")?;

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
                "/Applications/Google Chrome Canary.app/Contents/MacOS/Google Chrome Canary".to_string(),
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

    /// Navigate to Okta and perform authentication
    pub async fn navigate_to_okta(&mut self, url: &str) -> Result<()> {
        // Launch browser if not already initialized
        if self.browser.is_none() {
            let browser = self.launch_browser()?;
            self.browser = Some(browser);
        }

        let browser = self.browser.as_ref()
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
                browser.new_tab()
                    .context("Failed to create new browser tab")?
            };
            self.active_tab = Some(tab.clone());
            tab
        };

        info!("Navigating to: {}", url);
        tab.navigate_to(url)
            .context("Failed to navigate to URL")?;

        tab.wait_until_navigated()
            .context("Failed to wait for navigation")?;

        Ok(())
    }

    /// Extract cookies from authenticated session
    pub async fn extract_cookies(&self) -> Result<HashMap<String, String>> {
        let tab = self.active_tab.as_ref()
            .ok_or_else(|| anyhow!("No active tab"))?;

        // Get cookies from the page
        let cookies = tab.get_cookies()
            .context("Failed to get cookies")?;

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
        let tab = self.active_tab.as_ref()
            .ok_or_else(|| anyhow!("No active tab"))?;

        info!("Clicking element: {}", selector);

        // Find and click the element
        let element = tab.find_element(selector)
            .context("Failed to find element")?;

        element.click()
            .context("Failed to click element")?;

        Ok(())
    }

    /// Wait for element to appear
    pub async fn wait_for_element(&self, selector: &str, timeout_secs: u64) -> Result<()> {
        let tab = self.active_tab.as_ref()
            .ok_or_else(|| anyhow!("No active tab"))?;

        info!("Waiting for element: {} (timeout: {}s)", selector, timeout_secs);

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
        let tab = self.active_tab.as_ref()
            .ok_or_else(|| anyhow!("No active tab"))?;

        let url = tab.get_url();
        Ok(url)
    }

    /// Execute JavaScript in the page
    pub async fn execute_js(&self, script: &str) -> Result<String> {
        let tab = self.active_tab.as_ref()
            .ok_or_else(|| anyhow!("No active tab"))?;

        let result = tab.evaluate(script, false)
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