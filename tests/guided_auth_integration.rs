//! Guided Authentication Integration Tests
//! 
//! Tests the actual guided authentication flow using the real implementation.
//! These tests will launch lightpanda browser and attempt Okta SSO authentication.
//!
//! Run with: cargo test guided_auth --test guided_auth_integration -- --ignored --nocapture

use tokio;

// Import the actual guided auth implementation
use cs_cli::common::auth::GuidedAuth;

/// Test the complete guided authentication flow
/// 
/// This test will:
/// 1. Launch lightpanda browser 
/// 2. Navigate to postman.okta.com
/// 3. Click Okta Verify authentication option
/// 4. Wait for user TouchID authentication
/// 5. Navigate to each platform (Gong, Slack, Gainsight, Salesforce)  
/// 6. Extract cookies from authenticated sessions
///
/// User interaction required: TouchID authentication when prompted
#[tokio::test]
#[ignore = "requires user interaction and Okta Verify app"]
async fn test_full_guided_authentication_flow() {
    println!("🚀 Starting guided authentication flow test...");
    println!("📱 Make sure Okta Verify app is installed and you're ready to authenticate with TouchID");
    
    let mut guided_auth = GuidedAuth::new();
    
    match guided_auth.authenticate().await {
        Ok(cookies) => {
            println!("✅ Guided authentication completed successfully!");
            println!("📊 Collected cookies for {} platforms", cookies.len());
            
            // Verify we got cookies for expected platforms
            let expected_platforms = ["gong", "slack", "gainsight", "salesforce"];
            for platform in expected_platforms {
                if let Some(cookie_data) = cookies.get(platform) {
                    println!("✅ {}: {} characters of cookie data", platform, cookie_data.len());
                    assert!(!cookie_data.is_empty(), "Cookie data should not be empty for {}", platform);
                } else {
                    println!("⚠️  {}: No cookie data collected", platform);
                }
            }
            
            assert!(!cookies.is_empty(), "Should have collected at least some cookies");
            println!("🎉 Test completed successfully - guided authentication works!");
        }
        Err(e) => {
            println!("❌ Guided authentication failed: {}", e);
            panic!("Guided authentication should succeed with user interaction: {}", e);
        }
    }
}

/// Test lightpanda browser launch without full auth flow
/// 
/// This test verifies the browser can be started and stopped correctly
/// without requiring user authentication.
#[tokio::test]
#[ignore = "downloads and extracts lightpanda binary"]  
async fn test_lightpanda_browser_lifecycle() {
    use cs_cli::common::auth::guided_auth::LightpandaBrowser;
    
    println!("🔧 Testing lightpanda browser lifecycle...");
    
    let mut browser = LightpandaBrowser::new();
    
    // Test binary extraction and browser startup
    match browser.start().await {
        Ok(()) => {
            println!("✅ Lightpanda browser started successfully");
            
            // Test that we can get a WebSocket URL for CDP
            let ws_url = browser.get_websocket_url();
            println!("✅ CDP WebSocket URL: {}", ws_url);
            assert!(ws_url.starts_with("ws://127.0.0.1:") || ws_url.starts_with("ws://localhost:"));
            
            // Test browser shutdown
            match browser.stop() {
                Ok(()) => {
                    println!("✅ Lightpanda browser stopped successfully");
                }
                Err(e) => {
                    println!("❌ Failed to stop browser: {}", e);
                    panic!("Should be able to stop browser: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to start lightpanda browser: {}", e);
            panic!("Should be able to start lightpanda browser: {}", e);
        }
    }
    
    println!("🎉 Browser lifecycle test completed successfully!");
}

/// Test binary extraction and decompression
/// 
/// This test verifies the embedded zstd-compressed lightpanda binary
/// can be extracted and decompressed correctly.
#[tokio::test]
async fn test_binary_extraction() {
    use cs_cli::common::auth::guided_auth::LightpandaBrowser;
    
    println!("📦 Testing binary extraction and decompression...");
    
    let mut browser = LightpandaBrowser::new();
    let binary_path = browser.get_binary_path().to_string();
    
    println!("📁 Target binary path: {}", binary_path);
    
    // Clean up any existing binary
    if std::path::Path::new(&binary_path).exists() {
        std::fs::remove_file(&binary_path).expect("Should be able to clean up existing binary");
    }
    
    // Test extraction
    match browser.ensure_binary().await {
        Ok(()) => {
            println!("✅ Binary extraction completed successfully");
            
            // Verify the binary exists and is executable
            assert!(std::path::Path::new(&binary_path).exists(), "Extracted binary should exist");
            
            let metadata = std::fs::metadata(&binary_path).expect("Should be able to read binary metadata");
            assert!(metadata.is_file(), "Should be a regular file");
            assert!(metadata.len() > 50_000_000, "Decompressed binary should be ~55MB"); // Original size check
            
            // On Unix systems, check if executable bit is set
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let permissions = metadata.permissions();
                assert!(permissions.mode() & 0o111 != 0, "Binary should be executable");
            }
            
            println!("✅ Binary verification passed - {} bytes", metadata.len());
            
            // Clean up
            std::fs::remove_file(&binary_path).expect("Should be able to clean up test binary");
            println!("🧹 Cleaned up extracted binary");
        }
        Err(e) => {
            println!("❌ Binary extraction failed: {}", e);
            panic!("Should be able to extract embedded binary: {}", e);
        }
    }
    
    println!("🎉 Binary extraction test completed successfully!");
}

/// Test chromiumoxide connection to lightpanda browser
/// 
/// This test starts lightpanda browser and verifies chromiumoxide can connect
/// and perform basic browser automation tasks.
#[tokio::test]
#[ignore = "requires lightpanda browser launch"]
async fn test_chromiumoxide_connection() {
    use cs_cli::common::auth::guided_auth::LightpandaBrowser;
    use chromiumoxide::Browser;
    use futures::StreamExt;
    
    println!("🔌 Testing chromiumoxide connection to lightpanda...");
    
    let mut browser = LightpandaBrowser::new();
    
    // Start browser
    browser.start().await.expect("Should be able to start browser");
    
    // Get WebSocket URL
    let ws_url = browser.get_websocket_url();
    println!("📡 Connecting to lightpanda WebSocket: {}", ws_url);
    
    // Connect chromiumoxide
    match Browser::connect(&ws_url).await {
        Ok((browser, mut handler)) => {
            println!("✅ Chromiumoxide connected successfully");
            
            // Spawn handler
            tokio::spawn(async move {
                while let Some(event) = handler.next().await {
                    if let Err(e) = event {
                        println!("Browser handler error: {}", e);
                    }
                }
            });
            
            // Test creating a blank page (more compatible with beta lightpanda)
            match browser.new_page("about:blank").await {
                Ok(page) => {
                    println!("✅ Page created successfully");
                    
                    // Test getting current URL (should be about:blank)
                    if let Some(url) = page.url().await.expect("Should be able to get URL") {
                        println!("✅ Current URL: {}", url);
                        assert!(url.contains("about:blank") || url.is_empty(), "Should be blank page or empty");
                    }
                    
                    // Test basic DOM manipulation (lightpanda supports DOM APIs)
                    match page.set_content("<html><body><h1>Test Page</h1></body></html>").await {
                        Ok(_) => {
                            println!("✅ DOM content set successfully");
                            
                            // Test getting page title
                            if let Ok(Some(title)) = page.get_title().await {
                                println!("✅ Page title: '{}'", title);
                            } else {
                                println!("ℹ️  No page title found (expected for basic HTML)");
                            }
                        }
                        Err(e) => {
                            println!("⚠️  DOM content setting failed: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("⚠️  Page creation failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Chromiumoxide connection failed: {}", e);
            browser.stop().expect("Should be able to stop browser");
            panic!("Should be able to connect to lightpanda: {}", e);
        }
    }
    
    // Clean up
    browser.stop().expect("Should be able to stop browser");
    println!("🎉 Chromiumoxide connection test completed successfully!");
}

/// Test Okta Verify app detection on macOS
#[cfg(target_os = "macos")]
#[tokio::test]
async fn test_okta_verify_detection() {
    use cs_cli::common::auth::GuidedAuth;
    
    println!("🔍 Testing Okta Verify app detection...");
    
    let guided_auth = GuidedAuth::new();
    
    match guided_auth.check_okta_verify().await {
        Ok(is_running) => {
            if is_running {
                println!("✅ Okta Verify is currently running");
            } else {
                println!("ℹ️  Okta Verify is not currently running");
                println!("💡 You can install Okta Verify from the App Store if needed");
            }
            
            // Test should pass regardless of whether Okta Verify is running
            println!("✅ Okta Verify detection completed successfully");
        }
        Err(e) => {
            println!("❌ Okta Verify detection failed: {}", e);
            panic!("Should be able to check Okta Verify status: {}", e);
        }
    }
}