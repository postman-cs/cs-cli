//! Guided authentication module for automating Okta SSO login
//!
//! This module provides browser automation for guided authentication through Okta SSO.
//! It uses Fantoccini WebDriver to automate the login process and access your own session cookies,
//! with support for multiple browsers (Chrome, Firefox, Safari).

use crate::common::auth::cookie_retriever::Cookie;
use crate::common::auth::guided_auth_config::{DynamicCookieDomains, GuidedAuthConfig};
use crate::common::auth::guided_cookie_storage::are_cookies_expired;
use crate::common::auth::hybrid_cookie_storage::{
    get_cookies_hybrid, has_cookies_hybrid, store_cookies_hybrid,
};
use crate::common::auth::profile_detector::{BrowserProfile, BrowserType, PlatformType, ProfileDetector};
use crate::common::auth::temp_profile_manager::TempProfileManager;
use crate::common::drivers::{Browser, DriverManager};
use crate::common::error::{CsCliError, Result};
use cookie::Expiration;
use fantoccini::{Client, Locator};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Main guided authentication struct
pub struct GuidedAuth {
    client: Option<Client>,
    config: GuidedAuthConfig,
    cookie_domains: DynamicCookieDomains,
    #[allow(dead_code)]  // Will be used when headless mode is configured
    headless: bool,
    selected_profile: Option<BrowserProfile>,
    profile_detector: Option<ProfileDetector>,
    driver_manager: DriverManager,
    selected_browser: Browser,
    temp_profile_manager: Option<TempProfileManager>,
}

impl Default for GuidedAuth {
    fn default() -> Self {
        Self::new()
    }
}

impl GuidedAuth {
    /// Create new guided authentication instance
    pub fn new() -> Self {
        Self {
            client: None,
            config: GuidedAuthConfig::default(),
            cookie_domains: DynamicCookieDomains::new(),
            headless: true,
            selected_profile: None,
            profile_detector: None,
            driver_manager: DriverManager::new().unwrap_or_else(|_| {
                panic!("Failed to initialize driver manager")
            }),
            selected_browser: Browser::Chrome, // Default to Chrome
            temp_profile_manager: None,
        }
    }

    /// Create new guided authentication instance with browser visible (for testing)
    pub fn new_headed() -> Self {
        Self {
            client: None,
            config: GuidedAuthConfig::default(),
            cookie_domains: DynamicCookieDomains::new(),
            headless: false,
            selected_profile: None,
            profile_detector: None,
            driver_manager: DriverManager::new().unwrap_or_else(|_| {
                panic!("Failed to initialize driver manager")
            }),
            selected_browser: Browser::Chrome,
            temp_profile_manager: None,
        }
    }

    /// Check if browser is initialized (for testing)
    pub fn has_browser(&self) -> bool {
        self.client.is_some()
    }

    /// Create new guided authentication instance with specific browser profile
    pub fn with_profile(profile: BrowserProfile) -> Self {
        // Determine browser type from profile
        let browser_type = match profile.browser_type {
            BrowserType::Chrome => Browser::Chrome,
            BrowserType::Firefox => Browser::Firefox,
            BrowserType::Safari => Browser::Safari,
            BrowserType::Brave => Browser::Brave,
            BrowserType::Edge => Browser::Edge,
            BrowserType::Arc => Browser::Arc,
            BrowserType::Chromium => Browser::Chromium,
            BrowserType::LibreWolf => Browser::LibreWolf,
            BrowserType::Opera => Browser::Opera,
            BrowserType::OperaGX => Browser::OperaGX,
            BrowserType::Vivaldi => Browser::Vivaldi,
            BrowserType::Zen => Browser::Zen,
            BrowserType::Comet => Browser::Comet,
        };

        Self {
            client: None,
            config: GuidedAuthConfig::default(),
            cookie_domains: DynamicCookieDomains::new(),
            headless: true,
            selected_profile: Some(profile),
            profile_detector: None,
            driver_manager: DriverManager::new().unwrap_or_else(|_| {
                panic!("Failed to initialize driver manager")
            }),
            selected_browser: browser_type,
            temp_profile_manager: None,
        }
    }

    /// Create new guided authentication instance with platform-specific profile preference
    pub fn with_platform_preference(platform: PlatformType) -> Result<Self> {
        let detector = ProfileDetector::new()
            .map_err(|e| CsCliError::Authentication(format!("Failed to initialize profile detector: {e}")))?;

        let selected_profile = detector.find_best_profile_for_platform(&platform)
            .cloned();

        let browser_type = if let Some(ref profile) = selected_profile {
            info!("Selected profile for {:?}: {}", platform, profile.description());
            match profile.browser_type {
                BrowserType::Chrome => Browser::Chrome,
                BrowserType::Firefox => Browser::Firefox,
                BrowserType::Safari => Browser::Safari,
                BrowserType::Brave => Browser::Brave,
                BrowserType::Edge => Browser::Edge,
                BrowserType::Arc => Browser::Arc,
                BrowserType::Chromium => Browser::Chromium,
                BrowserType::LibreWolf => Browser::LibreWolf,
                BrowserType::Opera => Browser::Opera,
                BrowserType::OperaGX => Browser::OperaGX,
                BrowserType::Vivaldi => Browser::Vivaldi,
                BrowserType::Zen => Browser::Zen,
                BrowserType::Comet => Browser::Comet,
            }
        } else {
            warn!("No suitable profile found for platform {:?}, using default", platform);
            Browser::Chrome
        };

        Ok(Self {
            client: None,
            config: GuidedAuthConfig::default(),
            cookie_domains: DynamicCookieDomains::new(),
            headless: true,
            selected_profile,
            profile_detector: Some(detector),
            driver_manager: DriverManager::new()?,
            selected_browser: browser_type,
            temp_profile_manager: None,
        })
    }

    /// Get the currently selected profile
    pub fn get_selected_profile(&self) -> Option<&BrowserProfile> {
        self.selected_profile.as_ref()
    }

    /// Get available profiles (initializes detector if needed)
    pub fn get_available_profiles(&mut self) -> Result<&[BrowserProfile]> {
        if self.profile_detector.is_none() {
            self.profile_detector = Some(ProfileDetector::new()
                .map_err(|e| CsCliError::Authentication(format!("Failed to initialize profile detector: {e}")))?);
        }

        Ok(&self.profile_detector.as_ref().unwrap().detected_profiles)
    }

    /// Set a specific profile to use
    pub fn set_profile(&mut self, profile: BrowserProfile) {
        info!("Setting profile: {}", profile.description());

        // Update browser type based on profile
        self.selected_browser = match profile.browser_type {
            BrowserType::Chrome => Browser::Chrome,
            BrowserType::Firefox => Browser::Firefox,
            BrowserType::Safari => Browser::Safari,
            BrowserType::Brave => Browser::Brave,
            BrowserType::Edge => Browser::Edge,
            BrowserType::Arc => Browser::Arc,
            BrowserType::Chromium => Browser::Chromium,
            BrowserType::LibreWolf => Browser::LibreWolf,
            BrowserType::Opera => Browser::Opera,
            BrowserType::OperaGX => Browser::OperaGX,
            BrowserType::Vivaldi => Browser::Vivaldi,
            BrowserType::Zen => Browser::Zen,
            BrowserType::Comet => Browser::Comet,
        };

        self.selected_profile = Some(profile);
    }

    /// Initialize browser with the appropriate driver
    async fn init_browser(&mut self) -> Result<()> {
        if self.client.is_some() {
            return Ok(());
        }

        info!("Initializing browser: {:?}", self.selected_browser);

        // If we have a selected profile, try creating a temporary profile to avoid conflicts
        let profile_to_use = if let Some(ref original_profile) = self.selected_profile {
            // Initialize temp profile manager if not already done
            if self.temp_profile_manager.is_none() {
                info!("Initializing temporary profile manager to avoid browser conflicts");
                self.temp_profile_manager = Some(TempProfileManager::new()?);
            }

            if let Some(ref mut temp_manager) = self.temp_profile_manager {
                match temp_manager.create_temp_profile(original_profile) {
                    Ok(temp_profile) => {
                        info!("Created temporary profile to avoid browser conflicts: {}", temp_profile.profile_path.display());
                        Some(temp_profile)
                    }
                    Err(e) => {
                        warn!("Failed to create temporary profile, using original: {}", e);
                        Some(original_profile.clone())
                    }
                }
            } else {
                Some(original_profile.clone())
            }
        } else {
            None
        };

        // Connect using the driver manager with profile information
        let client = match self.driver_manager.connect_with_profile(
            Some(self.selected_browser),
            profile_to_use.as_ref()
        ).await {
            Ok(c) => c,
            Err(e) => {
                // If first attempt fails, try cleaning up any stale processes and retry
                warn!("Initial browser connection failed: {}, retrying after cleanup", e);
                let _ = self.driver_manager.cleanup_all_drivers().await;
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                self.driver_manager.connect_with_profile(
                    Some(self.selected_browser),
                    profile_to_use.as_ref()
                ).await?
            }
        };

        // Set window size
        client.set_window_rect(0, 0, 1280, 1024).await
            .map_err(|e| CsCliError::Authentication(format!("Failed to set window size: {e}")))?;

        self.client = Some(client);
        Ok(())
    }

    /// Navigate to a URL
    async fn navigate_to_okta(&mut self, url: &str) -> Result<()> {
        self.init_browser().await?;

        if let Some(client) = &self.client {
            client.goto(url).await
                .map_err(|e| CsCliError::Authentication(format!("Failed to navigate to URL: {e}")))?;
        }

        Ok(())
    }

    /// Get current URL
    async fn get_current_url(&self) -> Result<String> {
        if let Some(client) = &self.client {
            let url = client.current_url().await
                .map_err(|e| CsCliError::Authentication(format!("Failed to get current URL: {e}")))?;
            Ok(url.to_string())
        } else {
            Err(CsCliError::Authentication("Browser not initialized".to_string()))
        }
    }

    /// Execute JavaScript
    async fn execute_js(&self, script: &str) -> Result<String> {
        if let Some(client) = &self.client {
            let result = client.execute(script, vec![])
                .await
                .map_err(|e| CsCliError::Authentication(format!("Failed to execute JavaScript: {e}")))?;

            // Convert result to string
            let value_str = serde_json::to_string(&result)
                .unwrap_or_else(|_| "null".to_string());

            Ok(value_str)
        } else {
            Err(CsCliError::Authentication("Browser not initialized".to_string()))
        }
    }

    /// Check if an element exists
    async fn element_exists(&self, selector: &str) -> bool {
        if let Some(client) = &self.client {
            client.find(Locator::Css(selector)).await.is_ok()
        } else {
            false
        }
    }

    /// Click an element
    async fn click_element(&self, selector: &str) -> Result<()> {
        if let Some(client) = &self.client {
            let element = client.find(Locator::Css(selector)).await
                .map_err(|e| CsCliError::Authentication(format!("Failed to find element {selector}: {e}")))?;
            element.click().await
                .map_err(|e| CsCliError::Authentication(format!("Failed to click element {selector}: {e}")))?;
            Ok(())
        } else {
            Err(CsCliError::Authentication("Browser not initialized".to_string()))
        }
    }

    /// Access your own cookies from the browser session
    async fn extract_cookies_from_browser(&self) -> Result<HashMap<String, Vec<Cookie>>> {
        if let Some(client) = &self.client {
            let cookies = client.get_all_cookies().await
                .map_err(|e| CsCliError::Authentication(format!("Failed to get cookies: {e}")))?;

            let mut result: HashMap<String, Vec<Cookie>> = HashMap::new();

            for cookie in cookies {
                let domain = cookie.domain().unwrap_or("").to_string();

                // Handle cookie expiration properly
                let expires = cookie.expires().and_then(|exp| {
                    match exp {
                        Expiration::DateTime(dt) => {
                            // Convert OffsetDateTime to Unix timestamp
                            Some(dt.unix_timestamp() as u64)
                        }
                        Expiration::Session => None, // Session cookies don't have explicit expiry
                    }
                });

                let c = Cookie {
                    name: cookie.name().to_string(),
                    value: cookie.value().to_string(),
                    domain: domain.clone(),
                    path: cookie.path().unwrap_or("/").to_string(),
                    expires,
                    http_only: cookie.http_only().unwrap_or(false),
                    secure: cookie.secure().unwrap_or(false),
                };

                result.entry(domain).or_default().push(c);
            }

            Ok(result)
        } else {
            Err(CsCliError::Authentication("Browser not initialized".to_string()))
        }
    }

    /// Check if already authenticated
    async fn check_already_authenticated(&self) -> Result<bool> {
        let url = self.get_current_url().await?;

        // Check if we've been redirected to a dashboard or app page
        Ok(url.contains("/app/") || url.contains("/dashboard") || url.contains("/home") || self.config.is_success_url(&url))
    }

    /// Check stored cookies
    async fn check_stored_cookies(&mut self) -> Result<HashMap<String, String>> {
        // Try to get cookies from storage
        if has_cookies_hybrid().await {
            let cookies = get_cookies_hybrid().await?;

            // Validate cookies aren't expired
            if !are_cookies_expired(&cookies) {
                return Ok(cookies);
            }
        }

        Err(CsCliError::Authentication("No valid stored cookies found".to_string()))
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

        // Check if we're already logged in
        if self.check_already_authenticated().await? {
            info!("Already authenticated! Proceeding directly to platform collection...");
        } else {
            // Wait for authentication options to load
            info!("Waiting for authentication options to load...");
            tokio::time::sleep(std::time::Duration::from_secs(
                self.config.timing.auth_options_load_wait,
            ))
            .await;

            // Try to detect and click Okta Verify button
            let okta_verify_clicked = self.try_click_okta_verify().await;

            if !okta_verify_clicked {
                warn!("Could not find Okta Verify button, waiting for manual authentication...");
            }

            // Wait for authentication completion
            info!("Waiting for authentication to complete...");
            let auth_timeout = std::time::Duration::from_secs(self.config.timing.auth_success_timeout);
            let start = std::time::Instant::now();

            while start.elapsed() < auth_timeout {
                if self.check_already_authenticated().await? {
                    info!("Authentication successful!");
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }

        // Collect cookies from all platforms
        let mut all_cookies = HashMap::new();

        for (platform_name, platform_url) in &self.config.platform_urls {
            info!("Navigating to {} to collect cookies...", platform_name);

            if let Some(client) = &self.client {
                // Navigate to platform
                client.goto(platform_url).await
                    .map_err(|e| CsCliError::Authentication(format!("Failed to navigate to {platform_name}: {e}")))?;

                // Wait for page load
                tokio::time::sleep(std::time::Duration::from_secs(
                    self.config.timing.platform_navigation_delay,
                ))
                .await;

                // Access platform-specific cookies from your authenticated session
                let browser_cookies = self.extract_cookies_from_browser().await?;

                // Store cookies for this platform
                for (_domain, cookies) in browser_cookies {
                    // Discover domains for this platform
                    let platform_cookies: Vec<Cookie> = cookies.clone();
                    let cookie_refs: Vec<&Cookie> = platform_cookies.iter().collect();
                    let discovered_domains = self.cookie_domains.discover_from_cookies(&cookie_refs, platform_name);

                    debug!("Discovered domains for {}: {:?}", platform_name, discovered_domains);

                    // Store cookies
                    for cookie in cookies {
                        all_cookies.insert(
                            format!("{}_{}", platform_name, cookie.name),
                            cookie.value,
                        );
                    }
                }
            }
        }

        // Store cookies for future use
        store_cookies_hybrid(&all_cookies).await?;

        Ok(all_cookies)
    }

    /// Try to click Okta Verify button
    async fn try_click_okta_verify(&self) -> bool {
        let okta_verify_selectors = vec![
            &self.config.selectors.okta_verify_primary,
            &self.config.selectors.okta_verify_fallback,
            "a[data-se='okta_verify-push']",
            "button[data-se='okta_verify-push']",
            "[data-se-for-name='okta_verify']",
        ];

        for selector in &okta_verify_selectors {
            if self.element_exists(selector).await {
                info!("Found Okta Verify button with selector: {}", selector);
                match self.click_element(selector).await {
                    Ok(_) => {
                        info!("Successfully clicked Okta Verify button");
                        return true;
                    }
                    Err(e) => {
                        warn!("Failed to click Okta Verify button: {}", e);
                    }
                }
            }
        }

        false
    }

    /// Get page HTML content
    pub async fn get_page_content(&self) -> Result<String> {
        self.execute_js("document.documentElement.outerHTML").await
    }

    /// Close the browser
    pub async fn close(&mut self) -> Result<()> {
        if let Some(client) = self.client.take() {
            client.close().await
                .map_err(|e| CsCliError::Authentication(format!("Failed to close browser: {e}")))?;
        }

        // Clean up temporary profiles
        if let Some(ref mut temp_manager) = self.temp_profile_manager {
            if let Err(e) = temp_manager.cleanup() {
                warn!("Failed to cleanup temporary profiles: {}", e);
            } else {
                info!("Successfully cleaned up temporary profiles");
            }
        }

        // The driver manager will handle process cleanup

        Ok(())
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
            .map_err(|e| CsCliError::Authentication(format!("Failed to check for Okta Verify process: {e}")))?;

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
}

impl Drop for GuidedAuth {
    fn drop(&mut self) {
        // Clean up the browser if it's still open
        if self.client.is_some() {
            let client = self.client.take();
            // Note: We can't await in drop, so we spawn a task
            if let Some(client) = client {
                tokio::spawn(async move {
                    let _ = client.close().await;
                });
            }
        }

        // Clean up temporary profiles
        if let Some(ref mut temp_manager) = self.temp_profile_manager {
            if let Err(e) = temp_manager.cleanup() {
                warn!("Failed to cleanup temporary profiles in Drop: {}", e);
            }
        }

        // The driver manager will handle process cleanup
    }
}