//! Extracted authentication flow methods to eliminate DRY violations
//!
//! This module contains the broken-down authentication methods that were
//! previously part of the massive authenticate() method in authenticator.rs

use super::GongAuthenticator;
use crate::common::auth::cookies::{Cookie, CookieRetrieval};
use crate::common::http::client::HttpClient;
use crate::Result;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};

impl GongAuthenticator {
    /// Main authentication orchestrator - now simplified and focused
    pub async fn authenticate(&mut self) -> Result<bool> {
        info!("Starting Gong authentication");

        // Try authentication methods in order of preference
        if self.try_hybrid_storage_authentication().await? {
            info!("Successfully authenticated using stored session data");
            return Ok(true);
        }

        if self.try_browser_cookies_authentication().await? {
            info!("Successfully authenticated using browser cookies");
            return Ok(true);
        }

        // Fall back to guided authentication
        self.try_guided_authentication().await
    }

    /// Try to authenticate using browser cookies (Firefox, Safari, etc.)
    pub async fn try_browser_cookies_authentication(&mut self) -> Result<bool> {
        info!("Checking for existing browser cookies...");

        // Use the new cookie retrieval trait
        let cookies = match self.cookie_retriever.retrieve_cookies().await {
            Ok(cookies) => cookies,
            Err(e) => {
                warn!("Browser cookie retrieval failed: {}", e);
                return Ok(false);
            }
        };

        if cookies.is_empty() {
            debug!("No browser cookies found");
            return Ok(false);
        }

        info!("Found {} browser cookies, attempting authentication", cookies.len());

        // Try to authenticate with the retrieved cookies
        self.authenticate_with_cookies(cookies, "Browser Cookies").await
    }


    /// Authenticate using a set of cookies from any source
    async fn authenticate_with_cookies(
        &mut self,
        cookies: Vec<Cookie>,
        source: &str,
    ) -> Result<bool> {
        if cookies.is_empty() {
            return Ok(false);
        }

        info!("Attempting authentication with {} cookies from {}", cookies.len(), source);

        // Extract cell identifier from cookies
        let cell = match self.retrieve_cell_from_cookies(&cookies) {
            Ok(cell) => {
                info!("Successfully extracted Gong cell: {}", cell);
                cell
            }
            Err(e) => {
                debug!("Could not retrieve cell from {} cookies: {}", source, e);
                return Ok(false);
            }
        };

        // Build session cookies map
        let mut session_cookies = HashMap::new();
        for cookie in &cookies {
            session_cookies.insert(cookie.name.clone(), cookie.value.clone());
        }

        // Set up authentication state
        self.setup_authentication_state(cell.clone(), session_cookies, source.to_string()).await?;

        // Validate cookies with CSRF token
        if self.validate_cookies_with_csrf(&cell).await? {
            // Retrieve workspace ID
            self.retrieve_and_set_workspace_id().await?;
            info!("Authentication completed successfully with {}", source);
            Ok(true)
        } else {
            warn!("Cookie validation failed for {}", source);
            self.clear_authentication_state();
            Ok(false)
        }
    }

    /// Set up the authentication state with cookies and cell
    async fn setup_authentication_state(
        &mut self,
        cell: String,
        session_cookies: HashMap<String, String>,
        source: String,
    ) -> Result<()> {
        // Store authentication state
        self.gong_cookies = Some(super::GongCookies {
            cell: cell.clone(),
            session_cookies: session_cookies.clone(),
            retrieved_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
            browser: source,
        });

        self.base_url = Some(format!("https://{cell}.app.gong.io"));

        // Set cookies on HTTP client
        info!("Setting cookies on HTTP client for validation...");
        self.http_client
            .set_cookies(session_cookies)
            .await?;

        Ok(())
    }

    /// Validate cookies by attempting to get a CSRF token
    async fn validate_cookies_with_csrf(&self, cell: &str) -> Result<bool> {
        info!("Attempting CSRF token validation with 5-second timeout...");
        
        match tokio::time::timeout(
            std::time::Duration::from_secs(5),
            self.csrf_manager.get_csrf_token(cell, false),
        )
        .await
        {
            Ok(Ok(Some(_token))) => {
                info!("CSRF token validation successful");
                Ok(true)
            }
            Ok(Ok(None)) | Ok(Err(_)) | Err(_) => {
                warn!("CSRF token validation failed - cookies may be expired");
                Ok(false)
            }
        }
    }

    /// Retrieve workspace ID and set it in the authenticator
    async fn retrieve_and_set_workspace_id(&mut self) -> Result<()> {
        if let Ok(Some(workspace_id)) = self.retrieve_workspace_id().await {
            self.workspace_id = Some(workspace_id);
            info!("Workspace ID retrieved successfully");
        } else {
            warn!("Could not retrieve workspace ID - some API calls may fail");
        }
        Ok(())
    }

    /// Clear authentication state on failure
    fn clear_authentication_state(&mut self) {
        self.gong_cookies = None;
        self.base_url = None;
    }

}