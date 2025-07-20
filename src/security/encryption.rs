//! # Encryption and Cryptographic Functions
//!
//! This module provides secure encryption, hashing, and random token generation
//! for the Torch security system.

use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;
use sha2::{Sha256, Digest};
use std::time::{SystemTime, UNIX_EPOCH};
use base64::{Engine as _, engine::general_purpose};

use crate::security::{SecurityError, SecurityResult};

/// Generate a cryptographically secure random token
pub fn generate_random_token(length: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

/// Generate a random hex token
pub fn generate_hex_token(byte_length: usize) -> String {
    let mut rng = thread_rng();
    let bytes: Vec<u8> = (0..byte_length).map(|_| rng.gen::<u8>()).collect();
    hex::encode(bytes)
}

/// Generate a secure session ID
pub fn generate_session_id() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    
    let random_part = generate_hex_token(16);
    let combined = format!("{}{}", timestamp, random_part);
    
    // Hash the combined value for additional security
    let mut hasher = Sha256::new();
    hasher.update(combined.as_bytes());
    hex::encode(hasher.finalize())
}

/// Generate a secure API key
pub fn generate_api_key() -> String {
    format!("torch_{}", generate_hex_token(32))
}

/// Hash data using SHA-256
pub fn hash_sha256(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    hex::encode(hasher.finalize())
}

/// Generate a secure salt for password hashing
pub fn generate_salt() -> String {
    generate_hex_token(16)
}

/// Create a secure hash with salt
pub fn hash_with_salt(data: &str, salt: &str) -> String {
    let combined = format!("{}{}", data, salt);
    hash_sha256(&combined)
}

/// Verify data against a salted hash
pub fn verify_salted_hash(data: &str, salt: &str, hash: &str) -> bool {
    let computed_hash = hash_with_salt(data, salt);
    // Use constant-time comparison to prevent timing attacks
    constant_time_eq(&computed_hash, hash)
}

/// Constant-time string comparison to prevent timing attacks
pub fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    
    let mut result = 0u8;
    for (byte_a, byte_b) in a.bytes().zip(b.bytes()) {
        result |= byte_a ^ byte_b;
    }
    
    result == 0
}

/// Generate a time-based one-time password (TOTP) secret
pub fn generate_totp_secret() -> String {
    generate_random_token(32)
}

/// Simple XOR encryption (for demonstration - use proper encryption in production)
pub fn simple_encrypt(data: &str, key: &str) -> String {
    let key_bytes = key.as_bytes();
    let encrypted: Vec<u8> = data
        .bytes()
        .enumerate()
        .map(|(i, byte)| byte ^ key_bytes[i % key_bytes.len()])
        .collect();
    
    general_purpose::STANDARD.encode(&encrypted)
}

/// Simple XOR decryption (for demonstration - use proper encryption in production)
pub fn simple_decrypt(encrypted_data: &str, key: &str) -> SecurityResult<String> {
    let encrypted_bytes = general_purpose::STANDARD.decode(encrypted_data.as_bytes())
        .map_err(|_| SecurityError::EncryptionError("Invalid base64 data".to_string()))?;
    
    let key_bytes = key.as_bytes();
    let decrypted: Vec<u8> = encrypted_bytes
        .iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key_bytes[i % key_bytes.len()])
        .collect();
    
    String::from_utf8(decrypted)
        .map_err(|_| SecurityError::EncryptionError("Invalid UTF-8 data".to_string()))
}

/// Generate a secure encryption key
pub fn generate_encryption_key() -> String {
    generate_hex_token(32) // 256-bit key
}

/// Create a secure hash for file integrity checking
pub fn hash_file_content(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    hex::encode(hasher.finalize())
}

/// Generate a secure nonce for cryptographic operations
pub fn generate_nonce() -> String {
    generate_hex_token(12) // 96-bit nonce
}

/// Create a secure signature for data integrity
pub fn create_signature(data: &str, secret: &str) -> String {
    let combined = format!("{}.{}", data, secret);
    hash_sha256(&combined)
}

/// Verify a signature
pub fn verify_signature(data: &str, signature: &str, secret: &str) -> bool {
    let expected_signature = create_signature(data, secret);
    constant_time_eq(&expected_signature, signature)
}

/// Generate a secure password reset token
pub fn generate_password_reset_token() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let random_part = generate_hex_token(20);
    format!("{}_{}", timestamp, random_part)
}

/// Validate a password reset token (check if not expired)
pub fn validate_password_reset_token(token: &str, max_age_seconds: u64) -> SecurityResult<()> {
    let parts: Vec<&str> = token.split('_').collect();
    if parts.len() != 2 {
        return Err(SecurityError::AuthenticationFailed("Invalid token format".to_string()));
    }
    
    let timestamp = parts[0].parse::<u64>()
        .map_err(|_| SecurityError::AuthenticationFailed("Invalid timestamp".to_string()))?;
    
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    if current_time - timestamp > max_age_seconds {
        return Err(SecurityError::AuthenticationFailed("Token expired".to_string()));
    }
    
    Ok(())
}

/// Generate a secure email verification token
pub fn generate_email_verification_token(email: &str) -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let data = format!("{}:{}", email, timestamp);
    let signature = hash_sha256(&data);
    
    general_purpose::STANDARD.encode(format!("{}:{}", data, signature).as_bytes())
}

/// Verify an email verification token
pub fn verify_email_verification_token(token: &str, expected_email: &str, max_age_seconds: u64) -> SecurityResult<()> {
    let decoded = general_purpose::STANDARD.decode(token.as_bytes())
        .map_err(|_| SecurityError::AuthenticationFailed("Invalid token format".to_string()))?;
    
    let decoded_str = String::from_utf8(decoded)
        .map_err(|_| SecurityError::AuthenticationFailed("Invalid token encoding".to_string()))?;
    
    let parts: Vec<&str> = decoded_str.split(':').collect();
    if parts.len() != 3 {
        return Err(SecurityError::AuthenticationFailed("Invalid token structure".to_string()));
    }
    
    let email = parts[0];
    let timestamp = parts[1].parse::<u64>()
        .map_err(|_| SecurityError::AuthenticationFailed("Invalid timestamp".to_string()))?;
    let signature = parts[2];
    
    // Verify email matches
    if email != expected_email {
        return Err(SecurityError::AuthenticationFailed("Email mismatch".to_string()));
    }
    
    // Verify signature
    let data = format!("{}:{}", email, timestamp);
    let expected_signature = hash_sha256(&data);
    if !constant_time_eq(signature, &expected_signature) {
        return Err(SecurityError::AuthenticationFailed("Invalid signature".to_string()));
    }
    
    // Check expiration
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    if current_time - timestamp > max_age_seconds {
        return Err(SecurityError::AuthenticationFailed("Token expired".to_string()));
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_token_generation() {
        let token1 = generate_random_token(32);
        let token2 = generate_random_token(32);
        
        assert_eq!(token1.len(), 32);
        assert_eq!(token2.len(), 32);
        assert_ne!(token1, token2); // Should be different
    }

    #[test]
    fn test_hex_token_generation() {
        let token = generate_hex_token(16);
        assert_eq!(token.len(), 32); // 16 bytes = 32 hex chars
        
        // Should only contain hex characters
        assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_session_id_generation() {
        let id1 = generate_session_id();
        let id2 = generate_session_id();
        
        assert_ne!(id1, id2);
        assert_eq!(id1.len(), 64); // SHA-256 hash = 64 hex chars
    }

    #[test]
    fn test_hash_with_salt() {
        let data = "password123";
        let salt = generate_salt();
        let hash = hash_with_salt(data, &salt);
        
        assert!(verify_salted_hash(data, &salt, &hash));
        assert!(!verify_salted_hash("wrong_password", &salt, &hash));
    }

    #[test]
    fn test_constant_time_eq() {
        assert!(constant_time_eq("hello", "hello"));
        assert!(!constant_time_eq("hello", "world"));
        assert!(!constant_time_eq("hello", "hello_world"));
    }

    #[test]
    fn test_simple_encryption() {
        let data = "secret message";
        let key = "encryption_key";
        
        let encrypted = simple_encrypt(data, key);
        let decrypted = simple_decrypt(&encrypted, key).unwrap();
        
        assert_eq!(data, decrypted);
        assert_ne!(data, encrypted);
    }

    #[test]
    fn test_signature_verification() {
        let data = "important data";
        let secret = "secret_key";
        
        let signature = create_signature(data, secret);
        assert!(verify_signature(data, &signature, secret));
        assert!(!verify_signature("tampered data", &signature, secret));
        assert!(!verify_signature(data, &signature, "wrong_secret"));
    }

    #[test]
    fn test_password_reset_token() {
        let token = generate_password_reset_token();
        assert!(validate_password_reset_token(&token, 3600).is_ok());
        
        // Test with expired token (simulate old timestamp)
        let old_token = "1000000000_randompart";
        assert!(validate_password_reset_token(old_token, 3600).is_err());
    }
}
