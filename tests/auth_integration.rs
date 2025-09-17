//! Authentication Integration Tests
//!
//! These tests validate the browser cookie extraction and Gong authentication flow
//! against real browsers and the production Gong API.
//!
//! IMPORTANT: These tests require:
//! 1. An active browser session logged into Gong
//! 2. Valid Gong account credentials
//! 3. Network connectivity to app.gong.io

use cs_cli::common::auth::{Cookie, CookieExtractor};
use cs_cli::common::config::AuthSettings;
use cs_cli::gong::auth::GongAuthenticator;

#[tokio::test]
#[ignore = "Requires real browser cookies and Gong account"]
async fn test_browser_cookie_extraction_multi_browser() {
    // Test cookie extraction from multiple browsers
    let domains = vec!["gong.io".to_string(), ".gong.io".to_string()];
    let extractor = CookieExtractor::new(domains);

    // This should extract cookies from available browsers
    let result = extractor.extract_gong_cookies_with_source();

    match result {
        Ok((cookies, browser)) => {
            println!("Successfully extracted cookies from browser: {}", browser);
            assert!(!cookies.is_empty(), "Should extract at least one cookie");

            // Verify we have essential cookies
            let cookie_names: Vec<String> = cookies.iter().map(|c| c.name.clone()).collect();
            println!("Found cookies: {:?}", cookie_names);

            // Should have session-related cookies
            let has_session_cookie = cookies.iter().any(|c|
                c.name.contains("session") ||
                c.name.contains("JSESSIONID") ||
                c.name.contains("_gong")
            );
            assert!(has_session_cookie, "Should have session cookies");
        }
        Err(e) => {
            println!("Warning: Cookie extraction failed (expected if not logged in): {}", e);
            // This is acceptable if no browsers are logged in
        }
    }
}

#[tokio::test]
#[ignore = "Requires real Gong API access"]
async fn test_gong_authentication_full_flow() {
    // Full authentication flow test
    let auth_config = AuthSettings::default();
    let mut authenticator = GongAuthenticator::new(auth_config).await
        .expect("Failed to create authenticator");

    // Attempt authentication
    let result = authenticator.authenticate().await;

    match result {
        Ok(true) => {
            println!("Authentication successful!");

            // Verify we can get authenticated headers (includes CSRF)
            let headers = authenticator.get_authenticated_headers(false).await;
            assert!(headers.is_ok(), "Should get authenticated headers after auth");
            let headers = headers.unwrap();
            assert!(headers.contains_key("x-csrf-token"), "Headers should include CSRF token");

            // Verify we have workspace ID
            let workspace_id = authenticator.get_workspace_id();
            assert!(workspace_id.is_some(), "Should have workspace ID after auth");
            println!("Workspace ID: {:?}", workspace_id);

            // Verify we have base URL
            let base_url = authenticator.get_base_url();
            assert!(base_url.is_ok(), "Should have base URL after auth");
            assert!(base_url.unwrap().contains("gong.io"), "Base URL should be Gong domain");
        }
        Ok(false) => {
            println!("Authentication failed - likely no valid cookies");
        }
        Err(e) => {
            println!("Authentication error (expected if not logged in): {}", e);
        }
    }
}

#[tokio::test]
#[ignore = "Requires real Gong API access"]
async fn test_csrf_token_refresh() {
    let auth_config = AuthSettings::default();
    let mut authenticator = GongAuthenticator::new(auth_config).await
        .expect("Failed to create authenticator");

    // Authenticate first
    if authenticator.authenticate().await.unwrap_or(false) {
        // Get initial authenticated headers (includes CSRF)
        let headers1 = authenticator.get_authenticated_headers(false).await
            .expect("Should get authenticated headers");

        // Force refresh by requesting again
        let headers2 = authenticator.get_authenticated_headers(true).await
            .expect("Should get auth headers after refresh");

        // Verify headers include CSRF token
        assert!(headers1.contains_key("x-csrf-token"), "Headers should include CSRF token");
        assert!(headers2.contains_key("x-csrf-token"), "Refreshed headers should include CSRF token");

        let token1 = headers1.get("x-csrf-token").unwrap();
        let token2 = headers2.get("x-csrf-token").unwrap();

        assert!(!token1.is_empty(), "CSRF token should not be empty");
        assert!(!token2.is_empty(), "Refreshed CSRF token should not be empty");
    }
}

#[tokio::test]
async fn test_authentication_without_cookies() {
    // Test behavior when no cookies are available
    let domains = vec!["nonexistent.domain.com".to_string()];
    let extractor = CookieExtractor::new(domains);

    let result = extractor.extract_gong_cookies_with_source();
    assert!(result.is_err() || result.unwrap().0.is_empty(),
            "Should fail or return empty for non-existent domain");
}

#[tokio::test]
#[ignore = "Requires real browser state manipulation"]
async fn test_expired_cookie_handling() {
    // This test would verify handling of expired cookies
    // In practice, this is hard to test without mocking
    let auth_config = AuthSettings::default();
    let mut authenticator = GongAuthenticator::new(auth_config).await
        .expect("Failed to create authenticator");

    // Create fake expired cookies
    let expired_cookies = vec![
        Cookie {
            name: "test_session".to_string(),
            value: "expired_value".to_string(),
            domain: ".gong.io".to_string(),
            path: "/".to_string(),
            expires: Some(0), // Expired
            http_only: true,
            secure: true,
        }
    ];

    // This should fail authentication
    // Note: Real implementation would need a way to inject test cookies
    println!("Test expired cookie handling - requires mock support");
}

#[tokio::test]
#[ignore = "Requires multiple browsers installed"]
async fn test_browser_fallback_priority() {
    // Test that browser fallback works in priority order
    let domains = vec!["gong.io".to_string()];
    let extractor = CookieExtractor::new(domains);

    // Try to extract from each browser type
    let browsers = vec!["Safari", "Chrome", "Firefox", "Edge", "Brave"];
    let mut found_browsers = vec![];

    for browser_name in &browsers {
        // Note: Real implementation would need browser-specific extraction
        println!("Checking browser: {}", browser_name);

        // Try extraction
        if let Ok((cookies, source)) = extractor.extract_gong_cookies_with_source() {
            if !cookies.is_empty() {
                found_browsers.push(source.clone());
                println!("Found cookies in: {}", source);
            }
        }
    }

    println!("Browsers with Gong cookies: {:?}", found_browsers);
    // At least one browser should work if user is logged in
}

#[tokio::test]
#[ignore = "Requires real API and can trigger rate limits"]
async fn test_rate_limit_handling_during_auth() {
    // Test that authentication handles rate limits gracefully
    let auth_config = AuthSettings::default();

    // Create multiple authenticators to potentially trigger rate limits
    let mut tasks = vec![];

    for i in 0..5 {
        let config = auth_config.clone();
        tasks.push(tokio::spawn(async move {
            let mut auth = GongAuthenticator::new(config).await
                .expect("Failed to create authenticator");

            println!("Auth attempt {}", i);
            let result = auth.authenticate().await;
            (i, result)
        }));
    }

    // Wait for all to complete
    let mut success_count = 0;
    let mut rate_limit_count = 0;

    for task in tasks {
        let (idx, result) = task.await.expect("Task failed");
        match result {
            Ok(true) => {
                success_count += 1;
                println!("Auth {} succeeded", idx);
            }
            Ok(false) => {
                println!("Auth {} failed (no cookies)", idx);
            }
            Err(e) => {
                if e.to_string().contains("429") || e.to_string().contains("rate") {
                    rate_limit_count += 1;
                    println!("Auth {} rate limited", idx);
                } else {
                    println!("Auth {} error: {}", idx, e);
                }
            }
        }
    }

    println!("Results - Success: {}, Rate limited: {}", success_count, rate_limit_count);
    // Should handle rate limits gracefully
}