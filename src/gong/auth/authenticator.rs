use super::CSRFManager;
use crate::common::auth::hybrid_cookie_storage::{get_cookies_hybrid, has_cookies_hybrid};
use crate::common::auth::{Cookie, CookieRetriever, GuidedAuth, PlatformType};
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

    /// Your own session cookies from your authenticated browser
    pub session_cookies: HashMap<String, String>,

    /// When cookies were retrieved (Unix timestamp)
    pub retrieved_at: f64,

    /// Which browser was used for cookie retrieval
    pub browser: String,
}

/// Main authentication manager for Gong API with multi-browser support
///
/// Orchestrates the complete authentication flow using YOUR OWN authenticated sessions:
/// - Multi-browser cookie access from your authenticated browser sessions (Firefox, Chrome, Safari, Edge, Brave)
/// - CSRF token management with async caching
/// - Workspace ID access from your Gong home page
/// - Header generation for authenticated requests
/// - Error handling and token refresh logic
pub struct GongAuthenticator {
    /// HTTP client for API requests (using impit)
    pub http_client: Arc<GongHttpClient>,

    /// Multi-browser cookie accessor using direct database access
<<<<<<< HEAD
    pub cookie_retriever: CookieRetriever,
=======
    cookie_retriever: CookieRetriever,
>>>>>>> 30887b9 (github auth improvements)

    /// CSRF token manager with async safety
    pub csrf_manager: CSRFManager,

    /// Retrieved Gong cookies and metadata
<<<<<<< HEAD
    pub gong_cookies: Option<GongCookies>,
=======
    gong_cookies: Option<GongCookies>,
>>>>>>> 30887b9 (github auth improvements)

    /// Base URL for API calls (e.g., "<https://us-14496.app.gong.io>")
    pub base_url: Option<String>,

    /// Gong workspace ID retrieved from home page
<<<<<<< HEAD
    pub workspace_id: Option<String>,
=======
    workspace_id: Option<String>,
>>>>>>> 30887b9 (github auth improvements)
}

impl GongAuthenticator {
    /// Create a new Gong authenticator with production configuration
    pub async fn new(config: AuthSettings) -> Result<Self> {
        // Create HTTP client using common platform configuration
        let http_client = Arc::new(GongHttpClient::new(
            crate::common::http::PlatformConfigs::gong(None)
                .build_settings()
        ).await?);

        // Set up cookie retriever for Gong domains (*.gong.io)
        let domains = vec![
            "gong.io".to_string(),  // Exact domain
            ".gong.io".to_string(), // All subdomains (*.gong.io)
        ];
        let cookie_retriever = CookieRetriever::new(domains);
        let csrf_manager = CSRFManager::new(config, http_client.clone());

        Ok(Self {
            http_client,
            cookie_retriever,
            csrf_manager,
            gong_cookies: None,
            base_url: None,
            workspace_id: None,
        })
    }

<<<<<<< HEAD
    // Note: The authenticate() method has been moved to authentication_flow.rs
    // to eliminate DRY violations and improve code organization.
=======
    /// Perform complete authentication flow
    ///
    /// This is the main entry point that:
    /// 1. Checks hybrid storage (gist + local keychain) for stored cookies
    /// 2. Retrieves cookies from available browsers
    /// 3. Validates cookies with server (tries multiple browsers if needed)
    /// 4. Determines Gong cell identifier
    /// 5. Sets up base URL and session cookies
    /// 6. Fetches initial CSRF token
    /// 7. Retrieves workspace ID
    ///
    /// # Returns
    /// true if authentication succeeds, false otherwise
    pub async fn authenticate(&mut self) -> Result<bool> {
        info!("Starting Gong authentication");

        // First check hybrid storage (gist + local keychain) for stored cookies
        info!("Checking for stored session data from previous authentication...");
        if let Ok(authenticated) = self.try_hybrid_storage_authentication().await {
            if authenticated {
                info!("Successfully authenticated using stored session data");
                return Ok(true);
            }
        }

        // Try Firefox cookies next (quick, no user interaction needed)
        info!("Checking for existing browser cookies...");
        let cookie_retriever = self.cookie_retriever.clone();
        info!("Spawning blocking task for Firefox cookie retrieval...");
        let firefox_cookies = match tokio::task::spawn_blocking(move || {
            cookie_retriever.retrieve_firefox_cookies_ephemeral()
        }).await {
            Ok(Ok(cookies)) => {
                info!("Firefox cookie retrieval completed successfully, found {} cookies", cookies.len());
                cookies
            },
            Ok(Err(e)) => {
                warn!("Firefox cookie retrieval failed: {}", e);
                Vec::new()
            },
            Err(e) => {
                warn!("Firefox cookie retrieval task failed: {}", e);
                Vec::new()
            }
        };

        // If we have cookies, try to use them
        if !firefox_cookies.is_empty() {
            info!(
                "Found {} Firefox cookies, validating...",
                firefox_cookies.len()
            );

            let all_browser_cookies = vec![(firefox_cookies, "Firefox".to_string())];

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

                // Try to retrieve cell and test authentication
                info!("Attempting to extract Gong cell identifier from {} cookies", browser_name);
                match self.retrieve_cell_from_cookies(&cookies) {
                    Ok(cell) => {
                        info!("Successfully extracted Gong cell: {}", cell);
                        // Build session cookies map
                        let mut session_cookies = HashMap::new();
                        for cookie in &cookies {
                            session_cookies.insert(cookie.name.clone(), cookie.value.clone());
                        }

                        // Store authentication state temporarily
                        self.gong_cookies = Some(GongCookies {
                            cell: cell.clone(),
                            session_cookies: session_cookies.clone(),
                            retrieved_at: SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs_f64(),
                            browser: browser_name.clone(),
                        });

                        self.base_url = Some(format!("https://{cell}.app.gong.io"));

                        // Set cookies on HTTP client
                        info!("Setting cookies on HTTP client for validation...");
                        self.http_client
                            .set_cookies(session_cookies.clone())
                            .await?;

                        // Try to get CSRF token to validate the cookies (with short timeout)
                        info!("Attempting CSRF token validation with 5-second timeout...");
                        match tokio::time::timeout(
                            std::time::Duration::from_secs(5),
                            self.csrf_manager.get_csrf_token(&cell, false),
                        )
                        .await
                        {
                            Ok(Ok(Some(_token))) => {
                                info!("Successfully authenticated with {} cookies", browser_name);
                                working_cookies = Some((cookies, browser_name));
                                break;
                            }
                            Ok(Ok(None)) | Ok(Err(_)) | Err(_) => {
                                warn!("{} cookies are expired or invalid", browser_name);
                                // Clear the failed state
                                self.gong_cookies = None;
                                self.base_url = None;
                                continue;
                            }
                        }
                    }
                    Err(e) => {
                        debug!(
                            "Could not retrieve cell from {} cookies: {}",
                            browser_name, e
                        );
                        continue;
                    }
                }
            }

            // Check if we found working cookies
            if let Some((cookies, browser_source)) = working_cookies {
                info!(
                    cookies_found = cookies.len(),
                    browser_used = %browser_source,
                    "Successfully authenticated with existing cookies"
                );

                // Continue with the rest of the authentication flow
                let (cookies, browser_source) = (cookies, browser_source);

                // The authentication state is already set up from the successful attempt above
                // Just need to get the cell for logging
                let cell = self.gong_cookies.as_ref().unwrap().cell.clone();

                // CSRF token was already validated above, no need to get it again

                // Retrieve workspace ID from home page
                if let Ok(Some(workspace_id)) = self.retrieve_workspace_id().await {
                    self.workspace_id = Some(workspace_id);
                    info!(workspace_id = %self.workspace_id.as_ref().unwrap(), "Workspace ID retrieved successfully");
                } else {
                    warn!("Could not retrieve workspace ID - some API calls may fail");
                }

                info!(
                    cell = %cell,
                    browser = %browser_source,
                    cookies_count = cookies.len(),
                    "Authentication flow completed successfully"
                );

                return Ok(true);
            }
        }

        // No working cookies found - try guided authentication
        info!("No valid browser cookies found, attempting guided authentication...");
        match self.try_guided_authentication().await {
            Ok(true) => {
                info!("Guided authentication successful");
                Ok(true)
            }
            Ok(false) => {
                error!("Authentication failed: User declined guided authentication");
                Ok(false)
            }
            Err(e) => {
                error!("Guided authentication failed: {}", e);
                Ok(false)
            }
        }
    }
>>>>>>> 30887b9 (github auth improvements)

    /// Try to authenticate using cookies from hybrid storage (gist + local keychain)
    pub async fn try_hybrid_storage_authentication(&mut self) -> Result<bool> {
        // Check if we have cookies in hybrid storage
        if !has_cookies_hybrid().await {
            debug!("No cookies found in hybrid storage");
            return Ok(false);
        }

        // Get cookies from hybrid storage
        let stored_cookies = match get_cookies_hybrid().await {
            Ok(cookies) => cookies,
            Err(e) => {
                debug!("Failed to retrieve cookies from hybrid storage: {}", e);
                return Ok(false);
            }
        };

        if stored_cookies.is_empty() {
            debug!("Retrieved empty cookie collection from hybrid storage");
            return Ok(false);
        }

        info!(
            "Found {} cookies in hybrid storage, processing Gong cookies...",
            stored_cookies.len()
        );

        // Retrieve Gong-specific cookies and convert to Cookie format
        let mut gong_cookies = Vec::new();
        let mut session_cookies = HashMap::new();
        let mut cell_cookie_value = None;

        for (name, value) in &stored_cookies {
            // Check if this is a Gong cookie (prefixed with "gong_")
            if let Some(actual_name) = name.strip_prefix("gong_") {
                // Create a Cookie struct for cell retrieval
                if actual_name == "cell" {
                    cell_cookie_value = Some(value.clone());
                    gong_cookies.push(Cookie {
                        name: actual_name.to_string(),
                        value: value.clone(),
                        domain: ".gong.io".to_string(),
                        path: "/".to_string(),
                        secure: true,
                        http_only: true,
                        expires: None,
                    });
                }

                // Store in session cookies map
                session_cookies.insert(actual_name.to_string(), value.clone());
            }
        }

        if session_cookies.is_empty() {
            debug!("No Gong cookies found in stored data");
            return Ok(false);
        }

        info!(
            "Found {} Gong cookies in stored data",
            session_cookies.len()
        );

        // Try to retrieve cell from the cookies
        let cell = if let Some(cell_value) = cell_cookie_value {
            match self.decode_cell_cookie(&cell_value) {
                Ok(cell) => {
                    info!("Successfully decoded cell from stored cookies: {}", cell);
                    cell
                }
                Err(e) => {
                    warn!("Failed to decode cell cookie from stored data: {}", e);
                    return Ok(false);
                }
            }
        } else {
            warn!("No cell cookie found in stored data");
            return Ok(false);
        };

        // Set up authentication state
        self.gong_cookies = Some(GongCookies {
            cell: cell.clone(),
            session_cookies: session_cookies.clone(),
            retrieved_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
            browser: "Hybrid Storage".to_string(),
        });

        self.base_url = Some(format!("https://{cell}.app.gong.io"));

        // Set cookies on HTTP client
        if let Err(e) = self.http_client.set_cookies(session_cookies.clone()).await {
            warn!("Failed to set cookies on HTTP client: {}", e);
            return Ok(false);
        }

        // Validate cookies by trying to get CSRF token
        info!("Validating stored cookies with CSRF token check...");
        match tokio::time::timeout(
            std::time::Duration::from_secs(5),
            self.csrf_manager.get_csrf_token(&cell, false),
        )
        .await
        {
            Ok(Ok(Some(token))) => {
                info!("Stored cookies are valid, CSRF token obtained: {}", token);

                // Retrieve workspace ID
                if let Ok(Some(workspace_id)) = self.retrieve_workspace_id().await {
                    self.workspace_id = Some(workspace_id);
                    info!(
                        "Workspace ID retrieved: {}",
                        self.workspace_id.as_ref().unwrap()
                    );
                } else {
                    warn!("Could not retrieve workspace ID - some API calls may fail");
                }

                Ok(true)
            }
            Ok(Ok(None)) | Ok(Err(_)) | Err(_) => {
                warn!("Stored cookies are invalid or expired, clearing state");
                // Clear the failed state
                self.gong_cookies = None;
                self.base_url = None;
                Ok(false)
            }
        }
    }

    /// Retrieve Gong cell identifier from cookies by decrypting the JWT "cell" cookie
<<<<<<< HEAD
    pub fn retrieve_cell_from_cookies(&self, cookies: &[Cookie]) -> Result<String> {
=======
    fn retrieve_cell_from_cookies(&self, cookies: &[Cookie]) -> Result<String> {
>>>>>>> 30887b9 (github auth improvements)
        // Only strategy: Decode from JWT "cell" cookie
        for cookie in cookies {
            if cookie.name == "cell" {
                if let Ok(cell) = self.decode_cell_cookie(&cookie.value) {
                    debug!(cell = %cell, strategy = "jwt_decode", "Retrieved cell from JWT cookie");
                    return Ok(cell);
                }
            }
        }

        Err(CsCliError::Authentication(
            "Could not find or decode 'cell' cookie - make sure you're logged into Gong"
                .to_string(),
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

    /// Retrieve workspace ID from Gong home page
    /// TODO: Implement in Phase 2 when HTTP client is available
    pub async fn retrieve_workspace_id(&mut self) -> Result<Option<String>> {
        let base_url = self.base_url.as_ref().ok_or_else(|| {
            CsCliError::Authentication("Cannot retrieve workspace ID without base URL".to_string())
        })?;

        // Make actual HTTP request to retrieve workspace ID
        let url = format!("{base_url}/home");
        let headers = self.get_read_headers()?;

        // Update headers on HTTP client and make request
        self.http_client.update_headers(headers).await?;
        let response = self.http_client.get(&url).await?;

        if !response.status().is_success() {
            warn!(
                status = response.status().as_u16(),
                "Failed to fetch home page for workspace ID retrieval"
            );
            return Ok(None);
        }

        let html_content = response
            .text()
            .await
            .map_err(|e| CsCliError::Generic(format!("Failed to read home page: {e}")))?;

        debug!("Retrieving workspace ID from home page HTML");

        // Use regex for workspace ID extraction (optimized with aho-corasick already in tree)
        let workspace_regex = Regex::new(r#"workspaceId:\s*"(\d+)""#)
            .map_err(|e| CsCliError::Generic(format!("Regex compilation failed: {e}")))?;

        if let Some(captures) = workspace_regex.captures(&html_content) {
            if let Some(workspace_id) = captures.get(1) {
                let id = workspace_id.as_str().to_string();
                self.workspace_id = Some(id.clone());

                info!(
                    workspace_id = %id,
                    "Successfully retrieved workspace ID"
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
                    retrieval_method = "alternative_pattern",
                    "Successfully retrieved workspace ID"
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

    /// Attempt guided authentication via Okta SSO
    ///
    /// Returns Ok(true) if guided auth was attempted and completed successfully,
    /// Ok(false) if guided auth was not attempted (user declined),
    /// Err if guided auth failed with errors.
    pub async fn try_guided_authentication(&mut self) -> Result<bool> {
        info!("Checking if guided authentication is available...");

        // Only attempt guided auth on macOS where we can detect/launch Okta Verify
        #[cfg(not(target_os = "macos"))]
        {
            info!("Guided authentication is only available on macOS");
            return Ok(false);
        }

        #[cfg(target_os = "macos")]
        {
            // Guided authentication is mandatory - no user choice to decline
            // This will automatically proceed with Okta SSO authentication
            info!("Starting mandatory guided Okta authentication...");

            info!("Starting guided Okta authentication...");

            // Create guided auth instance with work profile preference for Gong
            let mut guided_auth = match GuidedAuth::with_platform_preference(PlatformType::Gong) {
                Ok(auth) => {
                    if let Some(profile) = auth.get_selected_profile() {
                        info!("Using browser profile for Gong authentication: {}", profile.description());
                    }
                    auth
                }
                Err(e) => {
                    warn!("Failed to initialize profile-specific authentication: {}", e);
                    info!("Falling back to default authentication method");
                    GuidedAuth::new()
                }
            };

            match guided_auth.authenticate().await {
                Ok(collected_cookies) => {
                    info!("Guided authentication completed successfully");
                    info!(
                        "Collected cookies for {} platforms",
                        collected_cookies.len()
                    );

                    // Process the collected cookies to set up authenticator state
                    // Guided auth returns a flat HashMap with platform-prefixed cookie names
                    // e.g., "gong_cell" -> "value", "gong_session" -> "value"

                    // Extract Gong-specific cookies
                    let mut session_cookies = HashMap::new();
                    let mut cell_value = None;

                    for (cookie_name, cookie_value) in &collected_cookies {
                        // Check if this is a Gong cookie
                        if cookie_name.starts_with("gong_") {
                            // Remove the "gong_" prefix to get the actual cookie name
                            let actual_name =
                                cookie_name.strip_prefix("gong_").unwrap_or(cookie_name);
                            session_cookies.insert(actual_name.to_string(), cookie_value.clone());

                            // Check specifically for the cell cookie
                            if actual_name == "cell" {
                                cell_value = Some(cookie_value.clone());
                            }
                        }
                    }

                    if !session_cookies.is_empty() {
                        info!(
                            "Found {} Gong cookies from guided authentication",
                            session_cookies.len()
                        );

                        // If we found a cell cookie, extract the cell value and set up state
                        if let Some(cell_cookie_value) = cell_value {
                            match self.decode_cell_cookie(&cell_cookie_value) {
                                Ok(cell) => {
                                    info!("Successfully extracted cell from guided auth: {}", cell);

                                    // Set up authenticator state
                                    self.gong_cookies = Some(GongCookies {
                                        cell: cell.clone(),
                                        session_cookies: session_cookies.clone(),
                                        retrieved_at: SystemTime::now()
                                            .duration_since(UNIX_EPOCH)
                                            .unwrap()
                                            .as_secs_f64(),
                                        browser: "Guided Authentication".to_string(),
                                    });

                                    self.base_url = Some(format!("https://{cell}.app.gong.io"));

                                    // Set cookies on HTTP client
                                    if let Err(e) =
                                        self.http_client.set_cookies(session_cookies).await
                                    {
                                        warn!("Failed to set cookies on HTTP client: {}", e);
                                    }

                                    // Retrieve workspace ID
                                    if let Ok(Some(workspace_id)) =
                                        self.retrieve_workspace_id().await
                                    {
                                        self.workspace_id = Some(workspace_id);
                                        info!(workspace_id = %self.workspace_id.as_ref().unwrap(), "Workspace ID retrieved from guided auth");
                                    } else {
                                        warn!("Could not retrieve workspace ID from guided auth");
                                    }

                                    return Ok(true);
                                }
                                Err(e) => {
                                    warn!("Failed to decode cell cookie from guided auth: {}", e);
                                }
                            }
                        } else {
                            warn!("No cell cookie found in guided authentication cookies");
                        }
                    }

                    // If we get here, we didn't find valid Gong cookies
                    warn!("No valid Gong cookies found in guided authentication results");
                    warn!(
                        "Available cookies: {:?}",
                        collected_cookies
                            .keys()
                            .filter(|k| k.starts_with("gong_"))
                            .collect::<Vec<_>>()
                    );
                    Ok(false)
                }
                Err(e) => {
                    error!("Guided authentication failed: {}", e);
                    Err(CsCliError::Authentication(format!(
                        "Guided authentication failed: {e}"
                    )))
                }
            }
        }
    }
}
