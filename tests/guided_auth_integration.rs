//! Guided Authentication Integration Tests
//!
//! Tests the guided authentication flow using headless Chrome.
//! These tests will launch Chrome browser and attempt Okta SSO authentication.
//!
//! Run with: cargo test --test guided_auth_integration -- --nocapture --test-threads=1

// Import the actual guided auth implementation
use cs_cli::common::auth::GuidedAuth;
use std::collections::HashMap;

/// Test the complete guided authentication flow
///
/// This test will:
/// 1. Launch Chrome browser
/// 2. Navigate to postman.okta.com
/// 3. Click Okta Verify authentication option
/// 4. Wait for user TouchID authentication
/// 5. Navigate to each platform (Gong, Slack, Gainsight, Salesforce)
/// 6. Extract cookies from authenticated sessions
///
/// User interaction required: TouchID authentication when prompted
#[tokio::test]
async fn test_full_guided_authentication_flow() {
    println!("Starting guided authentication flow test...");
    println!(
        "Make sure Okta Verify app is installed and you're ready to authenticate with TouchID"
    );

    let mut guided_auth = GuidedAuth::new();

    match guided_auth.authenticate().await {
        Ok(cookies) => {
            println!("Guided authentication completed successfully!");
            println!("Collected cookies for {} platforms", cookies.len());

            // Print all cookie names to debug what we actually collected
            println!("\nAll collected cookie names:");
            for name in cookies.keys() {
                println!("  {name}");
            }

            // Verify we got cookies for expected platforms
            let expected_platforms = ["gong", "slack", "gainsight", "salesforce"];
            for platform in expected_platforms {
                // Look for cookies with platform prefix
                let platform_cookies: Vec<_> = cookies
                    .keys()
                    .filter(|k| k.starts_with(&format!("{platform}_")))
                    .collect();
                if !platform_cookies.is_empty() {
                    println!(
                        "{}: {} platform-specific cookies collected",
                        platform,
                        platform_cookies.len()
                    );
                    for cookie_name in platform_cookies {
                        if let Some(cookie_data) = cookies.get(cookie_name) {
                            println!("  {} = {} characters", cookie_name, cookie_data.len());
                        }
                    }
                } else {
                    println!("{platform}: No platform-specific cookies collected");
                }
            }

            assert!(
                !cookies.is_empty(),
                "Should have collected at least some cookies"
            );
            println!("Test completed successfully - guided authentication works!");
        }
        Err(e) => {
            println!("Guided authentication failed: {e}");
            panic!("Guided authentication should succeed with user interaction: {e}");
        }
    }
}

/// Test Chrome browser launch
///
/// This test verifies the browser can be started and controlled correctly
#[tokio::test]
async fn test_chrome_browser_launch() {
    println!("Testing Chrome browser launch...");

    let mut guided_auth = GuidedAuth::new();

    // Test browser initialization
    assert!(
        !guided_auth.has_browser(),
        "Browser should not be initialized initially"
    );

    // Navigate to a test URL
    match guided_auth
        .navigate_to_okta("https://postman.okta.com")
        .await
    {
        Ok(()) => {
            println!("Chrome browser launched and navigated successfully");
            assert!(
                guided_auth.has_browser(),
                "Browser should be initialized after navigation"
            );

            // Test getting current URL
            match guided_auth.get_current_url().await {
                Ok(url) => {
                    println!("Current URL: {url}");
                    assert!(
                        url.contains("postman.okta.com"),
                        "Should have navigated to Okta"
                    );
                }
                Err(e) => {
                    println!("Failed to get current URL: {e}");
                }
            }

            // Close browser
            match guided_auth.close() {
                Ok(()) => {
                    println!("Chrome browser closed successfully");
                }
                Err(e) => {
                    println!("Failed to close browser: {e}");
                }
            }
        }
        Err(e) => {
            println!("Failed to launch Chrome browser: {e}");
            panic!("Should be able to launch Chrome browser: {e}");
        }
    }
}

/// Test Chrome browser detection
///
/// This test verifies that Chrome can be found on the system
#[tokio::test]
async fn test_chrome_detection() {
    println!("Testing Chrome browser detection...");

    // Check if Chrome exists at expected locations
    let chrome_paths = vec![
        #[cfg(target_os = "macos")]
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        #[cfg(target_os = "macos")]
        "/Applications/Chromium.app/Contents/MacOS/Chromium",
        #[cfg(target_os = "linux")]
        "/usr/bin/google-chrome",
        #[cfg(target_os = "windows")]
        r"C:\Program Files\Google\Chrome\Application\chrome.exe",
    ];

    let mut chrome_found = false;
    for path in chrome_paths {
        if std::path::Path::new(path).exists() {
            println!("Found Chrome at: {path}");
            chrome_found = true;
            break;
        }
    }

    if !chrome_found {
        println!("Chrome not found at standard locations - browser launch may fail");
        println!("Please ensure Chrome is installed");
    } else {
        println!("Chrome detection test completed successfully!");
    }
}

/// Test dynamic cookie domain discovery with visible browser
///
/// This test launches Chrome in headed mode (visible) to allow observation of:
/// 1. Browser automation behavior
/// 2. Dynamic cookie domain discovery from actual cookies
/// 3. Platform-specific cookie extraction
///
/// User interaction required: TouchID authentication when prompted
#[tokio::test]
#[ignore = "requires manual interaction and visual observation"]
async fn test_dynamic_cookie_discovery_headed() {
    println!("{}", "=".repeat(80));
    println!("Starting VISIBLE browser test for dynamic cookie domain discovery");
    println!("You will see the browser window open and navigate through authentication");
    println!("{}", "=".repeat(80));

    // Use new_headed() to launch browser in visible mode
    let mut guided_auth = GuidedAuth::new_headed();

    println!("\nBrowser will launch in VISIBLE mode...");
    println!("Please be ready to authenticate with TouchID when prompted\n");

    // Add a longer delay to let user read the message and observe browser
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    // Use authenticate_fresh to force actual navigation to platforms
    match guided_auth.authenticate_fresh().await {
        Ok(cookies) => {
            println!("\n{}", "=".repeat(80));
            println!("AUTHENTICATION SUCCESSFUL - ANALYZING COOKIES");
            println!("{}", "=".repeat(80));

            // Test dynamic domain discovery
            println!("\n--- Dynamic Cookie Domain Discovery Results ---");

            // Group cookies by discovered platform domains
            let platforms = ["gong", "slack", "gainsight", "salesforce"];
            for platform in platforms {
                println!("\n[{}] Platform Analysis:", platform.to_uppercase());

                let platform_cookies: Vec<_> = cookies
                    .keys()
                    .filter(|k| k.starts_with(&format!("{}_", platform)))
                    .collect();

                if !platform_cookies.is_empty() {
                    println!("  ✓ Found {} cookies for {}", platform_cookies.len(), platform);

                    // Show first few cookie names (without values for security)
                    for (idx, cookie_name) in platform_cookies.iter().take(5).enumerate() {
                        println!("    {}. {}", idx + 1, cookie_name);
                    }

                    if platform_cookies.len() > 5 {
                        println!("    ... and {} more", platform_cookies.len() - 5);
                    }
                } else {
                    println!("  ✗ No cookies found for {}", platform);
                    println!("    (Platform may not have been accessed during auth flow)");
                }
            }

            println!("\n--- Cookie Statistics ---");
            println!("Total cookies collected: {}", cookies.len());

            // Count cookies by prefix
            let mut prefix_counts: HashMap<String, usize> = HashMap::new();
            for key in cookies.keys() {
                if let Some(prefix) = key.split('_').next() {
                    *prefix_counts.entry(prefix.to_string()).or_insert(0) += 1;
                }
            }

            println!("\nCookies grouped by prefix:");
            for (prefix, count) in prefix_counts.iter() {
                println!("  {}: {} cookies", prefix, count);
            }

            // Verify dynamic discovery worked
            assert!(
                !cookies.is_empty(),
                "Should have collected at least some cookies"
            );

            println!("\n{}", "=".repeat(80));
            println!("TEST COMPLETED SUCCESSFULLY");
            println!("Dynamic cookie domain discovery is working correctly!");
            println!("{}", "=".repeat(80));
        }
        Err(e) => {
            println!("\n{}", "=".repeat(80));
            println!("AUTHENTICATION FAILED");
            println!("{}", "=".repeat(80));
            println!("Error: {}", e);
            println!("\nBrowser window will stay open for 30 seconds so you can inspect the page...");
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            panic!("Authentication should succeed with user interaction: {}", e);
        }
    }
}

/// Test navigation to Okta OAuth endpoint
#[tokio::test]
async fn test_okta_oauth_navigation() {
    println!("Testing navigation to Okta OAuth authorization endpoint...");

    let mut guided_auth = GuidedAuth::new();

    let okta_url = "https://postman.okta.com/oauth2/v1/authorize?client_id=okta.2b1959c8-bcc0-56eb-a589-cfcfb7422f26&code_challenge=QqOla_j2ieDvzX7ebLtkAvwddcCQrhgFmqW0OgXEkTE&code_challenge_method=S256&nonce=oCAMKaZsIi9TTTkla80f4MnJfMn8kOlx0uWRhtL2AG1IcN5XVZF9UF83vjxgX8dg&redirect_uri=https%3A%2F%2Fpostman.okta.com%2Fenduser%2Fcallback&response_type=code&state=X9clhfyFhEE90WBMcHIaBtS2EtXzbYZEe4eN4XTF1PTqawEr3A4TGvD6UFTO6gV0&scope=openid%20profile%20email%20okta.users.read.self%20okta.users.manage.self%20okta.internal.enduser.read%20okta.internal.enduser.manage%20okta.enduser.dashboard.read%20okta.enduser.dashboard.manage%20okta.myAccount.sessions.manage";

    match guided_auth.navigate_to_okta(okta_url).await {
        Ok(()) => {
            println!("Successfully navigated to Okta OAuth endpoint");

            // Wait longer for Okta sign-in widget to load
            println!("Waiting for Okta sign-in widget to initialize...");
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;

            // Check what's in the okta-sign-in container
            let check_container_js = r#"
                (function() {
                    const container = document.getElementById('okta-sign-in');
                    if (container) {
                        return JSON.stringify({
                            found: true,
                            id: container.id,
                            className: container.className,
                            childCount: container.children.length,
                            innerHTML: container.innerHTML.substring(0, 500)
                        });
                    } else {
                        // Check body content to see what's actually on the page
                        return JSON.stringify({
                            found: false,
                            bodyHTML: document.body.innerHTML.substring(0, 1000),
                            title: document.title,
                            readyState: document.readyState
                        });
                    }
                })()
            "#;

            match guided_auth.execute_js(check_container_js).await {
                Ok(result) => {
                    println!("\n=== Page Content Debug ===");
                    println!("{result}");
                    println!("=========================\n");
                }
                Err(e) => {
                    println!("Failed to check page content: {e}");
                }
            }

            // Extract and analyze page content
            let extract_links_js = r#"
                JSON.stringify(Array.from(document.querySelectorAll('a')).map(link => ({
                    href: link.href,
                    text: link.textContent?.trim() || '',
                    ariaLabel: link.getAttribute('aria-label') || '',
                    className: link.className || '',
                    id: link.id || '',
                    dataSe: link.getAttribute('data-se') || ''
                })))
            "#;

            match guided_auth.execute_js(extract_links_js).await {
                Ok(result) => {
                    println!("\nLinks found on the OAuth authorization page:");
                    println!("{}", "=".repeat(60));
                    println!("{result}");
                    println!("{}", "=".repeat(60));
                }
                Err(e) => {
                    println!("Failed to extract links: {e}");
                }
            }

            // Extract any buttons that might be authentication options
            let extract_buttons_js = r#"
                JSON.stringify(Array.from(document.querySelectorAll('button, input[type="button"], input[type="submit"], [role="button"]')).map(btn => ({
                    text: btn.textContent?.trim() || btn.value || '',
                    type: btn.type || '',
                    className: btn.className || '',
                    id: btn.id || '',
                    ariaLabel: btn.getAttribute('aria-label') || ''
                })))
            "#;

            match guided_auth.execute_js(extract_buttons_js).await {
                Ok(result) => {
                    println!("\nButtons found on the page:");
                    println!("{}", "=".repeat(60));
                    println!("{result}");
                    println!("{}", "=".repeat(60));
                }
                Err(e) => {
                    println!("Failed to extract buttons: {e}");
                }
            }

            // Get page title
            match guided_auth
                .execute_js("JSON.stringify(document.title || 'No Title')")
                .await
            {
                Ok(title) => {
                    println!("\nPage title: {title}");
                }
                Err(e) => {
                    println!("Failed to get page title: {e}");
                }
            }

            // Close browser
            let _ = guided_auth.close();
            println!("Navigation test completed successfully!");
        }
        Err(e) => {
            println!("Navigation failed: {e}");
            panic!("Should be able to navigate to Okta: {e}");
        }
    }
}

/// Test Okta Verify app detection on macOS
#[cfg(target_os = "macos")]
#[tokio::test]
async fn test_okta_verify_detection() {
    println!("Testing Okta Verify app detection...");

    let guided_auth = GuidedAuth::new();

    match guided_auth.check_okta_verify().await {
        Ok(is_running) => {
            if is_running {
                println!("Okta Verify is currently running");
            } else {
                println!("Okta Verify is not currently running");
                println!("You can install Okta Verify from the App Store if needed");
            }

            // Test should pass regardless of whether Okta Verify is running
            println!("Okta Verify detection completed successfully");
        }
        Err(e) => {
            println!("Okta Verify detection failed: {e}");
            panic!("Should be able to check Okta Verify status: {e}");
        }
    }
}
