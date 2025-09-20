//! Guided authentication module for automating Okta SSO login
//!
//! This module provides browser automation for guided authentication through Okta SSO.
//! It uses headless Chrome to automate the login process and extract session cookies.

use crate::common::auth::guided_auth_config::{DynamicCookieDomains, GuidedAuthConfig};
use crate::common::auth::guided_cookie_storage::{are_cookies_expired, validate_cookies};
use crate::common::auth::hybrid_cookie_storage::{
    get_cookies_hybrid, has_cookies_hybrid, store_cookies_hybrid,
};
use crate::common::auth::profile_detector::{BrowserProfile, BrowserType, PlatformType, ProfileDetector};
use anyhow::{anyhow, Context, Result};
use headless_chrome::{protocol::cdp::Network::Cookie, Browser, LaunchOptions, Tab};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tracing::{debug, info, warn};

    /// Main guided authentication struct
    pub struct GuidedAuth {
        browser: Option<Browser>,
        active_tab: Option<Arc<Tab>>,
        config: GuidedAuthConfig,
        cookie_domains: DynamicCookieDomains,
        headless: bool,
        selected_profile: Option<BrowserProfile>,
        profile_detector: Option<ProfileDetector>,
    }

impl GuidedAuth {
    /// Create new guided authentication instance
    pub fn new() -> Self {
        Self {
            browser: None,
            active_tab: None,
            config: GuidedAuthConfig::default(),
            cookie_domains: DynamicCookieDomains::new(),
            headless: true,
            selected_profile: None,
            profile_detector: None,
        }
    }

    /// Create new guided authentication instance with browser visible (for testing)
    pub fn new_headed() -> Self {
        Self {
            browser: None,
            active_tab: None,
            config: GuidedAuthConfig::default(),
            cookie_domains: DynamicCookieDomains::new(),
            headless: false,
            selected_profile: None,
            profile_detector: None,
        }
    }

    /// Check if browser is initialized (for testing)
    pub fn has_browser(&self) -> bool {
        self.browser.is_some()
    }

    /// Create new guided authentication instance with specific browser profile
    pub fn with_profile(profile: BrowserProfile) -> Self {
        Self {
            browser: None,
            active_tab: None,
            config: GuidedAuthConfig::default(),
            cookie_domains: DynamicCookieDomains::new(),
            headless: true,
            selected_profile: Some(profile),
            profile_detector: None,
        }
    }

    /// Create new guided authentication instance with platform-specific profile preference
    pub fn with_platform_preference(platform: PlatformType) -> Result<Self> {
        let detector = ProfileDetector::new()
            .context("Failed to initialize profile detector")?;
            
        let selected_profile = detector.find_best_profile_for_platform(&platform)
            .map(|p| p.clone());
            
        if let Some(ref profile) = selected_profile {
            info!("Selected profile for {:?}: {}", platform, profile.description());
        } else {
            warn!("No suitable profile found for platform {:?}, using default", platform);
        }
        
        Ok(Self {
            browser: None,
            active_tab: None,
            config: GuidedAuthConfig::default(),
            cookie_domains: DynamicCookieDomains::new(),
            headless: true,
            selected_profile,
            profile_detector: Some(detector),
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
                .context("Failed to initialize profile detector")?);
        }
        
        Ok(&self.profile_detector.as_ref().unwrap().detected_profiles)
    }

    /// Set a specific profile to use
    pub fn set_profile(&mut self, profile: BrowserProfile) {
        info!("Setting profile: {}", profile.description());
        self.selected_profile = Some(profile);
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
            tokio::time::sleep(std::time::Duration::from_secs(
                self.config.timing.auth_options_load_wait,
            ))
            .await;
            
            // Log what's actually on the page after waiting
            let current_url = self.get_current_url().await?;
            info!("URL after waiting for auth options: {}", current_url);
            
            // Get the full page HTML to see what's actually there
            let full_page_html = self.execute_js("document.documentElement.outerHTML").await.unwrap_or_else(|_| "Failed to get HTML".to_string());
            info!("Full page HTML length: {} characters", full_page_html.len());
            
            // Look for any elements that might be auth-related
            let auth_search_script = r#"
                try {
                    const searchTerms = ['okta', 'verify', 'factor', 'auth', 'login', 'signin'];
                    const allElements = document.querySelectorAll('*');
                    const matches = [];
                    
                    for (let el of allElements) {
                        const text = el.textContent?.toLowerCase() || '';
                        const className = el.className?.toLowerCase() || '';
                        const id = el.id?.toLowerCase() || '';
                        
                        for (let term of searchTerms) {
                            if (text.includes(term) || className.includes(term) || id.includes(term)) {
                                matches.push({
                                    tag: el.tagName,
                                    text: text.substring(0, 100),
                                    className: className,
                                    id: id
                                });
                                break;
                            }
                        }
                        
                        if (matches.length >= 20) break; // Limit results
                    }
                    
                    return JSON.stringify(matches);
                } catch(e) {
                    return 'Error: ' + e.message;
                }
            "#;
            
            match self.execute_js(auth_search_script).await {
                Ok(search_results) => {
                    if search_results == "null" || search_results == "None" {
                        warn!("No auth-related elements found on page");
                    } else {
                        info!("Auth-related elements found: {}", search_results);
                    }
                }
                Err(e) => {
                    warn!("Failed to search for auth elements: {}", e);
                }
            }

            // Check if we're already in the Okta Verify flow (waiting for TouchID)
            let auth_in_progress = self.element_exists("a[data-se='cancel'].link.js-cancel").await;
            if auth_in_progress {
                info!("Already in Okta Verify flow - 'Back to sign in' link detected. Waiting for TouchID...");
            } else {
                // Click on Okta Verify authentication option
                info!("Selecting Okta Verify authentication...");
                match self.click_okta_verify_option().await {
                    Ok(()) => info!("Successfully selected Okta Verify"),
                    Err(e) => {
                    warn!(
                        "Failed to click Okta Verify, trying Okta FastPass button: {}",
                        e
                    );
                    // Try Okta FastPass button (this will take us to the page with the primary selector)
                    let fastpass_selector = self.config.selectors.okta_verify_fallback.clone();
                    info!("Attempting Okta FastPass selector: {}", fastpass_selector);
                    
                    // Check if FastPass element exists
                    let fastpass_exists = self.element_exists(&fastpass_selector).await;
                    info!("FastPass selector '{}' element exists: {}", fastpass_selector, fastpass_exists);
                    
                    if !fastpass_exists {
                        // Log available buttons for debugging
                        let button_search_script = r#"
                            try {
                                const buttons = Array.from(document.querySelectorAll('a, button')).map(el => ({
                                    tag: el.tagName,
                                    text: el.textContent?.trim().substring(0, 100) || '',
                                    className: el.className || '',
                                    href: el.getAttribute('href') || '',
                                    ariaLabel: el.getAttribute('aria-label') || ''
                                }));
                                return JSON.stringify(buttons.slice(0, 10));
                            } catch(e) {
                                return 'Error: ' + e.message;
                            }
                        "#;
                        
                        match self.execute_js(button_search_script).await {
                            Ok(buttons_json) => {
                                if buttons_json == "null" || buttons_json == "None" {
                                    warn!("JavaScript returned null/None for buttons");
                                } else {
                                    info!("Available buttons on page: {}", buttons_json);
                                }
                            }
                            Err(e) => {
                                warn!("Failed to get buttons: {}", e);
                            }
                        }
                        
                        // Try a more aggressive fallback - look for any clickable auth elements
                        info!("Attempting fallback authentication element detection...");
                        let fallback_search = r#"
                            try {
                                // Look for any element that might be an auth option
                                const allClickable = document.querySelectorAll('a, button, [role="button"], [onclick]');
                                const authOptions = [];
                                
                                for (let el of allClickable) {
                                    const text = el.textContent?.toLowerCase() || '';
                                    const ariaLabel = el.getAttribute('aria-label')?.toLowerCase() || '';
                                    const className = el.className?.toLowerCase() || '';
                                    
                                    // Look for any auth-related keywords
                                    const authKeywords = ['okta', 'verify', 'push', 'fastpass', 'factor', 'authenticate', 'login', 'signin'];
                                    const hasAuthKeyword = authKeywords.some(keyword => 
                                        text.includes(keyword) || ariaLabel.includes(keyword) || className.includes(keyword)
                                    );
                                    
                                    if (hasAuthKeyword && el.offsetParent !== null) { // Element is visible
                                        authOptions.push({
                                            tag: el.tagName,
                                            text: text.substring(0, 100),
                                            ariaLabel: ariaLabel,
                                            className: className,
                                            href: el.getAttribute('href') || '',
                                            onclick: el.getAttribute('onclick') || ''
                                        });
                                    }
                                }
                                
                                return JSON.stringify(authOptions);
                            } catch(e) {
                                return 'Error: ' + e.message;
                            }
                        "#;
                        
                        match self.execute_js(fallback_search).await {
                            Ok(fallback_results) => {
                                if fallback_results == "null" || fallback_results == "None" || fallback_results == "[]" {
                                    warn!("No fallback authentication elements found");
                                    return Err(anyhow!("Neither Okta Verify nor FastPass buttons found"));
                                } else {
                                    info!("Fallback authentication elements found: {}", fallback_results);
                                    
                                    // Try to click the first available auth option
                                    let click_script = r#"
                                        try {
                                            const allClickable = document.querySelectorAll('a, button, [role="button"], [onclick]');
                                            for (let el of allClickable) {
                                                const text = el.textContent?.toLowerCase() || '';
                                                const ariaLabel = el.getAttribute('aria-label')?.toLowerCase() || '';
                                                const className = el.className?.toLowerCase() || '';
                                                
                                                const authKeywords = ['okta', 'verify', 'push', 'fastpass', 'factor', 'authenticate', 'login', 'signin'];
                                                const hasAuthKeyword = authKeywords.some(keyword => 
                                                    text.includes(keyword) || ariaLabel.includes(keyword) || className.includes(keyword)
                                                );
                                                
                                                if (hasAuthKeyword && el.offsetParent !== null) {
                                                    el.click();
                                                    return 'Clicked: ' + el.tagName + ' with text: ' + text.substring(0, 50);
                                                }
                                            }
                                            return 'No clickable auth element found';
                                        } catch(e) {
                                            return 'Error: ' + e.message;
                                        }
                                    "#;
                                    
                                    match self.execute_js(click_script).await {
                                        Ok(click_result) => {
                                            info!("Fallback click result: {}", click_result);
                                            // Wait a moment for any navigation
                                            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                                        }
                                        Err(e) => {
                                            warn!("Failed to execute fallback click: {}", e);
                                            return Err(anyhow!("Neither Okta Verify nor FastPass buttons found"));
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                warn!("Failed to execute fallback search: {}", e);
                                return Err(anyhow!("Neither Okta Verify nor FastPass buttons found"));
                            }
                        }
                    }
                    
                    // Click FastPass button
                    self.click_element(&fastpass_selector).await?;
                    info!("Successfully clicked Okta FastPass button");
                    
                    // Wait for page to load and then try primary selector again
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    
                    // Now try the primary selector on the new page
                    info!("Trying primary Okta Verify selector on new page...");
                    self.click_element(&self.config.selectors.okta_verify_primary).await?;
                    info!("Successfully clicked Okta Verify on redirected page");
                    }
                }
            }

            // Wait for Okta Verify to process and detect authentication success
            info!("Waiting for Okta Verify authentication...");
            self.wait_for_authentication_success().await?;
        }

        // Navigate to platform apps and collect cookies
        let all_cookies = self.navigate_to_platforms().await?;

        // Store the fresh cookies for future use with hybrid storage (gist + local)
        let cookies_to_store = all_cookies.clone();
        match store_cookies_hybrid(&cookies_to_store).await {
            Ok(()) => {
                info!(
                    "Successfully stored {} cookies for future use",
                    all_cookies.len()
                );
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
        info!("Checking for stored session data...");

        // Check if we have stored cookies using hybrid storage
        let has_cookies = has_cookies_hybrid().await;

        if !has_cookies {
            debug!("No stored session data found");
            return Err(anyhow!("No stored session data found"));
        }

        // Retrieve stored cookies using hybrid storage
        let stored_cookies = get_cookies_hybrid()
            .await
            .map_err(|e| anyhow!("Failed to retrieve stored session data: {}", e))?;

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
                warn!(
                    "Cookie validation failed: {}, will perform fresh authentication",
                    e
                );
                Err(anyhow!("Cookie validation failed: {}", e))
            }
        }
    }

    /// Check if we're already authenticated by detecting dashboard page or authentication in progress
    async fn check_already_authenticated(&self) -> Result<bool> {
        info!("Checking if already authenticated...");

        // Check for dashboard-specific elements that indicate we're already logged in
        let current_url = self.get_current_url().await?;
        info!("Current URL after navigation: {}", current_url);
        
        // Primary method: Check for "My Apps" dashboard title element
        let my_apps_check = self.element_exists("h1.dashboard--my-apps-title").await;
        if my_apps_check {
            info!("Dashboard detected via 'My Apps' title element");
            return Ok(true);
        }

        // Check if authentication is already in progress (Okta Verify launching, TouchID waiting, etc.)
        let auth_in_progress_check = self.element_exists("a[data-se='cancel'].link.js-cancel").await;
        if auth_in_progress_check {
            info!("Authentication already in progress - 'Back to sign in' link found");
            return Ok(true);
        }

        // Fallback method: Check for window._oktaEnduser object
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

        // Fallback method: Check URL patterns
        if current_url.contains("enduser/dashboard")
            || current_url.contains("app/UserHome")
            || (current_url.contains("postman.okta.com") && current_url.ends_with("/"))
        {
            info!("Dashboard detected via URL pattern: {}", current_url);
            return Ok(true);
        }

        info!("Not authenticated - need to proceed with login flow");
        Ok(false)
    }

    /// Click on Okta Verify authentication option
    async fn click_okta_verify_option(&self) -> Result<()> {
        // Try to click using configured primary selector
        let primary_selector = &self.config.selectors.okta_verify_primary;
        info!("Attempting to click Okta Verify with primary selector: {}", primary_selector);
        
        // Log current page state before attempting click
        let current_url = self.get_current_url().await?;
        info!("Current URL before clicking Okta Verify: {}", current_url);
        
        // Check if the element exists before trying to click
        let element_exists = self.element_exists(primary_selector).await;
        info!("Primary selector '{}' element exists: {}", primary_selector, element_exists);
        
        if !element_exists {
            // Log page content for debugging
            let page_title = self.execute_js("document.title").await.unwrap_or_else(|_| "Unknown".to_string());
            info!("Page title: {}", page_title);
            
            // Try a simpler approach to get basic page info
            let simple_page_info = self.execute_js("document.body.innerHTML.substring(0, 1000)").await.unwrap_or_else(|_| "Failed to get page content".to_string());
            info!("Page content preview: {}", simple_page_info);
            
            // Log available authentication options
            let auth_options_script = r#"
                try {
                    const authElements = document.querySelectorAll('a[class*="select-factor"], button[class*="select-factor"], .select-factor');
                    const options = Array.from(authElements).map(el => ({
                        text: el.textContent?.trim() || '',
                        className: el.className || '',
                        ariaLabel: el.getAttribute('aria-label') || '',
                        href: el.getAttribute('href') || ''
                    }));
                    return JSON.stringify(options);
                } catch(e) {
                    return 'Error: ' + e.message;
                }
            "#;
            
            match self.execute_js(auth_options_script).await {
                Ok(options_json) => {
                    if options_json == "null" || options_json == "None" {
                        warn!("JavaScript returned null/None for authentication options");
                    } else {
                        info!("Available authentication options: {}", options_json);
                    }
                }
                Err(e) => {
                    warn!("Failed to get authentication options: {}", e);
                }
            }
            
            // Log all clickable elements that might be auth options
            let clickable_elements_script = r#"
                try {
                    const clickableElements = document.querySelectorAll('a, button, [role="button"], [onclick]');
                    const elements = Array.from(clickableElements).slice(0, 10).map(el => ({
                        tagName: el.tagName,
                        text: el.textContent?.trim().substring(0, 50) || '',
                        className: el.className || '',
                        ariaLabel: el.getAttribute('aria-label') || ''
                    }));
                    return JSON.stringify(elements);
                } catch(e) {
                    return 'Error: ' + e.message;
                }
            "#;
            
            match self.execute_js(clickable_elements_script).await {
                Ok(elements_json) => {
                    if elements_json == "null" || elements_json == "None" {
                        warn!("JavaScript returned null/None for clickable elements");
                    } else {
                        info!("First 10 clickable elements on page: {}", elements_json);
                    }
                }
                Err(e) => {
                    warn!("Failed to get clickable elements: {}", e);
                }
            }
        }
        
        self.click_element(primary_selector).await
    }

    /// Wait for authentication success by monitoring URL changes and dashboard elements
    async fn wait_for_authentication_success(&self) -> Result<()> {
        info!("Waiting for authentication to complete...");

        let max_attempts = self.config.timing.auth_success_timeout;
        let mut attempts = 0;

        while attempts < max_attempts {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;

            let current_url = self.get_current_url().await?;
            info!("Authentication check attempt {}/{} - Current URL: {}", attempts + 1, max_attempts, current_url);

            // Primary check: Look for "My Apps" dashboard element
            let my_apps_exists = self.element_exists("h1.dashboard--my-apps-title").await;
            if my_apps_exists {
                info!("Authentication successful - 'My Apps' dashboard element found!");
                return Ok(());
            }

            // Check if authentication is in progress (Okta Verify launching, TouchID waiting, etc.)
            let auth_in_progress = self.element_exists("a[data-se='cancel'].link.js-cancel").await;
            if auth_in_progress {
                info!("Authentication in progress - 'Back to sign in' link found, continuing to wait...");
                // Don't return yet, keep waiting for completion
            }

            // Fallback checks: URL patterns and other indicators
            let is_success_url = self.config.is_success_url(&current_url);
            let is_authenticated = self.check_already_authenticated().await?;
            
            info!("URL success check: {}, Authentication check: {}", is_success_url, is_authenticated);

            if is_success_url || is_authenticated {
                info!("Authentication successful, redirected to: {}", current_url);
                return Ok(());
            }

            // Log page state every 5 attempts for debugging
            if attempts % 5 == 0 && attempts > 0 {
                let page_state_script = r#"
                    const state = {
                        url: window.location.href,
                        title: document.title,
                        hasOktaEnduser: typeof window._oktaEnduser !== 'undefined',
                        bodyClasses: document.body.className,
                        myAppsElement: document.querySelector('h1.dashboard--my-apps-title') ? 'found' : 'not found',
                        errorElements: Array.from(document.querySelectorAll('[class*="error"], [class*="alert"]')).map(el => el.textContent?.trim()).filter(Boolean)
                    };
                    return JSON.stringify(state, null, 2);
                "#;
                
                match self.execute_js(page_state_script).await {
                    Ok(state_json) => {
                        info!("Page state at attempt {}: {}", attempts, state_json);
                    }
                    Err(e) => {
                        debug!("Failed to get page state: {}", e);
                    }
                }
            }

            attempts += 1;
        }

        let current_url = self.get_current_url().await?;
        warn!(
            "Authentication timeout after {}s, current URL: {}",
            max_attempts, current_url
        );

        // Log final page state for debugging
        let final_state_script = r#"
            const finalState = {
                url: window.location.href,
                title: document.title,
                bodyClasses: document.body.className,
                allLinks: Array.from(document.querySelectorAll('a')).map(a => ({
                    text: a.textContent?.trim().substring(0, 50) || '',
                    href: a.getAttribute('href') || ''
                })).slice(0, 5),
                allButtons: Array.from(document.querySelectorAll('button')).map(b => ({
                    text: b.textContent?.trim().substring(0, 50) || '',
                    className: b.className || ''
                })).slice(0, 5)
            };
            return JSON.stringify(finalState, null, 2);
        "#;
        
        match self.execute_js(final_state_script).await {
            Ok(final_state_json) => {
                warn!("Final page state after timeout: {}", final_state_json);
            }
            Err(e) => {
                warn!("Failed to get final page state: {}", e);
            }
        }

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

            match self
                .navigate_to_platform(&platform_name, &platform_url)
                .await
            {
                Ok(platform_cookies) => {
                    info!(
                        "Successfully collected {} cookies from {}",
                        platform_cookies.len(),
                        platform_name
                    );
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

        info!(
            "Total cookies collected from all platforms: {}",
            all_cookies.len()
        );
        Ok(all_cookies)
    }

    /// Navigate to a specific platform and extract cookies
    async fn navigate_to_platform(
        &mut self,
        platform_name: &str,
        platform_url: &str,
    ) -> Result<HashMap<String, String>> {
        info!("Loading {} app from Okta: {}", platform_name, platform_url);

        // Navigate to the platform URL
        self.navigate_to_url(platform_url).await?;

        // Wait for SSO redirection to complete to the actual platform domain
        info!("Waiting for {} SSO login to complete...", platform_name);
        let expected_domains = self.get_expected_platform_domain(platform_name);

        // Log initial URL after clicking Okta app tile
        let initial_url = self.get_current_url().await?;
        info!("{} initial URL after Okta navigation: {}", platform_name, initial_url);

        // Wait for redirect to platform domain (not just a fixed time)
        let max_wait = self.config.timing.sso_redirect_wait;
        let mut waited = 0;
        let mut reached_platform = false;

        while waited < max_wait {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            waited += 1;

            let current_url = self.get_current_url().await?;
            info!("{} checking URL (attempt {}): {}", platform_name, waited, current_url);

            // Check if we've reached the actual platform domain
            for domain in &expected_domains {
                if current_url.contains(domain) {
                    info!("✓ {} successfully redirected to platform domain: {}", platform_name, current_url);
                    reached_platform = true;
                    break;
                }
            }

            if reached_platform {
                break;
            }
        }

        let final_url = self.get_current_url().await?;

        if !reached_platform {
            warn!("✗ {} SSO redirect failed or stayed on Okta. Final URL: {}", platform_name, final_url);
            warn!("Expected one of these domains: {:?}", expected_domains);

            // Log all available cookie domains for debugging
            let (_, browser_cookies) = self.extract_cookies_with_details().await?;
            let found_domains: HashSet<String> = browser_cookies
                .iter()
                .map(|c| c.domain.clone())
                .collect();
            warn!("Available cookie domains at this URL: {:?}", found_domains);
        }

        // Extract cookies with full details for domain discovery
        let (_, browser_cookies) = self.extract_cookies_with_details().await?;

        // Filter cookies relevant to this platform using proper domain filtering
        let platform_cookies = self.filter_platform_cookies(platform_name, &browser_cookies);

        // Validate we got actual platform cookies
        if platform_cookies.is_empty() {
            warn!("No valid cookies collected for {}. SSO may have failed.", platform_name);
        } else {
            info!("Collected {} authentication cookies for {}", platform_cookies.len(), platform_name);
        }

        Ok(platform_cookies)
    }

    /// Get expected platform domains for validation
    fn get_expected_platform_domain(&self, platform_name: &str) -> Vec<String> {
        match platform_name {
            "gong" => vec!["gong.io".to_string(), "postman-success.gong.io".to_string()],
            "slack" => vec!["slack.com".to_string(), "postman-eng.slack.com".to_string()],
            "gainsight" => vec!["gainsight.com".to_string(), "postman.gainsight.com".to_string()],
            "salesforce" => vec!["salesforce.com".to_string(), "postman.my.salesforce.com".to_string()],
            _ => {
                warn!("Unknown platform: {}", platform_name);
                vec![]
            }
        }
    }

    /// Filter cookies relevant to the specific platform using proper domain filtering
    fn filter_platform_cookies(
        &mut self,
        platform_name: &str,
        browser_cookies: &[Cookie],
    ) -> HashMap<String, String> {
        let expected_domains = self.get_expected_platform_domain(platform_name);
        let mut platform_cookies = HashMap::new();

        // Only include cookies from the actual platform domains
        for cookie in browser_cookies {
            // Check if this cookie belongs to one of the platform's expected domains
            let is_platform_cookie = expected_domains.iter().any(|domain| {
                cookie.domain.contains(domain) || domain.contains(&cookie.domain)
            });

            if is_platform_cookie {
                // Add platform prefix to avoid conflicts between platforms
                let prefixed_name = format!("{}_{}", platform_name, cookie.name);
                platform_cookies.insert(prefixed_name, cookie.value.clone());

                debug!(
                    "Found {} cookie: {} from domain {}",
                    platform_name, cookie.name, cookie.domain
                );
            }
        }

        // Log what we found
        if platform_cookies.is_empty() {
            warn!(
                "No cookies found for {} from expected domains: {:?}",
                platform_name, expected_domains
            );

            // Log what domains we did find for debugging
            let found_domains: HashSet<String> = browser_cookies
                .iter()
                .map(|c| c.domain.clone())
                .collect();
            debug!(
                "Available cookie domains: {:?}",
                found_domains
            );
        } else {
            info!(
                "Filtered {} valid cookies for {} from domains: {:?}",
                platform_cookies.len(),
                platform_name,
                expected_domains
            );
        }

        platform_cookies
    }

    /// Launch Chrome browser with appropriate options
    fn launch_browser(&self) -> Result<Browser> {
        // Check if we have a specific profile selected
        if let Some(profile) = &self.selected_profile {
            match profile.browser_type {
                BrowserType::Chrome => return self.launch_chrome_with_profile(profile),
                BrowserType::Firefox => return self.launch_firefox_with_profile(profile),
                BrowserType::Safari => {
                    warn!("Safari profile selected but not supported for automation, falling back to default Chrome");
                }
            }
        }

        // Fallback to default Chrome launch
        self.launch_default_chrome()
    }

    /// Launch Chrome with a specific profile
    fn launch_chrome_with_profile(&self, profile: &BrowserProfile) -> Result<Browser> {
        info!("Launching Chrome with profile: {}", profile.description());

        // Try to find Chrome in common locations
        let chrome_paths = self.find_chrome_paths();
        info!("Checking Chrome paths: {:?}", chrome_paths);

        // Configure launch options
        let mut launch_options = LaunchOptions::default_builder();

        // Try each Chrome path until one works
        let mut chrome_found = false;
        for path in &chrome_paths {
            if std::path::Path::new(path).exists() {
                info!("Found Chrome at: {}", path);
                launch_options.path(Some(std::path::PathBuf::from(path)));
                chrome_found = true;
                break;
            } else {
                debug!("Chrome not found at: {}", path);
            }
        }

        if !chrome_found {
            warn!("No Chrome executable found in expected locations, using system default");
        }

        // Use the user's actual Chrome profile directory
        let chrome_dir = profile.profile_path.parent()
            .ok_or_else(|| anyhow!("Invalid Chrome profile path"))?;
        let user_data_dir = chrome_dir.to_path_buf();

        info!("Using Chrome user data directory: {}", user_data_dir.display());
        info!("Using Chrome profile: {} ({})", profile.profile_name, profile.profile_id);

        // Build Chrome arguments
        let user_data_arg = format!("--user-data-dir={}", user_data_dir.display());
        let profile_dir_arg = if profile.profile_id == "Default" {
            None
        } else {
            Some(format!("--profile-directory={}", profile.profile_id))
        };

        let mut args = vec![
            std::ffi::OsStr::new("--disable-blink-features=AutomationControlled"),
            std::ffi::OsStr::new("--user-agent=Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36"),
            std::ffi::OsStr::new(&user_data_arg),
        ];

        // Add profile directory argument if not Default profile
        if let Some(ref profile_arg) = profile_dir_arg {
            args.push(std::ffi::OsStr::new(profile_arg));
        }

        launch_options
            .headless(self.headless)
            .window_size(Some((1280, 1024)))
            .args(args);

        // Launch browser
        info!("Launching Chrome with profile-specific options: headless={}, profile={}", self.headless, profile.profile_id);
        let browser =
            Browser::new(launch_options.build()?).context("Failed to launch Chrome browser with profile")?;

        info!("Chrome browser launched successfully with profile: {}", profile.profile_name);
        Ok(browser)
    }

    /// Launch Firefox with a specific profile (not fully implemented - Chrome fallback)
    fn launch_firefox_with_profile(&self, profile: &BrowserProfile) -> Result<Browser> {
        warn!("Firefox profile automation not yet implemented: {}", profile.description());
        warn!("Falling back to default Chrome launch");
        self.launch_default_chrome()
    }

    /// Launch Chrome with default cs-cli profile (original behavior)
    fn launch_default_chrome(&self) -> Result<Browser> {
        info!("Launching Chrome browser with default cs-cli profile...");

        // Try to find Chrome in common locations
        let chrome_paths = self.find_chrome_paths();
        info!("Checking Chrome paths: {:?}", chrome_paths);

        // Configure launch options
        let mut launch_options = LaunchOptions::default_builder();

        // Try each Chrome path until one works
        let mut chrome_found = false;
        for path in &chrome_paths {
            if std::path::Path::new(path).exists() {
                info!("Found Chrome at: {}", path);
                launch_options.path(Some(std::path::PathBuf::from(path)));
                chrome_found = true;
                break;
            } else {
                debug!("Chrome not found at: {}", path);
            }
        }

        if !chrome_found {
            warn!("No Chrome executable found in expected locations, using system default");
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
            .headless(self.headless)
            .window_size(Some((1280, 1024)))
            .args(vec![
                std::ffi::OsStr::new("--disable-blink-features=AutomationControlled"),
                std::ffi::OsStr::new("--user-agent=Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36"),
                std::ffi::OsStr::new(&user_data_arg),
            ]);

        // Launch browser
        info!("Launching browser with options: headless={}, window_size=(1280,1024)", self.headless);
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

        // Get the tab once and store it for reuse
        if self.active_tab.is_none() {
            let browser = self
                .browser
                .as_ref()
                .ok_or_else(|| anyhow!("Browser not initialized"))?;

            // Wait for Chrome to fully initialize and create its default tab
            let mut attempts = 0;
            let max_attempts = 10;
            let tab = loop {
                {
                    let tabs = browser.get_tabs().lock().unwrap();
                    if let Some(first_tab) = tabs.first() {
                        let tab = first_tab.clone();
                        drop(tabs); // Release lock
                        break tab;
                    }
                    drop(tabs); // Release lock
                }
                
                attempts += 1;
                if attempts >= max_attempts {
                    return Err(anyhow!("Chrome failed to create initial tab after {} attempts", max_attempts));
                }
                
                // Wait a bit for Chrome to initialize
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            };

            self.active_tab = Some(tab);
            info!("Using Chrome's default tab for navigation");
        }

        let tab = self
            .active_tab
            .as_ref()
            .ok_or_else(|| anyhow!("Active tab not initialized"))?;

        info!("Navigating to: {}", url);
        tab.navigate_to(url).context("Failed to navigate to URL")?;

        info!("Waiting for navigation to complete...");
        tab.wait_until_navigated()
            .context("Failed to wait for navigation")?;

        // Log final URL after navigation
        let final_url = tab.get_url();
        info!("Navigation completed. Final URL: {}", final_url);

        // Log page title for verification
        let page_title_script = "document.title";
        match tab.evaluate(page_title_script, false) {
            Ok(result) => {
                info!("Page title after navigation: {:?}", result.value);
            }
            Err(e) => {
                debug!("Failed to get page title: {}", e);
            }
        }

        Ok(())
    }

    /// Navigate to Okta and perform authentication  
    pub async fn navigate_to_okta(&mut self, url: &str) -> Result<()> {
        self.navigate_to_url(url).await
    }

    /// Extract cookies from authenticated session
    pub async fn extract_cookies(&self) -> Result<HashMap<String, String>> {
        let (cookie_map, _) = self.extract_cookies_with_details().await?;
        Ok(cookie_map)
    }

    /// Extract cookies with full details for domain discovery
    pub async fn extract_cookies_with_details(
        &self,
    ) -> Result<(
        HashMap<String, String>,
        Vec<Cookie>,
    )> {
        let tab = self
            .active_tab
            .as_ref()
            .ok_or_else(|| anyhow!("No active tab"))?;

        // Get cookies from the page
        let cookies = tab.get_cookies().context("Failed to get cookies")?;

        let mut cookie_map = HashMap::new();

        // Convert cookies to map
        for cookie in &cookies {
            debug!("Cookie: {} = {} (domain: {})", cookie.name, cookie.value, cookie.domain);
            cookie_map.insert(cookie.name.clone(), cookie.value.clone());
        }

        info!("Extracted {} cookies", cookie_map.len());
        Ok((cookie_map, cookies))
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
