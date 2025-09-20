use crate::{CsCliError, Result};
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;
use tracing::debug;

use super::cookie_retrieval_trait::{Cookie, CookieRetrieval};

/// Generic multi-browser cookie retriever using the rookie crate
/// Supports Firefox, Chrome, Safari, Edge, Brave, Arc, Opera, and more browsers
/// Platform-agnostic - can be used by any service (Gong, Slack, Gainsight, etc.)
#[derive(Clone)]
pub struct CookieRetriever {
    /// Target domains to extract cookies from
    domains: Vec<String>,
}

#[async_trait::async_trait]
impl CookieRetrieval for CookieRetriever {
    async fn retrieve_cookies(&self) -> Result<Vec<Cookie>> {
        // Try Firefox first, then Safari
        let firefox_result = self.try_retrieve_firefox_cookies().await;
        if firefox_result.is_ok() && !firefox_result.as_ref().unwrap().is_empty() {
            return firefox_result;
        }
        
        let safari_result = self.try_retrieve_safari_cookies().await;
        if safari_result.is_ok() && !safari_result.as_ref().unwrap().is_empty() {
            return safari_result;
        }
        
        // If both fail, return Firefox error (more descriptive)
        firefox_result
    }
}

impl CookieRetriever {
    /// Create a new cookie retriever for specific domains
    pub fn new(domains: Vec<String>) -> Self {
        Self { domains }
    }

    /// Retrieve cookies from Firefox using ephemeral database copies
    pub fn retrieve_firefox_cookies_ephemeral(&self) -> Result<Vec<Cookie>> {
        self.retrieve_gecko_cookies_ephemeral("Firefox")
    }
    
    /// Try to retrieve Firefox cookies with proper error handling
    async fn try_retrieve_firefox_cookies(&self) -> Result<Vec<Cookie>> {
        let cookie_retriever = self.clone();
        tokio::task::spawn_blocking(move || {
            cookie_retriever.retrieve_firefox_cookies_ephemeral()
        }).await.map_err(|e| CsCliError::Generic(format!("Firefox cookie retrieval task failed: {}", e)))?
    }
    
    /// Try to retrieve Safari cookies with proper error handling
    async fn try_retrieve_safari_cookies(&self) -> Result<Vec<Cookie>> {
        let cookie_retriever = self.clone();
        tokio::task::spawn_blocking(move || {
            cookie_retriever.retrieve_safari_cookies_ephemeral()
        }).await.map_err(|e| CsCliError::Generic(format!("Safari cookie retrieval task failed: {}", e)))?
    }


    /// Find all Gecko browser profile directories on macOS (Firefox, LibreWolf, Zen)
    fn find_gecko_profiles(&self, browser_name: &str) -> Vec<PathBuf> {
        let mut profiles = Vec::new();

        if let Some(home) = std::env::var_os("HOME") {
            let profiles_dir = PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join(browser_name)
                .join("Profiles");

            if profiles_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&profiles_dir) {
                    for entry in entries.flatten() {
                        if entry.path().is_dir() {
                            let profile_name = entry.file_name().to_string_lossy().to_string();
                            // Skip hidden directories and profiles that look invalid
                            // Gecko profiles typically have format: random.profile-name
                            if !profile_name.starts_with('.') && profile_name.contains('.') {
                                profiles.push(entry.path());
                            }
                        }
                    }
                }
            }
        }

        profiles
    }


    /// Filter cookies by domain validity (delegates to trait implementation)
    fn filter_valid_cookies(&self, cookies: Vec<Cookie>) -> Vec<Cookie> {
        self.filter_by_domains(cookies, &self.domains)
    }

    /// Get the target domains this extractor is configured for
    pub fn get_domains(&self) -> &[String] {
        &self.domains
    }

    /// Read Gecko browser cookies from a SQLite database with optimized query
    fn read_gecko_cookies_from_db(&self, db_path: &Path) -> Result<Vec<Cookie>> {
        if !db_path.exists() {
            return Err(CsCliError::CookieRetrieval(format!(
                "Gecko browser cookies database not found at: {}",
                db_path.display()
            )));
        }

        debug!("Reading Gecko browser cookies from: {}", db_path.display());

        let conn = Connection::open(db_path).map_err(|e| {
            CsCliError::CookieRetrieval(format!(
                "Failed to open Gecko browser cookies database at {}: {}",
                db_path.display(),
                e
            ))
        })?;

        // Optimize: Build a single query with multiple LIKE conditions instead of looping
        if self.domains.is_empty() {
            return Ok(Vec::new());
        }

        // Build WHERE clause with multiple domain conditions
        let like_conditions: Vec<String> = self.domains
            .iter()
            .map(|_| "host LIKE ?".to_string())
            .collect();
        let where_clause = like_conditions.join(" OR ");

        let query = format!(
            "SELECT name, value, host, path, isSecure, isHttpOnly, expiry FROM moz_cookies WHERE {}",
            where_clause
        );

        debug!("Executing optimized query with {} domain conditions", self.domains.len());

        let mut stmt = conn.prepare(&query).map_err(|e| {
            CsCliError::CookieRetrieval(format!(
                "Failed to prepare Gecko browser query for {}: {}",
                db_path.display(),
                e
            ))
        })?;

        // Bind all domain patterns as parameters
        let domain_patterns: Vec<String> = self.domains
            .iter()
            .map(|domain| format!("%{}", domain))
            .collect();

        let cookie_rows = stmt
            .query_map(rusqlite::params_from_iter(domain_patterns.iter()), |row| {
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
                CsCliError::CookieRetrieval(format!(
                    "Failed to query Gecko browser cookies from {}: {}",
                    db_path.display(),
                    e
                ))
            })?;

        let mut cookies = Vec::new();
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

        debug!("Found {} Gecko browser cookies matching domains from {}", cookies.len(), db_path.display());
        Ok(cookies)
    }

    /// Read cookies directly from a specific Firefox profile's SQLite database
    #[allow(dead_code)]
    fn read_firefox_profile_cookies(&self, profile_path: &Path) -> Result<Vec<Cookie>> {
        let cookies_db_path = profile_path.join("cookies.sqlite");
        self.read_gecko_cookies_from_db(&cookies_db_path)
    }

    /// Read Gecko browser cookies directly from a database file
    fn read_gecko_cookies_from_file(&self, db_path: &Path) -> Result<Vec<Cookie>> {
        self.read_gecko_cookies_from_db(db_path)
    }

    /// Retrieve cookies from Safari using ephemeral binary cookie copies
    pub fn retrieve_safari_cookies_ephemeral(&self) -> Result<Vec<Cookie>> {
        tracing::info!("Starting Safari cookie retrieval using ephemeral binary cookie copies...");
        tracing::debug!("Target domains: {:?}", self.domains);

        let safari_cookies_path = self.find_safari_cookies_path();

        if !safari_cookies_path.exists() {
            return Err(CsCliError::CookieRetrieval(
                "Safari cookies file not found".to_string(),
            ));
        }

        tracing::info!("Found Safari cookies at: {}", safari_cookies_path.display());

        // Create ephemeral copy of cookies file to avoid lock issues
        tracing::debug!("Creating secure temporary copy of Safari binary cookies...");
        match NamedTempFile::with_prefix("cs-cli-safari-cookies-") {
            Ok(temp_file) => {
                let temp_path = temp_file.path();
                tracing::debug!("Created secure temporary file at {}", temp_path.display());

                // Copy binary cookies to temporary location
                match std::fs::copy(&safari_cookies_path, temp_path) {
                    Ok(_) => {
                        tracing::info!("Created ephemeral copy at {}", temp_path.display());

                        // Parse cookies from temporary copy
                        tracing::debug!("Parsing cookies from temporary binary file...");
                        match self.read_safari_cookies_from_file(temp_path) {
                            Ok(cookies) => {
                                tracing::debug!("Raw cookies retrieved: {}", cookies.len());
                                let filtered = self.filter_valid_cookies(cookies);
                                if !filtered.is_empty() {
                                    tracing::info!(
                                        "Found {} valid Safari cookies",
                                        filtered.len()
                                    );
                                    Ok(filtered)
                                } else {
                                    tracing::debug!("No valid Safari cookies found for target domains");
                                    Err(CsCliError::CookieRetrieval(
                                        "No authentication methods available.".to_string(),
                                    ))
                                }
                            }
                            Err(e) => {
                                tracing::warn!("Failed to read Safari cookies: {}", e);
                                Err(e)
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Failed to copy Safari cookies file: {}",
                            e
                        );
                        Err(CsCliError::CookieRetrieval(format!(
                            "Failed to copy Safari cookies: {}",
                            e
                        )))
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to create temporary file for Safari cookies: {}", e);
                Err(CsCliError::CookieRetrieval(format!(
                    "Failed to create temporary file: {}",
                    e
                )))
            }
        }
    }

    /// Find Safari's binary cookies file on macOS
    fn find_safari_cookies_path(&self) -> PathBuf {
        if let Some(home) = std::env::var_os("HOME") {
            PathBuf::from(home)
                .join("Library")
                .join("Containers")
                .join("com.apple.Safari")
                .join("Data")
                .join("Library")
                .join("Cookies")
                .join("Cookies.binarycookies")
        } else {
            PathBuf::from("/tmp/nonexistent")
        }
    }

    /// Parse Safari's binary cookie format using strings extraction
    fn read_safari_cookies_from_file(&self, cookies_path: &Path) -> Result<Vec<Cookie>> {
        if !cookies_path.exists() {
            return Err(CsCliError::CookieRetrieval(format!(
                "Safari cookies file not found at: {}",
                cookies_path.display()
            )));
        }

        debug!("Parsing Safari binary cookies from: {}", cookies_path.display());

        // Use strings command to extract readable text from binary file
        use std::process::Command;
        let output = Command::new("strings")
            .arg(cookies_path)
            .output()
            .map_err(|e| {
                CsCliError::CookieRetrieval(format!(
                    "Failed to execute strings command on Safari cookies: {}",
                    e
                ))
            })?;

        if !output.status.success() {
            return Err(CsCliError::CookieRetrieval(
                "Failed to extract strings from Safari cookies file".to_string(),
            ));
        }

        let strings_output = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = strings_output.lines().collect();

        debug!("Extracted {} text lines from Safari binary cookies", lines.len());

        // Parse the strings output to extract cookie information
        self.parse_safari_cookie_strings(&lines)
    }

    /// Parse cookie information from Safari binary cookies strings output
    fn parse_safari_cookie_strings(&self, lines: &[&str]) -> Result<Vec<Cookie>> {
        let mut cookies = Vec::new();
        let mut current_domain = None;
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();

            // Look for domain patterns (lines starting with 'A.')
            if line.starts_with("A.") && line.contains('.') {
                current_domain = Some(line.strip_prefix("A.").unwrap_or(line));
                i += 1;
                continue;
            }

            // Skip if we don't have a domain context
            let domain = match current_domain {
                Some(d) => d,
                None => {
                    i += 1;
                    continue;
                }
            };

            // Check if this domain matches our target domains
            let domain_matches = self.domains.iter().any(|target_domain| {
                domain.ends_with(target_domain)
                    || domain == format!(".{target_domain}")
                    || domain == target_domain.trim_start_matches('.')
            });

            if !domain_matches {
                i += 1;
                continue;
            }

            // Look for cookie name and value pairs
            // Cookie names are typically followed by their values
            if i + 1 < lines.len() && !line.is_empty() && !line.starts_with("A.") && !line.starts_with("bplist") {
                let cookie_name = line;
                let cookie_value = lines[i + 1].trim();

                // Skip obviously non-cookie data
                if cookie_name.len() > 100 || cookie_value.len() > 5000 {
                    i += 1;
                    continue;
                }

                // Skip binary data markers
                if cookie_name.starts_with("bplist") || cookie_value.starts_with("bplist") ||
                   cookie_name == "ZAccessTime#A" || cookie_value == "ZAccessTime#A" {
                    i += 1;
                    continue;
                }

                debug!("Found potential cookie: {} = {} (domain: {})", cookie_name, cookie_value, domain);

                cookies.push(Cookie {
                    name: cookie_name.to_string(),
                    value: cookie_value.to_string(),
                    domain: domain.to_string(),
                    path: "/".to_string(),
                    secure: true, // Assume secure for Gong cookies
                    http_only: false,
                    expires: None, // Session cookie assumption
                });

                i += 2; // Skip both name and value lines
            } else {
                i += 1;
            }
        }

        debug!("Parsed {} cookies from Safari binary format for target domains", cookies.len());
        Ok(cookies)
    }

    /// Retrieve cookies from any Gecko browser using ephemeral database copies
    pub fn retrieve_gecko_cookies_ephemeral(&self, browser_name: &str) -> Result<Vec<Cookie>> {
        tracing::info!("Starting {} cookie retrieval using ephemeral database copies...", browser_name);
        tracing::debug!("Target domains: {:?}", self.domains);

        let gecko_profiles = self.find_gecko_profiles(browser_name);
        tracing::info!("Found {} {} profiles to check", gecko_profiles.len(), browser_name);

        if gecko_profiles.is_empty() {
            return Err(CsCliError::CookieRetrieval(
                format!("No {} profiles found", browser_name),
            ));
        }

        let mut all_cookies = Vec::new();

        for profile_path in gecko_profiles {
            tracing::info!("Checking {} profile: {}", browser_name, profile_path.display());

            // Create ephemeral copy of cookies database to avoid lock issues
            let cookies_db_path = profile_path.join("cookies.sqlite");

            if !cookies_db_path.exists() {
                tracing::debug!("No cookies.sqlite in profile {}", profile_path.display());
                continue;
            }

            // Create secure temporary file with proper cleanup
            tracing::debug!("Creating secure temporary copy of cookies database...");
            match NamedTempFile::with_prefix(&format!("cs-cli-{}-cookies-", browser_name.to_lowercase())) {
                Ok(temp_file) => {
                    let temp_path = temp_file.path();
                    tracing::debug!("Created secure temporary file at {}", temp_path.display());

                    // Copy database to temporary location
                    match std::fs::copy(&cookies_db_path, temp_path) {
                        Ok(_) => {
                            tracing::info!("Created ephemeral copy at {}", temp_path.display());

                            // Read directly from temporary copy
                            tracing::debug!("Reading cookies from temporary database...");
                            match self.read_gecko_cookies_from_file(temp_path) {
                                Ok(cookies) => {
                                    tracing::debug!("Raw cookies retrieved: {}", cookies.len());
                                    let filtered = self.filter_valid_cookies(cookies);
                                    if !filtered.is_empty() {
                                        tracing::info!(
                                            "Found {} valid cookies in {} profile {}",
                                            filtered.len(),
                                            browser_name,
                                            profile_path.display()
                                        );
                                        all_cookies.extend(filtered);
                                    } else {
                                        tracing::debug!("No valid cookies found in {} profile {}", browser_name, profile_path.display());
                                    }
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        "Failed to read cookies from {} profile {}: {}",
                                        browser_name,
                                        profile_path.display(),
                                        e
                                    );
                                }
                            }
                            // temp_file is automatically cleaned up when it goes out of scope
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Failed to copy cookies.sqlite from {} profile {}: {}",
                                browser_name,
                                profile_path.display(),
                                e
                            );
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to create temporary file for {} profile {}: {}",
                        browser_name,
                        profile_path.display(),
                        e
                    );
                }
            }
        }

        if all_cookies.is_empty() {
            Err(CsCliError::CookieRetrieval(
                "No authentication methods available.".to_string(),
            ))
        } else {
            debug!(
                "Found {} total cookies across all {} profiles",
                all_cookies.len(),
                browser_name
            );
            Ok(all_cookies)
        }
    }

}
