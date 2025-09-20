//! Enhanced Session Metadata with Validation and Security Features
//!
//! Provides session validation, expiration checks, and replay protection
//! for secure session data management.

use chrono::{DateTime, Utc, Duration};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Enhanced session metadata with security features
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SessionMetadata {
    /// Version for future compatibility
    pub version: u32,
    /// When the session was created
    pub created_at: DateTime<Utc>,
    /// When the session was last updated
    pub updated_at: DateTime<Utc>,
    /// When the session expires
    pub expires_at: DateTime<Utc>,
    /// List of platforms included in this session
    pub platforms: Vec<String>,
    /// Unique session identifier for replay protection
    pub session_id: String,
    /// Hash of session content for integrity verification
    pub content_hash: String,
    /// Device identifier for multi-device tracking
    pub device_id: String,
}

impl SessionMetadata {
    /// Create new session metadata with security features
    pub fn new(platforms: Vec<String>) -> Self {
        let now = Utc::now();
        let session_id = generate_session_id();
        let device_id = get_device_id();
        
        Self {
            version: 1,
            created_at: now,
            updated_at: now,
            expires_at: now + Duration::days(30), // 30-day expiration
            platforms,
            session_id,
            content_hash: String::new(), // Will be set when content is available
            device_id,
        }
    }

    /// Update the session with new content
    pub fn update(&mut self, platforms: Vec<String>, content_hash: String) {
        self.updated_at = Utc::now();
        self.platforms = platforms;
        self.content_hash = content_hash;
        
        // Extend expiration by 30 days from now
        self.expires_at = Utc::now() + Duration::days(30);
    }

    /// Validate session metadata
    pub fn validate(&self) -> Result<(), SessionValidationError> {
        let now = Utc::now();

        // Check if session has expired
        if now > self.expires_at {
            return Err(SessionValidationError::Expired {
                expired_at: self.expires_at,
                current_time: now,
            });
        }

        // Check if session is too old (security measure)
        let max_age = Duration::days(90); // Maximum 90 days old
        if now - self.created_at > max_age {
            return Err(SessionValidationError::TooOld {
                created_at: self.created_at,
                max_age_days: 90,
            });
        }

        // Validate session ID format
        if self.session_id.len() != 32 || !self.session_id.chars().all(|c| c.is_alphanumeric()) {
            return Err(SessionValidationError::InvalidSessionId {
                session_id: self.session_id.clone(),
            });
        }

        // Validate device ID format
        if self.device_id.len() != 16 || !self.device_id.chars().all(|c| c.is_alphanumeric()) {
            return Err(SessionValidationError::InvalidDeviceId {
                device_id: self.device_id.clone(),
            });
        }

        // Check version compatibility
        if self.version > 1 {
            return Err(SessionValidationError::UnsupportedVersion {
                version: self.version,
                supported_version: 1,
            });
        }

        Ok(())
    }

    /// Check if session is still valid (not expired)
    pub fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }

    /// Check if session needs refresh (expires within 7 days)
    pub fn needs_refresh(&self) -> bool {
        let now = Utc::now();
        let refresh_threshold = Duration::days(7);
        now + refresh_threshold > self.expires_at
    }

    /// Get time until expiration
    pub fn time_until_expiration(&self) -> Duration {
        let now = Utc::now();
        if now > self.expires_at {
            Duration::zero()
        } else {
            self.expires_at - now
        }
    }

    /// Generate content hash for integrity verification
    pub fn generate_content_hash(cookies: &HashMap<String, String>) -> String {
        // Create a deterministic representation of the cookies
        let mut content = String::new();
        let mut sorted_keys: Vec<_> = cookies.keys().collect();
        sorted_keys.sort();
        
        for key in sorted_keys {
            if let Some(value) = cookies.get(key) {
                content.push_str(&format!("{}={}", key, value));
            }
        }
        
        format!("{:x}", Sha256::digest(content.as_bytes()))
    }
}

/// Session validation errors
#[derive(Debug, thiserror::Error)]
pub enum SessionValidationError {
    #[error("Session expired at {expired_at}, current time: {current_time}")]
    Expired {
        expired_at: DateTime<Utc>,
        current_time: DateTime<Utc>,
    },

    #[error("Session too old: created at {created_at}, maximum age: {max_age_days} days")]
    TooOld {
        created_at: DateTime<Utc>,
        max_age_days: u64,
    },

    #[error("Invalid session ID format: {session_id}")]
    InvalidSessionId { session_id: String },

    #[error("Invalid device ID format: {device_id}")]
    InvalidDeviceId { device_id: String },

    #[error("Unsupported session version: {version}, supported: {supported_version}")]
    UnsupportedVersion { version: u32, supported_version: u32 },

    #[error("Content hash mismatch - possible tampering")]
    ContentHashMismatch,
}

/// Generate a secure session ID
fn generate_session_id() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}

/// Get or generate a device identifier
fn get_device_id() -> String {
    // Try to get a stable device identifier
    if let Ok(hostname) = std::env::var("HOSTNAME") {
        // Use hostname as base for device ID
        let hash = Sha256::digest(hostname.as_bytes());
        format!("{:x}", hash)[..16].to_string()
    } else {
        // Fallback to random device ID
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(16)
            .map(char::from)
            .collect()
    }
}

/// Session data container with metadata and cookies
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SessionData {
    pub metadata: SessionMetadata,
    pub cookies: HashMap<String, String>,
}

impl SessionData {
    /// Create new session data with validation
    pub fn new(cookies: HashMap<String, String>) -> Self {
        let platforms: Vec<String> = cookies.keys().cloned().collect();
        let content_hash = SessionMetadata::generate_content_hash(&cookies);
        
        let mut metadata = SessionMetadata::new(platforms);
        metadata.content_hash = content_hash;
        
        Self { metadata, cookies }
    }

    /// Validate the entire session data
    pub fn validate(&self) -> Result<(), SessionValidationError> {
        // Validate metadata
        self.metadata.validate()?;
        
        // Verify content hash
        let expected_hash = SessionMetadata::generate_content_hash(&self.cookies);
        if self.metadata.content_hash != expected_hash {
            return Err(SessionValidationError::ContentHashMismatch);
        }
        
        Ok(())
    }

    /// Check if session is valid and not expired
    pub fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }

    /// Update session with new cookies
    pub fn update(&mut self, cookies: HashMap<String, String>) {
        let platforms: Vec<String> = cookies.keys().cloned().collect();
        let content_hash = SessionMetadata::generate_content_hash(&cookies);
        
        self.metadata.update(platforms, content_hash);
        self.cookies = cookies;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_session_metadata_creation() {
        let platforms = vec!["gong".to_string(), "slack".to_string()];
        let metadata = SessionMetadata::new(platforms.clone());
        
        assert_eq!(metadata.version, 1);
        assert_eq!(metadata.platforms, platforms);
        assert_eq!(metadata.session_id.len(), 32);
        assert_eq!(metadata.device_id.len(), 16);
        assert!(metadata.is_valid());
    }

    #[test]
    fn test_session_expiration() {
        let platforms = vec!["gong".to_string()];
        let mut metadata = SessionMetadata::new(platforms);
        
        // Set expiration to past
        metadata.expires_at = Utc::now() - Duration::days(1);
        
        assert!(!metadata.is_valid());
        assert!(matches!(
            metadata.validate(),
            Err(SessionValidationError::Expired { .. })
        ));
    }

    #[test]
    fn test_session_data_validation() {
        let mut cookies = HashMap::new();
        cookies.insert("session_id".to_string(), "test_value".to_string());
        cookies.insert("auth_token".to_string(), "another_value".to_string());
        
        let session_data = SessionData::new(cookies);
        assert!(session_data.is_valid());
        
        // Test content hash validation
        let validation_result = session_data.validate();
        assert!(validation_result.is_ok());
    }

    #[test]
    fn test_content_hash_generation() {
        let mut cookies1 = HashMap::new();
        cookies1.insert("a".to_string(), "1".to_string());
        cookies1.insert("b".to_string(), "2".to_string());
        
        let mut cookies2 = HashMap::new();
        cookies2.insert("b".to_string(), "2".to_string());
        cookies2.insert("a".to_string(), "1".to_string());
        
        // Same content, different order should produce same hash
        let hash1 = SessionMetadata::generate_content_hash(&cookies1);
        let hash2 = SessionMetadata::generate_content_hash(&cookies2);
        
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_session_refresh_needed() {
        let platforms = vec!["gong".to_string()];
        let mut metadata = SessionMetadata::new(platforms);
        
        // Set expiration to 3 days from now (within refresh threshold)
        metadata.expires_at = Utc::now() + Duration::days(3);
        
        assert!(metadata.needs_refresh());
        
        // Set expiration to 10 days from now (beyond refresh threshold)
        metadata.expires_at = Utc::now() + Duration::days(10);
        
        assert!(!metadata.needs_refresh());
    }
}