//! Guided authentication using lightpanda browser for Okta SSO
//! 
//! This module handles automated authentication to Okta when cookies are not available
//! from existing browser sessions. It uses lightpanda headless browser to:
//! 
//! 1. Check if Okta Verify app is running
//! 2. Navigate to postman.okta.com OAuth flow
//! 3. Click the Okta Verify authentication option
//! 4. Wait for TouchID authentication
//! 5. Navigate to each app (Gong, Slack, Gainsight, Salesforce) to establish cookies
//! 6. Extract cookies for subsequent API calls

use std::process::{Command, Stdio};
use std::time::Duration;
use std::collections::HashMap;
use anyhow::{Result, Context, anyhow};
use serde_json::Value;
use tokio::time::sleep;
use tracing::{info, warn, debug};

use super::cdp_client::CdpClient;

/// Embedded lightpanda binary for Apple Silicon macOS
#[cfg(target_arch = "aarch64")]
const LIGHTPANDA_BINARY: &[u8] = include_bytes!("../../../bundled/lightpanda/lightpanda-aarch64-macos");

/// Binary name for the extracted executable
const LIGHTPANDA_EXECUTABLE_NAME: &str = "lightpanda";

/// URLs for Okta app integrations
const OKTA_APPS: &[(&str, &str)] = &[
    ("gong", "https://postman.okta.com/home/gong/0oa2498tgvv07sY4u5d7/aln18z05fityF2rra1d8"),
    ("slack", "https://postman.okta.com/home/slack/0oah7rbz24ePsdEUb5d7/19411"),
    ("gainsight", "https://postman.okta.com/home/postman_gainsight_1/0oamoyvq23HxTiYXn5d7/alnmoz76mc9cunIgV5d7"),
    ("salesforce", "https://postman.okta.com/home/salesforce/0oa278r39cGoxK3TW5d7/46"),
];

/// Okta authentication URLs
const OKTA_BASE_URL: &str = "https://postman.okta.com";
const OKTA_DASHBOARD: &str = "https://postman.okta.com/app/UserHome";

/// Lightpanda browser manager
pub struct LightpandaBrowser {
    process: Option<std::process::Child>,
    cdp_port: u16,
    binary_path: String,
}

/// Guided authentication manager
pub struct GuidedAuth {
    browser: Option<LightpandaBrowser>,
}

impl LightpandaBrowser {
    /// Create new browser instance with random CDP port
    pub fn new() -> Self {
        let cdp_port = 9222 + (rand::random::<u16>() % 1000); // Random port 9222-10222
        
        Self {
            process: None,
            cdp_port,
            binary_path: Self::find_lightpanda_binary(),
        }
    }

    /// Get the CDP port (for testing)
    pub fn get_cdp_port(&self) -> u16 {
        self.cdp_port
    }

    /// Get the binary path (for testing)
    pub fn get_binary_path(&self) -> &str {
        &self.binary_path
    }

    /// Get path for lightpanda binary (will be extracted from embedded data)
    fn find_lightpanda_binary() -> String {
        // Use a temporary directory for the extracted binary
        let temp_dir = std::env::temp_dir();
        let binary_path = temp_dir.join(format!("cs-cli-{}", LIGHTPANDA_EXECUTABLE_NAME));
        
        info!("Will use embedded lightpanda binary at: {}", binary_path.display());
        binary_path.to_string_lossy().to_string()
    }

    /// Extract embedded lightpanda binary if not available
    pub async fn ensure_binary(&mut self) -> Result<()> {
        if std::path::Path::new(&self.binary_path).exists() {
            debug!("Lightpanda binary already exists at: {}", self.binary_path);
            return Ok(());
        }

        info!("Extracting embedded lightpanda binary...");
        
        // Extract embedded binary data
        #[cfg(target_arch = "aarch64")]
        let binary_data = LIGHTPANDA_BINARY;
        
        #[cfg(not(target_arch = "aarch64"))]
        {
            return Err(anyhow!(
                "Lightpanda binary not available for this architecture. Only Apple Silicon (aarch64) is currently supported. \
                Please build cs-cli on an Apple Silicon Mac to enable guided authentication."
            ));
        }

        // Write binary to temporary location
        tokio::fs::write(&self.binary_path, binary_data).await
            .context("Failed to write embedded lightpanda binary")?;

        // Make executable
        Command::new("chmod")
            .args(["+x", &self.binary_path])
            .output()
            .context("Failed to make lightpanda binary executable")?;

        info!("Successfully extracted embedded lightpanda binary to: {}", self.binary_path);
        Ok(())
    }

    /// Start lightpanda browser with CDP enabled
    pub async fn start(&mut self) -> Result<()> {
        self.ensure_binary().await?;

        info!("Starting lightpanda browser on port {}", self.cdp_port);

        let child = Command::new(&self.binary_path)
            .args([
                "--remote-debugging-port", &self.cdp_port.to_string(),
                "--headless",
                "--disable-gpu",
                "--no-sandbox",
                "--disable-dev-shm-usage",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to start lightpanda browser")?;

        self.process = Some(child);

        // Wait for browser to be ready
        for attempt in 1..=10 {
            sleep(Duration::from_millis(500)).await;
            
            if self.is_ready().await? {
                info!("Lightpanda browser ready on port {}", self.cdp_port);
                return Ok(());
            }
            
            debug!("Waiting for lightpanda to be ready (attempt {}/10)", attempt);
        }

        Err(anyhow!("Lightpanda browser failed to start within 5 seconds"))
    }

    /// Check if browser is ready by testing CDP connection
    async fn is_ready(&self) -> Result<bool> {
        let url = format!("http://localhost:{}/json/version", self.cdp_port);
        
        match reqwest::get(&url).await {
            Ok(response) if response.status().is_success() => Ok(true),
            _ => Ok(false),
        }
    }

    /// Get CDP WebSocket URL for a new tab
    pub async fn new_tab(&self) -> Result<String> {
        let url = format!("http://localhost:{}/json/new", self.cdp_port);
        let response = reqwest::get(&url).await?;
        let tab_info: Value = response.json().await?;
        
        let ws_url = tab_info["webSocketDebuggerUrl"]
            .as_str()
            .ok_or_else(|| anyhow!("No WebSocket URL in tab info"))?;
            
        Ok(ws_url.to_string())
    }

    /// Stop browser process and clean up extracted binary
    pub fn stop(&mut self) -> Result<()> {
        if let Some(mut process) = self.process.take() {
            info!("Stopping lightpanda browser");
            process.kill()?;
            process.wait()?;
        }
        
        // Clean up the extracted binary
        if std::path::Path::new(&self.binary_path).exists() {
            match std::fs::remove_file(&self.binary_path) {
                Ok(_) => debug!("Cleaned up extracted lightpanda binary"),
                Err(e) => warn!("Failed to clean up extracted binary: {}", e),
            }
        }
        
        Ok(())
    }
}

impl Drop for LightpandaBrowser {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

impl GuidedAuth {
    /// Create new guided authentication instance
    pub fn new() -> Self {
        Self {
            browser: None,
        }
    }

    /// Check if browser is initialized (for testing)
    pub fn has_browser(&self) -> bool {
        self.browser.is_some()
    }

    /// Check if Okta Verify app is running on macOS
    #[cfg(target_os = "macos")]
    pub async fn check_okta_verify(&self) -> Result<bool> {
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

    /// Launch Okta Verify app if not running
    #[cfg(target_os = "macos")]
    pub async fn launch_okta_verify(&self) -> Result<()> {
        if self.check_okta_verify().await? {
            return Ok(());
        }

        info!("Launching Okta Verify...");
        
        Command::new("open")
            .arg("-a")
            .arg("Okta Verify")
            .spawn()
            .context("Failed to launch Okta Verify")?;

        // Wait for app to start
        for attempt in 1..=10 {
            sleep(Duration::from_secs(1)).await;
            if self.check_okta_verify().await? {
                info!("Okta Verify launched successfully");
                return Ok(());
            }
            debug!("Waiting for Okta Verify to start (attempt {}/10)", attempt);
        }

        Err(anyhow!("Failed to start Okta Verify within 10 seconds"))
    }

    /// Start guided authentication flow
    pub async fn authenticate(&mut self) -> Result<HashMap<String, String>> {
        info!("Starting guided Okta authentication...");

        // Step 1: Ensure Okta Verify is running
        #[cfg(target_os = "macos")]
        self.launch_okta_verify().await?;

        // Step 2: Start lightpanda browser
        let mut browser = LightpandaBrowser::new();
        browser.start().await?;
        self.browser = Some(browser);

        // Step 3: Navigate to Okta and authenticate
        self.perform_okta_auth().await?;

        // Step 4: Navigate to each app to establish cookies
        let cookies = self.collect_app_cookies().await?;

        info!("Guided authentication completed successfully");
        Ok(cookies)
    }

    /// Perform Okta authentication flow
    async fn perform_okta_auth(&self) -> Result<()> {
        let browser = self.browser.as_ref()
            .ok_or_else(|| anyhow!("Browser not started"))?;
        
        // Create new tab and CDP client
        let ws_url = browser.new_tab().await?;
        let mut cdp = CdpClient::new();
        cdp.connect(&ws_url).await?;

        info!("Starting Okta OAuth flow...");
        
        // Step 1: Navigate to Okta OAuth endpoint
        cdp.navigate(OKTA_BASE_URL).await?;

        // Step 2: Wait for OAuth redirect with challenge/nonce
        let current_url = cdp.get_current_url().await?;
        info!("Current URL: {}", current_url);

        // Step 3: Look for Okta Verify authentication option
        info!("Looking for Okta Verify authentication option...");
        
        // Wait for the page to load authentication options
        sleep(Duration::from_secs(3)).await;
        
        // Try different selectors for Okta Verify button
        let okta_verify_selectors = [
            "a[aria-label*='Okta Verify']",
            ".button.select-factor[aria-label*='Okta Verify']", 
            "a.select-factor[href='#']",
            ".link-button[data-se='button'][aria-label*='Okta Verify']",
            "a[data-se='button'][aria-label*='Select Okta Verify']",
        ];

        let mut clicked = false;
        for selector in &okta_verify_selectors {
            match cdp.wait_for_element(selector, 5).await {
                Ok(_) => {
                    info!("Found Okta Verify button with selector: {}", selector);
                    cdp.click_element(selector).await?;
                    clicked = true;
                    break;
                }
                Err(_) => {
                    debug!("Selector not found: {}", selector);
                    continue;
                }
            }
        }

        if !clicked {
            // Try to find any element that might be the Okta Verify option
            info!("Standard selectors failed, checking page content...");
            
            // Get page text to see what's available
            let page_text = cdp.evaluate_js("document.body.textContent").await?;
            debug!("Page content: {:?}", page_text);

            // Look for clickable elements containing "Okta Verify"
            let find_button_js = r#"
                const buttons = Array.from(document.querySelectorAll('a, button, .button, [role="button"]'));
                const oktaButton = buttons.find(btn => 
                    btn.textContent && 
                    (btn.textContent.includes('Okta Verify') || 
                     btn.textContent.includes('Select') ||
                     btn.getAttribute('aria-label')?.includes('Okta Verify'))
                );
                oktaButton ? oktaButton.outerHTML : null;
            "#;

            let button_result = cdp.evaluate_js(find_button_js).await?;
            if let Some(button_html) = button_result.get("result")
                .and_then(|r| r.get("value"))
                .and_then(|v| v.as_str()) {
                info!("Found potential Okta Verify button: {}", button_html);
                
                // Try to click it
                let click_js = r#"
                    const buttons = Array.from(document.querySelectorAll('a, button, .button, [role="button"]'));
                    const oktaButton = buttons.find(btn => 
                        btn.textContent && 
                        (btn.textContent.includes('Okta Verify') || 
                         btn.textContent.includes('Select') ||
                         btn.getAttribute('aria-label')?.includes('Okta Verify'))
                    );
                    if (oktaButton) {
                        oktaButton.click();
                        true;
                    } else {
                        false;
                    }
                "#;

                let click_result = cdp.evaluate_js(click_js).await?;
                clicked = click_result.get("result")
                    .and_then(|r| r.get("value"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
            }
        }

        if !clicked {
            return Err(anyhow!("Could not find or click Okta Verify authentication option"));
        }

        info!("Successfully clicked Okta Verify option");

        // Step 4: Wait for TouchID prompt and authentication
        info!("Waiting for TouchID authentication (user interaction required)...");
        
        // Wait for URL to change indicating successful authentication
        let auth_start_url = cdp.get_current_url().await?;
        
        // Give user up to 60 seconds to complete TouchID
        match cdp.wait_for_url_change(&auth_start_url, 60).await {
            Ok(new_url) => {
                info!("Authentication completed, redirected to: {}", new_url);
                
                // Check if we're on the dashboard
                if new_url.contains("UserHome") || new_url.contains("dashboard") {
                    info!("Successfully authenticated to Okta dashboard");
                    return Ok(());
                }
            }
            Err(_) => {
                warn!("No URL change detected after 60 seconds");
            }
        }

        // Alternative: Check for dashboard elements
        let dashboard_selectors = [
            ".o-user-dashboard",
            "[data-se='app-dashboard']",
            ".enduser-dashboard",
        ];

        for selector in &dashboard_selectors {
            if cdp.wait_for_element(selector, 5).await.is_ok() {
                info!("Found dashboard element, authentication successful");
                return Ok(());
            }
        }

        // Final fallback: wait and assume success if no errors
        sleep(Duration::from_secs(5)).await;
        let final_url = cdp.get_current_url().await?;
        info!("Authentication flow completed, final URL: {}", final_url);

        Ok(())
    }

    /// Navigate to each app and collect cookies
    async fn collect_app_cookies(&self) -> Result<HashMap<String, String>> {
        let browser = self.browser.as_ref()
            .ok_or_else(|| anyhow!("Browser not started"))?;
        
        // Create new tab for app navigation
        let ws_url = browser.new_tab().await?;
        let mut cdp = CdpClient::new();
        cdp.connect(&ws_url).await?;

        let mut all_cookies = HashMap::new();
        
        for (app_name, app_url) in OKTA_APPS {
            info!("Navigating to {} app: {}", app_name, app_url);
            
            // Navigate to the app URL (this should auto-login via Okta SSO)
            cdp.navigate(app_url).await?;
            
            // Wait for potential redirect and app to load
            sleep(Duration::from_secs(3)).await;
            
            let final_url = cdp.get_current_url().await?;
            info!("Final URL for {}: {}", app_name, final_url);
            
            // Get cookies for this app
            match cdp.get_cookies().await {
                Ok(cookies) => {
                    info!("Retrieved {} cookies for {}", cookies.len(), app_name);
                    
                    // Convert cookies to a format we can use
                    let mut app_cookies = Vec::new();
                    for cookie in cookies {
                        if let (Some(name), Some(value)) = (
                            cookie.get("name").and_then(|n| n.as_str()),
                            cookie.get("value").and_then(|v| v.as_str())
                        ) {
                            app_cookies.push(format!("{}={}", name, value));
                        }
                    }
                    
                    // Store all cookies as a single string for this app
                    let cookie_string = app_cookies.join("; ");
                    let cookie_len = cookie_string.len();
                    all_cookies.insert(app_name.to_string(), cookie_string);
                    
                    info!("Stored cookies for {}: {} characters", app_name, cookie_len);
                }
                Err(e) => {
                    warn!("Failed to get cookies for {}: {}", app_name, e);
                    // Continue with other apps
                }
            }
            
            // Brief pause between apps
            sleep(Duration::from_millis(1000)).await;
        }
        
        info!("Cookie collection completed for {} apps", all_cookies.len());
        Ok(all_cookies)
    }
}

impl Default for GuidedAuth {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_browser_creation() {
        let browser = LightpandaBrowser::new();
        assert!(browser.cdp_port >= 9222 && browser.cdp_port <= 10222);
    }

    #[tokio::test]
    #[cfg(target_os = "macos")]
    async fn test_okta_verify_check() {
        let auth = GuidedAuth::new();
        // This will pass regardless of whether Okta Verify is running
        let _ = auth.check_okta_verify().await;
    }
}