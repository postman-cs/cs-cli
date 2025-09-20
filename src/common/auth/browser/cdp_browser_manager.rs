//! CDP Browser Manager for direct browser launching and automation
//!
//! This module provides a complete browser automation solution using Chrome DevTools Protocol (CDP)
//! It launches browsers directly with debug ports and connects via WebSocket.

use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
use anyhow::{Result, Context, anyhow};
use serde_json::{Value, json};
use tokio::process;
use tokio::net::TcpStream;
use tokio::time::{timeout, sleep};
use tokio_tungstenite::{connect_async, WebSocketStream, MaybeTlsStream};
use tokio_tungstenite::tungstenite::Message;
use futures::{SinkExt, StreamExt};
use tracing::{debug, info, warn};

use crate::common::error::{CsCliError, Result as CsResult};
use crate::common::auth::browser::profile_detector::{BrowserProfile, BrowserType};

#[derive(Debug, Clone, Copy)]
pub enum Browser {
    Chrome,
    Firefox,
    Safari,
    Brave,
    Edge,
    Arc,
    Chromium,
    LibreWolf,
    Opera,
    OperaGX,
    Vivaldi,
    Zen,
    Comet,
}

impl From<BrowserType> for Browser {
    fn from(browser_type: BrowserType) -> Self {
        match browser_type {
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
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Platform {
    MacosArm64,
    MacosX64,
    LinuxX64,
    WindowsX64,
}

impl Platform {
    pub fn current() -> CsResult<Self> {
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        return Ok(Self::MacosArm64);

        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        return Ok(Self::MacosX64);

        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        return Ok(Self::LinuxX64);

        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        return Ok(Self::WindowsX64);

        #[cfg(not(any(
            all(target_os = "macos", target_arch = "aarch64"),
            all(target_os = "macos", target_arch = "x86_64"),
            all(target_os = "linux", target_arch = "x86_64"),
            all(target_os = "windows", target_arch = "x86_64")
        )))]
        return Err(CsCliError::Generic("Unsupported platform".to_string()));
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MacosArm64 => "macos-arm64",
            Self::MacosX64 => "macos-x64",
            Self::LinuxX64 => "linux-x64",
            Self::WindowsX64 => "windows-x64",
        }
    }
}


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

/// Browser process information
#[derive(Debug)]
pub struct BrowserProcess {
    pub debug_port: u16,
    pub ws_url: String,
}

/// CDP Browser Manager that launches browsers and manages CDP connections
pub struct CdpBrowserManager {
    ws: Option<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    command_id: AtomicU32,
    browser_process: Option<BrowserProcess>,
    browser_child: Option<process::Child>,
    debug_port: u16,
}

impl CdpBrowserManager {
    /// Create new CDP browser manager
    pub fn new() -> Self {
        Self {
            ws: None,
            command_id: AtomicU32::new(1),
            browser_process: None,
            browser_child: None,
            debug_port: 9222, // Default Chrome debug port
        }
    }


    /// Launch browser with CDP debug port and connect
    pub async fn launch_browser(&mut self, browser: Browser, profile: Option<&BrowserProfile>) -> CsResult<()> {
        info!("Launching {:?} browser with CDP debug port", browser);
        
        // Find available debug port
        self.debug_port = self.find_available_debug_port().await?;
        
        // Launch browser process
        let (browser_process, child) = self.launch_browser_process(browser, profile).await?;
        
        // Store the child separately for cleanup
        self.browser_child = Some(child);
        
        // Register for global cleanup
        if let Some(_child) = &self.browser_child {
            // We can't move the child here, so we'll register it differently
            // For now, we'll skip the global cleanup registration
            // TODO: Implement proper cleanup registration
        }
        
        self.browser_process = Some(browser_process);
        
        // Wait for browser to start
        sleep(Duration::from_secs(2)).await;
        
        // Connect to CDP WebSocket
        let ws_url = format!("ws://localhost:{}/json/version", self.debug_port);
        self.connect_to_cdp(&ws_url).await?;
        
        info!("Successfully launched {:?} browser with CDP on port {}", browser, self.debug_port);
        Ok(())
    }

    /// Find available debug port starting from default
    async fn find_available_debug_port(&self) -> CsResult<u16> {
        let start_port = 9222;
        
        for port in start_port..start_port + 100 {
            if let Ok(_) = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}")).await {
                return Ok(port);
            }
        }
        
        Err(CsCliError::Generic("No available debug ports found".to_string()))
    }

    /// Launch browser process with CDP debug port
    async fn launch_browser_process(&self, browser: Browser, profile: Option<&BrowserProfile>) -> CsResult<(BrowserProcess, process::Child)> {
        let binary_path = Self::get_browser_binary_path(browser)
            .ok_or_else(|| CsCliError::Generic(format!("No binary path configured for {:?}", browser)))?;

        if !std::path::Path::new(binary_path).exists() {
            return Err(CsCliError::Generic(format!("Browser binary not found: {}", binary_path)));
        }

        let mut cmd = process::Command::new(binary_path);
        
        // Add CDP debug port
        cmd.arg(format!("--remote-debugging-port={}", self.debug_port));
        
        // Add browser-specific automation arguments
        let automation_args = Self::get_browser_automation_args(browser);
        for arg in automation_args {
            cmd.arg(arg);
        }
        
        // Add profile-specific arguments
        if let Some(profile) = profile {
            Self::add_profile_args(&mut cmd, browser, profile)?;
        }
        
        // Configure stdio redirection for TUI mode
        self.configure_stdio_for_tui(&mut cmd);
        
        info!("Launching browser: {} with debug port {}", binary_path, self.debug_port);
        
        let child = cmd.spawn()
            .map_err(|e| CsCliError::Generic(format!("Failed to launch browser: {e}")))?;
        
        let ws_url = format!("ws://localhost:{}/json/version", self.debug_port);
        
        Ok((BrowserProcess {
            debug_port: self.debug_port,
            ws_url,
        }, child))
    }

    /// Add profile-specific arguments to browser command
    fn add_profile_args(cmd: &mut process::Command, browser: Browser, profile: &BrowserProfile) -> CsResult<()> {
        match browser {
            Browser::Chrome | Browser::Brave | Browser::Edge | Browser::Arc | Browser::Chromium
            | Browser::Opera | Browser::OperaGX | Browser::Vivaldi | Browser::Comet => {
                // Set user data directory (parent of profile path)
                if let Some(chrome_dir) = profile.profile_path.parent() {
                    cmd.arg(format!("--user-data-dir={}", chrome_dir.display()));
                    info!("Setting user data directory: {}", chrome_dir.display());
                }
                
                // Set profile directory if not the default profile
                if profile.profile_id != "Default" {
                    cmd.arg(format!("--profile-directory={}", profile.profile_id));
                    info!("Setting profile directory: {}", profile.profile_id);
                }
            }
            _ => {
                warn!("Profile arguments not supported for {:?}, using default profile", browser);
            }
        }
        
        Ok(())
    }

    /// Configure stdio redirection for TUI mode
    fn configure_stdio_for_tui(&self, cmd: &mut process::Command) {
        if std::env::var("CS_CLI_TUI_MODE").is_ok() {
            match std::env::var("CS_CLI_LOG_FILE").ok().and_then(|path| {
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&path)
                    .ok()
            }) {
                Some(log_file) => {
                    cmd.stdout(log_file.try_clone().unwrap());
                    cmd.stderr(log_file);
                }
                None => {
                    cmd.stdout(std::process::Stdio::null());
                    cmd.stderr(std::process::Stdio::null());
                }
            }
        }
    }

    /// Connect to CDP WebSocket endpoint
    async fn connect_to_cdp(&mut self, ws_url: &str) -> CsResult<()> {
        info!("Connecting to CDP WebSocket: {}", ws_url);
        
        // First, get the actual WebSocket URL from the browser
        let actual_ws_url = self.get_cdp_websocket_url().await?;
        
        let (ws_stream, _) = connect_async(&actual_ws_url).await
            .map_err(|e| CsCliError::Generic(format!("Failed to connect to CDP WebSocket: {e}")))?;
        
        self.ws = Some(ws_stream);
        
        // Enable required CDP domains
        self.enable_domains().await?;
        
        info!("CDP client connected successfully");
        Ok(())
    }

    /// Get the actual WebSocket URL from browser's CDP endpoint
    async fn get_cdp_websocket_url(&self) -> CsResult<String> {
        let http_url = format!("http://localhost:{}/json/version", self.debug_port);
        
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| CsCliError::Generic(format!("Failed to create HTTP client: {e}")))?;
        
        let response = client.get(&http_url).send().await
            .map_err(|e| CsCliError::Generic(format!("Failed to get CDP info: {e}")))?;
        
        if !response.status().is_success() {
            return Err(CsCliError::Generic(format!("CDP endpoint returned status: {}", response.status())));
        }
        
        let info: Value = response.json().await
            .map_err(|e| CsCliError::Generic(format!("Failed to parse CDP info: {e}")))?;
        
        let ws_url = info.get("webSocketDebuggerUrl")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CsCliError::Generic("No WebSocket URL found in CDP info".to_string()))?;
        
        Ok(ws_url.to_string())
    }

    /// Enable required CDP domains
    async fn enable_domains(&mut self) -> CsResult<()> {
        // Enable Page domain for navigation
        self.send_command("Page.enable", json!({})).await?;
        
        // Enable Runtime domain for JavaScript execution
        self.send_command("Runtime.enable", json!({})).await?;
        
        // Enable Network domain for cookie management
        self.send_command("Network.enable", json!({})).await?;
        
        // Enable Target domain for context management
        self.send_command("Target.enable", json!({})).await?;
        
        Ok(())
    }

    /// Send CDP command and wait for response
    pub async fn send_command(&mut self, method: &str, params: Value) -> CsResult<Value> {
        let id = self.command_id.fetch_add(1, Ordering::SeqCst);
        
        let command = json!({
            "id": id,
            "method": method,
            "params": params
        });

        debug!("Sending CDP command: {} (id: {})", method, id);
        
        let ws = self.ws.as_mut()
            .ok_or_else(|| CsCliError::Generic("CDP client not connected".to_string()))?;

        // Send command
        ws.send(Message::Text(command.to_string())).await
            .map_err(|e| CsCliError::Generic(format!("Failed to send CDP command: {e}")))?;

        // Wait for response
        let response = timeout(Duration::from_secs(30), self.wait_for_response(id)).await
            .map_err(|_| CsCliError::Generic("CDP command timeout".to_string()))?
            .map_err(|e| CsCliError::Generic(format!("Failed to receive CDP response: {e}")))?;

        if let Some(error) = response.error {
            return Err(CsCliError::Generic(format!("CDP command failed: {}", error)));
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
    pub async fn navigate(&mut self, url: &str) -> CsResult<()> {
        info!("Navigating to: {}", url);
        
        let params = json!({
            "url": url
        });

        self.send_command("Page.navigate", params).await?;
        
        // Wait for load complete
        self.wait_for_load().await?;
        
        Ok(())
    }

    /// Alias for navigate method
    pub async fn goto(&mut self, url: &str) -> CsResult<()> {
        self.navigate(url).await
    }

    /// Wait for page to load
    async fn wait_for_load(&mut self) -> CsResult<()> {
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
    pub async fn evaluate_js(&mut self, expression: &str) -> CsResult<Value> {
        debug!("Evaluating JavaScript: {}", expression);
        
        let params = json!({
            "expression": expression,
            "returnByValue": true
        });

        let result = self.send_command("Runtime.evaluate", params).await?;
        Ok(result)
    }

    /// Click element by selector
    pub async fn click_element(&mut self, selector: &str) -> CsResult<()> {
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
            return Err(CsCliError::Generic(format!("Element not found: {}", selector)));
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
    pub async fn wait_for_element(&mut self, selector: &str, timeout_secs: u64) -> CsResult<()> {
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

        Err(CsCliError::Generic(format!("Element not found within {} seconds: {}", timeout_secs, selector)))
    }

    /// Get all cookies for current domain
    pub async fn get_cookies(&mut self) -> CsResult<Vec<Value>> {
        debug!("Getting cookies for current domain");
        
        let result = self.send_command("Network.getCookies", json!({})).await?;
        
        let cookies = result.get("cookies")
            .and_then(|c| c.as_array())
            .ok_or_else(|| CsCliError::Generic("No cookies found in response".to_string()))?;
        
        info!("Retrieved {} cookies", cookies.len());
        Ok(cookies.clone())
    }

    /// Get current page URL
    pub async fn get_current_url(&mut self) -> CsResult<String> {
        let result = self.evaluate_js("window.location.href").await?;
        
        let url = result.get("result")
            .and_then(|r| r.get("value"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| CsCliError::Generic("Failed to get current URL".to_string()))?;
            
        Ok(url.to_string())
    }

    /// Check if element contains text
    pub async fn element_contains_text(&mut self, selector: &str, text: &str) -> CsResult<bool> {
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

    /// Get browser binary path for a given browser type on macOS
    fn get_browser_binary_path(browser: Browser) -> Option<&'static str> {
        match browser {
            Browser::Chrome => Some("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"),
            Browser::Brave => Some("/Applications/Brave Browser.app/Contents/MacOS/Brave Browser"),
            Browser::Edge => Some("/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge"),
            Browser::Arc => Some("/Applications/Arc.app/Contents/MacOS/Arc"),
            Browser::Chromium => Some("/Applications/Chromium.app/Contents/MacOS/Chromium"),
            Browser::Opera => Some("/Applications/Opera.app/Contents/MacOS/Opera"),
            Browser::OperaGX => Some("/Applications/Opera GX.app/Contents/MacOS/Opera GX"),
            Browser::Vivaldi => Some("/Applications/Vivaldi.app/Contents/MacOS/Vivaldi"),
            Browser::Comet => Some("/Applications/Comet.app/Contents/MacOS/Comet"),
            Browser::Firefox => Some("/Applications/Firefox.app/Contents/MacOS/firefox"),
            Browser::LibreWolf => Some("/Applications/LibreWolf.app/Contents/MacOS/LibreWolf"),
            Browser::Zen => Some("/Applications/Zen.app/Contents/MacOS/Zen"),
            Browser::Safari => Some("/Applications/Safari.app/Contents/MacOS/Safari"),
        }
    }

    /// Get browser-specific automation arguments
    fn get_browser_automation_args(browser: Browser) -> Vec<String> {
        let mut args = Vec::new();

        // Essential stability arguments for all Chromium browsers
        args.push("--no-sandbox".to_string());
        args.push("--disable-dev-shm-usage".to_string());
        args.push("--disable-gpu".to_string());
        args.push("--disable-web-security".to_string());
        args.push("--no-first-run".to_string());
        args.push("--disable-default-apps".to_string());
        args.push("--disable-sync".to_string());
        args.push("--disable-translate".to_string());
        args.push("--disable-background-timer-throttling".to_string());
        args.push("--disable-backgrounding-occluded-windows".to_string());
        args.push("--disable-renderer-backgrounding".to_string());
        args.push("--disable-field-trial-config".to_string());
        args.push("--disable-ipc-flooding-protection".to_string());
        args.push("--disable-hang-monitor".to_string());
        args.push("--disable-prompt-on-repost".to_string());
        args.push("--disable-component-extensions-with-background-pages".to_string());
        args.push("--disable-extensions".to_string());
        args.push("--disable-plugins".to_string());
        args.push("--disable-plugins-discovery".to_string());
        args.push("--disable-print-preview".to_string());
        args.push("--disable-save-password-bubble".to_string());
        args.push("--disable-single-click-autofill".to_string());
        args.push("--disable-speech-api".to_string());
        args.push("--disable-speech-synthesis-api".to_string());
        args.push("--disable-xss-auditor".to_string());
        args.push("--disable-background-networking".to_string());
        args.push("--disable-client-side-phishing-detection".to_string());
        args.push("--disable-logging".to_string());
        args.push("--disable-notifications".to_string());
        args.push("--disable-popup-blocking".to_string());
        args.push("--disable-permissions-api".to_string());
        args.push("--metrics-recording-only".to_string());
        args.push("--safebrowsing-disable-auto-update".to_string());
        args.push("--enable-automation".to_string());
        args.push("--password-store=basic".to_string());
        args.push("--use-mock-keychain".to_string());
        args.push("--disable-blink-features=AutomationControlled".to_string());

        // Browser-specific arguments to prevent conflicts and crashes
        match browser {
            Browser::Brave => {
                // Brave-specific: minimal intervention - let Brave run with its default features
            }
            Browser::Edge => {
                args.push("--disable-edge-extensions".to_string());
                args.push("--disable-edge-sync".to_string());
                args.push("--disable-edge-translate".to_string());
                args.push("--disable-edge-news".to_string());
                args.push("--disable-edge-shopping".to_string());
                args.push("--disable-features=msEdgeShopping".to_string());
                args.push("--disable-features=msEdgeCollections".to_string());
            }
            Browser::Arc => {
                args.push("--disable-arc-extensions".to_string());
                args.push("--disable-arc-sync".to_string());
                args.push("--disable-arc-translate".to_string());
                args.push("--disable-features=ArcCollections".to_string());
                args.push("--disable-features=ArcSidebar".to_string());
            }
            Browser::Opera | Browser::OperaGX => {
                args.push("--disable-opera-extensions".to_string());
                args.push("--disable-opera-sync".to_string());
                args.push("--disable-opera-translate".to_string());
                args.push("--disable-opera-news".to_string());
            }
            Browser::Vivaldi => {
                args.push("--disable-vivaldi-extensions".to_string());
                args.push("--disable-vivaldi-sync".to_string());
                args.push("--disable-vivaldi-translate".to_string());
            }
            Browser::Comet => {
                args.push("--disable-comet-extensions".to_string());
                args.push("--disable-comet-sync".to_string());
            }
            Browser::Chrome | Browser::Chromium => {
                // Chrome/Chromium: minimal additional args needed
            }
            // Firefox-based browsers handled separately
            Browser::Firefox | Browser::LibreWolf | Browser::Zen | Browser::Safari => {}
        }

        args
    }

    /// Close browser and cleanup
    pub async fn close(&mut self) -> CsResult<()> {
        info!("Closing CDP browser manager");
        
        // Close WebSocket connection
        if let Some(mut ws) = self.ws.take() {
            let _ = ws.close(None).await;
        }
        
        // Kill browser process
        if let Some(mut child) = self.browser_child.take() {
            if let Err(e) = child.kill().await {
                warn!("Failed to kill browser process: {}", e);
            } else {
                info!("Successfully killed browser process");
            }
        }
        
        Ok(())
    }
}

impl Default for CdpBrowserManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for CdpBrowserManager {
    fn drop(&mut self) {
        // Clean up browser process if still running
        if let Some(child) = self.browser_child.take() {
            if let Some(pid) = child.id() {
                if let Err(e) = std::process::Command::new("kill")
                    .arg("-TERM")
                    .arg(pid.to_string())
                    .output()
                {
                    warn!("Failed to kill browser process {}: {}", pid, e);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cdp_browser_manager_creation() {
        let manager = CdpBrowserManager::new();
        assert_eq!(manager.command_id.load(Ordering::SeqCst), 1);
        assert_eq!(manager.debug_port, 9222);
    }

    #[test]
    fn test_browser_binary_paths() {
        assert!(CdpBrowserManager::get_browser_binary_path(Browser::Chrome).is_some());
        assert!(CdpBrowserManager::get_browser_binary_path(Browser::Brave).is_some());
        assert!(CdpBrowserManager::get_browser_binary_path(Browser::Edge).is_some());
    }

    #[test]
    fn test_automation_args() {
        let args = CdpBrowserManager::get_browser_automation_args(Browser::Chrome);
        assert!(args.contains(&"--no-sandbox".to_string()));
        assert!(args.contains(&"--disable-dev-shm-usage".to_string()));
        assert!(args.contains(&"--remote-debugging-port".to_string()));
    }
}