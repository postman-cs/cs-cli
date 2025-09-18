//! Guided Authentication Cookie Storage
//!
//! Stores and retrieves authentication cookies from macOS keychain.
//! Similar to password storage but designed for platform-specific session cookies.

use crate::{CsCliError, Result};
use serde_json;
use std::collections::HashMap;
use std::process::Command;
use tracing::{debug, info, warn};

const KEYCHAIN_SERVICE_NAME: &str = "com.postman.cs-cli.cookies";
const KEYCHAIN_ACCOUNT_NAME: &str = "guided-auth-cookies";

/// Store platform cookies in keychain
pub fn store_cookies(cookies: &HashMap<String, String>) -> Result<()> {
    if cookies.is_empty() {
        return Err(CsCliError::Authentication(
            "Cannot store empty cookie collection".to_string(),
        ));
    }

    info!("Storing {} platform cookies in keychain...", cookies.len());

    // Serialize cookies to JSON
    let cookies_json = serde_json::to_string(cookies).map_err(|e| {
        CsCliError::Authentication(format!("Failed to serialize cookies: {e}"))
    })?;

    // Get current executable path for ACL
    let current_exe = std::env::current_exe().map_err(|e| {
        CsCliError::Authentication(format!("Could not determine executable path: {e}"))
    })?;

    // First, try to delete existing item (ignore errors)
    let _ = Command::new("security")
        .args([
            "delete-generic-password",
            "-a",
            KEYCHAIN_ACCOUNT_NAME,
            "-s",
            KEYCHAIN_SERVICE_NAME,
        ])
        .output();

    // Add new keychain item with signature-based ACL for our signed binary
    let output = Command::new("security")
        .args([
            "add-generic-password",
            "-a",
            KEYCHAIN_ACCOUNT_NAME,
            "-s",
            KEYCHAIN_SERVICE_NAME,
            "-w",
            &cookies_json,
            "-A",  // Allow access without user interaction for trusted apps
            "-T",
            current_exe.to_str().unwrap_or(""),
            "-T",
            "/usr/bin/security", // Allow security command access
        ])
        .stdin(std::process::Stdio::null()) // Don't wait for input
        .output()
        .map_err(|e| CsCliError::Authentication(format!("Failed to store cookies: {e}")))?;

    if output.status.success() {
        info!("Platform cookies stored successfully in keychain");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(CsCliError::Authentication(format!(
            "Failed to store cookies in keychain: {}",
            stderr.trim()
        )))
    }
}

/// Retrieve platform cookies from keychain
pub fn get_stored_cookies() -> Result<HashMap<String, String>> {
    use std::process::Stdio;
    
    debug!("Attempting to retrieve stored cookies from keychain");

    let output = Command::new("security")
        .args([
            "find-generic-password",
            "-a",
            KEYCHAIN_ACCOUNT_NAME,
            "-s",
            KEYCHAIN_SERVICE_NAME,
            "-w", // Return password only
        ])
        .stdin(Stdio::null()) // Don't wait for input
        .output()
        .map_err(|e| CsCliError::Authentication(format!("Failed to access keychain: {e}")))?;

    if output.status.success() {
        let cookies_json = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !cookies_json.is_empty() {
            debug!("Successfully retrieved stored cookies from keychain");
            
            // Deserialize cookies from JSON
            let cookies: HashMap<String, String> = serde_json::from_str(&cookies_json)
                .map_err(|e| {
                    CsCliError::Authentication(format!("Failed to deserialize cookies: {e}"))
                })?;
            
            info!("Retrieved {} platform cookies from keychain", cookies.len());
            Ok(cookies)
        } else {
            Err(CsCliError::Authentication(
                "Retrieved empty cookie data from keychain".to_string(),
            ))
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        debug!("Keychain cookie retrieval failed: {}", stderr);
        Err(CsCliError::Authentication(
            "Could not retrieve stored cookies from keychain".to_string(),
        ))
    }
}

/// Check if stored cookies exist in keychain
pub fn has_stored_cookies() -> bool {
    use std::process::Stdio;
    
    debug!("Checking if stored cookies exist in keychain...");
    
    // Use a more lenient approach that doesn't block
    let result = std::process::Command::new("security")
        .args([
            "find-generic-password",
            "-a",
            KEYCHAIN_ACCOUNT_NAME,
            "-s",
            KEYCHAIN_SERVICE_NAME,
        ])
        .stdin(Stdio::null()) // Don't wait for input
        .stdout(Stdio::null()) // Don't capture output
        .stderr(Stdio::null()) // Don't capture errors
        .status(); // Just get exit status
        
    match result {
        Ok(status) => {
            let exists = status.success();
            debug!("Stored cookies exist: {}", exists);
            exists
        }
        Err(e) => {
            debug!("Failed to check stored cookies: {}", e);
            false
        }
    }
}

/// Delete stored cookies from keychain
pub fn delete_stored_cookies() -> Result<()> {
    info!("Deleting stored platform cookies from keychain...");

    let output = Command::new("security")
        .args([
            "delete-generic-password",
            "-a",
            KEYCHAIN_ACCOUNT_NAME,
            "-s",
            KEYCHAIN_SERVICE_NAME,
        ])
        .output()
        .map_err(|e| CsCliError::Authentication(format!("Failed to delete cookies: {e}")))?;

    if output.status.success() {
        info!("Platform cookies deleted successfully from keychain");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!("Failed to delete cookies from keychain: {}", stderr);
        // This is non-fatal - cookies might not have existed
        Ok(())
    }
}

/// Check if cookies are expired by examining common session cookie patterns
pub fn are_cookies_expired(cookies: &HashMap<String, String>) -> bool {
    // Look for common expiration indicators in cookie values
    for (name, value) in cookies {
        // Check for expired session tokens or timestamps
        if name.contains("session") || name.contains("token") || name.contains("auth") {
            // Simple heuristic: if the value looks like a timestamp and it's old, consider expired
            // This is a basic implementation - real expiration checking would be more sophisticated
            if value.len() < 10 || value == "expired" || value == "invalid" {
                debug!("Cookie {} appears to be expired", name);
                return true;
            }
        }
    }
    
    // For now, assume cookies are valid
    // In a real implementation, we'd check actual expiration times or test with API calls
    false
}

/// Validate cookies by testing them against a simple API endpoint
pub async fn validate_cookies(_cookies: &HashMap<String, String>) -> Result<bool> {
    // TODO: Implement actual cookie validation
    // This would involve making test API calls to each platform to verify session validity
    // For now, assume cookies are valid if they exist
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cookie_expiration_detection() {
        let mut cookies = HashMap::new();
        cookies.insert("valid_session".to_string(), "abc123def456".to_string());
        cookies.insert("user_id".to_string(), "12345".to_string());
        
        assert!(!are_cookies_expired(&cookies));
        
        // Add an expired-looking cookie
        cookies.insert("session_token".to_string(), "expired".to_string());
        assert!(are_cookies_expired(&cookies));
    }

    #[test]
    fn test_has_stored_cookies() {
        // This test doesn't actually create keychain entries
        // It just verifies the function doesn't panic
        let _result = has_stored_cookies();
    }
}