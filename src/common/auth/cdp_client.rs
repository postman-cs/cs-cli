//! Chrome DevTools Protocol client for lightpanda browser automation
//! 
//! This module provides a simple CDP client for communicating with lightpanda browser
//! through WebSocket connections. It handles the core CDP commands needed for
//! Okta authentication automation.

use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
use anyhow::{Result, Context, anyhow};
use serde_json::{Value, json};
use tokio::net::TcpStream;
use tokio::time::{timeout, sleep};
use tokio_tungstenite::{connect_async, WebSocketStream, MaybeTlsStream};
use tokio_tungstenite::tungstenite::Message;
use futures_util::{SinkExt, StreamExt};
use tracing::{debug, info};

/// CDP method results
#[derive(Debug, Clone)]
pub struct CdpResponse {
    pub id: u32,
    pub result: Option<Value>,
    pub error: Option<Value>,
}

/// CDP event notification
#[derive(Debug, Clone)]
pub struct CdpEvent {
    pub method: String,
    pub params: Value,
}

/// Simple CDP client for browser automation
pub struct CdpClient {
    ws: Option<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    command_id: AtomicU32,
}

impl CdpClient {
    /// Create new CDP client
    pub fn new() -> Self {
        Self {
            ws: None,
            command_id: AtomicU32::new(1),
        }
    }

    /// Connect to browser via WebSocket
    pub async fn connect(&mut self, ws_url: &str) -> Result<()> {
        info!("Connecting to CDP WebSocket: {}", ws_url);
        
        let (ws_stream, _) = connect_async(ws_url).await
            .context("Failed to connect to CDP WebSocket")?;
        
        self.ws = Some(ws_stream);
        
        // Enable required CDP domains
        self.enable_domains().await?;
        
        info!("CDP client connected successfully");
        Ok(())
    }

    /// Enable required CDP domains
    async fn enable_domains(&mut self) -> Result<()> {
        // Try to create a target first (may be required for lightpanda)
        match self.send_command("Target.createTarget", json!({
            "url": "about:blank"
        })).await {
            Ok(result) => debug!("Target created: {:?}", result),
            Err(_) => debug!("Target creation failed, continuing with default context"),
        }
        
        // Create browser context (required for lightpanda)
        match self.send_command("Browser.createBrowserContext", json!({})).await {
            Ok(_) => debug!("Browser context created successfully"),
            Err(_) => debug!("Browser context creation failed, continuing anyway"),
        }
        
        // Enable Page domain for navigation
        self.send_command("Page.enable", json!({})).await?;
        
        // Enable Runtime domain for JavaScript execution
        self.send_command("Runtime.enable", json!({})).await?;
        
        // Enable Network domain for cookie management
        self.send_command("Network.enable", json!({})).await?;
        
        Ok(())
    }

    /// Send CDP command and wait for response
    pub async fn send_command(&mut self, method: &str, params: Value) -> Result<Value> {
        let id = self.command_id.fetch_add(1, Ordering::SeqCst);
        
        let command = json!({
            "id": id,
            "method": method,
            "params": params
        });

        debug!("Sending CDP command: {} (id: {})", method, id);
        
        let ws = self.ws.as_mut()
            .ok_or_else(|| anyhow!("CDP client not connected"))?;

        // Send command
        ws.send(Message::Text(command.to_string())).await
            .context("Failed to send CDP command")?;

        // Wait for response
        let response = timeout(Duration::from_secs(30), self.wait_for_response(id)).await
            .context("CDP command timeout")?
            .context("Failed to receive CDP response")?;

        if let Some(error) = response.error {
            return Err(anyhow!("CDP command failed: {}", error));
        }

        Ok(response.result.unwrap_or(Value::Null))
    }

    /// Wait for specific response ID
    async fn wait_for_response(&mut self, expected_id: u32) -> Result<CdpResponse> {
        let ws = self.ws.as_mut()
            .ok_or_else(|| anyhow!("CDP client not connected"))?;

        while let Some(message) = ws.next().await {
            let message = message.context("WebSocket error")?;
            
            if let Message::Text(text) = message {
                let parsed: Value = serde_json::from_str(&text)
                    .context("Failed to parse CDP message")?;

                // Check if this is a response to our command
                if let Some(id) = parsed.get("id").and_then(|v| v.as_u64()) {
                    if id as u32 == expected_id {
                        return Ok(CdpResponse {
                            id: id as u32,
                            result: parsed.get("result").cloned(),
                            error: parsed.get("error").cloned(),
                        });
                    }
                } else if let Some(method) = parsed.get("method").and_then(|v| v.as_str()) {
                    // This is an event, log it but continue waiting
                    debug!("Received CDP event: {}", method);
                }
            }
        }

        Err(anyhow!("WebSocket closed while waiting for response"))
    }

    /// Navigate to URL
    pub async fn navigate(&mut self, url: &str) -> Result<()> {
        info!("Navigating to: {}", url);
        
        let params = json!({
            "url": url
        });

        self.send_command("Page.navigate", params).await?;
        
        // Wait for load complete
        self.wait_for_load().await?;
        
        Ok(())
    }

    /// Wait for page to load
    async fn wait_for_load(&mut self) -> Result<()> {
        info!("Waiting for page to load...");
        
        // Give the page time to load
        sleep(Duration::from_secs(3)).await;
        
        // Check if page is loaded by evaluating document.readyState
        let result = self.evaluate_js("document.readyState").await?;
        
        if let Some(state) = result.get("result").and_then(|r| r.get("value")).and_then(|v| v.as_str()) {
            if state == "complete" {
                info!("Page loaded successfully");
                return Ok(());
            }
        }
        
        // Wait a bit more if not complete
        sleep(Duration::from_secs(2)).await;
        info!("Page load completed (with timeout)");
        Ok(())
    }

    /// Execute JavaScript and return result
    pub async fn evaluate_js(&mut self, expression: &str) -> Result<Value> {
        debug!("Evaluating JavaScript: {}", expression);
        
        let params = json!({
            "expression": expression,
            "returnByValue": true
        });

        let result = self.send_command("Runtime.evaluate", params).await?;
        Ok(result)
    }

    /// Click element by selector
    pub async fn click_element(&mut self, selector: &str) -> Result<()> {
        info!("Clicking element: {}", selector);
        
        // First check if element exists
        let check_js = format!(
            "document.querySelector('{}') !== null", 
            selector.replace('\'', "\\'")
        );
        
        let exists = self.evaluate_js(&check_js).await?;
        if !exists.get("result")
            .and_then(|r| r.get("value"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false) {
            return Err(anyhow!("Element not found: {}", selector));
        }

        // Click the element
        let click_js = format!(
            "document.querySelector('{}').click()", 
            selector.replace('\'', "\\'")
        );
        
        self.evaluate_js(&click_js).await?;
        
        info!("Successfully clicked element: {}", selector);
        sleep(Duration::from_millis(500)).await; // Brief pause after click
        
        Ok(())
    }

    /// Wait for element to appear
    pub async fn wait_for_element(&mut self, selector: &str, timeout_secs: u64) -> Result<()> {
        info!("Waiting for element: {} (timeout: {}s)", selector, timeout_secs);
        
        let check_js = format!(
            "document.querySelector('{}') !== null", 
            selector.replace('\'', "\\'")
        );

        for attempt in 1..=timeout_secs {
            let exists = self.evaluate_js(&check_js).await?;
            
            if exists.get("result")
                .and_then(|r| r.get("value"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false) {
                info!("Element found: {}", selector);
                return Ok(());
            }

            debug!("Waiting for element (attempt {}/{})", attempt, timeout_secs);
            sleep(Duration::from_secs(1)).await;
        }

        Err(anyhow!("Element not found within {} seconds: {}", timeout_secs, selector))
    }

    /// Get all cookies for current domain
    pub async fn get_cookies(&mut self) -> Result<Vec<Value>> {
        debug!("Getting cookies for current domain");
        
        let result = self.send_command("Network.getCookies", json!({})).await?;
        
        let cookies = result.get("cookies")
            .and_then(|c| c.as_array())
            .ok_or_else(|| anyhow!("No cookies found in response"))?;
        
        info!("Retrieved {} cookies", cookies.len());
        Ok(cookies.clone())
    }

    /// Get current page URL
    pub async fn get_current_url(&mut self) -> Result<String> {
        let result = self.evaluate_js("window.location.href").await?;
        
        let url = result.get("result")
            .and_then(|r| r.get("value"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Failed to get current URL"))?;
            
        Ok(url.to_string())
    }

    /// Wait for URL to change (useful for redirects)
    pub async fn wait_for_url_change(&mut self, current_url: &str, timeout_secs: u64) -> Result<String> {
        info!("Waiting for URL to change from: {} (timeout: {}s)", current_url, timeout_secs);
        
        for attempt in 1..=timeout_secs {
            let new_url = self.get_current_url().await?;
            
            if new_url != current_url {
                info!("URL changed to: {}", new_url);
                return Ok(new_url);
            }

            debug!("Waiting for URL change (attempt {}/{})", attempt, timeout_secs);
            sleep(Duration::from_secs(1)).await;
        }

        Err(anyhow!("URL did not change within {} seconds", timeout_secs))
    }

    /// Check if element contains text
    pub async fn element_contains_text(&mut self, selector: &str, text: &str) -> Result<bool> {
        let js = format!(
            "document.querySelector('{}')?.textContent?.includes('{}')", 
            selector.replace('\'', "\\'"),
            text.replace('\'', "\\'")
        );
        
        let result = self.evaluate_js(&js).await?;
        
        Ok(result.get("result")
            .and_then(|r| r.get("value"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false))
    }
}

impl Default for CdpClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cdp_client_creation() {
        let client = CdpClient::new();
        assert_eq!(client.command_id.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_cdp_response_parsing() {
        let response = CdpResponse {
            id: 1,
            result: Some(json!({"success": true})),
            error: None,
        };
        
        assert_eq!(response.id, 1);
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }
}