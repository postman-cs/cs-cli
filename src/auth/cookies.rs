use crate::{CsCliError, Result};
use rookie;
use std::collections::HashMap;

/// Multi-browser cookie extractor using the rookie crate
/// Supports Firefox, Chrome, Safari, Edge, Brave, and more
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

impl CookieExtractor {
    pub fn new(domains: Vec<String>) -> Self {
        Self { domains }
    }

    /// Extract Gong cookies from Firefox
    pub fn extract_firefox_cookies(&self) -> Result<Vec<Cookie>> {
        match rookie::firefox(Some(self.domains.clone())) {
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
                "Failed to extract Firefox cookies: {e}"
            ))),
        }
    }

    /// Extract Gong cookies from Chrome
    pub fn extract_chrome_cookies(&self) -> Result<Vec<Cookie>> {
        match rookie::chrome(Some(self.domains.clone())) {
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
                "Failed to extract Chrome cookies: {e}"
            ))),
        }
    }

    /// Extract Gong cookies from Safari (macOS only)
    #[cfg(target_os = "macos")]
    pub fn extract_safari_cookies(&self) -> Result<Vec<Cookie>> {
        match rookie::safari(Some(self.domains.clone())) {
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
                "Failed to extract Safari cookies: {e}"
            ))),
        }
    }

    /// Extract Gong cookies from Edge
    pub fn extract_edge_cookies(&self) -> Result<Vec<Cookie>> {
        match rookie::edge(Some(self.domains.clone())) {
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
                "Failed to extract Edge cookies: {e}"
            ))),
        }
    }

    /// Extract cookies from all browsers that rookie supports with browser detection
    pub fn extract_gong_cookies_with_source(&self) -> Result<(Vec<Cookie>, String)> {
        // Try Firefox first (most common for technical users)
        if let Ok(cookies) = self.extract_firefox_cookies() {
            if !cookies.is_empty() {
                return Ok((cookies, "Firefox".to_string()));
            }
        }

        // Try Chrome
        if let Ok(cookies) = self.extract_chrome_cookies() {
            if !cookies.is_empty() {
                return Ok((cookies, "Chrome".to_string()));
            }
        }

        // Try Edge
        if let Ok(cookies) = self.extract_edge_cookies() {
            if !cookies.is_empty() {
                return Ok((cookies, "Edge".to_string()));
            }
        }

        // Try Arc
        if let Ok(cookies) = self.extract_arc_cookies() {
            if !cookies.is_empty() {
                return Ok((cookies, "Arc".to_string()));
            }
        }

        // Try Brave
        if let Ok(cookies) = self.extract_brave_cookies() {
            if !cookies.is_empty() {
                return Ok((cookies, "Brave".to_string()));
            }
        }

        // Try Chromium
        if let Ok(cookies) = self.extract_chromium_cookies() {
            if !cookies.is_empty() {
                return Ok((cookies, "Chromium".to_string()));
            }
        }

        // Try LibreWolf
        if let Ok(cookies) = self.extract_librewolf_cookies() {
            if !cookies.is_empty() {
                return Ok((cookies, "LibreWolf".to_string()));
            }
        }

        // Try Opera
        if let Ok(cookies) = self.extract_opera_cookies() {
            if !cookies.is_empty() {
                return Ok((cookies, "Opera".to_string()));
            }
        }

        // Try Opera GX
        if let Ok(cookies) = self.extract_opera_gx_cookies() {
            if !cookies.is_empty() {
                return Ok((cookies, "Opera GX".to_string()));
            }
        }

        // Try Vivaldi
        if let Ok(cookies) = self.extract_vivaldi_cookies() {
            if !cookies.is_empty() {
                return Ok((cookies, "Vivaldi".to_string()));
            }
        }

        // Try Zen
        if let Ok(cookies) = self.extract_zen_cookies() {
            if !cookies.is_empty() {
                return Ok((cookies, "Zen".to_string()));
            }
        }

        // Try Safari on macOS
        #[cfg(target_os = "macos")]
        if let Ok(cookies) = self.extract_safari_cookies() {
            if !cookies.is_empty() {
                return Ok((cookies, "Safari".to_string()));
            }
        }

        // Try Cachy Browser on Linux
        #[cfg(target_os = "linux")]
        if let Ok(cookies) = self.extract_cachy_cookies() {
            if !cookies.is_empty() {
                return Ok((cookies, "Cachy Browser".to_string()));
            }
        }

        Err(CsCliError::CookieExtraction(
            "No Gong cookies found in any supported browser (Firefox, Chrome, Edge, Arc, Brave, Chromium, LibreWolf, Opera, Opera GX, Vivaldi, Zen, Safari, Cachy)".to_string(),
        ))
    }

    /// Extract cookies from multiple browsers and return the first successful result
    pub fn extract_gong_cookies(&self) -> Result<Vec<Cookie>> {
        let (cookies, _browser) = self.extract_gong_cookies_with_source()?;
        Ok(cookies)
    }

    /// Extract cookies from Arc browser
    fn extract_arc_cookies(&self) -> Result<Vec<Cookie>> {
        match rookie::arc(Some(self.domains.clone())) {
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
                "Failed to extract Arc cookies: {e}"
            ))),
        }
    }

    /// Extract cookies from Brave browser  
    fn extract_brave_cookies(&self) -> Result<Vec<Cookie>> {
        match rookie::brave(Some(self.domains.clone())) {
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
                "Failed to extract Brave cookies: {e}"
            ))),
        }
    }

    /// Extract cookies from Chromium browser
    fn extract_chromium_cookies(&self) -> Result<Vec<Cookie>> {
        match rookie::chromium(Some(self.domains.clone())) {
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
                "Failed to extract Chromium cookies: {e}"
            ))),
        }
    }

    /// Extract cookies from LibreWolf browser
    fn extract_librewolf_cookies(&self) -> Result<Vec<Cookie>> {
        match rookie::librewolf(Some(self.domains.clone())) {
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
                "Failed to extract LibreWolf cookies: {e}"
            ))),
        }
    }

    /// Extract cookies from Opera browser
    fn extract_opera_cookies(&self) -> Result<Vec<Cookie>> {
        match rookie::opera(Some(self.domains.clone())) {
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
                "Failed to extract Opera cookies: {e}"
            ))),
        }
    }

    /// Extract cookies from Opera GX browser
    fn extract_opera_gx_cookies(&self) -> Result<Vec<Cookie>> {
        match rookie::opera_gx(Some(self.domains.clone())) {
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
                "Failed to extract Opera GX cookies: {e}"
            ))),
        }
    }

    /// Extract cookies from Vivaldi browser
    fn extract_vivaldi_cookies(&self) -> Result<Vec<Cookie>> {
        match rookie::vivaldi(Some(self.domains.clone())) {
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
                "Failed to extract Vivaldi cookies: {e}"
            ))),
        }
    }

    /// Extract cookies from Zen browser
    fn extract_zen_cookies(&self) -> Result<Vec<Cookie>> {
        match rookie::zen(Some(self.domains.clone())) {
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
                "Failed to extract Zen cookies: {e}"
            ))),
        }
    }

    /// Extract cookies from Cachy Browser (Linux only)
    #[cfg(target_os = "linux")]
    fn extract_cachy_cookies(&self) -> Result<Vec<Cookie>> {
        match rookie::cachy(Some(self.domains.clone())) {
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
                "Failed to extract Cachy cookies: {e}"
            ))),
        }
    }

    /// Extract authentication cookies from available browsers and return as headers map
    pub fn get_auth_headers(&self) -> Result<HashMap<String, String>> {
        let cookies = self.extract_gong_cookies()?;
        let mut headers = HashMap::new();

        // Build cookie header string
        let cookie_header = cookies
            .iter()
            .map(|c| format!("{}={}", c.name, c.value))
            .collect::<Vec<_>>()
            .join("; ");

        if !cookie_header.is_empty() {
            headers.insert("Cookie".to_string(), cookie_header);
        }

        Ok(headers)
    }
}

// Convenience functions for backward compatibility
pub fn extract_gong_cookies() -> Result<Vec<Cookie>> {
    let domains = vec!["gong.io".to_string(), ".gong.io".to_string()];
    let extractor = CookieExtractor::new(domains);
    extractor.extract_gong_cookies()
}
