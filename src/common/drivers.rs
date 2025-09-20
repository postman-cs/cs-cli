use std::fs;
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::common::error::{CsCliError, Result};
use fantoccini::ClientBuilder;
use serde_json;
use tokio::process;
use tokio::signal;
use tokio::net::TcpListener;
use tracing::{error, info, warn};
use zstd::stream::read::Decoder as ZstdDecoder;
use regex::Regex;
use reqwest;

// Constants for WebDriver configuration
const DEFAULT_WEBDRIVER_PORT: u16 = 9515;
const PORT_SEARCH_RANGE: u16 = 100;
const DRIVER_INIT_WAIT_SECS: u64 = 2;

#[derive(Debug, Clone, Copy)]
pub enum Browser {
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

#[derive(Debug, Clone, Copy)]
pub enum Platform {
    MacosArm64,
    MacosX64,
    LinuxX64,
    WindowsX64,
}

impl Platform {
    pub fn current() -> Result<Self> {
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        return Ok(Self::MacosArm64);

        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        return Ok(Self::MacosX64);

        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        return Ok(Self::LinuxX64);

        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        return Ok(Self::WindowsX64);

        #[cfg(not(any(
            all(target_os = "macos", target_arch = "aarch64"),
            all(target_os = "macos", target_arch = "x86_64"),
            all(target_os = "linux", target_arch = "x86_64"),
            all(target_os = "windows", target_arch = "x86_64")
        )))]
        return Err(CsCliError::Generic("Unsupported platform".to_string()));
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MacosArm64 => "macos-arm64",
            Self::MacosX64 => "macos-x64",
            Self::LinuxX64 => "linux-x64",
            Self::WindowsX64 => "windows-x64",
        }
    }
}

/// Browser version information
#[derive(Debug, Clone)]
pub struct BrowserVersion {
    pub major: u32,
    pub minor: u32,
    pub build: u32,
    pub patch: u32,
}

impl BrowserVersion {
    /// Parse version string like "140.0.7339.80" into components
    pub fn parse(version_str: &str) -> Result<Self> {
        let re = Regex::new(r"^(\d+)\.(\d+)\.(\d+)\.(\d+)$")
            .map_err(|e| CsCliError::Generic(format!("Failed to compile version regex: {}", e)))?;
        
        if let Some(caps) = re.captures(version_str) {
            Ok(BrowserVersion {
                major: caps[1].parse()
                    .map_err(|e| CsCliError::Generic(format!("Failed to parse major version: {}", e)))?,
                minor: caps[2].parse()
                    .map_err(|e| CsCliError::Generic(format!("Failed to parse minor version: {}", e)))?,
                build: caps[3].parse()
                    .map_err(|e| CsCliError::Generic(format!("Failed to parse build version: {}", e)))?,
                patch: caps[4].parse()
                    .map_err(|e| CsCliError::Generic(format!("Failed to parse patch version: {}", e)))?,
            })
        } else {
            Err(CsCliError::Generic(format!("Invalid version format: {}", version_str)))
        }
    }

    /// Get the major version for ChromeDriver compatibility
    pub fn major_version(&self) -> u32 {
        self.major
    }

    /// Format as string
    pub fn to_string(&self) -> String {
        format!("{}.{}.{}.{}", self.major, self.minor, self.build, self.patch)
    }
}

pub struct DriverManager {
    cache_dir: PathBuf,
    platform: Platform,
    active_drivers: Arc<Mutex<Vec<process::Child>>>,
}

impl DriverManager {
    pub fn new() -> Result<Self> {
        let cache_dir = dirs::home_dir()
            .ok_or_else(|| CsCliError::Generic("Could not find home directory".to_string()))?
            .join(".cs-cli")
            .join("drivers");

        fs::create_dir_all(&cache_dir)
            .map_err(|e| CsCliError::Generic(format!("Failed to create cache dir: {e}")))?;

        let manager = Self {
            cache_dir,
            platform: Platform::current()?,
            active_drivers: Arc::new(Mutex::new(Vec::new())),
        };

        // Set up signal handlers for graceful shutdown
        manager.setup_signal_handlers()?;

        Ok(manager)
    }

    /// Detect browser version by running the browser binary
    pub fn detect_browser_version(&self, browser: Browser) -> Result<BrowserVersion> {
        let binary_path = Self::get_browser_binary_path(browser)
            .ok_or_else(|| CsCliError::Generic(format!("No binary path configured for {:?}", browser)))?;

        if !std::path::Path::new(binary_path).exists() {
            return Err(CsCliError::Generic(format!("Browser binary not found: {}", binary_path)));
        }

        // Try different version flags for different browsers
        let version_flags = match browser {
            Browser::Chrome | Browser::Brave | Browser::Edge | Browser::Arc | Browser::Chromium
            | Browser::Opera | Browser::OperaGX | Browser::Vivaldi | Browser::Comet => {
                vec!["--version", "-version", "--product-version"]
            }
            Browser::Firefox | Browser::LibreWolf | Browser::Zen => {
                vec!["--version", "-v"]
            }
            Browser::Safari => {
                // Safari version detection is more complex on macOS
                return self.detect_safari_version();
            }
        };

        for flag in version_flags {
            if let Ok(output) = Command::new(binary_path).arg(flag).output() {
                if output.status.success() {
                    let version_str = String::from_utf8_lossy(&output.stdout);
                    if let Ok(version) = self.parse_browser_version_output(&version_str, browser) {
                        info!("Detected {:?} version: {}", browser, version.to_string());
                        return Ok(version);
                    }
                }
            }
        }

        Err(CsCliError::Generic(format!("Could not detect version for {:?}", browser)))
    }

    /// Parse browser version from command output
    fn parse_browser_version_output(&self, output: &str, browser: Browser) -> Result<BrowserVersion> {
        // Common version patterns
        let patterns = match browser {
            Browser::Chrome | Browser::Brave | Browser::Edge | Browser::Arc | Browser::Chromium
            | Browser::Opera | Browser::OperaGX | Browser::Vivaldi | Browser::Comet => {
                vec![
                    r"(\d+\.\d+\.\d+\.\d+)",  // Standard Chrome format
                    r"ChromeDriver (\d+\.\d+\.\d+\.\d+)",  // ChromeDriver output
                ]
            }
            Browser::Firefox | Browser::LibreWolf | Browser::Zen => {
                vec![
                    r"Mozilla Firefox (\d+\.\d+\.\d+\.\d+)",
                    r"(\d+\.\d+\.\d+\.\d+)",
                ]
            }
            Browser::Safari => {
                vec![r"Version (\d+\.\d+\.\d+\.\d+)"]
            }
        };

        for pattern in patterns {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(caps) = re.captures(output) {
                    let version_str = caps.get(1).map_or(&caps[0], |m| m.as_str());
                    return BrowserVersion::parse(version_str);
                }
            }
        }

        Err(CsCliError::Generic(format!("Could not parse version from output: {}", output)))
    }

    /// Detect Safari version using system information
    fn detect_safari_version(&self) -> Result<BrowserVersion> {
        // On macOS, we can get Safari version from system info
        if cfg!(target_os = "macos") {
            if let Ok(output) = Command::new("defaults")
                .args(&["read", "/Applications/Safari.app/Contents/Info.plist", "CFBundleShortVersionString"])
                .output() {
                if output.status.success() {
                    let version_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    return BrowserVersion::parse(&version_str);
                }
            }
        }
        
        Err(CsCliError::Generic("Could not detect Safari version".to_string()))
    }

    /// Download ChromeDriver for a specific browser version
    pub async fn download_chromedriver_for_version(&self, browser_version: &BrowserVersion) -> Result<PathBuf> {
        let version_str = browser_version.to_string();
        let driver_name = format!("chromedriver-{}-{}", version_str, self.platform.as_str());
        let driver_path = self.cache_dir.join(&driver_name);

        // Check if we already have this version
        if driver_path.exists() && driver_path.metadata()?.permissions().mode() & 0o111 != 0 {
            info!("ChromeDriver {} already exists", version_str);
            return Ok(driver_path);
        }

        info!("Downloading ChromeDriver for browser version {}", version_str);

        // Try Chrome for Testing API first (more reliable for newer versions)
        if let Ok(path) = self.download_from_chrome_for_testing(browser_version, &driver_path).await {
            return Ok(path);
        }

        // Fallback to legacy ChromeDriver API
        if let Ok(path) = self.download_from_legacy_api(browser_version, &driver_path).await {
            return Ok(path);
        }

        Err(CsCliError::Generic(format!(
            "Could not download ChromeDriver for version {}", version_str
        )))
    }

    /// Download from Chrome for Testing API
    async fn download_from_chrome_for_testing(&self, browser_version: &BrowserVersion, target_path: &Path) -> Result<PathBuf> {
        let version_str = browser_version.to_string();
        let platform_str = match self.platform {
            Platform::MacosArm64 => "mac-arm64",
            Platform::MacosX64 => "mac-x64", 
            Platform::LinuxX64 => "linux64",
            Platform::WindowsX64 => "win64",
        };

        let url = format!(
            "https://storage.googleapis.com/chrome-for-testing-public/{}/mac-arm64/chromedriver-mac-arm64.zip",
            version_str
        );

        info!("Attempting to download from Chrome for Testing: {}", url);

        let client = reqwest::Client::new();
        let response = client.get(&url).send().await
            .map_err(|e| CsCliError::Generic(format!("Failed to download ChromeDriver: {}", e)))?;

        if !response.status().is_success() {
            return Err(CsCliError::Generic(format!("Download failed with status: {}", response.status())));
        }

        let bytes = response.bytes().await
            .map_err(|e| CsCliError::Generic(format!("Failed to read response: {}", e)))?;

        // Create temporary directory for extraction
        let temp_dir = self.cache_dir.join("temp");
        fs::create_dir_all(&temp_dir)
            .map_err(|e| CsCliError::Generic(format!("Failed to create temp dir: {}", e)))?;

        let zip_path = temp_dir.join("chromedriver.zip");
        fs::write(&zip_path, bytes)
            .map_err(|e| CsCliError::Generic(format!("Failed to write zip file: {}", e)))?;

        // Extract the zip file
        self.extract_chromedriver(&zip_path, target_path)?;

        // Clean up temp files
        fs::remove_file(&zip_path).ok();
        fs::remove_dir_all(&temp_dir).ok();

        Ok(target_path.to_path_buf())
    }

    /// Download from legacy ChromeDriver API (fallback)
    async fn download_from_legacy_api(&self, browser_version: &BrowserVersion, target_path: &Path) -> Result<PathBuf> {
        let major_version = browser_version.major_version();
        
        // Get the latest version for this major version
        let url = format!("https://chromedriver.storage.googleapis.com/LATEST_RELEASE_{}", major_version);
        
        let client = reqwest::Client::new();
        let response = client.get(&url).send().await
            .map_err(|e| CsCliError::Generic(format!("Failed to get latest version: {}", e)))?;

        if !response.status().is_success() {
            return Err(CsCliError::Generic(format!("Failed to get latest version: {}", response.status())));
        }

        let version_str = response.text().await
            .map_err(|e| CsCliError::Generic(format!("Failed to read version: {}", e)))?
            .trim().to_string();

        let platform_str = match self.platform {
            Platform::MacosArm64 => "mac64",
            Platform::MacosX64 => "mac64",
            Platform::LinuxX64 => "linux64", 
            Platform::WindowsX64 => "win32",
        };

        let download_url = format!(
            "https://chromedriver.storage.googleapis.com/{}/chromedriver_{}.zip",
            version_str, platform_str
        );

        info!("Downloading ChromeDriver {} from legacy API", version_str);

        let response = client.get(&download_url).send().await
            .map_err(|e| CsCliError::Generic(format!("Failed to download ChromeDriver: {}", e)))?;

        if !response.status().is_success() {
            return Err(CsCliError::Generic(format!("Download failed with status: {}", response.status())));
        }

        let bytes = response.bytes().await
            .map_err(|e| CsCliError::Generic(format!("Failed to read response: {}", e)))?;

        // Create temporary directory for extraction
        let temp_dir = self.cache_dir.join("temp");
        fs::create_dir_all(&temp_dir)
            .map_err(|e| CsCliError::Generic(format!("Failed to create temp dir: {}", e)))?;

        let zip_path = temp_dir.join("chromedriver.zip");
        fs::write(&zip_path, bytes)
            .map_err(|e| CsCliError::Generic(format!("Failed to write zip file: {}", e)))?;

        // Extract the zip file
        self.extract_chromedriver(&zip_path, target_path)?;

        // Clean up temp files
        fs::remove_file(&zip_path).ok();
        fs::remove_dir_all(&temp_dir).ok();

        Ok(target_path.to_path_buf())
    }

    /// Extract ChromeDriver from zip file
    fn extract_chromedriver(&self, zip_path: &Path, target_path: &Path) -> Result<()> {
        use zip::ZipArchive;
        use std::io::Cursor;

        let file = fs::File::open(zip_path)
            .map_err(|e| CsCliError::Generic(format!("Failed to open zip file: {}", e)))?;

        let mut archive = ZipArchive::new(file)
            .map_err(|e| CsCliError::Generic(format!("Failed to read zip archive: {}", e)))?;

        // Find the chromedriver executable in the archive
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)
                .map_err(|e| CsCliError::Generic(format!("Failed to read file from archive: {}", e)))?;

            let file_name = file.name();
            if file_name.ends_with("chromedriver") || file_name.ends_with("chromedriver.exe") {
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)
                    .map_err(|e| CsCliError::Generic(format!("Failed to read chromedriver: {}", e)))?;

                fs::write(target_path, buffer)
                    .map_err(|e| CsCliError::Generic(format!("Failed to write chromedriver: {}", e)))?;

                // Make executable
                let mut perms = fs::metadata(target_path)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(target_path, perms)
                    .map_err(|e| CsCliError::Generic(format!("Failed to set permissions: {}", e)))?;

                info!("Extracted ChromeDriver to {}", target_path.display());
                return Ok(());
            }
        }

        Err(CsCliError::Generic("ChromeDriver executable not found in archive".to_string()))
    }

    /// Download GeckoDriver for a specific Firefox version
    pub async fn download_geckodriver_for_version(&self, browser_version: &BrowserVersion) -> Result<PathBuf> {
        let version_str = browser_version.to_string();
        let driver_name = format!("geckodriver-{}-{}", version_str, self.platform.as_str());
        let driver_path = self.cache_dir.join(&driver_name);

        // Check if we already have this version
        if driver_path.exists() && driver_path.metadata()?.permissions().mode() & 0o111 != 0 {
            info!("GeckoDriver {} already exists", version_str);
            return Ok(driver_path);
        }

        info!("Downloading GeckoDriver for Firefox version {}", version_str);

        // Get the latest GeckoDriver version from GitHub releases
        let geckodriver_version = self.get_latest_geckodriver_version().await?;
        let platform_str = self.get_geckodriver_platform_string();

        let url = format!(
            "https://github.com/mozilla/geckodriver/releases/download/v{}/geckodriver-v{}-{}.tar.gz",
            geckodriver_version, geckodriver_version, platform_str
        );

        info!("Downloading GeckoDriver {} from GitHub", geckodriver_version);

        let client = reqwest::Client::new();
        let response = client.get(&url).send().await
            .map_err(|e| CsCliError::Generic(format!("Failed to download GeckoDriver: {}", e)))?;

        if !response.status().is_success() {
            return Err(CsCliError::Generic(format!("Download failed with status: {}", response.status())));
        }

        let bytes = response.bytes().await
            .map_err(|e| CsCliError::Generic(format!("Failed to read response: {}", e)))?;

        // Create temporary directory for extraction
        let temp_dir = self.cache_dir.join("temp");
        fs::create_dir_all(&temp_dir)
            .map_err(|e| CsCliError::Generic(format!("Failed to create temp dir: {}", e)))?;

        let tar_path = temp_dir.join("geckodriver.tar.gz");
        fs::write(&tar_path, bytes)
            .map_err(|e| CsCliError::Generic(format!("Failed to write tar file: {}", e)))?;

        // Extract the tar.gz file
        self.extract_geckodriver(&tar_path, &driver_path)?;

        // Clean up temp files
        fs::remove_file(&tar_path).ok();
        fs::remove_dir_all(&temp_dir).ok();

        Ok(driver_path)
    }

    /// Get the latest GeckoDriver version from GitHub API
    async fn get_latest_geckodriver_version(&self) -> Result<String> {
        let client = reqwest::Client::new();
        let response = client
            .get("https://api.github.com/repos/mozilla/geckodriver/releases/latest")
            .header("User-Agent", "cs-cli")
            .send().await
            .map_err(|e| CsCliError::Generic(format!("Failed to get GeckoDriver releases: {}", e)))?;

        if !response.status().is_success() {
            return Err(CsCliError::Generic(format!("Failed to get releases: {}", response.status())));
        }

        let json: serde_json::Value = response.json().await
            .map_err(|e| CsCliError::Generic(format!("Failed to parse JSON: {}", e)))?;

        let tag_name = json["tag_name"].as_str()
            .ok_or_else(|| CsCliError::Generic("No tag_name in response".to_string()))?;

        // Remove 'v' prefix if present
        Ok(tag_name.trim_start_matches('v').to_string())
    }

    /// Get platform string for GeckoDriver downloads
    fn get_geckodriver_platform_string(&self) -> &'static str {
        match self.platform {
            Platform::MacosArm64 => "macos-aarch64",
            Platform::MacosX64 => "macos",
            Platform::LinuxX64 => "linux64",
            Platform::WindowsX64 => "win64",
        }
    }

    /// Extract GeckoDriver from tar.gz file
    fn extract_geckodriver(&self, tar_path: &Path, target_path: &Path) -> Result<()> {
        use flate2::read::GzDecoder;
        use tar::Archive;

        let file = fs::File::open(tar_path)
            .map_err(|e| CsCliError::Generic(format!("Failed to open tar file: {}", e)))?;

        let gz_decoder = GzDecoder::new(file);
        let mut archive = Archive::new(gz_decoder);

        for entry in archive.entries()
            .map_err(|e| CsCliError::Generic(format!("Failed to read tar archive: {}", e)))? {
            let mut entry = entry
                .map_err(|e| CsCliError::Generic(format!("Failed to read tar entry: {}", e)))?;

            let path = entry.path()
                .map_err(|e| CsCliError::Generic(format!("Failed to get entry path: {}", e)))?;

            if path.file_name().and_then(|n| n.to_str()) == Some("geckodriver") {
                let mut buffer = Vec::new();
                entry.read_to_end(&mut buffer)
                    .map_err(|e| CsCliError::Generic(format!("Failed to read geckodriver: {}", e)))?;

                fs::write(target_path, buffer)
                    .map_err(|e| CsCliError::Generic(format!("Failed to write geckodriver: {}", e)))?;

                // Make executable
                let mut perms = fs::metadata(target_path)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(target_path, perms)
                    .map_err(|e| CsCliError::Generic(format!("Failed to set permissions: {}", e)))?;

                info!("Extracted GeckoDriver to {}", target_path.display());
                return Ok(());
            }
        }

        Err(CsCliError::Generic("GeckoDriver executable not found in archive".to_string()))
    }

    /// Ensure SafariDriver is available and compatible
    pub async fn ensure_safaridriver(&self, browser_version: &BrowserVersion) -> Result<PathBuf> {
        if !cfg!(target_os = "macos") {
            return Err(CsCliError::Generic("SafariDriver is only supported on macOS".to_string()));
        }

        let safaridriver_path = PathBuf::from("/usr/bin/safaridriver");
        
        if !safaridriver_path.exists() {
            return Err(CsCliError::Generic("SafariDriver not found at /usr/bin/safaridriver".to_string()));
        }

        // Check if SafariDriver version is compatible
        if let Ok(output) = Command::new(&safaridriver_path).arg("--version").output() {
            if output.status.success() {
                let version_output = String::from_utf8_lossy(&output.stdout);
                info!("SafariDriver version: {}", version_output.trim());
                
                // SafariDriver version should match Safari version
                // For now, we'll assume compatibility if SafariDriver exists
                return Ok(safaridriver_path);
            }
        }

        Err(CsCliError::Generic("SafariDriver is not working properly".to_string()))
    }

    /// Clean up old driver versions to save disk space
    pub fn cleanup_old_drivers(&self, keep_latest: usize) -> Result<()> {
        info!("Cleaning up old driver versions, keeping latest {}", keep_latest);

        // Group drivers by type and platform
        let mut driver_groups: std::collections::HashMap<String, Vec<(PathBuf, std::time::SystemTime)>> = std::collections::HashMap::new();

        if let Ok(entries) = fs::read_dir(&self.cache_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                        // Parse driver name pattern: driver-version-platform
                        if let Some((driver_type, version_platform)) = file_name.split_once('-') {
                            if driver_type == "chromedriver" || driver_type == "geckodriver" {
                                if let Ok(metadata) = entry.metadata() {
                                    if metadata.permissions().mode() & 0o111 != 0 {
                                        // It's an executable driver
                                        let modified = metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                                        driver_groups.entry(driver_type.to_string()).or_default().push((path.clone(), modified));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Clean up each driver type
        for (driver_type, mut drivers) in driver_groups {
            if drivers.len() <= keep_latest {
                continue; // Not enough drivers to clean up
            }

            // Sort by modification time (newest first)
            drivers.sort_by(|a, b| b.1.cmp(&a.1));

            // Remove old drivers
            for (path, _) in drivers.into_iter().skip(keep_latest) {
                if let Err(e) = fs::remove_file(&path) {
                    warn!("Failed to remove old driver {}: {}", path.display(), e);
                } else {
                    info!("Removed old driver: {}", path.display());
                }
            }
        }

        Ok(())
    }

    /// Get information about cached drivers
    pub fn get_driver_cache_info(&self) -> Result<Vec<(String, String, u64)>> {
        let mut drivers = Vec::new();

        if let Ok(entries) = fs::read_dir(&self.cache_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                        if let Some((driver_type, version_platform)) = file_name.split_once('-') {
                            if driver_type == "chromedriver" || driver_type == "geckodriver" {
                                if let Ok(metadata) = entry.metadata() {
                                    if metadata.permissions().mode() & 0o111 != 0 {
                                        let size = metadata.len();
                                        drivers.push((driver_type.to_string(), version_platform.to_string(), size));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(drivers)
    }

    fn setup_signal_handlers(&self) -> Result<()> {
        let drivers = Arc::clone(&self.active_drivers);
        
        tokio::spawn(async move {
            let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("Failed to create SIGTERM handler");
            let mut sigint = signal::unix::signal(signal::unix::SignalKind::interrupt())
                .expect("Failed to create SIGINT handler");

            tokio::select! {
                _ = sigterm.recv() => {
                    info!("Received SIGTERM, cleaning up driver processes");
                }
                _ = sigint.recv() => {
                    info!("Received SIGINT, cleaning up driver processes");
                }
            }

            // Clean up all active drivers - collect PIDs first to avoid Send issues
            let pids: Vec<u32> = {
                let drivers_guard = drivers.lock().unwrap();
                drivers_guard.iter()
                    .filter_map(|child| child.id())
                    .collect()
            };

            // Kill processes by PID to avoid Send issues
            for pid in pids {
                if let Err(e) = tokio::process::Command::new("kill")
                    .arg("-TERM")
                    .arg(pid.to_string())
                    .output()
                    .await
                {
                    warn!("Failed to kill driver process {}: {}", pid, e);
                }
            }
        });

        Ok(())
    }

    async fn find_available_port(&self, start_port: u16) -> Result<u16> {
        for port in start_port..start_port + PORT_SEARCH_RANGE {
            if let Ok(_) = TcpListener::bind(format!("127.0.0.1:{port}")).await {
                return Ok(port);
            }
        }
        Err(CsCliError::Generic("No available ports found in range".to_string()))
    }

    fn configure_stdio_for_tui(&self, cmd: &mut process::Command) {
        if std::env::var("CS_CLI_TUI_MODE").is_ok() {
            match std::env::var("CS_CLI_LOG_FILE").ok().and_then(|path| {
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&path)
                    .ok()
            }) {
                Some(log_file) => {
                    cmd.stdout(log_file.try_clone().unwrap());
                    cmd.stderr(log_file);
                }
                None => {
                    cmd.stdout(std::process::Stdio::null());
                    cmd.stderr(std::process::Stdio::null());
                }
            }
        }
    }

    pub async fn ensure_driver(&self, browser: Browser) -> Result<PathBuf> {
        match browser {
            Browser::Chrome | Browser::Brave | Browser::Edge | Browser::Arc | Browser::Chromium 
            | Browser::Opera | Browser::OperaGX | Browser::Vivaldi | Browser::Comet => {
                // For Chromium-based browsers, try automatic version detection and download
                self.ensure_chromedriver(browser).await
            }
            Browser::Firefox | Browser::LibreWolf | Browser::Zen => {
                // Firefox-based browsers use GeckoDriver (fallback to embedded)
                self.ensure_geckodriver(browser).await
            }
            Browser::Safari => {
                // SafariDriver is built into macOS, but we can check compatibility
                if cfg!(target_os = "macos") {
                    match self.detect_browser_version(browser) {
                        Ok(browser_version) => {
                            info!("Detected Safari version: {}", browser_version.to_string());
                            self.ensure_safaridriver(&browser_version).await
                        }
                        Err(e) => {
                            warn!("Failed to detect Safari version: {}", e);
                            // Fall back to basic SafariDriver check
                            let safaridriver_path = PathBuf::from("/usr/bin/safaridriver");
                            if safaridriver_path.exists() {
                                Ok(safaridriver_path)
                            } else {
                                Err(CsCliError::Generic("SafariDriver not found at /usr/bin/safaridriver".to_string()))
                            }
                        }
                    }
                } else {
                    Err(CsCliError::Generic("Safari is only supported on macOS".to_string()))
                }
            }
        }
    }

    /// Ensure ChromeDriver is available with automatic version detection
    async fn ensure_chromedriver(&self, browser: Browser) -> Result<PathBuf> {
        // First, try to detect browser version
        match self.detect_browser_version(browser) {
            Ok(browser_version) => {
                info!("Detected {:?} version: {}", browser, browser_version.to_string());
                
                // Try to download ChromeDriver for this specific version
                match self.download_chromedriver_for_version(&browser_version).await {
                    Ok(path) => {
                        info!("Successfully downloaded ChromeDriver for {:?} version {}", browser, browser_version.to_string());
                        return Ok(path);
                    }
                    Err(e) => {
                        warn!("Failed to download ChromeDriver for {:?} version {}: {}", browser, browser_version.to_string(), e);
                        // Fall back to embedded driver
                    }
                }
            }
            Err(e) => {
                warn!("Failed to detect {:?} version: {}", browser, e);
                // Fall back to embedded driver
            }
        }

        // Fallback to embedded ChromeDriver
        info!("Falling back to embedded ChromeDriver for {:?}", browser);
        self.ensure_embedded_chromedriver(browser).await
    }

    /// Ensure embedded ChromeDriver is available
    async fn ensure_embedded_chromedriver(&self, browser: Browser) -> Result<PathBuf> {
        let driver_path = self.cache_dir.join(format!("chromedriver-{}", self.platform.as_str()));

        // Check if driver already exists and is executable
        if driver_path.exists() {
            if let Ok(metadata) = fs::metadata(&driver_path) {
                let permissions = metadata.permissions();
                if permissions.mode() & 0o111 != 0 {
                    // Driver exists and is executable
                    return Ok(driver_path);
                }
            }
        }

        // Retrieve from embedded compressed binary
        self.retrieve_driver(browser, &driver_path)?;

        Ok(driver_path)
    }

    /// Ensure GeckoDriver is available with automatic version detection
    async fn ensure_geckodriver(&self, browser: Browser) -> Result<PathBuf> {
        // First, try to detect browser version
        match self.detect_browser_version(browser) {
            Ok(browser_version) => {
                info!("Detected {:?} version: {}", browser, browser_version.to_string());
                
                // Try to download GeckoDriver for this specific version
                match self.download_geckodriver_for_version(&browser_version).await {
                    Ok(path) => {
                        info!("Successfully downloaded GeckoDriver for {:?} version {}", browser, browser_version.to_string());
                        return Ok(path);
                    }
                    Err(e) => {
                        warn!("Failed to download GeckoDriver for {:?} version {}: {}", browser, browser_version.to_string(), e);
                        // Fall back to embedded driver
                    }
                }
            }
            Err(e) => {
                warn!("Failed to detect {:?} version: {}", browser, e);
                // Fall back to embedded driver
            }
        }

        // Fallback to embedded GeckoDriver
        info!("Falling back to embedded GeckoDriver for {:?}", browser);
        self.ensure_embedded_geckodriver(browser).await
    }

    /// Ensure embedded GeckoDriver is available
    async fn ensure_embedded_geckodriver(&self, browser: Browser) -> Result<PathBuf> {
        let driver_path = self.cache_dir.join(format!("geckodriver-{}", self.platform.as_str()));

        // Check if driver already exists and is executable
        if driver_path.exists() {
            if let Ok(metadata) = fs::metadata(&driver_path) {
                let permissions = metadata.permissions();
                if permissions.mode() & 0o111 != 0 {
                    // Driver exists and is executable
                    return Ok(driver_path);
                }
            }
        }

        // Retrieve from embedded compressed binary
        self.retrieve_driver(browser, &driver_path)?;

        Ok(driver_path)
    }

    fn retrieve_driver(&self, browser: Browser, target_path: &Path) -> Result<()> {
        let compressed_data = self.get_compressed_driver_data(browser)?;

        // Decompress with zstd
        let mut decoder = ZstdDecoder::new(compressed_data)
            .map_err(|e| CsCliError::Generic(format!("Failed to create zstd decoder: {e}")))?;

        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)
            .map_err(|e| CsCliError::Generic(format!("Failed to decompress driver: {e}")))?;

        // Write to file
        let mut file = fs::File::create(target_path)
            .map_err(|e| CsCliError::Generic(format!("Failed to create driver file: {e}")))?;
        file.write_all(&decompressed)
            .map_err(|e| CsCliError::Generic(format!("Failed to write driver file: {e}")))?;

        // Make executable (Unix only)
        #[cfg(unix)]
        {
            let mut permissions = fs::metadata(target_path)
                .map_err(|e| CsCliError::Generic(format!("Failed to get file metadata: {e}")))?
                .permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(target_path, permissions)
                .map_err(|e| CsCliError::Generic(format!("Failed to set permissions: {e}")))?;
        }

        Ok(())
    }

    fn get_compressed_driver_data(&self, browser: Browser) -> Result<&'static [u8]> {
        match (browser, self.platform) {
            (Browser::Chrome, Platform::MacosArm64) => {
                Ok(include_bytes!("../../vendor/drivers/compressed/macos-arm64/chromedriver.zst"))
            }
            (Browser::Firefox, Platform::MacosArm64) => {
                Ok(include_bytes!("../../vendor/drivers/compressed/macos-arm64/geckodriver.zst"))
            }
            // Add other platform combinations as needed
            _ => Err(CsCliError::Generic(format!(
                "No embedded driver for {:?} on {:?}",
                browser, self.platform
            )))
        }
    }

    pub async fn start_driver(&self, browser: Browser, preferred_port: u16) -> Result<(process::Child, u16)> {
        self.start_driver_with_profile(browser, preferred_port, None).await
    }

    pub async fn start_driver_with_profile(
        &self, 
        browser: Browser, 
        preferred_port: u16, 
        _profile_info: Option<&crate::common::auth::profile_detector::BrowserProfile>
    ) -> Result<(process::Child, u16)> {
        let driver_path = self.ensure_driver(browser).await?;
        let port = self.find_available_port(preferred_port).await?;
        info!("Starting {:?} driver on port {}", browser, port);

        let mut cmd = process::Command::new(&driver_path);

        match browser {
            Browser::Chrome | Browser::Brave | Browser::Edge | Browser::Arc | Browser::Chromium 
            | Browser::Opera | Browser::OperaGX | Browser::Vivaldi | Browser::Comet => {
                // All Chromium-based browsers use ChromeDriver
                cmd.arg(format!("--port={port}"));
                cmd.arg("--silent"); // Silent mode (suppresses all logs)
            }
            Browser::Firefox | Browser::LibreWolf | Browser::Zen => {
                // Firefox-based browsers use GeckoDriver
                cmd.arg("--port").arg(port.to_string());
                cmd.arg("--log-level").arg("fatal"); // Only fatal errors
            }
            Browser::Safari => {
                cmd.arg("--port").arg(port.to_string());
                // Safari driver doesn't have a quiet flag
            }
        }

        // Configure stdio redirection for TUI mode
        self.configure_stdio_for_tui(&mut cmd);

        // Start driver process
        let mut child = cmd.spawn()
            .map_err(|e| CsCliError::Generic(format!("Failed to start driver: {e}")))?;

        // Give driver time to start up
        tokio::time::sleep(Duration::from_secs(DRIVER_INIT_WAIT_SECS)).await;

        // Check if the process is still running
        if let Some(status) = child.try_wait().map_err(|e| CsCliError::Generic(format!("Failed to check driver status: {e}")))? {
            error!("Driver exited immediately with status: {:?}", status);
            return Err(CsCliError::Generic(format!("Driver exited immediately with status: {status:?}")));
        }

        // Additional validation: try to connect to the WebDriver endpoint to ensure it's responsive
        let test_url = format!("http://localhost:{port}/status");
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .map_err(|e| CsCliError::Generic(format!("Failed to create HTTP client for validation: {e}")))?;

        let mut retries = 3;
        let mut connected = false;

        while retries > 0 && !connected {
            match client.get(&test_url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        connected = true;
                        info!("WebDriver endpoint validated successfully on port {}", port);
                    } else {
                        warn!("WebDriver endpoint returned status {} on port {}", response.status(), port);
                    }
                }
                Err(e) => {
                    warn!("Failed to connect to WebDriver on port {}: {}. Retries left: {}", port, e, retries - 1);
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    retries -= 1;
                }
            }
        }

        if !connected {
            error!("WebDriver failed to become responsive on port {}", port);
            // Clean up the failed driver process
            let _ = child.kill().await;
            return Err(CsCliError::Generic(format!(
                "WebDriver failed to start properly on port {port}. This usually indicates browser launch issues or incompatible arguments."
            )));
        }

        Ok((child, port))
    }

    pub fn detect_active_browser(&self) -> Option<Browser> {
        #[cfg(target_os = "macos")]
        {
            let browser_configs = [
                (Browser::Chrome, "-x", "Google Chrome"),
                (Browser::Firefox, "-x", "Firefox"),
                (Browser::Safari, "-x", "Safari"),
                (Browser::Brave, "-f", "Brave Browser"),
                (Browser::Edge, "-f", "Microsoft Edge"),
                (Browser::Arc, "-x", "Arc"),
                (Browser::Chromium, "-x", "Chromium"),
                (Browser::LibreWolf, "-x", "LibreWolf"),
                (Browser::Opera, "-x", "Opera"),
                (Browser::OperaGX, "-f", "Opera GX"),
                (Browser::Vivaldi, "-x", "Vivaldi"),
                (Browser::Zen, "-x", "Zen"),
                (Browser::Comet, "-x", "Comet"),
            ];

            for (browser, flag, process_name) in &browser_configs {
                if let Ok(output) = Command::new("pgrep")
                    .arg(flag)
                    .arg(process_name)
                    .output()
                {
                    if output.status.success() && !output.stdout.is_empty() {
                        return Some(*browser);
                    }
                }
            }
        }

        Some(Browser::Chrome)
    }

    pub async fn connect(&self, browser: Option<Browser>) -> Result<fantoccini::Client> {
        self.connect_with_profile(browser, None).await
    }

    pub async fn connect_with_profile(
        &self, 
        browser: Option<Browser>, 
        profile_info: Option<&crate::common::auth::profile_detector::BrowserProfile>
    ) -> Result<fantoccini::Client> {
        let browser = browser.or_else(|| self.detect_active_browser())
            .unwrap_or(Browser::Chrome);

        let preferred_port = DEFAULT_WEBDRIVER_PORT;

        // Start the driver and get the actual port used  
        let (_driver_process, port) = self.start_driver_with_profile(browser, preferred_port, profile_info).await?;

        // Connect with fantoccini and profile-specific capabilities
        let mut capabilities = serde_json::Map::new();
        
        if let Some(profile) = profile_info {
            self.configure_profile_capabilities(&mut capabilities, browser, profile)?;
        }

        let client = if capabilities.is_empty() {
            ClientBuilder::native()
                .connect(&format!("http://localhost:{port}"))
                .await
                .map_err(|e| CsCliError::Generic(format!("Failed to connect to driver on port {port}: {e}")))?
        } else {
            ClientBuilder::native()
                .capabilities(capabilities)
                .connect(&format!("http://localhost:{port}"))
                .await
                .map_err(|e| CsCliError::Generic(format!("Failed to connect to driver with profile on port {port}: {e}")))?
        };

        Ok(client)
    }

    /// Get the browser binary path for a given browser type on macOS
    fn get_browser_binary_path(browser: Browser) -> Option<&'static str> {
        match browser {
            Browser::Chrome => Some("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"),
            Browser::Brave => Some("/Applications/Brave Browser.app/Contents/MacOS/Brave Browser"),
            Browser::Edge => Some("/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge"),
            Browser::Arc => Some("/Applications/Arc.app/Contents/MacOS/Arc"),
            Browser::Chromium => Some("/Applications/Chromium.app/Contents/MacOS/Chromium"),
            Browser::Opera => Some("/Applications/Opera.app/Contents/MacOS/Opera"),
            Browser::OperaGX => Some("/Applications/Opera GX.app/Contents/MacOS/Opera GX"),
            Browser::Vivaldi => Some("/Applications/Vivaldi.app/Contents/MacOS/Vivaldi"),
            Browser::Comet => Some("/Applications/Comet.app/Contents/MacOS/Comet"),
            Browser::Firefox => Some("/Applications/Firefox.app/Contents/MacOS/firefox"),
            Browser::LibreWolf => Some("/Applications/LibreWolf.app/Contents/MacOS/LibreWolf"),
            Browser::Zen => Some("/Applications/Zen.app/Contents/MacOS/Zen"),
            Browser::Safari => Some("/Applications/Safari.app/Contents/MacOS/Safari"),
        }
    }

    /// Get browser-specific automation arguments
    fn get_browser_automation_args(browser: Browser) -> Vec<String> {
        let mut args = Vec::new();

        // Essential stability arguments for all Chromium browsers
        args.push("--no-sandbox".to_string());
        args.push("--disable-dev-shm-usage".to_string());
        args.push("--disable-gpu".to_string());
        args.push("--disable-web-security".to_string());
        args.push("--no-first-run".to_string());
        args.push("--disable-default-apps".to_string());
        args.push("--disable-sync".to_string());
        args.push("--disable-translate".to_string());
        args.push("--disable-background-timer-throttling".to_string());
        args.push("--disable-backgrounding-occluded-windows".to_string());
        args.push("--disable-renderer-backgrounding".to_string());
        args.push("--disable-field-trial-config".to_string());
        args.push("--disable-ipc-flooding-protection".to_string());
        args.push("--disable-hang-monitor".to_string());
        args.push("--disable-prompt-on-repost".to_string());
        args.push("--disable-component-extensions-with-background-pages".to_string());
        args.push("--disable-extensions".to_string());
        args.push("--disable-plugins".to_string());
        args.push("--disable-plugins-discovery".to_string());
        args.push("--disable-print-preview".to_string());
        args.push("--disable-save-password-bubble".to_string());
        args.push("--disable-single-click-autofill".to_string());
        args.push("--disable-speech-api".to_string());
        args.push("--disable-speech-synthesis-api".to_string());
        args.push("--disable-xss-auditor".to_string());
        args.push("--disable-background-networking".to_string());
        args.push("--disable-client-side-phishing-detection".to_string());
        args.push("--disable-logging".to_string());
        args.push("--disable-notifications".to_string());
        args.push("--disable-popup-blocking".to_string());
        args.push("--disable-permissions-api".to_string());
        args.push("--metrics-recording-only".to_string());
        args.push("--safebrowsing-disable-auto-update".to_string());
        args.push("--enable-automation".to_string());
        args.push("--password-store=basic".to_string());
        args.push("--use-mock-keychain".to_string());
        args.push("--disable-blink-features=AutomationControlled".to_string());

        // Browser-specific arguments to prevent conflicts and crashes
        match browser {
            Browser::Brave => {
                // Brave-specific: minimal intervention - let Brave run with its default features
                // Only add essential automation stability without disabling Brave features
            }
            Browser::Edge => {
                args.push("--disable-edge-extensions".to_string());
                args.push("--disable-edge-sync".to_string());
                args.push("--disable-edge-translate".to_string());
                args.push("--disable-edge-news".to_string());
                args.push("--disable-edge-shopping".to_string());
                // Edge-specific stability
                args.push("--disable-features=msEdgeShopping".to_string());
                args.push("--disable-features=msEdgeCollections".to_string());
            }
            Browser::Arc => {
                args.push("--disable-arc-extensions".to_string());
                args.push("--disable-arc-sync".to_string());
                args.push("--disable-arc-translate".to_string());
                // Arc-specific stability (Arc has unique features)
                args.push("--disable-features=ArcCollections".to_string());
                args.push("--disable-features=ArcSidebar".to_string());
            }
            Browser::Opera | Browser::OperaGX => {
                args.push("--disable-opera-extensions".to_string());
                args.push("--disable-opera-sync".to_string());
                args.push("--disable-opera-translate".to_string());
                args.push("--disable-opera-news".to_string());
            }
            Browser::Vivaldi => {
                args.push("--disable-vivaldi-extensions".to_string());
                args.push("--disable-vivaldi-sync".to_string());
                args.push("--disable-vivaldi-translate".to_string());
            }
            Browser::Comet => {
                args.push("--disable-comet-extensions".to_string());
                args.push("--disable-comet-sync".to_string());
            }
            Browser::Chrome | Browser::Chromium => {
                // Chrome/Chromium: minimal additional args needed
                // These browsers are the most stable for automation
            }
            // Firefox-based browsers handled separately
            Browser::Firefox | Browser::LibreWolf | Browser::Zen | Browser::Safari => {}
        }

        args
    }

    /// Validate that a browser profile exists and is accessible
    fn validate_profile_access(profile: &crate::common::auth::profile_detector::BrowserProfile) -> Result<()> {
        if !profile.profile_path.exists() {
            return Err(CsCliError::Generic(format!(
                "Profile path does not exist: {}",
                profile.profile_path.display()
            )));
        }
        
        if !profile.profile_path.is_dir() {
            return Err(CsCliError::Generic(format!(
                "Profile path is not a directory: {}",
                profile.profile_path.display()
            )));
        }
        
        // Check if we can read the directory
        if let Err(e) = std::fs::read_dir(&profile.profile_path) {
            return Err(CsCliError::Generic(format!(
                "Cannot access profile directory {}: {}",
                profile.profile_path.display(),
                e
            )));
        }
        
        Ok(())
    }

    /// Configure WebDriver capabilities for profile-specific browser launching
    fn configure_profile_capabilities(
        &self,
        capabilities: &mut serde_json::Map<String, serde_json::Value>,
        browser: Browser,
        profile: &crate::common::auth::profile_detector::BrowserProfile,
    ) -> Result<()> {
        use crate::common::auth::profile_detector::BrowserType;
        
        // Validate profile access before proceeding
        match Self::validate_profile_access(profile) {
            Ok(()) => {
                info!("Profile validation successful for {:?}", browser);
            }
            Err(e) => {
                warn!("Profile validation failed for {:?}: {}", browser, e);
                // For some browsers, profile issues are recoverable - try without profile-specific args
                if matches!(browser, Browser::Chrome | Browser::Chromium) {
                    warn!("Attempting to continue without profile-specific configuration for {:?}", browser);
                    // Don't return error - let it try with default profile
                } else {
                    return Err(e);
                }
            }
        }

        match browser {
            Browser::Chrome | Browser::Brave | Browser::Edge | Browser::Arc | Browser::Chromium
            | Browser::Opera | Browser::OperaGX | Browser::Vivaldi | Browser::Comet => {
                // All Chromium-based browsers use the same WebDriver capabilities format
                let mut chrome_options = serde_json::Map::new();
                let mut args = Vec::new();

                // Set browser binary path
                if let Some(binary_path) = Self::get_browser_binary_path(browser) {
                    chrome_options.insert("binary".to_string(), serde_json::Value::String(binary_path.to_string()));
                    info!("Setting browser binary path for {:?}: {}", browser, binary_path);

                    // Verify binary exists
                    if !std::path::Path::new(binary_path).exists() {
                        warn!("Browser binary path does not exist: {}", binary_path);
                    }
                } else {
                    warn!("No browser binary path configured for {:?}", browser);
                }

                // Only set profile-specific arguments if profile validation succeeded
                if Self::validate_profile_access(profile).is_ok() {
                    // Set user data directory (parent of profile path)
                    if let Some(chrome_dir) = profile.profile_path.parent() {
                        args.push(format!("--user-data-dir={}", chrome_dir.display()));
                        info!("Setting user data directory: {}", chrome_dir.display());
                    }

                    // Set profile directory if not the default profile
                    if profile.profile_id != "Default" {
                        args.push(format!("--profile-directory={}", profile.profile_id));
                        info!("Setting profile directory: {}", profile.profile_id);
                    }
                } else {
                    info!("Skipping profile-specific arguments due to validation failure");
                }

                // Add browser-specific automation arguments
                let automation_args = Self::get_browser_automation_args(browser);
                args.extend(automation_args.clone());
                info!("Added {} automation arguments for {:?}", automation_args.len(), browser);

                chrome_options.insert("args".to_string(), serde_json::Value::Array(
                    args.into_iter().map(serde_json::Value::String).collect()
                ));

                capabilities.insert("goog:chromeOptions".to_string(), chrome_options.into());
            }
            Browser::Firefox | Browser::LibreWolf | Browser::Zen => {
                // Firefox-based browsers use similar configuration
                let mut firefox_options = serde_json::Map::new();
                
                // Set browser binary path
                if let Some(binary_path) = Self::get_browser_binary_path(browser) {
                    firefox_options.insert("binary".to_string(), serde_json::Value::String(binary_path.to_string()));
                    info!("Setting browser binary path for {:?}: {}", browser, binary_path);
                }
                
                // For Firefox, we need to set the profile path differently
                // Firefox profiles are typically handled via profile directories
                if matches!(profile.browser_type, BrowserType::Firefox | BrowserType::LibreWolf | BrowserType::Zen) {
                    firefox_options.insert("profile".to_string(),
                        serde_json::Value::String(profile.profile_path.display().to_string()));
                }

                capabilities.insert("moz:firefoxOptions".to_string(), firefox_options.into());
            }
            Browser::Safari => {
                // Safari doesn't support profile-specific launching in the same way
                // It uses the system's default Safari profile
                warn!("Safari profile-specific launching not fully supported, using system default");
            }
        }

        Ok(())
    }


    pub async fn cleanup_all_drivers(&self) -> Result<()> {
        // Move drivers out of the mutex to avoid holding lock across await
        let drivers = {
            let mut drivers_guard = self.active_drivers.lock().unwrap();
            drivers_guard.drain(..).collect::<Vec<_>>()
        };

        for mut driver in drivers {
            if let Err(e) = driver.kill().await {
                warn!("Failed to kill driver process: {}", e);
            }
        }
        Ok(())
    }
}

impl Drop for DriverManager {
    fn drop(&mut self) {
        // Clean up any remaining driver processes
        let drivers = Arc::clone(&self.active_drivers);
        let mut drivers_guard = drivers.lock().unwrap();
        for driver in drivers_guard.drain(..) {
            // Use blocking kill since we're in Drop
            if let Some(pid) = driver.id() {
                if let Err(e) = std::process::Command::new("kill")
                    .arg("-TERM")
                    .arg(pid.to_string())
                    .output()
                {
                    warn!("Failed to kill driver process {}: {}", pid, e);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        let platform = Platform::current();
        assert!(platform.is_ok());
    }

    #[tokio::test]
    async fn test_driver_retrieval() {
        let manager = DriverManager::new().unwrap();

        // Test Chrome driver retrieval
        let chrome_path = manager.ensure_driver(Browser::Chrome).await;
        assert!(chrome_path.is_ok());

        // Test Firefox driver retrieval
        let firefox_path = manager.ensure_driver(Browser::Firefox).await;
        assert!(firefox_path.is_ok());
    }

    #[tokio::test]
    async fn test_port_fallback() {
        let manager = DriverManager::new().unwrap();
        
        // Test finding available port
        let port = manager.find_available_port(9515).await;
        assert!(port.is_ok());
        assert!(port.unwrap() >= 9515);
    }

    #[tokio::test]
    async fn test_driver_startup() {
        let manager = DriverManager::new().unwrap();
        
        // Test driver startup with port fallback
        let result = manager.start_driver(Browser::Chrome, 9515).await;
        if result.is_ok() {
            let (mut child, port) = result.unwrap();
            assert!(port >= 9515);
            
            // Clean up
            let _ = child.kill().await;
        }
    }
}