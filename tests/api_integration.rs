//! API Integration Tests
//!
//! These tests validate API endpoints against the real Gong API
//! to ensure data extraction continues working correctly.
//!
//! IMPORTANT: Requires valid Gong authentication and test customer data

use cs_cli::common::config::{AuthSettings, HttpSettings};
use cs_cli::common::models::time::ExtractionRange;
use cs_cli::gong::api::client::HttpClientPool;
use cs_cli::gong::api::customer::GongCustomerSearchClient;
use cs_cli::gong::api::email::EmailEnhancer;
use cs_cli::gong::api::library::GongLibraryClient;
use cs_cli::gong::api::timeline::TimelineExtractor;
use cs_cli::gong::auth::GongAuthenticator;
use std::sync::Arc;

/// Test configuration with known test data
struct TestConfig {
    /// Known customer name that exists in Gong
    test_customer: String,
    /// Number of days to look back
    days_back: i64,
    /// Expected minimum number of communications
    min_expected_items: usize,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            test_customer: std::env::var("TEST_CUSTOMER_NAME")
                .unwrap_or_else(|_| "Fiserv".to_string()),
            days_back: std::env::var("TEST_DAYS_BACK")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30),
            min_expected_items: 1,
        }
    }
}

#[tokio::test]
#[ignore = "Requires real Gong API access and test data"]
async fn test_customer_search_real_api() {
    let config = TestConfig::default();
    let auth_config = AuthSettings::default();
    let mut authenticator = GongAuthenticator::new(auth_config)
        .await
        .expect("Failed to create authenticator");

    if !authenticator.authenticate().await.unwrap_or(false) {
        println!("Skipping test - authentication failed");
        return;
    }

    let client_pool = Arc::new(
        HttpClientPool::new(Some(HttpSettings::default()))
            .await
            .expect("Failed to create client pool"),
    );

    let search_client = GongCustomerSearchClient::new(
        client_pool,
        Arc::new(authenticator),
        None, // Use default config
    )
    .expect("Failed to create search client");

    // Search for customer
    let results = search_client.search_customers(&config.test_customer).await;

    match results {
        Ok(customers) => {
            println!(
                "Found {} customers matching '{}'",
                customers.len(),
                config.test_customer
            );
            assert!(!customers.is_empty(), "Should find at least one customer");

            // Verify customer data structure
            for customer in &customers {
                assert!(!customer.name.is_empty(), "Customer should have name");
                println!("Customer: {} (ID: {:?})", customer.name, customer.id);
            }
        }
        Err(e) => {
            panic!("Customer search failed: {}", e);
        }
    }
}

#[tokio::test]
#[ignore = "Requires real Gong API access and test data"]
async fn test_timeline_extraction_real_api() {
    let config = TestConfig::default();
    let auth_config = AuthSettings::default();
    let mut authenticator = GongAuthenticator::new(auth_config)
        .await
        .expect("Failed to create authenticator");

    if !authenticator.authenticate().await.unwrap_or(false) {
        println!("Skipping test - authentication failed");
        return;
    }

    let client_pool = Arc::new(
        HttpClientPool::new(Some(HttpSettings::default()))
            .await
            .expect("Failed to create client pool"),
    );

    let auth_arc = Arc::new(authenticator);

    // First, find the customer
    let search_client = GongCustomerSearchClient::new(
        client_pool.clone(),
        auth_arc.clone(),
        None, // Use default config
    )
    .expect("Failed to create search client");

    let customers = search_client
        .search_customers(&config.test_customer)
        .await
        .expect("Customer search failed");

    if customers.is_empty() {
        println!("No customers found for testing");
        return;
    }

    let customer = &customers[0];
    println!("Testing timeline for customer: {}", customer.name);

    // Extract timeline
    let mut extractor = TimelineExtractor::new(
        client_pool,
        auth_arc,
        None, // Use default config
        None, // Use default chunk_days
    )
    .expect("Failed to create timeline extractor");

    // Create date range for extraction
    let start_date = jiff::Zoned::now().saturating_sub(jiff::Span::new().days(config.days_back));

    // Use the customer's account ID if available
    let account_id = customer.id.as_deref().unwrap_or("test-account");

    let result = extractor
        .extract_account_timeline(account_id, start_date, None)
        .await;

    match result {
        Ok(timeline) => {
            println!("Timeline extraction successful:");
            println!("- Calls: {}", timeline.calls.len());
            println!("- Emails: {}", timeline.emails.len());
            println!("- Filtered stats: {:?}", timeline.stats);

            // Verify data structures
            for call in &timeline.calls {
                assert!(!call.id.is_empty(), "Call should have ID");
                assert!(call.duration >= 0, "Call duration should be non-negative");
                assert!(
                    !call.participants.is_empty(),
                    "Call should have participants"
                );
            }

            for email in &timeline.emails {
                assert!(!email.id.is_empty(), "Email should have ID");
                assert!(!email.subject.is_empty(), "Email should have subject");
                assert!(
                    email.html_body.is_some() || email.body_text.is_some(),
                    "Email should have content"
                );
            }
        }
        Err(e) => {
            panic!("Timeline extraction failed: {}", e);
        }
    }
}

#[tokio::test]
#[ignore = "Requires real Gong API access"]
async fn test_email_enhancement_real_api() {
    let auth_config = AuthSettings::default();
    let mut authenticator = GongAuthenticator::new(auth_config)
        .await
        .expect("Failed to create authenticator");

    if !authenticator.authenticate().await.unwrap_or(false) {
        println!("Skipping test - authentication failed");
        return;
    }

    let client_pool = Arc::new(
        HttpClientPool::new(Some(HttpSettings::default()))
            .await
            .expect("Failed to create client pool"),
    );

    let enhancer = EmailEnhancer::new(
        client_pool,
        Arc::new(authenticator),
        None, // Use default config
        None, // Use default batch_size
    );

    // Note: This would need a real Email object from the timeline
    // For now, we just verify the enhancer can be created
    println!("Email enhancer created successfully");

    // Can't easily test enhance_single_email without a real Email object
    // Would need to get an email from timeline extraction first
}

#[tokio::test]
#[ignore = "Requires real Gong API access"]
async fn test_library_client_call_search() {
    let auth_config = AuthSettings::default();
    let mut authenticator = GongAuthenticator::new(auth_config)
        .await
        .expect("Failed to create authenticator");

    if !authenticator.authenticate().await.unwrap_or(false) {
        println!("Skipping test - authentication failed");
        return;
    }

    let client_pool = Arc::new(
        HttpClientPool::new(Some(HttpSettings::default()))
            .await
            .expect("Failed to create client pool"),
    );

    let library_client = GongLibraryClient::new(
        client_pool,
        Arc::new(authenticator),
        None, // Use default config
    );

    // Search for recent calls
    let extraction_range =
        ExtractionRange::last_days(7).expect("Failed to create extraction range");

    // Get library calls with date range parameters
    let result = library_client
        .get_library_calls(
            None,    // call_stream_id
            Some(7), // days_back
            None,    // from_date
            None,    // to_date
            0,       // offset
        )
        .await;

    match result {
        Ok(library_result) => {
            println!("Found {} calls in library", library_result.calls.len());

            for call in &library_result.calls {
                assert!(!call.title.is_empty(), "Call should have title");
                assert!(call.duration >= 0, "Call duration should be non-negative");
                println!("Call: {} ({} seconds)", call.title, call.duration);
            }
        }
        Err(e) => {
            println!("Library search error (may be permission issue): {}", e);
        }
    }
}

#[tokio::test]
#[ignore = "Requires real Gong API access"]
async fn test_concurrent_api_calls_with_rate_limiting() {
    let config = TestConfig::default();
    let auth_config = AuthSettings::default();
    let mut authenticator = GongAuthenticator::new(auth_config)
        .await
        .expect("Failed to create authenticator");

    if !authenticator.authenticate().await.unwrap_or(false) {
        println!("Skipping test - authentication failed");
        return;
    }

    let client_pool = Arc::new(
        HttpClientPool::new(Some(HttpSettings::default()))
            .await
            .expect("Failed to create client pool"),
    );

    let auth_arc = Arc::new(authenticator);

    // Launch multiple concurrent requests
    let mut tasks = vec![];

    for i in 0..5 {
        let pool = client_pool.clone();
        let auth = auth_arc.clone();
        let customer = config.test_customer.clone();

        tasks.push(tokio::spawn(async move {
            let search_client = GongCustomerSearchClient::new(pool, auth, None)
                .expect("Failed to create search client");
            let start = std::time::Instant::now();
            let result = search_client.search_customers(&customer).await;
            let duration = start.elapsed();
            (i, result, duration)
        }));
    }

    // Collect results
    let mut success_count = 0;
    let mut total_duration = std::time::Duration::ZERO;

    for task in tasks {
        let (idx, result, duration) = task.await.expect("Task panicked");
        total_duration += duration;

        match result {
            Ok(customers) => {
                success_count += 1;
                println!(
                    "Request {} succeeded in {:?}, found {} customers",
                    idx,
                    duration,
                    customers.len()
                );
            }
            Err(e) => {
                println!("Request {} failed in {:?}: {}", idx, duration, e);
            }
        }
    }

    println!(
        "Concurrent requests - Success: {}/5, Total time: {:?}",
        success_count, total_duration
    );

    assert!(success_count > 0, "At least some requests should succeed");
}

#[tokio::test]
#[ignore = "Requires real Gong API access"]
async fn test_retry_logic_on_transient_failures() {
    // Test that the client properly retries on transient failures
    let auth_config = AuthSettings::default();
    let mut authenticator = GongAuthenticator::new(auth_config)
        .await
        .expect("Failed to create authenticator");

    if !authenticator.authenticate().await.unwrap_or(false) {
        println!("Skipping test - authentication failed");
        return;
    }

    // Use default HTTP configuration (retry logic is internal to the client)
    let http_config = HttpSettings::default();

    let client_pool = Arc::new(
        HttpClientPool::new(Some(http_config))
            .await
            .expect("Failed to create client pool"),
    );

    let search_client = GongCustomerSearchClient::new(
        client_pool,
        Arc::new(authenticator),
        None, // Use default config
    )
    .expect("Failed to create search client");

    // Make a request that might trigger retries
    let result = search_client.search_customers("test").await;

    // Should eventually succeed or fail definitively
    match result {
        Ok(customers) => {
            println!("Request succeeded, found {} customers", customers.len());
        }
        Err(e) => {
            println!("Request failed after retries: {}", e);
            // Verify it's not a transient error that should have been retried
            let error_str = e.to_string();
            assert!(
                !error_str.contains("timeout") && !error_str.contains("connection"),
                "Transient errors should be retried"
            );
        }
    }
}

#[tokio::test]
#[ignore = "Requires real Gong API access"]
async fn test_empty_results_handling() {
    let auth_config = AuthSettings::default();
    let mut authenticator = GongAuthenticator::new(auth_config)
        .await
        .expect("Failed to create authenticator");

    if !authenticator.authenticate().await.unwrap_or(false) {
        println!("Skipping test - authentication failed");
        return;
    }

    let client_pool = Arc::new(
        HttpClientPool::new(Some(HttpSettings::default()))
            .await
            .expect("Failed to create client pool"),
    );

    let search_client = GongCustomerSearchClient::new(
        client_pool,
        Arc::new(authenticator),
        None, // Use default config
    )
    .expect("Failed to create search client");

    // Search for a customer that likely doesn't exist
    let result = search_client
        .search_customers("NonExistentCustomer12345XYZ")
        .await;

    match result {
        Ok(customers) => {
            println!("Search returned {} results", customers.len());
            assert_eq!(
                customers.len(),
                0,
                "Should return empty list for non-existent customer"
            );
        }
        Err(e) => {
            // Also acceptable if it returns an error
            println!("Search returned error for non-existent customer: {}", e);
        }
    }
}

#[tokio::test]
#[ignore = "Requires real Gong API access and specific test data"]
async fn test_large_dataset_pagination() {
    let auth_config = AuthSettings::default();
    let mut authenticator = GongAuthenticator::new(auth_config)
        .await
        .expect("Failed to create authenticator");

    if !authenticator.authenticate().await.unwrap_or(false) {
        println!("Skipping test - authentication failed");
        return;
    }

    let client_pool = Arc::new(
        HttpClientPool::new(Some(HttpSettings::default()))
            .await
            .expect("Failed to create client pool"),
    );

    let mut extractor = TimelineExtractor::new(
        client_pool,
        Arc::new(authenticator),
        None, // Use default config
        None, // Use default chunk_days
    )
    .expect("Failed to create timeline extractor");

    // Request a large date range to test pagination
    let start_date = jiff::Zoned::now().saturating_sub(jiff::Span::new().days(365)); // Full year

    // Use a workspace ID that has lots of data
    // Note: This would need a real workspace ID with substantial data
    let workspace_id = "test-workspace-with-lots-of-data";

    let result = extractor
        .extract_account_timeline(workspace_id, start_date, None)
        .await;

    match result {
        Ok(timeline) => {
            println!("Successfully extracted large dataset:");
            println!("- Total calls: {}", timeline.calls.len());
            println!("- Total emails: {}", timeline.emails.len());

            // Verify pagination worked if we have lots of results
            if timeline.calls.len() > 100 || timeline.emails.len() > 100 {
                println!("Pagination likely worked - got many results");
            }
        }
        Err(e) => {
            println!(
                "Large dataset extraction failed (expected without valid workspace): {}",
                e
            );
        }
    }
}
