use std::process::{Command, Stdio};
use tokio::time::{timeout, Duration};

/// Check for updates and automatically install if available
pub async fn check_and_update() -> crate::Result<()> {
    // Get current version from Cargo.toml
    let current_version = env!("CARGO_PKG_VERSION");

    // Check latest version from GitHub API with timeout
    let latest_version = match timeout(Duration::from_secs(5), get_latest_version()).await {
        Ok(Ok(version)) => version,
        Ok(Err(_)) => return Ok(()), // Network error, skip update
        Err(_) => return Ok(()),     // Timeout, skip update
    };

    // Compare versions
    if version_is_newer(&latest_version, current_version) {
        println!(
            "🔄 Update available: v{} → v{}",
            current_version, latest_version
        );
        println!("📦 Installing update...");

        // Run the install script
        let result = Command::new("bash")
            .arg("-c")
            .arg("curl -s https://raw.githubusercontent.com/postman-cs/cs-cli/main/install.sh | bash")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        match result {
            Ok(status) if status.success() => {
                println!("✅ Update completed! Please restart cs-cli.");
                std::process::exit(0);
            }
            Ok(_) => {
                println!("⚠️  Update failed. Continuing with current version.");
            }
            Err(_) => {
                println!("⚠️  Update failed. Continuing with current version.");
            }
        }
    }

    Ok(())
}

/// Get latest version from GitHub API
async fn get_latest_version() -> crate::Result<String> {
    let client = reqwest::Client::builder()
        .user_agent("cs-cli")
        .timeout(Duration::from_secs(3))
        .build()
        .map_err(|e| crate::CsCliError::Generic(format!("HTTP client error: {}", e)))?;

    let response = client
        .get("https://api.github.com/repos/postman-cs/cs-cli/releases/latest")
        .send()
        .await
        .map_err(|e| crate::CsCliError::Generic(format!("Network error: {}", e)))?;

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| crate::CsCliError::Generic(format!("JSON error: {}", e)))?;

    let tag_name = json["tag_name"]
        .as_str()
        .ok_or_else(|| crate::CsCliError::Generic("Invalid API response".to_string()))?;

    // Remove 'v' prefix if present
    Ok(tag_name.strip_prefix('v').unwrap_or(tag_name).to_string())
}

/// Simple version comparison
fn version_is_newer(latest: &str, current: &str) -> bool {
    let latest_parts: Vec<u32> = latest.split('.').filter_map(|s| s.parse().ok()).collect();
    let current_parts: Vec<u32> = current.split('.').filter_map(|s| s.parse().ok()).collect();

    // Pad with zeros if needed
    let max_len = latest_parts.len().max(current_parts.len());
    let latest_padded: Vec<u32> = (0..max_len)
        .map(|i| latest_parts.get(i).copied().unwrap_or(0))
        .collect();
    let current_padded: Vec<u32> = (0..max_len)
        .map(|i| current_parts.get(i).copied().unwrap_or(0))
        .collect();

    latest_padded > current_padded
}
