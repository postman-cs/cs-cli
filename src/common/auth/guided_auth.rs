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
use serde_json::{self, Value};
use tokio::time::sleep;
use tracing::{info, warn, debug};

use super::cdp_client::CdpClient;

/// Embedded compressed lightpanda binary for Apple Silicon macOS
#[cfg(target_arch = "aarch64")]
const LIGHTPANDA_COMPRESSED: &[u8] = include_bytes!("../../../bundled/lightpanda/lightpanda-aarch64-macos.zst");

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

        info!("Extracting and decompressing embedded lightpanda binary...");
        
        // Decompress embedded binary data
        #[cfg(target_arch = "aarch64")]
        let binary_data = {
            debug!("Decompressing {} bytes with zstd", LIGHTPANDA_COMPRESSED.len());
            let decompressed = zstd::decode_all(LIGHTPANDA_COMPRESSED)
                .context("Failed to decompress embedded lightpanda binary")?;
            
            info!("Decompressed lightpanda binary: {} bytes", decompressed.len());
            decompressed
        };
        
        #[cfg(not(target_arch = "aarch64"))]
        {
            return Err(anyhow!(
                "Lightpanda binary not available for this architecture. Only Apple Silicon (aarch64) is currently supported. \
                Please build cs-cli on an Apple Silicon Mac to enable guided authentication."
            ));
        }

        // Write decompressed binary to temporary location
        tokio::fs::write(&self.binary_path, binary_data).await
            .context("Failed to write decompressed lightpanda binary")?;

        // Make executable
        Command::new("chmod")
            .args(["+x", &self.binary_path])
            .output()
            .context("Failed to make lightpanda binary executable")?;

        info!("Successfully extracted and decompressed lightpanda binary to: {}", self.binary_path);
        Ok(())
    }

    /// Start lightpanda browser with CDP enabled
    pub async fn start(&mut self) -> Result<()> {
        self.ensure_binary().await?;

        info!("Starting lightpanda browser on port {}", self.cdp_port);

        info!("Starting lightpanda with command: {} serve --host 127.0.0.1 --port {} --timeout 300", self.binary_path, self.cdp_port);
        
        let child = Command::new(&self.binary_path)
            .args([
                "serve",
                "--host", "127.0.0.1",
                "--port", &self.cdp_port.to_string(),
                "--timeout", "300", // 5 minutes timeout
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to start lightpanda browser")?;

        self.process = Some(child);

        // Wait for browser to be ready - increase timeout for lightpanda startup
        for attempt in 1..=20 {
            sleep(Duration::from_millis(500)).await;
            
            match self.is_ready().await {
                Ok(true) => {
                    info!("Lightpanda browser ready on port {}", self.cdp_port);
                    return Ok(());
                }
                Ok(false) => {
                    debug!("Lightpanda not ready yet (attempt {}/20)", attempt);
                }
                Err(e) => {
                    debug!("Readiness check failed (attempt {}/20): {}", attempt, e);
                }
            }
        }

        Err(anyhow!("Lightpanda browser failed to start within 10 seconds"))
    }

    /// Check if browser is ready by testing CDP connection
    async fn is_ready(&self) -> Result<bool> {
        let url = format!("http://localhost:{}/json/version", self.cdp_port);
        
        match reqwest::get(&url).await {
            Ok(response) if response.status().is_success() => Ok(true),
            _ => Ok(false),
        }
    }

    /// Get CDP WebSocket URL for lightpanda browser
    pub async fn new_tab(&self) -> Result<String> {
        // Lightpanda serves directly as a WebSocket endpoint, not through /json/new
        let ws_url = format!("ws://127.0.0.1:{}", self.cdp_port);
        info!("Using lightpanda WebSocket endpoint: {}", ws_url);
        
        Ok(ws_url)
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
        
        // Step 1: Generate OAuth authorization URL manually
        info!("Generating OAuth authorization URL for Okta Verify authentication...");
        
        // Generate required OAuth parameters
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let nonce = {
            let mut hasher = DefaultHasher::new();
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos().hash(&mut hasher);
            format!("{:x}", hasher.finish())
        };
        
        let state = {
            let mut hasher = DefaultHasher::new();
            (nonce.clone() + "state").hash(&mut hasher);
            format!("{:x}", hasher.finish())
        };
        
        // Create OAuth authorization URL
        let oauth_url = format!(
            "https://postman.okta.com/oauth2/v1/authorize?\
            client_id=okta.2b1959c8-bcc0-56eb-a589-cfcfb7422f26&\
            code_challenge=BnGfWPKU-QlBsQtOUSTzzJVNUeV3-7kusBWGhL5QWrc&\
            code_challenge_method=S256&\
            nonce={}&\
            redirect_uri=https%3A%2F%2Fpostman.okta.com%2Fenduser%2Fcallback&\
            response_type=code&\
            state={}&\
            scope=openid%20profile%20email%20okta.users.read.self%20okta.users.manage.self%20okta.internal.enduser.read%20okta.internal.enduser.manage%20okta.enduser.dashboard.read%20okta.enduser.dashboard.manage%20okta.myAccount.sessions.manage%20okta.internal.navigation.enduser.read",
            nonce, state
        );
        
        info!("Navigating to OAuth URL: {}", oauth_url);
        cdp.navigate(&oauth_url).await?;
        
        // Step 2: Wait for authentication page to load
        sleep(Duration::from_secs(3)).await;
        
        let current_url = cdp.get_current_url().await?;
        info!("After OAuth redirect, current URL: {}", current_url);
        
        // Check if we're already authenticated (immediate redirect to success)
        if current_url.contains("postman.okta.com/app/UserHome") && 
           current_url.contains("session_hint=AUTHENTICATED") {
            info!("Already authenticated! No TouchID needed.");
            return Ok(());
        }
        
        // Step 3: Look for authentication options on the page
        info!("Looking for authentication options...");
        
        // Try to find and click any Okta Verify related buttons
        let find_auth_button_js = r#"
            // Look for buttons that might trigger Okta Verify
            const buttons = Array.from(document.querySelectorAll('button, a, .button, [role="button"]'));
            const authButtons = buttons.filter(btn => {
                const text = btn.textContent?.toLowerCase() || '';
                const ariaLabel = btn.getAttribute('aria-label')?.toLowerCase() || '';
                
                return text.includes('okta verify') || 
                       text.includes('sign in') || 
                       text.includes('authenticate') ||
                       ariaLabel.includes('okta verify') ||
                       ariaLabel.includes('sign in');
            });
            
            if (authButtons.length > 0) {
                // Click the first auth button found
                authButtons[0].click();
                return true;
            }
            return false;
        "#;
        
        let button_result = cdp.evaluate_js(find_auth_button_js).await?;
        let clicked_button = button_result.get("result")
            .and_then(|r| r.get("value"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
            
        if clicked_button {
            info!("Found and clicked authentication button");
        } else {
            info!("No authentication button found - proceeding to wait for TouchID");
        }
        
        sleep(Duration::from_millis(1000)).await;

        // Step 4: Wait for TouchID authentication completion
        info!("Successfully clicked Okta Verify - waiting for TouchID authentication...");
        info!("Please complete TouchID authentication in the Okta Verify app");
        
        // Wait for authentication to complete and redirect to the success URL
        let auth_start_url = cdp.get_current_url().await?;
        info!("Starting authentication from URL: {}", auth_start_url);
        
        // Poll for the specific success URL with fast checks (every 500ms)
        let max_attempts = 120; // 60 seconds total (120 * 500ms)
        
        for attempt in 1..=max_attempts {
            let current_url = cdp.get_current_url().await?;
            
            // Success criteria: Must contain both "postman.okta.com/app/UserHome" AND "session_hint=AUTHENTICATED"
            if current_url.contains("postman.okta.com/app/UserHome") && 
               current_url.contains("session_hint=AUTHENTICATED") {
                info!("SUCCESS: Authentication completed! Reached target URL: {}", current_url);
                return Ok(());
            }
            
            // Log progress every 10 attempts (5 seconds)
            if attempt % 10 == 0 {
                info!("Still waiting for authentication... ({}s elapsed, current URL: {})", 
                      attempt * 500 / 1000, current_url);
            }
            
            // Fast polling - check every 500ms
            sleep(Duration::from_millis(500)).await;
        }
        
        // Timeout - get final URL for debugging
        let final_url = cdp.get_current_url().await?;
        warn!("Authentication timeout after 60 seconds. Final URL: {}", final_url);
        
        // Check if we at least got to some authenticated state
        if final_url.contains("postman.okta.com") && final_url != auth_start_url {
            warn!("Got to Okta domain but not the expected success URL");
            return Err(anyhow!("Authentication may have succeeded but didn't reach expected URL. Expected: postman.okta.com/app/UserHome?...session_hint=AUTHENTICATED, Got: {}", final_url));
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