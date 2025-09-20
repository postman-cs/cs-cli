use std::fs;
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use crate::common::error::{CsCliError, Result};
use fantoccini::ClientBuilder;
use tokio::process;
use zstd::stream::read::Decoder as ZstdDecoder;

#[derive(Debug, Clone, Copy)]
pub enum Browser {
    Chrome,
    Firefox,
    Safari,
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

pub struct DriverManager {
    cache_dir: PathBuf,
    platform: Platform,
}

impl DriverManager {
    pub fn new() -> Result<Self> {
        let cache_dir = dirs::home_dir()
            .ok_or_else(|| CsCliError::Generic("Could not find home directory".to_string()))?
            .join(".cs-cli")
            .join("drivers");

        fs::create_dir_all(&cache_dir)
            .map_err(|e| CsCliError::Generic(format!("Failed to create cache dir: {}", e)))?;

        Ok(Self {
            cache_dir,
            platform: Platform::current()?,
        })
    }

    pub async fn ensure_driver(&self, browser: Browser) -> Result<PathBuf> {
        let driver_name = match browser {
            Browser::Chrome => "chromedriver",
            Browser::Firefox => "geckodriver",
            Browser::Safari => {
                // SafariDriver is built into macOS
                if cfg!(target_os = "macos") {
                    return Ok(PathBuf::from("/usr/bin/safaridriver"));
                } else {
                    return Err(CsCliError::Generic("Safari is only supported on macOS".to_string()));
                }
            }
        };

        let driver_path = self.cache_dir.join(format!("{}-{}", driver_name, self.platform.as_str()));

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

        // Extract from embedded compressed binary
        self.extract_driver(browser, &driver_path)?;

        Ok(driver_path)
    }

    fn extract_driver(&self, browser: Browser, target_path: &Path) -> Result<()> {
        let compressed_data = self.get_compressed_driver_data(browser)?;

        // Decompress with zstd
        let mut decoder = ZstdDecoder::new(&compressed_data[..])
            .map_err(|e| CsCliError::Generic(format!("Failed to create zstd decoder: {}", e)))?;

        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)
            .map_err(|e| CsCliError::Generic(format!("Failed to decompress driver: {}", e)))?;

        // Write to file
        let mut file = fs::File::create(target_path)
            .map_err(|e| CsCliError::Generic(format!("Failed to create driver file: {}", e)))?;
        file.write_all(&decompressed)
            .map_err(|e| CsCliError::Generic(format!("Failed to write driver file: {}", e)))?;

        // Make executable (Unix only)
        #[cfg(unix)]
        {
            let mut permissions = fs::metadata(target_path)
                .map_err(|e| CsCliError::Generic(format!("Failed to get file metadata: {}", e)))?
                .permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(target_path, permissions)
                .map_err(|e| CsCliError::Generic(format!("Failed to set permissions: {}", e)))?;
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

    pub async fn start_driver(&self, browser: Browser, port: u16) -> Result<process::Child> {
        let driver_path = self.ensure_driver(browser).await?;

        let mut cmd = process::Command::new(&driver_path);

        match browser {
            Browser::Chrome => {
                cmd.arg(format!("--port={}", port));
            }
            Browser::Firefox => {
                cmd.arg("--port").arg(port.to_string());
            }
            Browser::Safari => {
                cmd.arg("--port").arg(port.to_string());
            }
        }

        // Start driver process
        let child = cmd.spawn()
            .map_err(|e| CsCliError::Generic(format!("Failed to start driver: {}", e)))?;

        // Give driver time to start up
        tokio::time::sleep(Duration::from_secs(2)).await;

        Ok(child)
    }

    pub fn detect_active_browser(&self) -> Option<Browser> {
        // Try to detect which browser has an active session
        // This is a simplified version - you might want to check for cookies or running processes

        #[cfg(target_os = "macos")]
        {
            // Check for running browser processes
            if let Ok(output) = Command::new("pgrep")
                .arg("-x")
                .arg("Google Chrome")
                .output()
            {
                if output.status.success() && !output.stdout.is_empty() {
                    return Some(Browser::Chrome);
                }
            }

            if let Ok(output) = Command::new("pgrep")
                .arg("-x")
                .arg("Firefox")
                .output()
            {
                if output.status.success() && !output.stdout.is_empty() {
                    return Some(Browser::Firefox);
                }
            }

            if let Ok(output) = Command::new("pgrep")
                .arg("-x")
                .arg("Safari")
                .output()
            {
                if output.status.success() && !output.stdout.is_empty() {
                    return Some(Browser::Safari);
                }
            }
        }

        // Default to Chrome if no active browser detected
        Some(Browser::Chrome)
    }

    pub async fn connect(&self, browser: Option<Browser>) -> Result<fantoccini::Client> {
        let browser = browser.or_else(|| self.detect_active_browser())
            .unwrap_or(Browser::Chrome);

        let port = 9515; // Default WebDriver port

        // Start the driver
        let _driver_process = self.start_driver(browser, port).await?;

        // Connect with fantoccini
        let client = ClientBuilder::rustls()
            .map_err(|e| CsCliError::Generic(format!("Failed to create client builder: {}", e)))?
            .connect(&format!("http://localhost:{}", port))
            .await
            .map_err(|e| CsCliError::Generic(format!("Failed to connect to driver: {}", e)))?;

        Ok(client)
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
    async fn test_driver_extraction() {
        let manager = DriverManager::new().unwrap();

        // Test Chrome driver extraction
        let chrome_path = manager.ensure_driver(Browser::Chrome).await;
        assert!(chrome_path.is_ok());

        // Test Firefox driver extraction
        let firefox_path = manager.ensure_driver(Browser::Firefox).await;
        assert!(firefox_path.is_ok());
    }
}