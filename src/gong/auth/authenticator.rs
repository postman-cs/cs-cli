use super::CSRFManager;
use crate::common::auth::{Cookie, CookieExtractor};
use crate::gong::api::client::GongHttpClient;
use crate::common::config::{AuthSettings, HttpSettings};
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

    /// Authentication configuration settings
    config: AuthSettings,

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
        // Create HTTP client for API requests
        let http_config = HttpSettings::default();
        let http_client = Arc::new(GongHttpClient::new(http_config).await?);

        // Set up cookie extractor for Gong domains (*.gong.io)
        let domains = vec![
            "gong.io".to_string(),  // Exact domain
            ".gong.io".to_string(), // All subdomains (*.gong.io)
        ];
        let cookie_extractor = CookieExtractor::new(domains);
        let csrf_manager = CSRFManager::new(config.clone(), http_client.clone());

        Ok(Self {
            http_client,
            config,
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
    /// 2. Determines Gong cell identifier
    /// 3. Sets up base URL and session cookies
    /// 4. Fetches initial CSRF token
    /// 5. Extracts workspace ID
    ///
    /// # Returns
    /// true if authentication succeeds, false otherwise
    pub async fn authenticate(&mut self) -> Result<bool> {
        info!("Starting Gong authentication with multi-browser support");

        // Extract cookies from available browsers (rookie handles fallback)
        let (cookies, browser_source) = self.cookie_extractor.extract_gong_cookies_with_source()?;

        if cookies.is_empty() {
            error!("No Gong cookies found in any supported browser");
            info!("Make sure you're logged into Gong in any supported browser: Firefox, Chrome, Edge, Arc, Brave, Chromium, LibreWolf, Opera, Opera GX, Vivaldi, Zen, Safari, or Cachy");
            return Ok(false);
        }

        info!(
            cookies_found = cookies.len(),
            browser_used = %browser_source,
            browsers_supported = "Firefox, Chrome, Edge, Arc, Brave, Chromium, LibreWolf, Opera, Opera GX, Vivaldi, Zen, Safari, Cachy",
            "Successfully extracted browser cookies"
        );

        // Find cell identifier from cookies
        let cell = self.extract_cell_from_cookies(&cookies)?;

        // Build session cookies map
        let mut session_cookies = HashMap::new();
        for cookie in &cookies {
            session_cookies.insert(cookie.name.clone(), cookie.value.clone());
        }

        // Store authentication state (browser source already detected)
        self.gong_cookies = Some(GongCookies {
            cell: cell.clone(),
            session_cookies,
            extracted_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
            browser: browser_source.clone(),
        });

        self.base_url = Some(format!("https://{cell}.app.gong.io"));

        // Set cookies in authenticator's HTTP client for workspace ID extraction
        if let Some(cookies) = &self.gong_cookies {
            self.http_client
                .set_cookies(cookies.session_cookies.clone())
                .await?;
        }

        // Get initial CSRF token (retry based on config)
        match self.csrf_manager.get_csrf_token(&cell, false).await? {
            Some(_) => {
                info!(
                    cell = %cell,
                    base_url = %self.base_url.as_ref().unwrap(),
                    retry_attempts = self.config.retry_attempts,
                    "CSRF token obtained successfully"
                );
            }
            #[allow(non_snake_case)]
            None => {
                warn!(
                    retry_attempts = self.config.retry_attempts,
                    "Failed to obtain initial CSRF token, will retry on first authenticated request"
                );
            }
        }

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

    /// Extract Gong cell identifier from cookies using multiple strategies
    fn extract_cell_from_cookies(&self, cookies: &[Cookie]) -> Result<String> {
        // Strategy 1: Decode from JWT "cell" cookie
        for cookie in cookies {
            if cookie.name == "cell" {
                if let Ok(cell) = self.decode_cell_cookie(&cookie.value) {
                    debug!(cell = %cell, strategy = "jwt_decode", "Extracted cell from JWT cookie");
                    return Ok(cell);
                }
            }
        }

        // Strategy 2: Extract from cookie domains
        let domains: Vec<String> = cookies.iter().map(|c| c.domain.clone()).collect();
        if let Ok(cell) = self.extract_cell_from_domains(&domains) {
            debug!(cell = %cell, strategy = "domain_parsing", "Extracted cell from domain names");
            return Ok(cell);
        }

        Err(CsCliError::Authentication(
            "Could not determine Gong cell identifier from cookies".to_string(),
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

    /// Extract cell value from cookie domain names using regex patterns (public for testing)
    pub fn extract_cell_from_domains(&self, domains: &[String]) -> Result<String> {
        for domain in domains {
            if !domain.to_lowercase().contains("gong") {
                continue;
            }

            debug!(domain = %domain, "Analyzing domain for cell identifier");

            // Pattern 1: Direct cell domains like us-14496.app.gong.io
            if domain.contains(".app.gong") && !domain.starts_with("resource.") {
                let cell_part = domain
                    .split(".app.gong")
                    .next()
                    .unwrap_or("")
                    .replace('.', "");

                if !cell_part.is_empty()
                    && !cell_part.starts_with("resource")
                    && !cell_part.starts_with("gcell")
                    && !["app", "www", "api"].contains(&cell_part.as_str())
                {
                    debug!(cell = %cell_part, domain = %domain, "Found cell from domain pattern");
                    return Ok(cell_part);
                }
            }

            // Pattern 2: gcell patterns like resource.gcell-nam-01.app.gong.io
            if domain.contains("gcell-") {
                for part in domain.split('.') {
                    if part.starts_with("gcell-") {
                        let cell_value = part.strip_prefix("gcell-").unwrap_or("");
                        if !cell_value.is_empty() {
                            debug!(cell = %cell_value, domain = %domain, "Found gcell from domain");
                            return Ok(cell_value.to_string());
                        }
                    }
                }
            }

            // Pattern 3: Cell-like patterns in domain parts
            for part in domain.split('.') {
                if part.len() > 3
                    && part.chars().any(|c| c.is_ascii_digit())
                    && part.chars().any(|c| c.is_alphabetic())
                    && !["gong", "app", "www", "api", "resource"].contains(&part)
                {
                    debug!(cell = %part, domain = %domain, "Found potential cell from domain part");
                    return Ok(part.to_string());
                }
            }
        }

        Err(CsCliError::Authentication(
            "Could not extract cell identifier from any domain".to_string(),
        ))
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
