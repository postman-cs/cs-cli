//! Guided authentication module for automating Okta SSO login
//!
//! This module provides browser automation for guided authentication through Okta SSO.
//! It uses Chrome DevTools Protocol (CDP) to automate the login process and access your own session cookies,
//! with support for multiple browsers (Chrome, Firefox, Safari).

use crate::common::auth::cookies::Cookie;
use crate::common::auth::auth_flow::guided_auth_config::{DynamicCookieDomains, GuidedAuthConfig};
use crate::common::auth::cookies::guided_cookie_storage::are_cookies_expired;
use crate::common::auth::cookies::hybrid_cookie_storage::{
    get_cookies_hybrid, has_cookies_hybrid, store_cookies_hybrid,
};
use crate::common::auth::browser::profile_detector::{BrowserProfile, BrowserType, PlatformType, ProfileDetector};
use crate::common::auth::browser::temp_browser_manager::TempBrowserManager;
use crate::common::auth::browser::cdp_browser_manager::CdpBrowserManager;
use crate::common::auth::browser::cdp_browser_manager::Browser;
use crate::common::error::{CsCliError, Result};
use std::collections::HashMap;
use tracing::{debug, info, warn};

// Helper macro to simplify error handling
macro_rules! auth_err {
    ($msg:expr) => {
        CsCliError::Authentication($msg.to_string())
    };
    ($fmt:expr, $($arg:tt)*) => {
        CsCliError::Authentication(format!($fmt, $($arg)*))
    };
}

/// Main guided authentication struct
pub struct GuidedAuth {
    cdp_manager: Option<CdpBrowserManager>,
    config: GuidedAuthConfig,
    cookie_domains: DynamicCookieDomains,
    #[allow(dead_code)]  // Will be used when headless mode is configured
    headless: bool,
    selected_profile: Option<BrowserProfile>,
    profile_detector: Option<ProfileDetector>,
    selected_browser: Browser,
    temp_browser_manager: Option<TempBrowserManager>,
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
            cdp_manager: None,
            config: GuidedAuthConfig::default(),
            cookie_domains: DynamicCookieDomains::new(),
            headless: true,
            selected_profile: None,
            profile_detector: None,
            selected_browser: Browser::Chrome, // Default to Chrome
            temp_browser_manager: None,
        }
    }

    /// Create new guided authentication instance with browser visible (for testing)
    pub fn new_headed() -> Self {
        Self {
            cdp_manager: None,
            config: GuidedAuthConfig::default(),
            cookie_domains: DynamicCookieDomains::new(),
            headless: false,
            selected_profile: None,
            profile_detector: None,
            selected_browser: Browser::Chrome,
            temp_browser_manager: None,
        }
    }

    /// Check if browser is initialized (for testing)
    pub fn has_browser(&self) -> bool {
        self.cdp_manager.is_some()
    }

    /// Create new guided authentication instance with specific browser profile
    pub fn with_profile(profile: BrowserProfile) -> Self {
        let browser_type = Self::get_guided_auth_browser(profile.browser_type);

        Self {
            cdp_manager: None,
            config: GuidedAuthConfig::default(),
            cookie_domains: DynamicCookieDomains::new(),
            headless: true,
            selected_profile: Some(profile),
            profile_detector: None,
            selected_browser: browser_type,
            temp_browser_manager: None,
        }
    }

    /// Create new guided authentication instance with platform-specific profile preference
    pub fn with_platform_preference(platform: PlatformType) -> Result<Self> {
        let detector = ProfileDetector::new()
            .map_err(|e| auth_err!("Failed to initialize profile detector: {}", e))?;

        let selected_profile = detector.find_best_profile_for_platform(&platform)
            .cloned();

        let browser_type = if let Some(ref profile) = selected_profile {
            info!("Selected profile for {:?}: {}", platform, profile.description());
            Self::get_guided_auth_browser(profile.browser_type)
        } else {
            warn!("No suitable profile found for platform {:?}, using default", platform);
            Browser::Chrome
        };

        Ok(Self {
            cdp_manager: None,
            config: GuidedAuthConfig::default(),
            cookie_domains: DynamicCookieDomains::new(),
            headless: true,
            selected_profile,
            profile_detector: Some(detector),
            selected_browser: browser_type,
            temp_browser_manager: None,
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
                .map_err(|e| auth_err!("Failed to initialize profile detector: {}", e))?);
        }

        Ok(&self.profile_detector.as_ref().unwrap().detected_profiles)
    }

    /// Set a specific profile to use
    pub fn set_profile(&mut self, profile: BrowserProfile) {
        info!("Setting profile: {}", profile.description());

        // Update browser type based on profile
        self.selected_browser = Self::get_guided_auth_browser(profile.browser_type);

        self.selected_profile = Some(profile);
    }

    /// Get the appropriate browser for guided authentication, with fallbacks for unsupported browsers
    fn get_guided_auth_browser(browser_type: BrowserType) -> Browser {
        match browser_type {
            BrowserType::Firefox | BrowserType::LibreWolf | BrowserType::Zen | BrowserType::Safari => {
                info!("{:?} detected but falling back to Chrome for guided authentication", browser_type);
                Browser::Chrome
            }
            _ => Browser::from(browser_type)
        }
    }

    /// Initialize browser with CDP
    async fn init_browser(&mut self) -> Result<()> {
        if self.cdp_manager.is_some() {
            return Ok(());
        }

        info!("Initializing browser with CDP: {:?}", self.selected_browser);

        // If we have a selected profile, try creating a temporary browser to avoid conflicts
        let profile_to_use = if let Some(ref original_profile) = self.selected_profile {
            // Initialize temp browser manager if not already done
            if self.temp_browser_manager.is_none() {
                info!("Initializing temporary browser manager to avoid browser conflicts");
                self.temp_browser_manager = Some(TempBrowserManager::new()?);
            }

            if let Some(ref mut temp_manager) = self.temp_browser_manager {
                match temp_manager.create_temp_browser(original_profile) {
                    Ok(temp_profile) => {
                        info!("Created temporary browser to avoid browser conflicts: {}", temp_profile.profile_path.display());

                        // Log what essential files were copied to temp profile
                        info!("Verifying essential authentication files in temp profile...");
                        let temp_path = &temp_profile.profile_path;
                        let essential_files = match original_profile.browser_type {
                            BrowserType::Chrome | BrowserType::Brave | BrowserType::Edge |
                            BrowserType::Arc | BrowserType::Chromium | BrowserType::Opera |
                            BrowserType::OperaGX | BrowserType::Vivaldi | BrowserType::Comet => {
                                vec!["Cookies", "Preferences", "Secure Preferences", "Local State", "IndexedDB"]
                            }
                            BrowserType::Firefox | BrowserType::LibreWolf | BrowserType::Zen => {
                                vec!["cookies.sqlite", "prefs.js", "key4.db", "storage"]
                            }
                            BrowserType::Safari => vec![], // Safari doesn't copy files
                        };

                        let mut found_files = 0;
                        for file in &essential_files {
                            let file_path = temp_path.join(file);
                            if file_path.exists() {
                                if let Ok(metadata) = std::fs::metadata(&file_path) {
                                    if file_path.is_dir() {
                                        info!("✓ Found directory {}: exists", file);
                                    } else {
                                        info!("✓ Found file {}: {} bytes", file, metadata.len());
                                    }
                                    found_files += 1;
                                } else {
                                    info!("✓ Found {}: (unknown size)", file);
                                    found_files += 1;
                                }
                            } else {
                                warn!("✗ Missing essential file: {}", file);
                            }
                        }

                        if found_files > 0 {
                            info!("Authentication profile ready: {}/{} essential files present", found_files, essential_files.len());
                        } else {
                            warn!("No essential authentication files found in temp profile!");
                        }

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

        // Launch browser with CDP
        let mut cdp_manager = CdpBrowserManager::new();
        cdp_manager.launch_browser(self.selected_browser, profile_to_use.as_ref()).await?;

        self.cdp_manager = Some(cdp_manager);
        Ok(())
    }

    /// Navigate to a URL
    async fn navigate_to_okta(&mut self, url: &str) -> Result<()> {
        self.init_browser().await?;

        if let Some(cdp_manager) = &mut self.cdp_manager {
            cdp_manager.navigate(url).await
                .map_err(|e| auth_err!("Failed to navigate to URL: {}", e))?;
        }

        Ok(())
    }


    /// Execute JavaScript
    async fn execute_js(&mut self, script: &str) -> Result<String> {
        if let Some(cdp_manager) = &mut self.cdp_manager {
            let result = cdp_manager.evaluate_js(script).await
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
    async fn element_exists(&mut self, selector: &str) -> bool {
        if let Some(cdp_manager) = &mut self.cdp_manager {
            let check_js = format!(
                "document.querySelector('{}') !== null", 
                selector.replace('\'', "\\'")
            );
            
            match cdp_manager.evaluate_js(&check_js).await {
                Ok(result) => {
                    result.get("result")
                        .and_then(|r| r.get("value"))
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
                }
                Err(_) => false,
            }
        } else {
            false
        }
    }

    /// Click an element
    async fn click_element(&mut self, selector: &str) -> Result<()> {
        if let Some(cdp_manager) = &mut self.cdp_manager {
            cdp_manager.click_element(selector).await
                .map_err(|e| CsCliError::Authentication(format!("Failed to click element {selector}: {e}")))?;
            Ok(())
        } else {
            Err(CsCliError::Authentication("Browser not initialized".to_string()))
        }
    }

    /// Retrieve cookies from browser session
    async fn retrieve_cookies_from_browser(&mut self) -> Result<HashMap<String, Vec<Cookie>>> {
        if let Some(cdp_manager) = &mut self.cdp_manager {
            let cookies = cdp_manager.get_cookies().await
                .map_err(|e| CsCliError::Authentication(format!("Failed to get cookies: {e}")))?;

            let mut result: HashMap<String, Vec<Cookie>> = HashMap::new();

            for cookie_value in cookies {
                if let Some(cookie_obj) = cookie_value.as_object() {
                    let domain = cookie_obj.get("domain")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    // Handle cookie expiration properly
                    let expires = cookie_obj.get("expires")
                        .and_then(|v| v.as_f64())
                        .map(|exp| exp as u64);

                    let c = Cookie {
                        name: cookie_obj.get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        value: cookie_obj.get("value")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        domain: domain.clone(),
                        path: cookie_obj.get("path")
                            .and_then(|v| v.as_str())
                            .unwrap_or("/")
                            .to_string(),
                        expires,
                        http_only: cookie_obj.get("httpOnly")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false),
                        secure: cookie_obj.get("secure")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false),
                    };

                    result.entry(domain).or_default().push(c);
                }
            }

            Ok(result)
        } else {
            Err(CsCliError::Authentication("Browser not initialized".to_string()))
        }
    }

    /// Check if already authenticated
    async fn check_already_authenticated(&mut self) -> Result<bool> {
        // ONLY check for the "My Apps" title - this is the ONLY reliable indicator of authentication
        match self.execute_js(r#"
            // Look for the specific "My Apps" title element
            const myAppsTitle = document.querySelector('h1[data-se="dashboard-my-apps-title"]');
            return myAppsTitle !== null;
        "#).await {
            Ok(result) => {
                if let Ok(is_auth) = serde_json::from_str::<bool>(&result) {
                    info!("Authentication check: My Apps title found = {}", is_auth);
                    return Ok(is_auth);
                }
            }
            Err(e) => {
                warn!("Failed to check authentication status: {}", e);
            }
        }

        // If JS execution fails, assume not authenticated
        Ok(false)
    }

    /// Check stored cookies
    async fn check_stored_cookies(&mut self) -> Result<HashMap<String, String>> {
        info!("Checking for stored cookies in gist/keychain storage...");
        
        // Try to get cookies from hybrid storage (gist first, then keychain)
        if has_cookies_hybrid().await {
            let cookies = get_cookies_hybrid().await?;
            info!("Retrieved {} stored cookies from hybrid storage", cookies.len());

            // Log stored cookie details only in debug mode
            debug!("Stored cookies found:");
            for (cookie_name, cookie_value) in &cookies {
                let value_preview = if cookie_value.len() > 30 {
                    format!("{}...", &cookie_value[..30])
                } else {
                    cookie_value.clone()
                };
                debug!("  - {}: {}", cookie_name, value_preview);
            }

            // Validate cookies aren't expired
            if !are_cookies_expired(&cookies) {
                info!("Stored cookies are valid and not expired");
                return Ok(cookies);
            } else {
                warn!("Stored cookies are expired, will need fresh authentication");
            }
        } else {
            info!("No stored cookies found in gist/keychain storage");
        }

        Err(CsCliError::Authentication("No valid stored cookies found".to_string()))
    }

    /// Check for valid browser cookies for each platform
    async fn check_browser_cookies(&mut self) -> Result<HashMap<String, String>> {
        info!("Opening browser to check for valid browser cookies...");

        // Initialize browser if not already done
        if self.cdp_manager.is_none() {
            info!("Initializing browser with temp profile for cookie check...");
            self.init_browser().await?;
        }

        // Get all cookies from browser without navigating anywhere
        info!("Retrieving all cookies from browser session...");
        let all_browser_cookies = self.retrieve_cookies_from_browser().await?;
        info!("Retrieved {} cookie domains from browser", all_browser_cookies.len());

        if all_browser_cookies.is_empty() {
            warn!("No cookies found in browser session - temp profile may be empty or browser failed to load cookies");
            return Err(CsCliError::Authentication("No cookies found in browser session".to_string()));
        }

        // Log summary of all domains found
        info!("Browser cookie domains found:");
        for (domain, cookies) in &all_browser_cookies {
            info!("  - {}: {} cookies", domain, cookies.len());
        }
        
        let mut all_valid_cookies = HashMap::new();
        let mut platforms_needing_auth = Vec::new();
        let mut cookie_log = Vec::new();

        // Define platform domains to check
        let platform_domains = vec![
            ("gong", vec!["gong.io", ".gong.io"]),
            ("slack", vec!["slack.com", ".slack.com"]),
            ("gainsight", vec!["gainsight.com", ".gainsight.com"]),
            ("salesforce", vec!["salesforce.com", ".salesforce.com"]),
        ];

        let platform_urls = self.config.platform_urls.clone();
        
        for (platform_name, domains) in platform_domains {
            info!("Checking browser cookies for platform: {}", platform_name);
            
            // Find cookies for this platform's domains
            let mut platform_cookies = HashMap::new();
            let mut found_cookies = 0;

            info!("Searching for {} cookies in {} browser domains...", platform_name, all_browser_cookies.len());

            for (domain, cookies) in &all_browser_cookies {
                // Check if this domain matches any of the platform's domains
                let matches_platform = domains.iter().any(|platform_domain| {
                    domain.contains(platform_domain)
                });

                if matches_platform {
                    info!("Found {} cookies for matching domain: {} (matches {})", cookies.len(), domain, platform_name);
                    for cookie in cookies {
                        debug!("Checking cookie: {} = {} (domain: {}, expires: {:?})",
                               cookie.name,
                               &cookie.value[..std::cmp::min(20, cookie.value.len())],
                               cookie.domain,
                               cookie.expires);
                        if self.is_cookie_valid(cookie) {
                            platform_cookies.insert(
                                format!("{}_{}", platform_name, cookie.name),
                                cookie.value.clone(),
                            );
                            found_cookies += 1;
                            debug!("✓ Valid cookie: {}", cookie.name);
                        } else {
                            debug!("✗ Expired/invalid cookie: {}", cookie.name);
                        }
                    }
                } else {
                    debug!("Domain {} doesn't match {} (looking for: {:?})", domain, platform_name, domains);
                }
            }
            
            if !platform_cookies.is_empty() {
                info!("Found {} valid cookies for {}", found_cookies, platform_name);
                
                // Log detailed cookie information
                let mut platform_log = format!("=== {} COOKIES ===\n", platform_name.to_uppercase());
                platform_log.push_str(&format!("Total cookies found: {}\n", platform_cookies.len()));
                platform_log.push_str("Cookie details:\n");
                
                for (cookie_name, cookie_value) in &platform_cookies {
                    // Truncate long cookie values for logging
                    let display_value = if cookie_value.len() > 50 {
                        format!("{}... ({} chars)", &cookie_value[..50], cookie_value.len())
                    } else {
                        cookie_value.clone()
                    };
                    platform_log.push_str(&format!("  - {}: {}\n", cookie_name, display_value));
                }
                platform_log.push_str("\n");
                
                cookie_log.push(platform_log);
                all_valid_cookies.extend(platform_cookies);
            } else {
                info!("No valid cookies found for {}, will need authentication", platform_name);
                if let Some(platform_url) = platform_urls.get(platform_name) {
                    platforms_needing_auth.push((platform_name.to_string(), platform_url.clone()));
                }
                cookie_log.push(format!("=== {} COOKIES ===\nNo valid cookies found\n\n", platform_name.to_uppercase()));
            }
        }

        // Write detailed cookie log to file only in debug mode
        if tracing::enabled!(tracing::Level::DEBUG) {
            self.write_cookie_log(&cookie_log).await?;
        }

        // If all platforms have valid cookies, we're done
        if platforms_needing_auth.is_empty() {
            info!("All platforms have valid browser cookies, no authentication needed");
            return Ok(all_valid_cookies);
        }

        // For platforms without valid cookies, authenticate to Okta and navigate to those platforms
        info!("{} platforms need authentication, proceeding with Okta authentication", platforms_needing_auth.len());
        let auth_cookies = self.authenticate_for_platforms(&platforms_needing_auth).await?;
        
        // Merge all cookies
        all_valid_cookies.extend(auth_cookies);
        
        Ok(all_valid_cookies)
    }


    /// Check if a cookie is valid (not expired)
    fn is_cookie_valid(&self, cookie: &Cookie) -> bool {
        if let Some(expires) = cookie.expires {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            // Cookie is valid if it expires more than 1 hour from now
            expires > now + 3600
        } else {
            // Session cookies are considered valid
            true
        }
    }


    /// Write detailed cookie log to file using common utilities
    async fn write_cookie_log(&self, cookie_log: &[String]) -> Result<()> {
        // Create log file path using common utilities
        let log_file_path = crate::common::logging::init_cookie_validation_logging()
            .map_err(|e| CsCliError::Authentication(format!("Failed to create cookie log file: {}", e)))?;
        
        // Write cookie log using common utilities
        crate::common::logging::write_cookie_validation_log(&log_file_path, cookie_log)
            .map_err(|e| CsCliError::Authentication(format!("Failed to write cookie log: {}", e)))?;
        
        info!("Cookie validation log written to: {}", log_file_path.display());
        Ok(())
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

        // No valid stored cookies found, need to check browser cookies
        info!("No valid stored cookies found, checking browser cookies...");
        let browser_cookies = self.check_browser_cookies().await?;
        
        // Store the browser cookies for future use
        store_cookies_hybrid(&browser_cookies).await?;
        
        Ok(browser_cookies)
    }

    /// Authenticate to Okta and navigate to specific platforms that need authentication
    async fn authenticate_for_platforms(&mut self, platforms_needing_auth: &[(String, String)]) -> Result<HashMap<String, String>> {
        info!("Authenticating to Okta for {} platforms", platforms_needing_auth.len());

        // Navigate to Okta OAuth authorization endpoint
        let okta_url = self.config.okta_oauth_url.clone();
        self.navigate_to_okta(&okta_url).await?;

        // Check if we're already logged in
        if self.check_already_authenticated().await? {
            info!("Already authenticated to Okta, collecting cookies...");
        } else {
            // Wait for authentication options to load
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

        // Now navigate to each platform that needs authentication
        let mut all_cookies = HashMap::new();
        
        for (platform_name, platform_url) in platforms_needing_auth {
            info!("Navigating to {} to collect cookies", platform_name);
            
            if let Some(cdp_manager) = &mut self.cdp_manager {
                // Navigate to platform
                cdp_manager.goto(platform_url).await
                    .map_err(|e| CsCliError::Authentication(format!("Failed to navigate to {platform_name}: {e}")))?;

                // Wait for page load
                tokio::time::sleep(std::time::Duration::from_secs(
                    self.config.timing.platform_navigation_delay,
                ))
                .await;

                // Extract cookies from browser
                let browser_cookies = self.retrieve_cookies_from_browser().await?;

                // Store cookies for this platform
                for (_domain, cookies) in browser_cookies {
                    for cookie in cookies {
                        all_cookies.insert(
                            format!("{}_{}", platform_name, cookie.name),
                            cookie.value,
                        );
                    }
                }
                
                info!("Collected cookies for {}", platform_name);
            }
        }

        Ok(all_cookies)
    }

    /// Perform fresh authentication flow
    async fn perform_fresh_authentication(&mut self) -> Result<HashMap<String, String>> {
        info!("Starting authentication flow...");

        // Navigate to Okta OAuth authorization endpoint
        let okta_url = self.config.okta_oauth_url.clone();
        self.navigate_to_okta(&okta_url).await?;

        // Check if we're already logged in
        if self.check_already_authenticated().await? {
            info!("Already authenticated, collecting cookies...");
        } else {
            // Wait for authentication options to load
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
        let platform_urls = self.config.platform_urls.clone();

        for (platform_name, platform_url) in &platform_urls {
            debug!("Collecting cookies from {}", platform_name);

            if let Some(cdp_manager) = &mut self.cdp_manager {
                // Navigate to platform
                cdp_manager.goto(platform_url).await
                    .map_err(|e| CsCliError::Authentication(format!("Failed to navigate to {platform_name}: {e}")))?;

                // Wait for page load
                tokio::time::sleep(std::time::Duration::from_secs(
                    self.config.timing.platform_navigation_delay,
                ))
                .await;

                // Access platform-specific cookies from your authenticated session
                let browser_cookies = self.retrieve_cookies_from_browser().await?;

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
    async fn try_click_okta_verify(&mut self) -> bool {
        let okta_verify_primary = self.config.selectors.okta_verify_primary.clone();
        let okta_verify_fallback = self.config.selectors.okta_verify_fallback.clone();
        let okta_verify_selectors = vec![
            &okta_verify_primary,
            &okta_verify_fallback,
            "a[data-se='okta_verify-push']",
            "button[data-se='okta_verify-push']",
            "[data-se-for-name='okta_verify']",
            // Additional common Okta Verify selectors
            "a[href*='okta_verify']",
            "button[href*='okta_verify']",
            "[data-se='okta_verify-push']",
            "a[aria-label*='Okta Verify']",
            "button[aria-label*='Okta Verify']",
            "a[title*='Okta Verify']",
            "button[title*='Okta Verify']",
            "a:contains('Okta Verify')",
            "button:contains('Okta Verify')",
            "a:contains('Push')",
            "button:contains('Push')",
            // Generic push notification selectors
            "[data-se='push']",
            "[data-se='factor-push']",
            "a[data-se='factor-push']",
            "button[data-se='factor-push']",
        ];

        info!("Searching for Okta Verify button with {} selectors", okta_verify_selectors.len());
        
        for selector in &okta_verify_selectors {
            info!("Checking selector: {}", selector);
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
            } else {
                debug!("Selector '{}' not found on page", selector);
            }
        }

        // If no Okta Verify button found, log the page content for debugging
        warn!("No Okta Verify button found with any selector. Logging page content for debugging...");
        
        // First, try to find any authentication-related buttons
        self.log_available_auth_buttons().await;
        
        match self.get_page_content().await {
            Ok(content) => {
                // Log just the relevant parts of the page (auth options)
                if let Some(auth_section) = self.extract_auth_section(&content) {
                    info!("Authentication section HTML:\n{}", auth_section);
                } else {
                    debug!("Full page content (first 2000 chars):\n{}", 
                           &content.chars().take(2000).collect::<String>());
                }
            }
            Err(e) => {
                warn!("Failed to get page content for debugging: {}", e);
            }
        }

        false
    }

    /// Get page HTML content
    pub async fn get_page_content(&mut self) -> Result<String> {
        self.execute_js("document.documentElement.outerHTML").await
    }

    /// Log all available authentication buttons on the page
    async fn log_available_auth_buttons(&mut self) {
        let auth_button_selectors = vec![
            "a[href]",
            "button",
            "[role='button']",
            "[data-se]",
            "[aria-label]",
            "[title]",
        ];

        info!("Scanning page for authentication buttons...");
        
        for selector in &auth_button_selectors {
            match self.execute_js(&format!(
                r#"
                const elements = document.querySelectorAll('{}');
                const authButtons = [];
                for (let el of elements) {{
                    const text = el.textContent?.trim() || '';
                    const href = el.href || '';
                    const ariaLabel = el.getAttribute('aria-label') || '';
                    const title = el.getAttribute('title') || '';
                    const dataSe = el.getAttribute('data-se') || '';
                    const className = el.className || '';
                    
                    // Look for authentication-related content
                    if (text.toLowerCase().includes('verify') || 
                        text.toLowerCase().includes('push') || 
                        text.toLowerCase().includes('okta') ||
                        text.toLowerCase().includes('authenticate') ||
                        ariaLabel.toLowerCase().includes('verify') ||
                        ariaLabel.toLowerCase().includes('push') ||
                        ariaLabel.toLowerCase().includes('okta') ||
                        title.toLowerCase().includes('verify') ||
                        title.toLowerCase().includes('push') ||
                        title.toLowerCase().includes('okta') ||
                        dataSe.includes('verify') ||
                        dataSe.includes('push') ||
                        dataSe.includes('okta') ||
                        className.toLowerCase().includes('verify') ||
                        className.toLowerCase().includes('push') ||
                        className.toLowerCase().includes('okta')) {{
                        
                        authButtons.push({{
                            tagName: el.tagName,
                            text: text,
                            href: href,
                            ariaLabel: ariaLabel,
                            title: title,
                            dataSe: dataSe,
                            className: className,
                            outerHTML: el.outerHTML.substring(0, 200)
                        }});
                    }}
                }}
                JSON.stringify(authButtons, null, 2);
                "#,
                selector
            )).await {
                Ok(result) => {
                    if let Ok(buttons) = serde_json::from_str::<serde_json::Value>(&result) {
                        if let Some(buttons_array) = buttons.as_array() {
                            if !buttons_array.is_empty() {
                                info!("Found {} authentication-related elements with selector '{}':", 
                                      buttons_array.len(), selector);
                                for button in buttons_array {
                                    info!("  - {}", button);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    debug!("Failed to scan for auth buttons with selector '{}': {}", selector, e);
                }
            }
        }
    }

    /// Extract authentication section from page HTML for debugging
    fn extract_auth_section(&self, html: &str) -> Option<String> {
        // Look for common authentication container patterns
        let auth_patterns = vec![
            r#"<div[^>]*class="[^"]*auth[^"]*"[^>]*>.*?</div>"#,
            r#"<div[^>]*class="[^"]*factor[^"]*"[^>]*>.*?</div>"#,
            r#"<div[^>]*class="[^"]*verify[^"]*"[^>]*>.*?</div>"#,
            r#"<div[^>]*class="[^"]*push[^"]*"[^>]*>.*?</div>"#,
            r#"<div[^>]*class="[^"]*okta[^"]*"[^>]*>.*?</div>"#,
        ];

        for pattern in auth_patterns {
            if let Ok(regex) = regex::Regex::new(pattern) {
                if let Some(mat) = regex.find(html) {
                    return Some(mat.as_str().to_string());
                }
            }
        }

        // If no specific auth section found, look for any divs with authentication-related classes
        if let Ok(regex) = regex::Regex::new(r#"<div[^>]*class="[^"]*(?:auth|factor|verify|push|okta)[^"]*"[^>]*>.*?</div>"#) {
            let mut matches = Vec::new();
            for mat in regex.find_iter(html) {
                matches.push(mat.as_str().to_string());
            }
            if !matches.is_empty() {
                return Some(matches.join("\n\n"));
            }
        }

        None
    }

    /// Close the browser
    pub async fn close(&mut self) -> Result<()> {
        if let Some(mut cdp_manager) = self.cdp_manager.take() {
            cdp_manager.close().await
                .map_err(|e| CsCliError::Authentication(format!("Failed to close browser: {e}")))?;
        }

        // Clean up temporary browsers
        if let Some(ref mut temp_manager) = self.temp_browser_manager {
            if let Err(e) = temp_manager.cleanup() {
                warn!("Failed to cleanup temporary browsers: {}", e);
            } else {
                info!("Successfully cleaned up temporary browsers");
            }
        }

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
        if self.cdp_manager.is_some() {
            let cdp_manager = self.cdp_manager.take();
            // Note: We can't await in drop, so we spawn a task
            if let Some(mut cdp_manager) = cdp_manager {
                tokio::spawn(async move {
                    let _ = cdp_manager.close().await;
                });
            }
        }

        // Clean up temporary browsers
        if let Some(ref mut temp_manager) = self.temp_browser_manager {
            if let Err(e) = temp_manager.cleanup() {
                warn!("Failed to cleanup temporary browsers in Drop: {}", e);
            }
        }
    }
}