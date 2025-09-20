//! Unit tests for session encryption functionality
//!
//! Tests AES-256-GCM encryption, key derivation, and security properties.

use cs_cli::common::auth::session_encryption::SessionEncryption;

#[test]
fn test_encryption_roundtrip() {
    let encryption = SessionEncryption::new().unwrap();
    let test_data = b"test session cookies data with special chars: \x00\xFF";

    let encrypted = encryption.encrypt(test_data).unwrap();
    let decrypted = encryption.decrypt(&encrypted).unwrap();

    assert_eq!(test_data, decrypted.as_slice());
}

#[test]
fn test_empty_data_encryption() {
    let encryption = SessionEncryption::new().unwrap();
    let empty_data = b"";

    let encrypted = encryption.encrypt(empty_data).unwrap();
    let decrypted = encryption.decrypt(&encrypted).unwrap();

    assert_eq!(empty_data, decrypted.as_slice());
}

#[test]
fn test_large_data_encryption() {
    let encryption = SessionEncryption::new().unwrap();
    // Create 1MB of test data
    let large_data = vec![0x42u8; 1024 * 1024];

    let encrypted = encryption.encrypt(&large_data).unwrap();
    let decrypted = encryption.decrypt(&encrypted).unwrap();

    assert_eq!(large_data, decrypted);
}

#[test]
fn test_tampered_data_fails() {
    let encryption = SessionEncryption::new().unwrap();
    let test_data = b"sensitive session data";

    let mut encrypted = encryption.encrypt(test_data).unwrap();

    // Tamper with the data (flip last byte)
    let last_idx = encrypted.len() - 1;
    encrypted[last_idx] ^= 1;

    // Should fail with authentication error
    let result = encryption.decrypt(&encrypted);
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Decryption failed") || error_msg.contains("tampered"));
}

#[test]
fn test_tampered_nonce_fails() {
    let encryption = SessionEncryption::new().unwrap();
    let test_data = b"test data";

    let mut encrypted = encryption.encrypt(test_data).unwrap();

    // Tamper with the nonce (first 12 bytes)
    encrypted[0] ^= 1;

    // Should fail with authentication error
    let result = encryption.decrypt(&encrypted);
    assert!(result.is_err());
}

#[test]
fn test_different_nonces() {
    let encryption = SessionEncryption::new().unwrap();
    let test_data = b"same plaintext data";

    let encrypted1 = encryption.encrypt(test_data).unwrap();
    let encrypted2 = encryption.encrypt(test_data).unwrap();

    // Different nonces should produce different ciphertexts
    assert_ne!(encrypted1, encrypted2);

    // But both should decrypt to the same plaintext
    assert_eq!(encryption.decrypt(&encrypted1).unwrap(), test_data);
    assert_eq!(encryption.decrypt(&encrypted2).unwrap(), test_data);
}

#[test]
fn test_invalid_ciphertext_length() {
    let encryption = SessionEncryption::new().unwrap();

    // Ciphertext too short (less than nonce length)
    let invalid_ciphertext = vec![0u8; 5];
    let result = encryption.decrypt(&invalid_ciphertext);

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Invalid ciphertext length"));
}

#[test]
fn test_encryption_deterministic_key() {
    // Two encryption instances should use the same key (from keychain)
    let encryption1 = SessionEncryption::new().unwrap();
    let encryption2 = SessionEncryption::new().unwrap();

    let test_data = b"cross-instance test data";

    let encrypted = encryption1.encrypt(test_data).unwrap();
    let decrypted = encryption2.decrypt(&encrypted).unwrap();

    assert_eq!(test_data, decrypted.as_slice());
}

#[test]
fn test_unicode_data() {
    let encryption = SessionEncryption::new().unwrap();
    let unicode_data = "测试数据 with emojis 🔐🚀 and symbols ∑∞";
    let test_bytes = unicode_data.as_bytes();

    let encrypted = encryption.encrypt(test_bytes).unwrap();
    let decrypted = encryption.decrypt(&encrypted).unwrap();

    assert_eq!(test_bytes, decrypted.as_slice());

    // Verify we can reconstruct the original string
    let decrypted_string = String::from_utf8(decrypted).unwrap();
    assert_eq!(unicode_data, decrypted_string);
}

#[test]
fn test_json_session_data() {
    use std::collections::HashMap;

    let encryption = SessionEncryption::new().unwrap();

    // Simulate actual session cookie data
    let mut cookies = HashMap::new();
    cookies.insert("gong_session".to_string(), "abc123def456".to_string());
    cookies.insert("okta_sid".to_string(), "xyz789".to_string());
    cookies.insert("csrf_token".to_string(), "secure_token_value".to_string());

    let json_data = serde_json::to_vec(&cookies).unwrap();

    let encrypted = encryption.encrypt(&json_data).unwrap();
    let decrypted = encryption.decrypt(&encrypted).unwrap();

    let restored_cookies: HashMap<String, String> = serde_json::from_slice(&decrypted).unwrap();
    assert_eq!(cookies, restored_cookies);
}

#[cfg(test)]
mod encryption_security_tests {
    use super::*;

    #[test]
    fn test_ciphertext_appears_random() {
        let encryption = SessionEncryption::new().unwrap();
        let test_data = b"predictable test data pattern";

        let encrypted = encryption.encrypt(test_data).unwrap();

        // Ciphertext should not contain obvious patterns from plaintext
        assert!(!encrypted.windows(4).any(|w| w == b"test"));
        assert!(!encrypted.windows(4).any(|w| w == b"data"));
        assert!(!encrypted.windows(7).any(|w| w == b"pattern"));
    }

    #[test]
    fn test_ciphertext_length() {
        let encryption = SessionEncryption::new().unwrap();
        let test_data = b"test data of known length";

        let encrypted = encryption.encrypt(test_data).unwrap();

        // Ciphertext should be: nonce (12) + plaintext length + tag (16)
        let expected_length = 12 + test_data.len() + 16;
        assert_eq!(encrypted.len(), expected_length);
    }

    #[test]
    fn test_no_plaintext_leakage() {
        let encryption = SessionEncryption::new().unwrap();
        let secret_data = b"SECRET_PASSWORD_12345";

        let encrypted = encryption.encrypt(secret_data).unwrap();

        // Encrypted data should not contain any portion of the original secret
        let encrypted_string = String::from_utf8_lossy(&encrypted);
        assert!(!encrypted_string.contains("SECRET"));
        assert!(!encrypted_string.contains("PASSWORD"));
        assert!(!encrypted_string.contains("12345"));
    }
}
