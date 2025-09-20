//! Browser profile detection and management
//!
//! This module provides functionality to discover and select browser profiles
//! for platform-specific authentication. It supports all major browsers with
//! a configuration-driven approach for easy maintenance.

use anyhow::{Context, Result};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;
use tracing::{debug, info, warn};

/// Supported browser types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BrowserType {
    Chrome,
    Firefox,
    Safari,
    Brave,
    Edge,
    Arc,
    Chromium,
    LibreWolf,
    Opera,
    OperaGX,
    Vivaldi,
    Zen,
    Comet,
}

/// Platform types for profile association
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlatformType {
    Gong,
    Slack,
    Gainsight,
    GitHub,
    Personal,
    Work,
}

/// Represents a discovered browser profile
#[derive(Debug, Clone)]
pub struct BrowserProfile {
    pub browser_type: BrowserType,
    pub profile_id: String,
    pub profile_name: String,
    pub email: Option<String>,
    pub profile_path: PathBuf,
    pub last_active: Option<SystemTime>,
    pub is_managed: bool,
    pub hosted_domain: Option<String>,
}

impl BrowserProfile {
    /// Check if this profile is likely a work profile
    pub fn is_work_profile(&self) -> bool {
        if let Some(email) = &self.email {
            return email.contains("@postman.com") 
                || email.contains("@company.") 
                || email.contains("@corp.")
                || self.is_managed;
        }
        
        self.profile_name.to_lowercase().contains("work") 
            || self.profile_name.to_lowercase().contains("business")
            || self.is_managed
    }

    /// Check if this profile matches a specific platform requirement
    pub fn matches_platform(&self, platform: &PlatformType) -> bool {
        match platform {
            PlatformType::Gong | PlatformType::Slack | PlatformType::Work => {
                self.is_work_profile()
            }
            PlatformType::GitHub | PlatformType::Personal => {
                !self.is_work_profile()
            }
            PlatformType::Gainsight => true, // Could be either
        }
    }

    /// Get a human-readable description of the profile
    pub fn description(&self) -> String {
        let mut desc = format!("{} ({})", self.profile_name, self.profile_id);
        if let Some(email) = &self.email {
            desc.push_str(&format!(" [{email}]"));
        }
        desc
    }
}

/// Browser configuration for discovery
#[derive(Debug, Clone)]
struct BrowserConfig {
    name: &'static str,
    browser_type: BrowserType,
    engine: BrowserEngine,
    app_support_path: Vec<&'static str>,
}

/// Browser engine type determines profile format
#[derive(Debug, Clone)]
enum BrowserEngine {
    Chromium, // Uses Local State JSON format
    Firefox,  // Uses profiles directory format
}

/// Main profile detector
pub struct ProfileDetector {
    pub detected_profiles: Vec<BrowserProfile>,
}

impl ProfileDetector {
    /// Create a new ProfileDetector and discover all available profiles
    pub fn new() -> Result<Self> {
        info!("Starting browser profile discovery...");
        
        let mut profiles = Vec::new();
        
        // Discover Chrome profiles (special case - most common)
        match Self::discover_chrome_profiles() {
            Ok(mut chrome_profiles) => {
                info!("Found {} Chrome profiles", chrome_profiles.len());
                profiles.append(&mut chrome_profiles);
            }
            Err(e) => {
                warn!("Failed to discover Chrome profiles: {}", e);
            }
        }

        // Discover Firefox profiles (special case - different format)
        match Self::discover_firefox_profiles() {
            Ok(mut firefox_profiles) => {
                info!("Found {} Firefox profiles", firefox_profiles.len());
                profiles.append(&mut firefox_profiles);
            }
            Err(e) => {
                warn!("Failed to discover Firefox profiles: {}", e);
            }
        }
        
        // Discover all other browsers using configuration
        let browser_configs = Self::get_browser_configs();
        
        for config in browser_configs {
            match Self::discover_browser_profiles(&config) {
                Ok(mut browser_profiles) => {
                    if !browser_profiles.is_empty() {
                        info!("Found {} {} profiles", browser_profiles.len(), config.name);
                        profiles.append(&mut browser_profiles);
                    }
                }
                Err(e) => {
                    warn!("Failed to discover {} profiles: {}", config.name, e);
                }
            }
        }
        
        info!("Total discovered profiles: {}", profiles.len());
        for profile in &profiles {
            debug!("Profile: {}", profile.description());
        }
        
        Ok(Self {
            detected_profiles: profiles,
        })
    }

    /// Get configuration for all supported browsers (excluding Chrome/Firefox which are handled separately)
    fn get_browser_configs() -> Vec<BrowserConfig> {
        vec![
            BrowserConfig {
                name: "Brave",
                browser_type: BrowserType::Brave,
                engine: BrowserEngine::Chromium,
                app_support_path: vec!["BraveSoftware", "Brave-Browser"],
            },
            BrowserConfig {
                name: "Edge",
                browser_type: BrowserType::Edge,
                engine: BrowserEngine::Chromium,
                app_support_path: vec!["Microsoft Edge"],
            },
            BrowserConfig {
                name: "Arc",
                browser_type: BrowserType::Arc,
                engine: BrowserEngine::Chromium,
                app_support_path: vec!["Arc", "User Data"],
            },
            BrowserConfig {
                name: "Chromium",
                browser_type: BrowserType::Chromium,
                engine: BrowserEngine::Chromium,
                app_support_path: vec!["Chromium"],
            },
            BrowserConfig {
                name: "Opera",
                browser_type: BrowserType::Opera,
                engine: BrowserEngine::Chromium,
                app_support_path: vec!["com.operasoftware.Opera"],
            },
            BrowserConfig {
                name: "Opera GX",
                browser_type: BrowserType::OperaGX,
                engine: BrowserEngine::Chromium,
                app_support_path: vec!["com.operasoftware.OperaGX"],
            },
            BrowserConfig {
                name: "Vivaldi",
                browser_type: BrowserType::Vivaldi,
                engine: BrowserEngine::Chromium,
                app_support_path: vec!["Vivaldi"],
            },
            BrowserConfig {
                name: "Comet",
                browser_type: BrowserType::Comet,
                engine: BrowserEngine::Chromium,
                app_support_path: vec!["Comet"],
            },
            BrowserConfig {
                name: "LibreWolf",
                browser_type: BrowserType::LibreWolf,
                engine: BrowserEngine::Firefox,
                app_support_path: vec!["LibreWolf", "Profiles"],
            },
            BrowserConfig {
                name: "Zen",
                browser_type: BrowserType::Zen,
                engine: BrowserEngine::Firefox,
                app_support_path: vec!["Zen", "Profiles"],
            },
        ]
    }

    /// Generic browser profile discovery
    fn discover_browser_profiles(config: &BrowserConfig) -> Result<Vec<BrowserProfile>> {
        let home = std::env::var("HOME")
            .context("HOME environment variable not set")?;
            
        let mut browser_dir = PathBuf::from(home)
            .join("Library")
            .join("Application Support");
            
        for segment in &config.app_support_path {
            browser_dir = browser_dir.join(segment);
        }

        match config.engine {
            BrowserEngine::Chromium => Self::discover_chromium_based_profiles(config, &browser_dir),
            BrowserEngine::Firefox => Self::discover_firefox_based_profiles(config, &browser_dir),
        }
    }

    /// Discover Chromium-based browser profiles (uses Local State JSON)
    fn discover_chromium_based_profiles(
        config: &BrowserConfig, 
        browser_dir: &Path
    ) -> Result<Vec<BrowserProfile>> {
        let local_state_path = browser_dir.join("Local State");
        
        if !local_state_path.exists() {
            return Err(anyhow::anyhow!("{} Local State not found at: {}", config.name, local_state_path.display()));
        }
        
        debug!("Reading {} Local State from: {}", config.name, local_state_path.display());
        let local_state_content = fs::read_to_string(&local_state_path)
            .context("Failed to read Local State file")?;
            
        let local_state: Value = serde_json::from_str(&local_state_content)
            .context("Failed to parse Local State JSON")?;
            
        let mut profiles = Vec::new();
        
        if let Some(profile_info_cache) = local_state["profile"]["info_cache"].as_object() {
            for (profile_id, profile_data) in profile_info_cache {
                let profile = Self::parse_chromium_profile(profile_id, profile_data, browser_dir, config.browser_type)?;
                profiles.push(profile);
            }
        }
        
        Ok(profiles)
    }

    /// Parse Chromium-based browser profile from JSON data
    fn parse_chromium_profile(
        profile_id: &str, 
        profile_data: &Value, 
        browser_dir: &Path,
        browser_type: BrowserType,
    ) -> Result<BrowserProfile> {
        let profile_name = profile_data["name"].as_str().unwrap_or("Unknown").to_string();
        let email = profile_data["user_name"].as_str()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        let is_managed = profile_data["is_managed"].as_i64().unwrap_or(0) != 0;
        let hosted_domain = profile_data["hosted_domain"].as_str()
            .filter(|s| !s.is_empty() && *s != "NO_HOSTED_DOMAIN")
            .map(|s| s.to_string());
            
        let profile_path = if profile_id == "Default" {
            browser_dir.join("Default")
        } else {
            browser_dir.join(profile_id)
        };
        
        let last_active = profile_data["active_time"].as_f64()
            .map(|timestamp| {
                let duration = std::time::Duration::from_secs_f64(timestamp);
                SystemTime::UNIX_EPOCH + duration
            });
            
        Ok(BrowserProfile {
            browser_type,
            profile_id: profile_id.to_string(),
            profile_name,
            email,
            profile_path,
            last_active,
            is_managed,
            hosted_domain,
        })
    }

    /// Discover Firefox-based browser profiles (uses profiles directory)
    fn discover_firefox_based_profiles(
        config: &BrowserConfig, 
        profiles_dir: &Path
    ) -> Result<Vec<BrowserProfile>> {
        if !profiles_dir.exists() {
            return Err(anyhow::anyhow!("{} profiles directory not found: {}", config.name, profiles_dir.display()));
        }
        
        debug!("Scanning {} profiles directory: {}", config.name, profiles_dir.display());
        let entries = fs::read_dir(profiles_dir)
            .context("Failed to read profiles directory")?;
            
        let mut profiles = Vec::new();
        
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let profile_name = entry.file_name().to_string_lossy().to_string();
                
                if profile_name.starts_with('.') || !profile_name.contains('.') {
                    continue;
                }
                
                let profile = Self::parse_firefox_based_profile(&profile_name, &entry.path(), config.browser_type)?;
                profiles.push(profile);
            }
        }
        
        Ok(profiles)
    }

    /// Parse Firefox-based browser profile
    fn parse_firefox_based_profile(
        profile_name: &str, 
        profile_path: &Path,
        browser_type: BrowserType,
    ) -> Result<BrowserProfile> {
        let last_active = fs::metadata(profile_path)
            .ok()
            .and_then(|metadata| metadata.modified().ok());
            
        let display_name = if let Some(dot_pos) = profile_name.find('.') {
            profile_name[dot_pos + 1..].to_string()
        } else {
            profile_name.to_string()
        };
        
        Ok(BrowserProfile {
            browser_type,
            profile_id: profile_name.to_string(),
            profile_name: display_name,
            email: None,
            profile_path: profile_path.to_path_buf(),
            last_active,
            is_managed: false,
            hosted_domain: None,
        })
    }

    /// Discover Chrome profiles (kept separate for legacy compatibility)
    fn discover_chrome_profiles() -> Result<Vec<BrowserProfile>> {
        let chrome_dir = Self::get_chrome_directory()?;
        let local_state_path = chrome_dir.join("Local State");
        
        if !local_state_path.exists() {
            return Err(anyhow::anyhow!("Chrome Local State not found at: {}", local_state_path.display()));
        }
        
        debug!("Reading Chrome Local State from: {}", local_state_path.display());
        let local_state_content = fs::read_to_string(&local_state_path)
            .context("Failed to read Chrome Local State file")?;
            
        let local_state: Value = serde_json::from_str(&local_state_content)
            .context("Failed to parse Chrome Local State JSON")?;
            
        let mut profiles = Vec::new();
        
        if let Some(profile_info_cache) = local_state["profile"]["info_cache"].as_object() {
            for (profile_id, profile_data) in profile_info_cache {
                let profile = Self::parse_chrome_profile(profile_id, profile_data, &chrome_dir)?;
                profiles.push(profile);
            }
        }
        
        Ok(profiles)
    }
    
    /// Parse Chrome profile
    fn parse_chrome_profile(
        profile_id: &str, 
        profile_data: &Value, 
        chrome_dir: &Path
    ) -> Result<BrowserProfile> {
        let profile_name = profile_data["name"].as_str().unwrap_or("Unknown").to_string();
        let email = profile_data["user_name"].as_str()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        let is_managed = profile_data["is_managed"].as_i64().unwrap_or(0) != 0;
        let hosted_domain = profile_data["hosted_domain"].as_str()
            .filter(|s| !s.is_empty() && *s != "NO_HOSTED_DOMAIN")
            .map(|s| s.to_string());
            
        let profile_path = if profile_id == "Default" {
            chrome_dir.join("Default")
        } else {
            chrome_dir.join(profile_id)
        };
        
        let last_active = profile_data["active_time"].as_f64()
            .map(|timestamp| {
                let duration = std::time::Duration::from_secs_f64(timestamp);
                SystemTime::UNIX_EPOCH + duration
            });
            
        Ok(BrowserProfile {
            browser_type: BrowserType::Chrome,
            profile_id: profile_id.to_string(),
            profile_name,
            email,
            profile_path,
            last_active,
            is_managed,
            hosted_domain,
        })
    }

    /// Discover Firefox profiles (kept separate for legacy compatibility)
    fn discover_firefox_profiles() -> Result<Vec<BrowserProfile>> {
        let firefox_profiles_dir = Self::get_firefox_profiles_directory()?;
        
        if !firefox_profiles_dir.exists() {
            return Err(anyhow::anyhow!("Firefox profiles directory not found: {}", firefox_profiles_dir.display()));
        }
        
        debug!("Scanning Firefox profiles directory: {}", firefox_profiles_dir.display());
        let entries = fs::read_dir(&firefox_profiles_dir)
            .context("Failed to read Firefox profiles directory")?;
            
        let mut profiles = Vec::new();
        
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let profile_name = entry.file_name().to_string_lossy().to_string();
                
                if profile_name.starts_with('.') || !profile_name.contains('.') {
                    continue;
                }
                
                let profile = Self::parse_firefox_profile(&profile_name, &entry.path())?;
                profiles.push(profile);
            }
        }
        
        Ok(profiles)
    }
    
    /// Parse Firefox profile
    fn parse_firefox_profile(profile_name: &str, profile_path: &Path) -> Result<BrowserProfile> {
        let last_active = fs::metadata(profile_path)
            .ok()
            .and_then(|metadata| metadata.modified().ok());
            
        let display_name = if let Some(dot_pos) = profile_name.find('.') {
            profile_name[dot_pos + 1..].to_string()
        } else {
            profile_name.to_string()
        };
        
        Ok(BrowserProfile {
            browser_type: BrowserType::Firefox,
            profile_id: profile_name.to_string(),
            profile_name: display_name,
            email: None,
            profile_path: profile_path.to_path_buf(),
            last_active,
            is_managed: false,
            hosted_domain: None,
        })
    }

    /// Find the best profile for a specific platform
    pub fn find_best_profile_for_platform(&self, platform: &PlatformType) -> Option<&BrowserProfile> {
        let mut candidates: Vec<&BrowserProfile> = self.detected_profiles
            .iter()
            .filter(|profile| profile.matches_platform(platform))
            .collect();
            
        if candidates.is_empty() {
            return self.detected_profiles
                .iter()
                .max_by_key(|profile| profile.last_active);
        }
        
        // NEW APPROACH: Find the best browser first, then best profile for that browser
        let best_profile = self.find_best_browser_profile_combination(&candidates);
        if let Some(profile) = best_profile {
            return Some(profile);
        }
        
        // Fallback to old logic if no good combination found
        candidates.sort_by(|a, b| b.last_active.cmp(&a.last_active));
        candidates.first().copied()
    }
    
    /// Find the best browser-profile combination by checking running browsers and cookie quality
    fn find_best_browser_profile_combination<'a>(&self, candidates: &[&'a BrowserProfile]) -> Option<&'a BrowserProfile> {
        
        // Check which browsers are currently running
        let running_browsers = self.detect_running_browsers();
        
        // Group candidates by browser type
        let mut browser_groups: std::collections::HashMap<BrowserType, Vec<&BrowserProfile>> = 
            std::collections::HashMap::new();
        
        for profile in candidates {
            browser_groups.entry(profile.browser_type).or_default().push(*profile);
        }
        
        // Priority order: Running browsers first, then by browser preference
        let browser_priority = [
            BrowserType::Brave,    // Brave often has better enterprise cookie handling
            BrowserType::Chrome,   // Chrome is most common
            BrowserType::Edge,     // Edge has good enterprise support
            BrowserType::Arc,      // Arc is becoming popular
            BrowserType::Firefox,  // Firefox as fallback
            BrowserType::Safari,   // Safari last (limited automation support)
        ];
        
        // First, try running browsers
        for browser_type in &browser_priority {
            if running_browsers.contains(browser_type) {
                if let Some(profiles) = browser_groups.get(browser_type) {
                    // Return most recently active profile for this running browser
                    let best_profile = profiles.iter()
                        .max_by_key(|p| p.last_active)?;
                    info!("Selected profile from running browser {:?}: {}", browser_type, best_profile.description());
                    return Some(*best_profile);
                }
            }
        }
        
        // If no running browsers have profiles, use priority order
        for browser_type in &browser_priority {
            if let Some(profiles) = browser_groups.get(browser_type) {
                let best_profile = profiles.iter()
                    .max_by_key(|p| p.last_active)?;
                info!("Selected profile from preferred browser {:?}: {}", browser_type, best_profile.description());
                return Some(*best_profile);
            }
        }
        
        None
    }
    
    /// Detect which browsers are currently running
    fn detect_running_browsers(&self) -> std::collections::HashSet<BrowserType> {
        let mut running = std::collections::HashSet::new();
        
        #[cfg(target_os = "macos")]
        {
            let browser_processes = [
                (BrowserType::Chrome, vec!["Google Chrome"]),
                (BrowserType::Brave, vec!["Brave Browser"]),
                (BrowserType::Firefox, vec!["Firefox"]),
                (BrowserType::Safari, vec!["Safari"]),
                (BrowserType::Edge, vec!["Microsoft Edge"]),
                (BrowserType::Arc, vec!["Arc"]),
                (BrowserType::Chromium, vec!["Chromium"]),
            ];
            
            for (browser_type, process_names) in &browser_processes {
                for process_name in process_names {
                    if let Ok(output) = Command::new("pgrep")
                        .arg("-f")
                        .arg(process_name)
                        .output()
                    {
                        if output.status.success() && !output.stdout.is_empty() {
                            running.insert(*browser_type);
                            break;
                        }
                    }
                }
            }
        }
        
        running
    }

    /// Find work-related profiles
    pub fn find_work_profiles(&self) -> Vec<&BrowserProfile> {
        self.detected_profiles
            .iter()
            .filter(|profile| profile.is_work_profile())
            .collect()
    }

    /// Find personal profiles
    pub fn find_personal_profiles(&self) -> Vec<&BrowserProfile> {
        self.detected_profiles
            .iter()
            .filter(|profile| !profile.is_work_profile())
            .collect()
    }

    /// Get most recently active profile
    pub fn get_most_recent_profile(&self) -> Option<&BrowserProfile> {
        self.detected_profiles
            .iter()
            .max_by_key(|profile| profile.last_active)
    }

    /// Get Chrome directory
    fn get_chrome_directory() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .context("HOME environment variable not set")?;
            
        Ok(PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("Google")
            .join("Chrome"))
    }
    
    /// Get Firefox profiles directory
    fn get_firefox_profiles_directory() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .context("HOME environment variable not set")?;
            
        Ok(PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("Firefox")
            .join("Profiles"))
    }
}

impl Default for ProfileDetector {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            detected_profiles: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_work_profile_detection() {
        let work_profile = BrowserProfile {
            browser_type: BrowserType::Chrome,
            profile_id: "Profile 1".to_string(),
            profile_name: "Work".to_string(),
            email: Some("jared.boynton@postman.com".to_string()),
            profile_path: PathBuf::from("/test"),
            last_active: None,
            is_managed: true,
            hosted_domain: Some("postman.com".to_string()),
        };
        
        assert!(work_profile.is_work_profile());
        assert!(work_profile.matches_platform(&PlatformType::Gong));
        assert!(work_profile.matches_platform(&PlatformType::Work));
    }

    #[test]
    fn test_personal_profile_detection() {
        let personal_profile = BrowserProfile {
            browser_type: BrowserType::Chrome,
            profile_id: "Profile 2".to_string(),
            profile_name: "Personal".to_string(),
            email: Some("jared.boynton@gmail.com".to_string()),
            profile_path: PathBuf::from("/test"),
            last_active: None,
            is_managed: false,
            hosted_domain: None,
        };
        
        assert!(!personal_profile.is_work_profile());
        assert!(personal_profile.matches_platform(&PlatformType::Personal));
        assert!(personal_profile.matches_platform(&PlatformType::GitHub));
    }

    #[test]
    fn test_browser_config_generation() {
        let configs = ProfileDetector::get_browser_configs();
        
        // Should have all the expected browsers
        assert!(configs.iter().any(|c| c.name == "Brave"));
        assert!(configs.iter().any(|c| c.name == "Edge"));
        assert!(configs.iter().any(|c| c.name == "Arc"));
        assert!(configs.iter().any(|c| c.name == "Vivaldi"));
        assert!(configs.iter().any(|c| c.name == "Opera"));
        assert!(configs.iter().any(|c| c.name == "LibreWolf"));
        assert!(configs.iter().any(|c| c.name == "Zen"));
        
        // Verify engine assignments
        let brave_config = configs.iter().find(|c| c.name == "Brave").unwrap();
        assert!(matches!(brave_config.engine, BrowserEngine::Chromium));
        
        let librewolf_config = configs.iter().find(|c| c.name == "LibreWolf").unwrap();
        assert!(matches!(librewolf_config.engine, BrowserEngine::Firefox));
    }
}