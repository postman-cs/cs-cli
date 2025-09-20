use crate::{CsCliError, Result};
use rusqlite::Connection;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::debug;

/// Generic multi-browser cookie extractor using the rookie crate
/// Supports Firefox, Chrome, Safari, Edge, Brave, Arc, Opera, and more browsers
/// Platform-agnostic - can be used by any service (Gong, Slack, Gainsight, etc.)
pub struct CookieExtractor {
    /// Target domains to extract cookies from
    domains: Vec<String>,
}

/// Browser-agnostic cookie structure
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

impl CookieExtractor {
    /// Create a new cookie extractor for specific domains
    pub fn new(domains: Vec<String>) -> Self {
        Self { domains }
    }

    /// Extract cookies from Firefox using ephemeral database copies
    pub fn extract_firefox_cookies_ephemeral(&self) -> Result<Vec<Cookie>> {
        debug!("Extracting Firefox cookies using ephemeral database copies...");

        let firefox_profiles = self.find_firefox_profiles();
        debug!("Found {} Firefox profiles to check", firefox_profiles.len());

        if firefox_profiles.is_empty() {
            return Err(CsCliError::CookieExtraction(
                "No Firefox profiles found".to_string(),
            ));
        }

        let mut all_cookies = Vec::new();

        for profile_path in firefox_profiles {
            debug!("Checking Firefox profile: {}", profile_path.display());

            // Create ephemeral copy of cookies database to avoid lock issues
            let cookies_db_path = profile_path.join("cookies.sqlite");

            if !cookies_db_path.exists() {
                debug!("No cookies.sqlite in profile {}", profile_path.display());
                continue;
            }

            // Create temporary copy with unique name
            let temp_dir = std::env::temp_dir();
            let temp_db_name = format!(
                "cs-cli-firefox-cookies-{}-{}.sqlite",
                std::process::id(),
                profile_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
            );
            let temp_path = temp_dir.join(&temp_db_name);

            // Copy database to temporary location
            match std::fs::copy(&cookies_db_path, &temp_path) {
                Ok(_) => {
                    debug!("Created ephemeral copy at {}", temp_path.display());

                    // Read directly from temporary copy
                    match self.read_firefox_cookies_from_file(&temp_path) {
                        Ok(cookies) => {
                            let filtered = self.filter_valid_cookies(cookies);
                            if !filtered.is_empty() {
                                debug!(
                                    "Found {} valid cookies in profile {}",
                                    filtered.len(),
                                    profile_path.display()
                                );
                                all_cookies.extend(filtered);
                            }
                        }
                        Err(e) => {
                            debug!(
                                "Failed to read cookies from {}: {}",
                                profile_path.display(),
                                e
                            );
                        }
                    }

                    // Clean up temporary file
                    let _ = std::fs::remove_file(&temp_path);
                }
                Err(e) => {
                    debug!(
                        "Failed to copy cookies.sqlite from {}: {}",
                        profile_path.display(),
                        e
                    );
                }
            }
        }

        if all_cookies.is_empty() {
            Err(CsCliError::CookieExtraction(
                "No authentican methods available.".to_string(),
            ))
        } else {
            debug!(
                "Found {} total cookies across all Firefox profiles",
                all_cookies.len()
            );
            Ok(all_cookies)
        }
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

    /// Filter cookies by domain validity
    fn filter_valid_cookies(&self, cookies: Vec<Cookie>) -> Vec<Cookie> {
        cookies
            .into_iter()
            .filter(|cookie| {
                // Check if cookie domain matches any of our target domains
                self.domains.iter().any(|domain| {
                    cookie.domain.ends_with(domain)
                        || cookie.domain == format!(".{domain}")
                        || cookie.domain == domain.trim_start_matches('.')
                })
            })
            .collect()
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

    /// Read cookies directly from a specific Firefox profile's SQLite database
    #[allow(dead_code)]
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

    /// Read Firefox cookies directly from a database file
    fn read_firefox_cookies_from_file(&self, db_path: &Path) -> Result<Vec<Cookie>> {
        if !db_path.exists() {
            return Err(CsCliError::CookieExtraction(format!(
                "Firefox cookies database not found at: {}",
                db_path.display()
            )));
        }

        debug!("Reading Firefox cookies from: {}", db_path.display());

        let conn = Connection::open(db_path).map_err(|e| {
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

        debug!("Found {} Firefox cookies matching domains", cookies.len());
        Ok(cookies)
    }
}
