use crate::{CsCliError, Result};
use rusqlite::Connection;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;
use tracing::debug;

/// Generic multi-browser cookie retriever using the rookie crate
/// Supports Firefox, Chrome, Safari, Edge, Brave, Arc, Opera, and more browsers
/// Platform-agnostic - can be used by any service (Gong, Slack, Gainsight, etc.)
#[derive(Clone)]
pub struct CookieRetriever {
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

impl CookieRetriever {
    /// Create a new cookie retriever for specific domains
    pub fn new(domains: Vec<String>) -> Self {
        Self { domains }
    }

    /// Retrieve cookies from Firefox using ephemeral database copies
    pub fn retrieve_firefox_cookies_ephemeral(&self) -> Result<Vec<Cookie>> {
        tracing::info!("Starting Firefox cookie retrieval using ephemeral database copies...");
        tracing::debug!("Target domains: {:?}", self.domains);

        let firefox_profiles = self.find_firefox_profiles();
        tracing::info!("Found {} Firefox profiles to check", firefox_profiles.len());

        if firefox_profiles.is_empty() {
            return Err(CsCliError::CookieRetrieval(
                "No Firefox profiles found".to_string(),
            ));
        }

        let mut all_cookies = Vec::new();

        for profile_path in firefox_profiles {
            tracing::info!("Checking Firefox profile: {}", profile_path.display());

            // Create ephemeral copy of cookies database to avoid lock issues
            let cookies_db_path = profile_path.join("cookies.sqlite");

            if !cookies_db_path.exists() {
                tracing::debug!("No cookies.sqlite in profile {}", profile_path.display());
                continue;
            }

            // Create secure temporary file with proper cleanup
            tracing::debug!("Creating secure temporary copy of cookies database...");
            match NamedTempFile::with_prefix("cs-cli-firefox-cookies-") {
                Ok(temp_file) => {
                    let temp_path = temp_file.path();
                    tracing::debug!("Created secure temporary file at {}", temp_path.display());

                    // Copy database to temporary location
                    match std::fs::copy(&cookies_db_path, temp_path) {
                        Ok(_) => {
                            tracing::info!("Created ephemeral copy at {}", temp_path.display());

                            // Read directly from temporary copy
                            tracing::debug!("Reading cookies from temporary database...");
                            match self.read_firefox_cookies_from_file(temp_path) {
                                Ok(cookies) => {
                                    tracing::debug!("Raw cookies retrieved: {}", cookies.len());
                                    let filtered = self.filter_valid_cookies(cookies);
                                    if !filtered.is_empty() {
                                        tracing::info!(
                                            "Found {} valid cookies in profile {}",
                                            filtered.len(),
                                            profile_path.display()
                                        );
                                        all_cookies.extend(filtered);
                                    } else {
                                        tracing::debug!("No valid cookies found in profile {}", profile_path.display());
                                    }
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        "Failed to read cookies from profile {}: {}",
                                        profile_path.display(),
                                        e
                                    );
                                }
                            }
                            // temp_file is automatically cleaned up when it goes out of scope
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Failed to copy cookies.sqlite from profile {}: {}",
                                profile_path.display(),
                                e
                            );
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to create temporary file for profile {}: {}",
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
                            // Firefox profiles typically have format: random.profile-name
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

    /// Read Firefox cookies from a SQLite database with optimized query
    fn read_firefox_cookies_from_db(&self, db_path: &Path) -> Result<Vec<Cookie>> {
        if !db_path.exists() {
            return Err(CsCliError::CookieRetrieval(format!(
                "Firefox cookies database not found at: {}",
                db_path.display()
            )));
        }

        debug!("Reading Firefox cookies from: {}", db_path.display());

        let conn = Connection::open(db_path).map_err(|e| {
            CsCliError::CookieRetrieval(format!(
                "Failed to open Firefox cookies database at {}: {}",
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
            .map(|_| "host LIKE ?")
            .collect();
        let where_clause = like_conditions.join(" OR ");

        let query = format!(
            "SELECT name, value, host, path, isSecure, isHttpOnly, expiry FROM moz_cookies WHERE {}",
            where_clause
        );

        debug!("Executing optimized query with {} domain conditions", self.domains.len());

        let mut stmt = conn.prepare(&query).map_err(|e| {
            CsCliError::CookieRetrieval(format!(
                "Failed to prepare Firefox query for {}: {}",
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
                    "Failed to query Firefox cookies from {}: {}",
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

        debug!("Found {} Firefox cookies matching domains from {}", cookies.len(), db_path.display());
        Ok(cookies)
    }

    /// Read cookies directly from a specific Firefox profile's SQLite database
    #[allow(dead_code)]
    fn read_firefox_profile_cookies(&self, profile_path: &Path) -> Result<Vec<Cookie>> {
        let cookies_db_path = profile_path.join("cookies.sqlite");
        self.read_firefox_cookies_from_db(&cookies_db_path)
    }

    /// Read Firefox cookies directly from a database file
    fn read_firefox_cookies_from_file(&self, db_path: &Path) -> Result<Vec<Cookie>> {
        self.read_firefox_cookies_from_db(db_path)
    }
}
