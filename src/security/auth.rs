//! # Authentication and Authorization
//!
//! This module provides secure password hashing and authentication.

use crate::security::{PasswordComplexity, SecurityError, SecurityResult};
use crate::security::encryption::{generate_salt, hash_with_salt, verify_salted_hash};

/// Hash a password securely
pub fn hash_password(password: &str) -> SecurityResult<String> {
    let salt = generate_salt();
    let hash = hash_with_salt(password, &salt);
    Ok(format!("{}:{}", salt, hash))
}

/// Verify a password against its hash
pub fn verify_password(password: &str, stored_hash: &str) -> SecurityResult<bool> {
    let parts: Vec<&str> = stored_hash.split(':').collect();
    if parts.len() != 2 {
        return Err(SecurityError::AuthenticationFailed("Invalid hash format".to_string()));
    }
    
    let salt = parts[0];
    let hash = parts[1];
    
    Ok(verify_salted_hash(password, salt, hash))
}

/// Validate password strength
pub fn validate_password_strength(password: &str, config: &PasswordComplexity) -> SecurityResult<()> {
    if password.len() < 8 {
        return Err(SecurityError::WeakPassword);
    }
    
    let mut char_types = 0;
    
    if config.require_uppercase && password.chars().any(|c| c.is_uppercase()) {
        char_types += 1;
    }
    
    if config.require_lowercase && password.chars().any(|c| c.is_lowercase()) {
        char_types += 1;
    }
    
    if config.require_numbers && password.chars().any(|c| c.is_numeric()) {
        char_types += 1;
    }
    
    if config.require_special_chars && password.chars().any(|c| !c.is_alphanumeric()) {
        char_types += 1;
    }
    
    if char_types < config.min_char_types {
        return Err(SecurityError::WeakPassword);
    }
    
    Ok(())
}
