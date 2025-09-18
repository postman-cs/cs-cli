use super::CSRFManager;
use crate::common::auth::{Cookie, CookieExtractor};
use crate::common::config::AuthSettings;
use crate::common::http::HttpClient; // For trait methods
use crate::gong::api::client::GongHttpClient;
use crate::{CsCliError, Result};
use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, error, info, warn};

/// Gong authentication cookies with metadata
#[derive(Debug, Clone)]
pub struct GongCookies {
    /// Gong cell identifier (e.g., "us-14496")
    pub cell: String,

    /// Session cookies extracted from browser
    pub session_cookies: HashMap<String, String>,

    /// When cookies were extracted (Unix timestamp)
    pub extracted_at: f64,

    /// Which browser was used for extraction
    pub browser: String,
}

/// Main authentication manager for Gong API with multi-browser support
///
/// Orchestrates the complete authentication flow:
/// - Multi-browser cookie extraction (Firefox, Chrome, Safari, Edge, Brave)
/// - CSRF token management with async caching
/// - Workspace ID extraction from Gong home page
/// - Header generation for authenticated requests
/// - Error handling and token refresh logic
pub struct GongAuthenticator {
    /// HTTP client for API requests (using impit)
    http_client: Arc<GongHttpClient>,

    /// Multi-browser cookie extractor using rookie
    cookie_extractor: CookieExtractor,

    /// CSRF token manager with async safety
    csrf_manager: CSRFManager,

    /// Extracted Gong cookies and metadata
    gong_cookies: Option<GongCookies>,

    /// Base URL for API calls (e.g., "<https://us-14496.app.gong.io>")
    base_url: Option<String>,

    /// Gong workspace ID extracted from home page
    workspace_id: Option<String>,
}

impl GongAuthenticator {
    /// Create a new Gong authenticator with production configuration
    pub async fn new(config: AuthSettings) -> Result<Self> {
        // Create HTTP client for API requests with intelligent HTTP/3 -> HTTP/2 fallback
        use crate::common::config::HttpSettings;
        let http_config = HttpSettings::default();
        let http_client = Arc::new(GongHttpClient::new(http_config).await?);

        // Set up cookie extractor for Gong domains (*.gong.io)
        let domains = vec![
            "gong.io".to_string(),  // Exact domain
            ".gong.io".to_string(), // All subdomains (*.gong.io)
        ];
        let cookie_extractor = CookieExtractor::new(domains);
        let csrf_manager = CSRFManager::new(config, http_client.clone());

        Ok(Self {
            http_client,
            cookie_extractor,
            csrf_manager,
            gong_cookies: None,
            base_url: None,
            workspace_id: None,
        })
    }

    /// Perform complete authentication flow
    ///
    /// This is the main entry point that:
    /// 1. Extracts cookies from available browsers
    /// 2. Validates cookies with server (tries multiple browsers if needed)
    /// 3. Determines Gong cell identifier
    /// 4. Sets up base URL and session cookies
    /// 5. Fetches initial CSRF token
    /// 6. Extracts workspace ID
    ///
    /// # Returns
    /// true if authentication succeeds, false otherwise
    pub async fn authenticate(&mut self) -> Result<bool> {
        info!("Starting Gong authentication with multi-browser support");

        // Ensure keychain access for browser cookies (macOS-only)
        use crate::common::auth::smart_keychain::SmartKeychainManager;
        let keychain_manager = SmartKeychainManager::new()?;
        keychain_manager.ensure_keychain_access()?;

        // Extract cookies from ALL available browsers for validation
        let all_browser_cookies = self.cookie_extractor.extract_all_browsers_cookies();

        if all_browser_cookies.is_empty() {
            error!("No cookies found in any supported browser");
            info!("Make sure you're logged into Gong in any supported browser: Firefox, Chrome, Edge, Arc, Brave, Chromium, LibreWolf, Opera, Opera GX, Vivaldi, Zen, Safari, or Cachy");
            return Ok(false);
        }

        // Try each browser's cookies until we find working ones
        let mut working_cookies: Option<(Vec<Cookie>, String)> = None;

        for (cookies, browser_name) in all_browser_cookies {
            if cookies.is_empty() {
                continue;
            }

            info!(
                "Testing cookies from {} ({} cookies)",
                browser_name,
                cookies.len()
            );

            // Try to extract cell and test authentication
            match self.extract_cell_from_cookies(&cookies) {
                Ok(cell) => {
                    // Build session cookies map
                    let mut session_cookies = HashMap::new();
                    for cookie in &cookies {
                        session_cookies.insert(cookie.name.clone(), cookie.value.clone());
                    }

                    // Store authentication state temporarily
                    self.gong_cookies = Some(GongCookies {
                        cell: cell.clone(),
                        session_cookies: session_cookies.clone(),
                        extracted_at: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs_f64(),
                        browser: browser_name.clone(),
                    });

                    self.base_url = Some(format!("https://{cell}.app.gong.io"));

                    // Set cookies on HTTP client
                    self.http_client.set_cookies(session_cookies.clone()).await?;

                    // Try to get CSRF token to validate the cookies
                    match self.csrf_manager.get_csrf_token(&cell, false).await {
                        Ok(Some(_token)) => {
                            info!(
                                "Successfully authenticated with {} cookies",
                                browser_name
                            );
                            working_cookies = Some((cookies, browser_name));
                            break;
                        }
                        Ok(None) | Err(_) => {
                            warn!(
                                "{} cookies are expired or invalid, trying next browser",
                                browser_name
                            );
                            // Clear the failed state
                            self.gong_cookies = None;
                            self.base_url = None;
                            continue;
                        }
                    }
                }
                Err(e) => {
                    debug!("Could not extract cell from {} cookies: {}", browser_name, e);
                    continue;
                }
            }
        }

        // Check if we found working cookies
        let (cookies, browser_source) = match working_cookies {
            Some(result) => result,
            None => {
                error!("All browser cookies are expired or invalid");
                info!("Please log into Gong in any browser and try again");
                return Ok(false);
            }
        };

        info!(
            cookies_found = cookies.len(),
            browser_used = %browser_source,
            browsers_supported = "Firefox, Chrome, Edge, Arc, Brave, Chromium, LibreWolf, Opera, Opera GX, Vivaldi, Zen, Safari, Cachy",
            "Successfully authenticated"
        );

        // The authentication state is already set up from the successful attempt above
        // Just need to get the cell for logging
        let cell = self.gong_cookies.as_ref().unwrap().cell.clone();

        // CSRF token was already validated above, no need to get it again

        // Extract workspace ID from home page
        if let Ok(Some(workspace_id)) = self.extract_workspace_id().await {
            self.workspace_id = Some(workspace_id);
            info!(workspace_id = %self.workspace_id.as_ref().unwrap(), "Workspace ID extracted successfully");
        } else {
            warn!("Could not extract workspace ID - some API calls may fail");
        }

        info!(
            cell = %cell,
            browser = %browser_source,
            cookies_count = cookies.len(),
            "Authentication flow completed successfully"
        );

        Ok(true)
    }

    /// Extract Gong cell identifier from cookies by decrypting the JWT "cell" cookie
    fn extract_cell_from_cookies(&self, cookies: &[Cookie]) -> Result<String> {
        // Only strategy: Decode from JWT "cell" cookie
        for cookie in cookies {
            if cookie.name == "cell" {
                if let Ok(cell) = self.decode_cell_cookie(&cookie.value) {
                    debug!(cell = %cell, strategy = "jwt_decode", "Extracted cell from JWT cookie");
                    return Ok(cell);
                }
            }
        }

        Err(CsCliError::Authentication(
            "Could not find or decode 'cell' cookie - make sure you're logged into Gong".to_string(),
        ))
    }

    /// Decode JWT cell cookie to extract cell value (public for testing)
    pub fn decode_cell_cookie(&self, cell_cookie: &str) -> Result<String> {
        // JWT format: header.payload.signature
        let parts: Vec<&str> = cell_cookie.split('.').collect();
        if parts.len() != 3 {
            return Err(CsCliError::Authentication("Invalid JWT format".to_string()));
        }

        // Decode the payload (middle part) with base64-simd for 1.56x performance
        let mut payload = parts[1].to_string();

        // Add padding if needed for base64 decoding
        let padding = 4 - (payload.len() % 4);
        if padding != 4 {
            payload.push_str(&"=".repeat(padding));
        }

        // Use base64-simd for faster decoding
        let decoded_bytes = base64_simd::STANDARD
            .decode_to_vec(payload.as_bytes())
            .map_err(|e| CsCliError::Authentication(format!("Base64 decode failed: {e}")))?;

        let payload_json = String::from_utf8(decoded_bytes)
            .map_err(|e| CsCliError::Authentication(format!("UTF-8 decode failed: {e}")))?;

        let payload_data: serde_json::Value = serde_json::from_str(&payload_json)
            .map_err(|e| CsCliError::Authentication(format!("JSON parse failed: {e}")))?;

        payload_data
            .get("cell")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| CsCliError::Authentication("Cell not found in JWT payload".to_string()))
    }

    /// Get headers for read-only requests (GET) - no CSRF token needed
    pub fn get_read_headers(&self) -> Result<HashMap<String, String>> {
        let base_url = self.base_url.as_ref().ok_or_else(|| {
            CsCliError::Authentication("Not authenticated - no base URL".to_string())
        })?;

        let mut headers = HashMap::new();
        headers.insert("X-Requested-With".to_string(), "XMLHttpRequest".to_string());
        headers.insert("Referer".to_string(), base_url.clone());

        Ok(headers)
    }

    /// Get headers for state-changing requests (POST/PUT/DELETE) with CSRF token
    ///
    /// # Arguments
    /// * `retry_on_failure` - Whether to retry with fresh token on initial failure
    pub async fn get_authenticated_headers(
        &self,
        retry_on_failure: bool,
    ) -> Result<HashMap<String, String>> {
        let base_url = self.base_url.as_ref().ok_or_else(|| {
            CsCliError::Authentication("Not authenticated - no base URL".to_string())
        })?;

        let cell = &self
            .gong_cookies
            .as_ref()
            .ok_or_else(|| {
                CsCliError::Authentication("Not authenticated - no cookies".to_string())
            })?
            .cell;

        // Get CSRF token
        let csrf_token = match self.csrf_manager.get_csrf_token(cell, false).await? {
            Some(token) => token,
            #[allow(non_snake_case)]
            None => {
                if retry_on_failure {
                    warn!("CSRF token missing, attempting forced refresh");

                    #[allow(non_snake_case)]
                    match self.csrf_manager.get_csrf_token(cell, true).await? {
                        Some(token) => token,
                        #[allow(non_snake_case)]
                        None => {
                            return Err(CsCliError::Authentication(
                                "Failed to get CSRF token after refresh attempt".to_string(),
                            ));
                        }
                    }
                } else {
                    return Err(CsCliError::Authentication(
                        "CSRF token not available and retry disabled".to_string(),
                    ));
                }
            }
        };

        let mut headers = HashMap::new();
        headers.insert("X-Requested-With".to_string(), "XMLHttpRequest".to_string());
        headers.insert("X-CSRF-Token".to_string(), csrf_token);
        headers.insert("Referer".to_string(), base_url.clone());

        Ok(headers)
    }

    /// Handle authentication errors by refreshing tokens
    ///
    /// # Arguments
    /// * `status_code` - HTTP status code that triggered the error
    /// * `is_post_request` - Whether this was a POST/PUT/DELETE request needing CSRF tokens
    ///
    /// # Returns
    /// true if tokens were refreshed successfully
    pub async fn handle_auth_error(&self, status_code: u16, is_post_request: bool) -> Result<bool> {
        if [401, 403].contains(&status_code) {
            if is_post_request {
                warn!(
                    status = status_code,
                    "Authentication error on POST request, refreshing CSRF token"
                );

                // Invalidate current token
                self.csrf_manager.invalidate_token().await;

                // Try to get fresh token
                if let Some(cookies) = &self.gong_cookies {
                    match self
                        .csrf_manager
                        .get_csrf_token(&cookies.cell, true)
                        .await?
                    {
                        Some(_) => {
                            info!("CSRF token successfully refreshed after authentication error");
                            return Ok(true);
                        }
                        #[allow(non_snake_case)]
                        None => {
                            error!("Failed to refresh CSRF token after authentication error");
                            return Ok(false);
                        }
                    }
                }
            } else {
                warn!(
                    status = status_code,
                    "Authentication error on GET request - likely session issue"
                );
                // For GET requests, 401/403 likely means session cookies are invalid
                // We could potentially refresh the entire authentication here
                return Ok(false);
            }
        }

        Ok(false)
    }

    /// Get the base URL for API calls
    pub fn get_base_url(&self) -> Result<&str> {
        self.base_url.as_deref().ok_or_else(|| {
            CsCliError::Authentication("Not authenticated - no base URL available".to_string())
        })
    }

    /// Extract workspace ID from Gong home page
    /// TODO: Implement in Phase 2 when HTTP client is available
    pub async fn extract_workspace_id(&mut self) -> Result<Option<String>> {
        let base_url = self.base_url.as_ref().ok_or_else(|| {
            CsCliError::Authentication("Cannot extract workspace ID without base URL".to_string())
        })?;

        // Make actual HTTP request to extract workspace ID
        let url = format!("{base_url}/home");
        let headers = self.get_read_headers()?;

        // Update headers on HTTP client and make request
        self.http_client.update_headers(headers).await?;
        let response = self.http_client.get(&url).await?;

        if !response.status().is_success() {
            warn!(
                status = response.status().as_u16(),
                "Failed to fetch home page for workspace ID extraction"
            );
            return Ok(None);
        }

        let html_content = response
            .text()
            .await
            .map_err(|e| CsCliError::Generic(format!("Failed to read home page: {e}")))?;

        debug!("Extracting workspace ID from home page HTML");

        // Use regex for workspace ID extraction (optimized with aho-corasick already in tree)
        let workspace_regex = Regex::new(r#"workspaceId:\s*"(\d+)""#)
            .map_err(|e| CsCliError::Generic(format!("Regex compilation failed: {e}")))?;

        if let Some(captures) = workspace_regex.captures(&html_content) {
            if let Some(workspace_id) = captures.get(1) {
                let id = workspace_id.as_str().to_string();
                self.workspace_id = Some(id.clone());

                info!(
                    workspace_id = %id,
                    "Successfully extracted workspace ID"
                );

                return Ok(Some(id));
            }
        }

        // Try alternative pattern
        let alt_regex = Regex::new(r#""workspaceId"\s*:\s*"(\d+)""#).map_err(|e| {
            CsCliError::Generic(format!("Alternative regex compilation failed: {e}"))
        })?;

        if let Some(captures) = alt_regex.captures(&html_content) {
            if let Some(workspace_id) = captures.get(1) {
                let id = workspace_id.as_str().to_string();
                self.workspace_id = Some(id.clone());

                info!(
                    workspace_id = %id,
                    extraction_method = "alternative_pattern",
                    "Successfully extracted workspace ID"
                );

                return Ok(Some(id));
            }
        }

        warn!("Could not find workspace ID in home page HTML");
        Ok(None)
    }

    /// Get the cached workspace ID
    pub fn get_workspace_id(&self) -> Option<&str> {
        self.workspace_id.as_deref()
    }

    /// Get authentication state summary for debugging
    pub fn get_auth_state(&self) -> HashMap<String, String> {
        let mut state = HashMap::new();

        if let Some(cookies) = &self.gong_cookies {
            state.insert("authenticated".to_string(), "true".to_string());
            state.insert("cell".to_string(), cookies.cell.clone());
            state.insert("browser".to_string(), cookies.browser.clone());
            state.insert(
                "cookies_count".to_string(),
                cookies.session_cookies.len().to_string(),
            );

            if let Some(base_url) = &self.base_url {
                state.insert("base_url".to_string(), base_url.clone());
            }

            if let Some(workspace_id) = &self.workspace_id {
                state.insert("workspace_id".to_string(), workspace_id.clone());
            }
        } else {
            state.insert("authenticated".to_string(), "false".to_string());
        }

        state
    }

    /// Check if currently authenticated
    pub fn is_authenticated(&self) -> bool {
        self.gong_cookies.is_some() && self.base_url.is_some()
    }

    /// Get session cookies for setting on HTTP clients
    pub fn get_session_cookies(&self) -> Result<HashMap<String, String>> {
        match &self.gong_cookies {
            Some(cookies) => Ok(cookies.session_cookies.clone()),
            None => Err(CsCliError::Authentication(
                "Not authenticated - no cookies available".to_string(),
            )),
        }
    }

    /// Get the Gong cell identifier
    pub fn get_cell(&self) -> Option<&str> {
        self.gong_cookies.as_ref().map(|c| c.cell.as_str())
    }
}
