//! Authentication Integration Tests
//!
//! These tests validate the generic browser cookie extraction and platform-specific
//! authentication flows against real browsers and production APIs.
//!
//! IMPORTANT: These tests require:
//! 1. An active browser session logged into the tested service (Gong, etc.)
//! 2. Valid account credentials for the service
//! 3. Network connectivity to the service endpoints

use cs_cli::common::auth::{Cookie, CookieExtractor};
use cs_cli::common::config::AuthSettings;
use cs_cli::gong::auth::GongAuthenticator;

#[tokio::test]
async fn test_browser_cookie_extraction_multi_browser() {
    // Test generic cookie extraction from all available browsers
    
    // NOTE: This test may prompt for browser Safe Storage access on first run
    // Click "Allow Always" when prompted, then subsequent runs will be prompt-free
    println!("Testing cookie extraction (may prompt for browser Safe Storage access - click 'Allow Always')");
    
    let domains = vec!["gong.io".to_string(), ".gong.io".to_string()];
    let extractor = CookieExtractor::new(domains);

    // This should extract cookies from available browsers
    let all_browser_cookies = extractor.extract_all_browsers_cookies();

    if all_browser_cookies.is_empty() {
        println!("Warning: No cookies found in any browser (expected if not logged in)");
        return; // Skip test if no cookies available
    }

    for (cookies, browser) in all_browser_cookies {
        println!("Successfully extracted cookies from browser: {browser}");
        assert!(!cookies.is_empty(), "Should extract at least one cookie");

        // Verify we have essential cookies
        let cookie_names: Vec<String> = cookies.iter().map(|c| c.name.clone()).collect();
        println!("Found cookies: {cookie_names:?}");

        // Should have some domain-related cookies
        let has_domain_cookie = cookies.iter().any(|c| {
            c.domain.contains("gong") || c.name.contains("session") || c.name.contains("JSESSIONID")
        });
        if has_domain_cookie {
            println!("Found domain-specific cookies in {browser}");
        }
    }
}

#[tokio::test]

async fn test_gong_authentication_full_flow() {
    // Full authentication flow test
    let auth_config = AuthSettings::default();
    let mut authenticator = GongAuthenticator::new(auth_config)
        .await
        .expect("Failed to create authenticator");

    // Attempt authentication
    let result = authenticator.authenticate().await;

    match result {
        Ok(true) => {
            println!("Authentication successful!");

            // Verify we can get authenticated headers (includes CSRF)
            let headers = authenticator.get_authenticated_headers(false).await;
            assert!(
                headers.is_ok(),
                "Should get authenticated headers after auth"
            );
            let headers = headers.unwrap();
            assert!(
                headers.contains_key("x-csrf-token"),
                "Headers should include CSRF token"
            );

            // Verify we have workspace ID
            let workspace_id = authenticator.get_workspace_id();
            assert!(
                workspace_id.is_some(),
                "Should have workspace ID after auth"
            );
            println!("Workspace ID: {workspace_id:?}");

            // Verify we have base URL
            let base_url = authenticator.get_base_url();
            assert!(base_url.is_ok(), "Should have base URL after auth");
            assert!(
                base_url.unwrap().contains("gong.io"),
                "Base URL should be Gong domain"
            );
        }
        Ok(false) => {
            println!("Authentication failed - likely no valid cookies");
        }
        Err(e) => {
            println!("Authentication error (expected if not logged in): {e}");
        }
    }
}

#[tokio::test]

async fn test_csrf_token_refresh() {
    let auth_config = AuthSettings::default();
    let mut authenticator = GongAuthenticator::new(auth_config)
        .await
        .expect("Failed to create authenticator");

    // Authenticate first
    if authenticator.authenticate().await.unwrap_or(false) {
        // Get initial authenticated headers (includes CSRF)
        let headers1 = authenticator
            .get_authenticated_headers(false)
            .await
            .expect("Should get authenticated headers");

        // Force refresh by requesting again
        let headers2 = authenticator
            .get_authenticated_headers(true)
            .await
            .expect("Should get auth headers after refresh");

        // Verify headers include CSRF token
        assert!(
            headers1.contains_key("x-csrf-token"),
            "Headers should include CSRF token"
        );
        assert!(
            headers2.contains_key("x-csrf-token"),
            "Refreshed headers should include CSRF token"
        );

        let token1 = headers1.get("x-csrf-token").unwrap();
        let token2 = headers2.get("x-csrf-token").unwrap();

        assert!(!token1.is_empty(), "CSRF token should not be empty");
        assert!(
            !token2.is_empty(),
            "Refreshed CSRF token should not be empty"
        );
    }
}

#[tokio::test]
async fn test_authentication_without_cookies() {
    // Test behavior when no cookies are available
    let domains = vec!["nonexistent.domain.com".to_string()];
    let extractor = CookieExtractor::new(domains);

    let all_cookies = extractor.extract_all_browsers_cookies();
    assert!(
        all_cookies.is_empty(),
        "Should return empty for non-existent domain"
    );
}

#[tokio::test]

async fn test_expired_cookie_handling() {
    // This test would verify handling of expired cookies
    // In practice, this is hard to test without mocking
    let auth_config = AuthSettings::default();
    let _authenticator = GongAuthenticator::new(auth_config)
        .await
        .expect("Failed to create authenticator");

    // Create fake expired cookies
    let _expired_cookies = [Cookie {
        name: "test_session".to_string(),
        value: "expired_value".to_string(),
        domain: ".gong.io".to_string(),
        path: "/".to_string(),
        expires: Some(0), // Expired
        http_only: true,
        secure: true,
    }];

    // This should fail authentication
    // Note: Real implementation would need a way to inject test cookies
    println!("Test expired cookie handling - requires mock support");
}

#[tokio::test]

async fn test_browser_fallback_priority() {
    // Test that browser fallback works in priority order
    let domains = vec!["gong.io".to_string()];
    let extractor = CookieExtractor::new(domains);

    // Extract from all available browsers
    let all_browser_cookies = extractor.extract_all_browsers_cookies();
    let mut found_browsers = vec![];

    for (cookies, browser_name) in all_browser_cookies {
        println!("Checking browser: {browser_name}");
        
        if !cookies.is_empty() {
            found_browsers.push(browser_name.clone());
            println!("Found {} cookies in: {browser_name}", cookies.len());
        }
    }

    println!("Browsers with cookies: {found_browsers:?}");
    // At least one browser should work if user is logged in
    if found_browsers.is_empty() {
        println!("No browsers found with cookies (expected if not logged in)");
    }
}

#[tokio::test]

async fn test_rate_limit_handling_during_auth() {
    // Test that authentication handles rate limits gracefully
    let auth_config = AuthSettings::default();

    // Create multiple authenticators to potentially trigger rate limits
    let mut tasks = vec![];

    for i in 0..5 {
        let config = auth_config.clone();
        tasks.push(tokio::spawn(async move {
            let mut auth = GongAuthenticator::new(config)
                .await
                .expect("Failed to create authenticator");

            println!("Auth attempt {i}");
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
                println!("Auth {idx} succeeded");
            }
            Ok(false) => {
                println!("Auth {idx} failed (no cookies)");
            }
            Err(e) => {
                if e.to_string().contains("429") || e.to_string().contains("rate") {
                    rate_limit_count += 1;
                    println!("Auth {idx} rate limited");
                } else {
                    println!("Auth {idx} error: {e}");
                }
            }
        }
    }

    println!("Results - Success: {success_count}, Rate limited: {rate_limit_count}");
    // Should handle rate limits gracefully
}
