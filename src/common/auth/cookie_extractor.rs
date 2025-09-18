use crate::{CsCliError, Result};
use rookie;
use rusqlite::Connection;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

/// Generic multi-browser cookie extractor using the rookie crate
/// Supports Firefox, Chrome, Safari, Edge, Brave, Arc, Opera, and more browsers
/// Platform-agnostic - can be used by any service (Gong, Slack, Gainsight, etc.)
pub struct CookieExtractor {
    /// Target domains to extract cookies from
    domains: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub secure: bool,
    pub http_only: bool,
    pub expires: Option<u64>,
}

/// Macro to generate browser cookie extraction methods with less boilerplate
macro_rules! browser_extractor {
    ($method_name:ident, $browser_fn:ident, $browser_name:expr) => {
        pub fn $method_name(&self) -> Result<Vec<Cookie>> {
            match rookie::$browser_fn(Some(self.domains.clone())) {
                Ok(cookies) => Ok(cookies
                    .into_iter()
                    .map(|c| Cookie {
                        name: c.name,
                        value: c.value,
                        domain: c.domain,
                        path: c.path,
                        secure: c.secure,
                        http_only: c.http_only,
                        expires: c.expires,
                    })
                    .collect()),
                Err(e) => Err(CsCliError::CookieExtraction(format!(
                    "Failed to extract {} cookies: {e}",
                    $browser_name
                ))),
            }
        }
    };
}

impl CookieExtractor {
    pub fn new(domains: Vec<String>) -> Self {
        Self { domains }
    }

    /// Extract cookies from Firefox, checking ALL profiles
    pub fn extract_firefox_cookies(&self) -> Result<Vec<Cookie>> {
        self.extract_firefox_all_profiles()
    }

    /// Extract cookies from Chrome, checking ALL profiles  
    pub fn extract_chrome_cookies(&self) -> Result<Vec<Cookie>> {
        self.extract_chrome_all_profiles()
    }

    // Generate browser extraction methods using macro for single-profile browsers
    browser_extractor!(extract_edge_cookies, edge, "Edge");
    browser_extractor!(extract_arc_cookies, arc, "Arc");
    browser_extractor!(extract_brave_cookies, brave, "Brave");
    browser_extractor!(extract_chromium_cookies, chromium, "Chromium");
    browser_extractor!(extract_librewolf_cookies, librewolf, "LibreWolf");
    browser_extractor!(extract_opera_cookies, opera, "Opera");
    browser_extractor!(extract_opera_gx_cookies, opera_gx, "Opera GX");
    browser_extractor!(extract_vivaldi_cookies, vivaldi, "Vivaldi");
    browser_extractor!(extract_zen_cookies, zen, "Zen");

    #[cfg(target_os = "macos")]
    browser_extractor!(extract_safari_cookies, safari, "Safari");

    /// Extract Firefox cookies from ALL profiles (not just default)
    fn extract_firefox_all_profiles(&self) -> Result<Vec<Cookie>> {
        debug!("Extracting Firefox cookies from all profiles...");

        // Try the standard rookie call first (default profile)
        match rookie::firefox(Some(self.domains.clone())) {
            Ok(cookies) => {
                let filtered = self.filter_valid_cookies(
                    cookies
                        .into_iter()
                        .map(|c| Cookie {
                            name: c.name,
                            value: c.value,
                            domain: c.domain,
                            path: c.path,
                            secure: c.secure,
                            http_only: c.http_only,
                            expires: c.expires,
                        })
                        .collect(),
                );

                if !filtered.is_empty() {
                    debug!(
                        "Found {} cookies in Firefox default profile",
                        filtered.len()
                    );
                    return Ok(filtered);
                }
            }
            Err(e) => debug!("Firefox default profile extraction failed: {}", e),
        }

        // If default profile has no cookies, scan all Firefox profiles
        debug!("Default Firefox profile empty, scanning all profiles...");

        let firefox_profiles = self.find_firefox_profiles();
        debug!("Found {} Firefox profiles to check", firefox_profiles.len());

        for profile_path in firefox_profiles {
            debug!("Checking Firefox profile: {}", profile_path.display());

            // Read cookies directly from this profile's database
            match self.read_firefox_profile_cookies(&profile_path) {
                Ok(cookies) => {
                    let filtered = self.filter_valid_cookies(cookies);
                    if !filtered.is_empty() {
                        debug!(
                            "Found {} cookies in Firefox profile: {}",
                            filtered.len(),
                            profile_path.display()
                        );
                        return Ok(filtered);
                    }
                }
                Err(e) => debug!(
                    "Failed to read Firefox profile {}: {}",
                    profile_path.display(),
                    e
                ),
            }
        }

        Err(CsCliError::CookieExtraction(
            "No Firefox cookies found in any profile".to_string(),
        ))
    }

    /// Extract Chrome cookies from ALL profiles (not just default)
    fn extract_chrome_all_profiles(&self) -> Result<Vec<Cookie>> {
        debug!("Extracting Chrome cookies from all profiles...");

        // Try the standard rookie call first (default profile)
        match rookie::chrome(Some(self.domains.clone())) {
            Ok(cookies) => {
                let filtered = self.filter_valid_cookies(
                    cookies
                        .into_iter()
                        .map(|c| Cookie {
                            name: c.name,
                            value: c.value,
                            domain: c.domain,
                            path: c.path,
                            secure: c.secure,
                            http_only: c.http_only,
                            expires: c.expires,
                        })
                        .collect(),
                );

                if !filtered.is_empty() {
                    debug!("Found {} cookies in Chrome default profile", filtered.len());
                    return Ok(filtered);
                }
            }
            Err(e) => debug!("Chrome default profile extraction failed: {}", e),
        }

        // If default profile has no cookies, scan all Chrome profiles
        debug!("Default Chrome profile empty, scanning all profiles...");

        let chrome_profiles = self.find_chrome_profiles();
        debug!("Found {} Chrome profiles to check", chrome_profiles.len());

        for profile_path in chrome_profiles {
            debug!("Checking Chrome profile: {}", profile_path.display());

            // Read cookies directly from this profile's database
            match self.read_chrome_profile_cookies(&profile_path) {
                Ok(cookies) => {
                    let filtered = self.filter_valid_cookies(cookies);
                    if !filtered.is_empty() {
                        debug!(
                            "Found {} cookies in Chrome profile: {}",
                            filtered.len(),
                            profile_path.display()
                        );
                        return Ok(filtered);
                    }
                }
                Err(e) => debug!(
                    "Failed to read Chrome profile {}: {}",
                    profile_path.display(),
                    e
                ),
            }
        }

        Err(CsCliError::CookieExtraction(
            "No Chrome cookies found in any profile".to_string(),
        ))
    }

    /// Find all Firefox profile directories on macOS
    fn find_firefox_profiles(&self) -> Vec<PathBuf> {
        let mut profiles = Vec::new();

        if let Some(home) = std::env::var_os("HOME") {
            let firefox_profiles_dir = PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("Firefox")
                .join("Profiles");

            if firefox_profiles_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&firefox_profiles_dir) {
                    for entry in entries.flatten() {
                        if entry.path().is_dir() {
                            let profile_name = entry.file_name().to_string_lossy().to_string();
                            // Skip hidden directories and profiles that look invalid
                            if !profile_name.starts_with('.') && profile_name.contains(".") {
                                profiles.push(entry.path());
                            }
                        }
                    }
                }
            }
        }

        profiles
    }

    /// Find all Chrome profile directories on macOS
    fn find_chrome_profiles(&self) -> Vec<PathBuf> {
        let mut profiles = Vec::new();

        if let Some(home) = std::env::var_os("HOME") {
            let chrome_dir = PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("Google")
                .join("Chrome");

            if chrome_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&chrome_dir) {
                    for entry in entries.flatten() {
                        if entry.path().is_dir() {
                            let profile_name = entry.file_name().to_string_lossy().to_string();
                            // Look for Default, Profile 1, Profile 2, etc., but not system directories
                            if profile_name == "Default" || profile_name.starts_with("Profile ") {
                                let cookies_file = entry.path().join("Cookies");
                                // Only include profiles that have a Cookies file
                                if cookies_file.exists() {
                                    profiles.push(entry.path());
                                }
                            }
                        }
                    }
                }
            }
        }

        profiles
    }

    /// Filter cookies to remove expired ones
    pub fn filter_valid_cookies(&self, cookies: Vec<Cookie>) -> Vec<Cookie> {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        cookies
            .into_iter()
            .filter(|cookie| {
                // If expires is None, cookie is a session cookie (valid)
                // If expires is Some(timestamp), check if it's in the future
                match cookie.expires {
                    None => true, // Session cookies are always valid
                    Some(expires_timestamp) => expires_timestamp > current_time,
                }
            })
            .collect()
    }

    /// Extract cookies from all browsers and return them grouped by browser
    /// Returns a list of (cookies, browser_name) tuples for platform-specific validation
    pub fn extract_all_browsers_cookies(&self) -> Vec<(Vec<Cookie>, String)> {
        debug!("Extracting cookies for domains: {:?}", self.domains);
        let mut all_browser_cookies = Vec::new();

        // Firefox-based browsers (no keychain access needed)
        let firefox_browsers = vec![
            ("Firefox", self.extract_firefox_cookies()),
            ("LibreWolf", self.extract_librewolf_cookies()),
        ];

        for (browser_name, result) in firefox_browsers {
            if let Ok(cookies) = result {
                let valid_cookies = self.filter_valid_cookies(cookies);
                if !valid_cookies.is_empty() {
                    debug!("{} has {} valid cookies", browser_name, valid_cookies.len());
                    all_browser_cookies.push((valid_cookies, browser_name.to_string()));
                }
            }
        }

        // Chromium-based browsers (may require keychain access)
        let chromium_browsers = vec![
            ("Chrome", self.extract_chrome_cookies()),
            ("Brave", self.extract_brave_cookies()),
            ("Edge", self.extract_edge_cookies()),
            ("Arc", self.extract_arc_cookies()),
            ("Chromium", self.extract_chromium_cookies()),
            ("Opera", self.extract_opera_cookies()),
            ("Opera GX", self.extract_opera_gx_cookies()),
            ("Vivaldi", self.extract_vivaldi_cookies()),
            ("Zen", self.extract_zen_cookies()),
        ];

        for (browser_name, result) in chromium_browsers {
            if let Ok(cookies) = result {
                let valid_cookies = self.filter_valid_cookies(cookies);
                if !valid_cookies.is_empty() {
                    debug!("{} has {} valid cookies", browser_name, valid_cookies.len());
                    all_browser_cookies.push((valid_cookies, browser_name.to_string()));
                }
            }
        }

        // Safari on macOS
        #[cfg(target_os = "macos")]
        if let Ok(cookies) = self.extract_safari_cookies() {
            let valid_cookies = self.filter_valid_cookies(cookies);
            if !valid_cookies.is_empty() {
                debug!("Safari has {} valid cookies", valid_cookies.len());
                all_browser_cookies.push((valid_cookies, "Safari".to_string()));
            }
        }

        all_browser_cookies
    }

    /// Extract cookies from first available browser with valid cookies
    /// Returns the first successful extraction for simple use cases
    pub fn extract_first_available(&self) -> Result<(Vec<Cookie>, String)> {
        let all_browsers = self.extract_all_browsers_cookies();

        if let Some((cookies, browser_name)) = all_browsers.first() {
            Ok((cookies.clone(), browser_name.clone()))
        } else {
            Err(CsCliError::CookieExtraction(format!(
                "No cookies found for domains {:?} in any supported browser",
                self.domains
            )))
        }
    }

    /// Convert cookies to HTTP Cookie header format
    pub fn cookies_to_header(&self, cookies: &[Cookie]) -> String {
        cookies
            .iter()
            .map(|c| format!("{}={}", c.name, c.value))
            .collect::<Vec<_>>()
            .join("; ")
    }

    /// Convert cookies to HTTP headers HashMap
    pub fn cookies_to_headers(&self, cookies: &[Cookie]) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        let cookie_header = self.cookies_to_header(cookies);

        if !cookie_header.is_empty() {
            headers.insert("Cookie".to_string(), cookie_header);
        }

        headers
    }

    /// Get the target domains this extractor is configured for
    pub fn get_domains(&self) -> &[String] {
        &self.domains
    }

    /// Read cookies directly from a specific Chrome profile's SQLite database
    fn read_chrome_profile_cookies(&self, profile_path: &Path) -> Result<Vec<Cookie>> {
        let cookies_db_path = profile_path.join("Cookies");

        if !cookies_db_path.exists() {
            return Err(CsCliError::CookieExtraction(format!(
                "Cookies database not found at: {}",
                cookies_db_path.display()
            )));
        }

        debug!("Reading Chrome cookies from: {}", cookies_db_path.display());

        // Chrome encrypts cookies, so we can only check if cookies exist for our domains
        // The actual values would need Chrome's decryption APIs to be readable
        let conn = Connection::open(&cookies_db_path).map_err(|e| {
            CsCliError::CookieExtraction(format!("Failed to open Chrome cookies database: {e}"))
        })?;

        let mut cookies = Vec::new();

        // Query for cookies from our target domains
        let query = "SELECT name, host_key, path, is_secure, is_httponly, expires_utc FROM cookies WHERE host_key LIKE ? OR host_key LIKE ?";

        for domain in &self.domains {
            let domain_pattern = format!("%{domain}");

            let mut stmt = conn.prepare(query).map_err(|e| {
                CsCliError::CookieExtraction(format!("Failed to prepare query: {e}"))
            })?;

            let cookie_rows = stmt
                .query_map([&domain_pattern, &domain_pattern], |row| {
                    Ok((
                        row.get::<_, String>(0)?,   // name
                        row.get::<_, String>(1)?,   // host_key (domain)
                        row.get::<_, String>(2)?,   // path
                        row.get::<_, i64>(3)? != 0, // is_secure
                        row.get::<_, i64>(4)? != 0, // is_httponly
                        row.get::<_, i64>(5)?,      // expires_utc
                    ))
                })
                .map_err(|e| {
                    CsCliError::CookieExtraction(format!("Failed to query cookies: {e}"))
                })?;

            for (name, host_key, path, is_secure, is_httponly, expires_utc) in cookie_rows.flatten()
            {
                // Convert Chrome's WebKit timestamp to Unix timestamp
                let expires = if expires_utc == 0 {
                    None // Session cookie
                } else {
                    // Chrome uses WebKit timestamp (microseconds since Jan 1, 1601)
                    // Convert to Unix timestamp (seconds since Jan 1, 1970)
                    let webkit_epoch_diff = 11644473600; // Seconds between 1601 and 1970
                    let unix_timestamp = (expires_utc as u64 / 1_000_000) - webkit_epoch_diff;
                    Some(unix_timestamp)
                };

                cookies.push(Cookie {
                    name,
                    value: "[ENCRYPTED]".to_string(), // Chrome encrypts cookie values
                    domain: host_key,
                    path,
                    secure: is_secure,
                    http_only: is_httponly,
                    expires,
                });
            }
        }

        if cookies.is_empty() {
            debug!("No matching cookies found in Chrome profile database");
        } else {
            debug!(
                "Found {} matching cookie entries in Chrome profile",
                cookies.len()
            );
            warn!("Chrome cookies are encrypted - using rookie for decryption when possible");
        }

        // Chrome cookies are encrypted, so this mainly serves as detection
        // We still need rookie or the keychain to decrypt them properly
        Ok(cookies)
    }

    /// Read cookies directly from a specific Firefox profile's SQLite database  
    fn read_firefox_profile_cookies(&self, profile_path: &Path) -> Result<Vec<Cookie>> {
        let cookies_db_path = profile_path.join("cookies.sqlite");

        if !cookies_db_path.exists() {
            return Err(CsCliError::CookieExtraction(format!(
                "Firefox cookies database not found at: {}",
                cookies_db_path.display()
            )));
        }

        debug!(
            "Reading Firefox cookies from: {}",
            cookies_db_path.display()
        );

        let conn = Connection::open(&cookies_db_path).map_err(|e| {
            CsCliError::CookieExtraction(format!("Failed to open Firefox cookies database: {e}"))
        })?;

        let mut cookies = Vec::new();

        // Query for cookies from our target domains
        let query = "SELECT name, value, host, path, isSecure, isHttpOnly, expiry FROM moz_cookies WHERE host LIKE ? OR host LIKE ?";

        for domain in &self.domains {
            let domain_pattern = format!("%{domain}");

            let mut stmt = conn.prepare(query).map_err(|e| {
                CsCliError::CookieExtraction(format!("Failed to prepare Firefox query: {e}"))
            })?;

            let cookie_rows = stmt
                .query_map([&domain_pattern, &domain_pattern], |row| {
                    Ok((
                        row.get::<_, String>(0)?,   // name
                        row.get::<_, String>(1)?,   // value
                        row.get::<_, String>(2)?,   // host (domain)
                        row.get::<_, String>(3)?,   // path
                        row.get::<_, i64>(4)? != 0, // isSecure
                        row.get::<_, i64>(5)? != 0, // isHttpOnly
                        row.get::<_, i64>(6)?,      // expiry
                    ))
                })
                .map_err(|e| {
                    CsCliError::CookieExtraction(format!("Failed to query Firefox cookies: {e}"))
                })?;

            for (name, value, host, path, is_secure, is_httponly, expiry) in cookie_rows.flatten() {
                let expires = if expiry == 0 {
                    None // Session cookie
                } else {
                    Some(expiry as u64)
                };

                cookies.push(Cookie {
                    name,
                    value,
                    domain: host,
                    path,
                    secure: is_secure,
                    http_only: is_httponly,
                    expires,
                });
            }
        }

        debug!(
            "Found {} matching cookies in Firefox profile",
            cookies.len()
        );
        Ok(cookies)
    }
}
