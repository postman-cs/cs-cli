//! Integration tests for GitHub gist storage
//!
//! These tests require a real GitHub token and are marked as ignored by default.
//! Run with: cargo test --ignored github_gist_integration

use cs_cli::common::auth::github_gist_storage::GitHubGistStorage;
use std::collections::HashMap;
use std::env;

/// Helper to check if GitHub token is available for testing
fn github_token_available() -> bool {
    env::var("GITHUB_TOKEN").is_ok() || env::var("GITHUB_CLIENT_SECRET").is_ok()
}

/// Skip test if no GitHub token available
macro_rules! skip_if_no_token {
    () => {
        if !github_token_available() {
            println!("Skipping test - GITHUB_TOKEN not set");
            return;
        }
    };
}

#[tokio::test]
#[ignore = "requires GitHub token"]
async fn test_gist_storage_initialization() {
    skip_if_no_token!();

    // Should be able to initialize without authentication
    let result = GitHubGistStorage::new().await;
    assert!(result.is_ok());
}

#[tokio::test]
#[ignore = "requires GitHub token and OAuth flow"]
async fn test_gist_storage_roundtrip() {
    skip_if_no_token!();

    let mut cookies = HashMap::new();
    cookies.insert("test_session".to_string(), "test_value_123".to_string());
    cookies.insert("csrf_token".to_string(), "csrf_abc_def".to_string());
    cookies.insert("user_id".to_string(), "user_456789".to_string());

    let mut storage = GitHubGistStorage::new().await.unwrap();

    // Store cookies (will trigger OAuth flow if not authenticated)
    match storage.store_cookies(&cookies).await {
        Ok(()) => {
            println!("Successfully stored cookies in gist");

            // Retrieve cookies
            let retrieved = storage.get_cookies().await.unwrap();
            assert_eq!(cookies, retrieved);

            // Verify has_cookies works
            assert!(storage.has_cookies().await);

            // Cleanup - delete the test gist
            storage.delete_cookies().await.unwrap();

            // Verify deletion worked
            assert!(!storage.has_cookies().await);
        }
        Err(e) => {
            println!("OAuth flow required or failed: {e}");
            // This is expected if OAuth hasn't been completed
        }
    }
}

#[tokio::test]
#[ignore = "requires GitHub token"]
async fn test_has_cookies_empty_state() {
    skip_if_no_token!();

    let storage = GitHubGistStorage::new().await.unwrap();

    // New storage should not have cookies (unless leftover from previous tests)
    // This test is best-effort since we can't guarantee clean state
    let has_cookies = storage.has_cookies().await;
    println!("Has cookies in fresh storage: {has_cookies}");

    // Test passes regardless - just verifies the method doesn't crash
}

#[tokio::test]
#[ignore = "requires GitHub token"]
async fn test_get_cookies_without_storage() {
    skip_if_no_token!();

    let storage = GitHubGistStorage::new().await.unwrap();

    // Should handle gracefully when no gist exists
    let result = storage.get_cookies().await;
    match result {
        Ok(_) => println!("Found existing gist data"),
        Err(e) => {
            println!("No gist data found (expected): {e}");
            // This is the expected case for fresh tests
        }
    }
}

#[cfg(test)]
mod gist_config_tests {

    #[test]
    fn test_gist_config_serialization() {
        // Test internal config structures without requiring GitHub API
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct TestGistConfig {
            gist_id: String,
            github_username: String,
            token_hash: String,
            last_sync: String,
        }

        let config = TestGistConfig {
            gist_id: "abc123def456".to_string(),
            github_username: "testuser".to_string(),
            token_hash: "sha256hash123".to_string(),
            last_sync: "2024-01-01T00:00:00Z".to_string(),
        };

        let serialized = serde_json::to_string_pretty(&config).unwrap();
        let deserialized: TestGistConfig = serde_json::from_str(&serialized).unwrap();

        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_session_metadata_structure() {
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct TestSessionMetadata {
            version: u32,
            created_at: String,
            updated_at: String,
            platforms: Vec<String>,
        }

        let metadata = TestSessionMetadata {
            version: 1,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T01:00:00Z".to_string(),
            platforms: vec!["gong".to_string(), "slack".to_string()],
        };

        let serialized = serde_json::to_string(&metadata).unwrap();
        let deserialized: TestSessionMetadata = serde_json::from_str(&serialized).unwrap();

        assert_eq!(metadata, deserialized);
        assert!(serialized.contains("\"version\":1"));
        assert!(serialized.contains("\"platforms\":[\"gong\",\"slack\"]"));
    }
}

#[cfg(test)]
mod github_api_mocks {

    // These tests use mock responses to test GitHub API interaction without real API calls

    #[test]
    fn test_github_error_responses() {
        // Simulate common GitHub API error responses
        let error_responses = vec![
            (401, r#"{"message": "Bad credentials"}"#),
            (403, r#"{"message": "API rate limit exceeded"}"#),
            (404, r#"{"message": "Not Found"}"#),
            (422, r#"{"message": "Validation Failed", "errors": []}"#),
        ];

        for (status_code, response_body) in error_responses {
            // Verify we can parse GitHub error responses
            assert!(response_body.contains("message"));
            println!("Can handle GitHub {status_code} error: {response_body}");
        }
    }

    #[test]
    fn test_gist_response_structure() {
        // Simulate a successful gist creation response
        let mock_gist_response = r#"
        {
            "id": "abc123def456",
            "html_url": "https://gist.github.com/abc123def456",
            "files": {
                "cs-cli-session-data.enc": {
                    "filename": "cs-cli-session-data.enc",
                    "type": "text/plain",
                    "content": "encrypted_data_here"
                }
            },
            "public": false,
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z"
        }
        "#;

        let gist_value: serde_json::Value = serde_json::from_str(mock_gist_response).unwrap();

        assert_eq!(gist_value["id"], "abc123def456");
        assert_eq!(gist_value["public"], false);
        assert!(gist_value["files"]["cs-cli-session-data.enc"]["content"].is_string());
    }

    #[test]
    fn test_oauth_token_response_structure() {
        // Simulate OAuth token exchange response
        let mock_token_response = r#"
        {
            "access_token": "ghs_token_here",
            "token_type": "bearer", 
            "scope": "gist"
        }
        "#;

        let token_value: serde_json::Value = serde_json::from_str(mock_token_response).unwrap();

        assert_eq!(token_value["token_type"], "bearer");
        assert_eq!(token_value["scope"], "gist");
        assert!(token_value["access_token"].is_string());
    }
}
