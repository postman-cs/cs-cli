//! Async Session Encryption Wrapper
//!
//! Provides async wrappers for session encryption operations to avoid
//! blocking the async runtime with CPU-intensive crypto operations.

use super::session_encryption::SessionEncryption;
use super::session_metadata::SessionData;
use crate::common::auth::github::github_gist_errors::GistStorageError;
use std::collections::HashMap;
use tokio::task;

/// Async wrapper for session encryption operations
pub struct AsyncSessionEncryption {
    encryption: SessionEncryption,
}

impl AsyncSessionEncryption {
    /// Create new async encryption wrapper
    pub fn new() -> Result<Self, GistStorageError> {
        let encryption = SessionEncryption::new()
            .map_err(|e| GistStorageError::EncryptionFailed {
                reason: e.to_string(),
            })?;
        
        Ok(Self { encryption })
    }

    /// Encrypt session data asynchronously
    pub async fn encrypt_session(&self, session_data: &SessionData) -> Result<Vec<u8>, GistStorageError> {
        let encryption = self.encryption.clone();
        let session_data = session_data.clone();
        
        task::spawn_blocking(move || {
            // Serialize session data
            let json_data = serde_json::to_vec(&session_data)
                .map_err(|e| GistStorageError::SerializationFailed {
                    reason: e.to_string(),
                })?;
            
            // Encrypt the data
            encryption.encrypt(&json_data)
                .map_err(|e| GistStorageError::EncryptionFailed {
                    reason: e.to_string(),
                })
        })
        .await
        .map_err(|e| GistStorageError::EncryptionFailed {
            reason: format!("Task join error: {}", e),
        })?
    }

    /// Decrypt session data asynchronously
    pub async fn decrypt_session(&self, encrypted_data: &[u8]) -> Result<SessionData, GistStorageError> {
        let encryption = self.encryption.clone();
        let encrypted_data = encrypted_data.to_vec();
        
        task::spawn_blocking(move || {
            // Decrypt the data
            let decrypted_data = encryption.decrypt(&encrypted_data)
                .map_err(|e| GistStorageError::EncryptionFailed {
                    reason: e.to_string(),
                })?;
            
            // Deserialize session data
            let session_data: SessionData = serde_json::from_slice(&decrypted_data)
                .map_err(|e| GistStorageError::SerializationFailed {
                    reason: e.to_string(),
                })?;
            
            // Validate the session data
            session_data.validate()
                .map_err(|e| GistStorageError::SessionValidationFailed {
                    reason: e.to_string(),
                })?;
            
            Ok(session_data)
        })
        .await
        .map_err(|e| GistStorageError::EncryptionFailed {
            reason: format!("Task join error: {}", e),
        })?
    }

    /// Encrypt cookies directly (convenience method)
    pub async fn encrypt_cookies(&self, cookies: &HashMap<String, String>) -> Result<Vec<u8>, GistStorageError> {
        let session_data = SessionData::new(cookies.clone());
        self.encrypt_session(&session_data).await
    }

    /// Decrypt cookies directly (convenience method)
    pub async fn decrypt_cookies(&self, encrypted_data: &[u8]) -> Result<HashMap<String, String>, GistStorageError> {
        let session_data = self.decrypt_session(encrypted_data).await?;
        Ok(session_data.cookies)
    }
}

impl Clone for AsyncSessionEncryption {
    fn clone(&self) -> Self {
        Self {
            encryption: self.encryption.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_async_encryption_roundtrip() {
        let encryption = AsyncSessionEncryption::new().unwrap();
        
        let mut cookies = HashMap::new();
        cookies.insert("session_id".to_string(), "test_value".to_string());
        cookies.insert("auth_token".to_string(), "another_value".to_string());
        
        // Encrypt cookies
        let encrypted = encryption.encrypt_cookies(&cookies).await.unwrap();
        assert!(!encrypted.is_empty());
        
        // Decrypt cookies
        let decrypted = encryption.decrypt_cookies(&encrypted).await.unwrap();
        assert_eq!(cookies, decrypted);
    }

    #[tokio::test]
    async fn test_async_session_data_roundtrip() {
        let encryption = AsyncSessionEncryption::new().unwrap();
        
        let mut cookies = HashMap::new();
        cookies.insert("session_id".to_string(), "test_value".to_string());
        
        let original_session = SessionData::new(cookies);
        
        // Encrypt session
        let encrypted = encryption.encrypt_session(&original_session).await.unwrap();
        assert!(!encrypted.is_empty());
        
        // Decrypt session
        let decrypted_session = encryption.decrypt_session(&encrypted).await.unwrap();
        
        assert_eq!(original_session.metadata.session_id, decrypted_session.metadata.session_id);
        assert_eq!(original_session.cookies, decrypted_session.cookies);
        assert!(decrypted_session.is_valid());
    }

    #[tokio::test]
    async fn test_tampered_data_fails() {
        let encryption = AsyncSessionEncryption::new().unwrap();
        
        let mut cookies = HashMap::new();
        cookies.insert("session_id".to_string(), "test_value".to_string());
        
        let encrypted = encryption.encrypt_cookies(&cookies).await.unwrap();
        
        // Tamper with the encrypted data
        let mut tampered = encrypted.clone();
        if let Some(last_byte) = tampered.last_mut() {
            *last_byte ^= 1;
        }
        
        // Should fail with decryption error
        let result = encryption.decrypt_cookies(&tampered).await;
        assert!(result.is_err());
    }
}