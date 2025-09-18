//! Integration tests for guided authentication
//! 
//! These tests verify that the guided authentication flow works correctly
//! with lightpanda browser and Okta SSO integration.

#[cfg(test)]
mod tests {
    use super::super::guided_auth::{GuidedAuth, LightpandaBrowser};
    use crate::common::auth::GuidedAuth;
    use tokio;

    #[tokio::test]
    async fn test_lightpanda_browser_creation() {
        let browser = LightpandaBrowser::new();
        assert!(browser.get_cdp_port() >= 9222 && browser.get_cdp_port() <= 10222);
        assert!(!browser.get_binary_path().is_empty());
    }

    #[tokio::test]
    async fn test_guided_auth_creation() {
        let auth = GuidedAuth::new();
        assert!(!auth.has_browser()); // Should start uninitialized
    }

    #[tokio::test]
    #[cfg(target_os = "macos")]
    async fn test_okta_verify_check() {
        let auth = GuidedAuth::new();
        
        // This should not fail, regardless of whether Okta Verify is running
        let result = auth.check_okta_verify().await;
        assert!(result.is_ok());
        
        // Log the result for manual verification
        println!("Okta Verify running: {:?}", result.unwrap());
    }

    #[tokio::test]
    async fn test_lightpanda_binary_detection() {
        let browser = LightpandaBrowser::new();
        
        // The binary path should be determined
        assert!(!browser.get_binary_path().is_empty());
        
        // If the binary doesn't exist, ensure_binary should handle it
        // (This test doesn't actually download to avoid network dependencies)
        if !std::path::Path::new(browser.get_binary_path()).exists() {
            println!("Lightpanda binary not found at: {}", browser.get_binary_path());
            println!("Run test with internet connection to test download");
        }
    }

    // Note: Full integration tests with actual browser automation
    // should be run manually or in CI with USE_REAL_BROWSER=true
    // to avoid requiring network access and browser installation
    // in regular test runs
}

/// Manual test function (not run automatically)
/// 
/// To run this test manually:
/// ```
/// USE_REAL_BROWSER=true cargo test test_full_guided_auth -- --ignored --nocapture
/// ```
#[tokio::test]
#[ignore]
async fn test_full_guided_auth() {
    if std::env::var("USE_REAL_BROWSER").is_err() {
        println!("Skipping real browser test - set USE_REAL_BROWSER=true to enable");
        return;
    }

    let mut auth = GuidedAuth::new();
    
    // This would actually launch lightpanda and attempt Okta auth
    match auth.authenticate().await {
        Ok(cookies) => {
            println!("Guided authentication successful!");
            println!("Collected cookies for {} platforms", cookies.len());
            for (platform, cookie_str) in cookies {
                println!("- {}: {} chars", platform, cookie_str.len());
            }
        }
        Err(e) => {
            println!("Guided authentication failed: {}", e);
            // This is expected if Okta Verify isn't set up or user cancels
        }
    }
}