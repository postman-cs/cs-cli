//! Basic Slack API client using common abstractions
//!
//! Simple client for testing Slack workspace access without Qdrant complexity

use crate::common::auth::SessionManager;
use crate::common::config::HttpSettings;
use crate::common::http::{BrowserHttpClient, HttpClient};
use crate::slack::auth::SlackAuth;
use crate::{CsCliError, Result};
use serde_json::Value;
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Basic Slack API client
pub struct SlackClient {
    auth: SlackAuth,
    http_client: Option<BrowserHttpClient>,
}

impl SlackClient {
    /// Create new Slack client for a workspace
    pub fn new(workspace_domain: String) -> Self {
        let auth = SlackAuth::new(workspace_domain);

        Self {
            auth,
            http_client: None,
        }
    }

    /// Initialize authentication and HTTP client
    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing Slack client...");

        // Authenticate using Firefox session
        self.auth.authenticate().await?;

        // Create HTTP client with browser compatibility - match detected browser
        let browser_type = self.auth.detected_browser.as_deref().unwrap_or("firefox");

        let http_config = HttpSettings {
            pool_size: 1,
            max_concurrency_per_client: 5,
            timeout_seconds: 30.0,
            max_clients: Some(1),
            global_max_concurrency: Some(5),
            enable_http3: true, // Use HTTP/3 for better performance
            force_http3: false, // Allow fallback to HTTP/2
            tls_version: None,
            browser_type: browser_type.to_string(),
        };

        info!("Client HTTP using browser type: {}", browser_type);

        let client = BrowserHttpClient::new(http_config).await?;

        // Set authentication data
        if let Some(session) = self.auth.get_session_data() {
            client.set_cookies(session.cookies.clone()).await?;
            client.set_headers(self.auth.get_auth_headers()).await?;
        }

        self.http_client = Some(client);

        info!("Slack client initialized successfully");
        Ok(())
    }

    /// Test authentication with Slack's auth.test endpoint
    pub async fn auth_test(&self) -> Result<Value> {
        let client = self
            .http_client
            .as_ref()
            .ok_or_else(|| CsCliError::Generic("HTTP client not initialized".to_string()))?;

        let session = self
            .auth
            .get_session_data()
            .ok_or_else(|| CsCliError::Authentication("No active session".to_string()))?;

        // Prepare auth.test request data
        let form_data = format!("token={}", session.xoxc_token);

        let response = client
            .post(
                &format!("{}/api/auth.test", session.workspace_url),
                Some(&form_data),
            )
            .await?;

        let json: Value = response.json().await.map_err(|e| {
            CsCliError::ApiRequest(format!("Failed to parse auth.test response: {}", e))
        })?;

        debug!("auth.test response: {}", json);
        Ok(json)
    }

    /// Get basic workspace info
    pub async fn get_workspace_info(&self) -> Result<HashMap<String, String>> {
        let auth_result = self.auth_test().await?;

        let mut info = HashMap::new();

        if let Some(obj) = auth_result.as_object() {
            if let Some(team) = obj.get("team").and_then(|v| v.as_str()) {
                info.insert("team_name".to_string(), team.to_string());
            }
            if let Some(user) = obj.get("user").and_then(|v| v.as_str()) {
                info.insert("user_name".to_string(), user.to_string());
            }
            if let Some(user_id) = obj.get("user_id").and_then(|v| v.as_str()) {
                info.insert("user_id".to_string(), user_id.to_string());
            }
            if let Some(team_id) = obj.get("team_id").and_then(|v| v.as_str()) {
                info.insert("team_id".to_string(), team_id.to_string());
            }
            if let Some(url) = obj.get("url").and_then(|v| v.as_str()) {
                info.insert("workspace_url".to_string(), url.to_string());
            }
        }

        Ok(info)
    }

    /// Simple test to get conversations list (channels)
    pub async fn get_conversations(&self) -> Result<Value> {
        let client = self
            .http_client
            .as_ref()
            .ok_or_else(|| CsCliError::Generic("HTTP client not initialized".to_string()))?;

        let session = self
            .auth
            .get_session_data()
            .ok_or_else(|| CsCliError::Authentication("No active session".to_string()))?;

        // Use client.counts endpoint (reliable way to get conversations)
        let form_data = format!("token={}", session.xoxc_token);

        let response = client
            .post(
                &format!("{}/api/client.counts", session.workspace_url),
                Some(&form_data),
            )
            .await?;

        let json: Value = response.json().await.map_err(|e| {
            CsCliError::ApiRequest(format!("Failed to parse conversations response: {}", e))
        })?;

        debug!("client.counts response: {}", json);
        Ok(json)
    }

    /// Test basic functionality
    pub async fn test_basic_functionality(&mut self) -> Result<()> {
        println!("Testing Slack integration using common abstractions...");

        // Test 1: Authentication
        println!("1. Testing authentication...");
        self.initialize().await?;
        println!("   Authentication successful");

        // Test 2: Auth test API call
        println!("2. Testing auth.test API call...");
        let auth_info = self.auth_test().await?;
        if auth_info
            .get("ok")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            println!("   API authentication successful");

            let workspace_info = self.get_workspace_info().await?;
            if let Some(team) = workspace_info.get("team_name") {
                println!("   📍 Workspace: {}", team);
            }
            if let Some(user) = workspace_info.get("user_name") {
                println!("   👤 User: {}", user);
            }
        } else {
            return Err(CsCliError::Authentication(
                "auth.test returned ok=false".to_string(),
            ));
        }

        // Test 3: Get conversations
        println!("3. Testing conversations list...");
        let conversations = self.get_conversations().await?;
        if let Some(channels) = conversations.get("channels").and_then(|v| v.as_array()) {
            println!("   Found {} conversations", channels.len());

            // Show a few channel names
            for (i, channel) in channels.iter().take(3).enumerate() {
                if let Some(name) = channel.get("name").and_then(|v| v.as_str()) {
                    println!("   📱 {}: #{}", i + 1, name);
                }
            }
        } else {
            warn!("No channels found in response");
        }

        println!("Basic Slack integration test completed successfully");
        println!("    Common abstractions working properly:");
        println!("    common::auth::CookieRetriever - Firefox cookie retrieval");
        println!("    common::auth::SessionManager - Session management trait");
        println!("    common::http::BrowserHttpClient - Browser-impersonating HTTP client");
        println!("    common::config::HttpSettings - HTTP configuration");

        Ok(())
    }
}
