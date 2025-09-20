//! GitHub OAuth Configuration for Postman CS-CLI
//!
//! Configuration for OAuth app that allows CSMs to grant access to their
//! personal GitHub accounts for encrypted session data storage.

/// OAuth Application Configuration (registered under Postman org)
pub const GITHUB_OAUTH_SCOPES: &str = "gist";
pub const CALLBACK_URL: &str = "http://localhost:8080/auth/github/callback";
pub const OAUTH_AUTHORIZE_URL: &str = "https://github.com/login/oauth/authorize";
pub const OAUTH_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";

/// Get GitHub Client ID from environment variable
pub fn get_github_client_id() -> Option<String> {
    std::env::var("GITHUB_CLIENT_ID").ok()
}

/// Get GitHub Client Secret from environment variable
pub fn get_github_client_secret() -> Option<String> {
    std::env::var("GITHUB_CLIENT_SECRET").ok()
}

/// Application identification for gists
pub const GIST_FILENAME: &str = "cs-cli-session-data.enc";
pub const GIST_DESCRIPTION: &str = "CS-CLI encrypted session data - DO NOT EDIT";

/// Generate secure OAuth state parameter for CSRF protection
pub fn generate_oauth_state() -> String {
    use rand::{distributions::Alphanumeric, Rng};
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}

/// Build complete GitHub OAuth authorization URL
pub fn build_oauth_url(state: &str) -> Result<String, String> {
    let client_id = get_github_client_id()
        .ok_or_else(|| "GITHUB_CLIENT_ID environment variable not set".to_string())?;

    Ok(format!(
        "{}?client_id={}&scope={}&state={}&redirect_uri={}",
        OAUTH_AUTHORIZE_URL,
        client_id,
        urlencoding::encode(GITHUB_OAUTH_SCOPES),
        state,
        urlencoding::encode(CALLBACK_URL)
    ))
}

/// Validate callback parameters
pub fn validate_oauth_callback(state: &str, received_state: &str) -> bool {
    state == received_state
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth_url_generation() {
        // Set test environment variable
        std::env::set_var("GITHUB_CLIENT_ID", "test_client_id");

        let state = "test_state_123";
        let url = build_oauth_url(state).unwrap();

        assert!(url.contains("github.com/login/oauth/authorize"));
        assert!(url.contains("scope=gist"));
        assert!(url.contains(&format!("state={state}")));
        assert!(url.contains("client_id=test_client_id"));
    }

    #[test]
    fn test_oauth_state_validation() {
        let original_state = "secure_state_123";

        assert!(validate_oauth_callback(original_state, original_state));
        assert!(!validate_oauth_callback(original_state, "different_state"));
    }

    #[test]
    fn test_oauth_state_generation() {
        let state1 = generate_oauth_state();
        let state2 = generate_oauth_state();

        assert_eq!(state1.len(), 32);
        assert_eq!(state2.len(), 32);
        assert_ne!(state1, state2); // Should be unique
    }
}
