//! Unit tests for GitHub OAuth functionality
//!
//! Tests OAuth URL generation, parameter parsing, and error handling
//! without requiring actual GitHub API calls.

use cs_cli::common::auth::github_oauth_config::*;

#[test]
fn test_oauth_url_generation() {
    // Set test environment variable
    std::env::set_var("GITHUB_CLIENT_ID", "test_client_id_abc");

    let state = "test_state_abc123";
    let url = build_oauth_url(state).unwrap();

    // URL should contain all required components
    assert!(url.contains("github.com/login/oauth/authorize"));
    assert!(url.contains("scope=gist"));
    assert!(url.contains(&format!("state={}", state)));
    assert!(url.contains("redirect_uri="));
    assert!(url.contains("client_id=test_client_id_abc"));

    // Verify URL structure
    assert!(url.starts_with("https://github.com/login/oauth/authorize?"));
}

#[test]
fn test_oauth_state_validation() {
    let original_state = "secure_state_123456";

    // Same state should validate
    assert!(validate_oauth_callback(original_state, original_state));

    // Different state should fail
    assert!(!validate_oauth_callback(original_state, "different_state"));
    assert!(!validate_oauth_callback(original_state, ""));
    assert!(!validate_oauth_callback("", original_state));
}

#[test]
fn test_oauth_state_generation() {
    let state1 = generate_oauth_state();
    let state2 = generate_oauth_state();

    // States should be correct length
    assert_eq!(state1.len(), 32);
    assert_eq!(state2.len(), 32);

    // States should be unique
    assert_ne!(state1, state2);

    // States should be alphanumeric
    assert!(state1.chars().all(|c| c.is_ascii_alphanumeric()));
    assert!(state2.chars().all(|c| c.is_ascii_alphanumeric()));
}

#[test]
fn test_oauth_constants() {
    // Verify OAuth configuration constants are reasonable
    assert_eq!(GITHUB_OAUTH_SCOPES, "gist");
    assert_eq!(CALLBACK_URL, "http://localhost:8080/auth/github/callback");
    assert_eq!(
        OAUTH_AUTHORIZE_URL,
        "https://github.com/login/oauth/authorize"
    );
    assert_eq!(
        OAUTH_TOKEN_URL,
        "https://github.com/login/oauth/access_token"
    );
    assert_eq!(GIST_FILENAME, "cs-cli-session-data.enc");
    assert!(GIST_DESCRIPTION.contains("cs-cli"));
    assert!(GIST_DESCRIPTION.contains("DO NOT EDIT"));
}

#[test]
fn test_build_oauth_url_encoding() {
    std::env::set_var("GITHUB_CLIENT_ID", "test_client");
    let state = "state_with_special_chars_&=?#";
    let url = build_oauth_url(state).unwrap();

    // URL should be properly encoded
    assert!(!url.contains("&=?#"));
    assert!(url.contains("state_with_special_chars"));
}

#[test]
fn test_empty_state_handling() {
    std::env::set_var("GITHUB_CLIENT_ID", "test_client");
    let empty_state = "";
    let url = build_oauth_url(empty_state).unwrap();

    // Should still generate valid URL structure
    assert!(url.contains("state="));
    assert!(url.contains("github.com/login/oauth/authorize"));
}

#[test]
fn test_missing_client_id() {
    std::env::remove_var("GITHUB_CLIENT_ID");
    let state = "test_state";
    let result = build_oauth_url(state);

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.contains("GITHUB_CLIENT_ID"));
}

#[cfg(test)]
mod oauth_flow_tests {
    use super::*;
    use cs_cli::common::auth::github_oauth_flow::GitHubOAuthFlow;

    #[test]
    fn test_oauth_flow_initialization() {
        let _oauth_flow = GitHubOAuthFlow::new();
        // OAuth flow should be creatable without network calls
        // This tests that initialization doesn't require actual GitHub connection
    }

    // The internal implementation details (parse_query_params, extract_auth_code, etc.)
    // are now private and tested within the github_oauth_flow module itself.
    // These tests have been moved to the internal test module where they have access
    // to private methods.
}
