//! # CSRF Protection
//!
//! This module provides Cross-Site Request Forgery (CSRF) protection using
//! secure tokens and validation.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use once_cell::sync::Lazy;

use crate::security::{SecurityConfig, SecurityError, SecurityResult};
use crate::security::encryption::generate_random_token;

/// CSRF token store
static CSRF_STORE: Lazy<Arc<RwLock<HashMap<String, CsrfToken>>>> = 
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

/// CSRF configuration
static mut CSRF_CONFIG: Option<CsrfConfig> = None;

/// CSRF token information
#[derive(Debug, Clone)]
struct CsrfToken {
    token: String,
    created_at: u64,
    expires_at: u64,
    session_id: String,
}

/// CSRF configuration
#[derive(Debug, Clone)]
pub struct CsrfConfig {
    /// Token length in bytes
    pub token_length: usize,
    /// Token expiration time in seconds
    pub token_expiration: u64,
    /// Maximum number of tokens per session
    pub max_tokens_per_session: usize,
    /// Token cleanup interval in seconds
    pub cleanup_interval: u64,
}

impl Default for CsrfConfig {
    fn default() -> Self {
        Self {
            token_length: 32,
            token_expiration: 3600, // 1 hour
            max_tokens_per_session: 10,
            cleanup_interval: 300, // 5 minutes
        }
    }
}

/// Initialize CSRF protection
pub fn initialize(config: &SecurityConfig) -> SecurityResult<()> {
    if !config.csrf_protection {
        return Ok(());
    }
    
    let csrf_config = CsrfConfig::default();
    
    unsafe {
        CSRF_CONFIG = Some(csrf_config);
    }
    
    // Start cleanup task (in a real implementation, this would be a background task)
    println!("🛡️ CSRF protection initialized");
    Ok(())
}

/// Generate a new CSRF token for a session
pub fn generate_token(session_id: &str) -> SecurityResult<String> {
    let config = unsafe {
        CSRF_CONFIG.as_ref().ok_or_else(|| {
            SecurityError::AuthenticationFailed("CSRF not initialized".to_string())
        })?
    };
    
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let token = generate_random_token(config.token_length);
    let csrf_token = CsrfToken {
        token: token.clone(),
        created_at: now,
        expires_at: now + config.token_expiration,
        session_id: session_id.to_string(),
    };
    
    // Store the token
    {
        let mut store = CSRF_STORE.write().unwrap();
        
        // Clean up old tokens for this session
        cleanup_session_tokens(&mut store, session_id, config.max_tokens_per_session);
        
        store.insert(token.clone(), csrf_token);
    }
    
    Ok(token)
}

/// Validate a CSRF token
pub fn validate_token(token: &str, session_id: &str) -> SecurityResult<()> {
    let _config = unsafe {
        CSRF_CONFIG.as_ref().ok_or_else(|| {
            SecurityError::AuthenticationFailed("CSRF not initialized".to_string())
        })?
    };
    
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let mut store = CSRF_STORE.write().unwrap();
    
    // Find and validate the token
    if let Some(csrf_token) = store.get(token) {
        // Check if token belongs to the session
        if csrf_token.session_id != session_id {
            return Err(SecurityError::CsrfTokenMismatch);
        }
        
        // Check if token has expired
        if now > csrf_token.expires_at {
            store.remove(token);
            return Err(SecurityError::CsrfTokenMismatch);
        }
        
        // Token is valid, remove it (one-time use)
        store.remove(token);
        Ok(())
    } else {
        Err(SecurityError::CsrfTokenMismatch)
    }
}

/// Clean up expired tokens
pub fn cleanup_expired_tokens() {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let mut store = CSRF_STORE.write().unwrap();
    store.retain(|_, token| now <= token.expires_at);
}

/// Clean up old tokens for a specific session
fn cleanup_session_tokens(store: &mut HashMap<String, CsrfToken>, session_id: &str, max_tokens: usize) {
    let mut session_tokens: Vec<_> = store
        .iter()
        .filter(|(_, token)| token.session_id == session_id)
        .map(|(key, token)| (key.clone(), token.created_at))
        .collect();
    
    if session_tokens.len() >= max_tokens {
        // Sort by creation time (oldest first)
        session_tokens.sort_by_key(|(_, created_at)| *created_at);
        
        // Remove oldest tokens
        let tokens_to_remove = session_tokens.len() - max_tokens + 1;
        for (token_key, _) in session_tokens.iter().take(tokens_to_remove) {
            store.remove(token_key);
        }
    }
}

/// Get CSRF token from request headers or form data
pub fn extract_token_from_request(headers: &HashMap<String, String>, form_data: &HashMap<String, String>) -> Option<String> {
    // Try to get token from X-CSRF-Token header
    if let Some(token) = headers.get("x-csrf-token").or_else(|| headers.get("X-CSRF-Token")) {
        return Some(token.clone());
    }
    
    // Try to get token from form data
    if let Some(token) = form_data.get("_token").or_else(|| form_data.get("csrf_token")) {
        return Some(token.clone());
    }
    
    None
}

/// Generate CSRF token HTML input field
pub fn generate_token_field(session_id: &str) -> SecurityResult<String> {
    let token = generate_token(session_id)?;
    Ok(format!(r#"<input type="hidden" name="_token" value="{}" />"#, token))
}

/// Generate CSRF token meta tag for AJAX requests
pub fn generate_token_meta_tag(session_id: &str) -> SecurityResult<String> {
    let token = generate_token(session_id)?;
    Ok(format!(r#"<meta name="csrf-token" content="{}" />"#, token))
}

/// CSRF middleware function
pub fn csrf_middleware(
    session_id: &str,
    method: &str,
    headers: &HashMap<String, String>,
    form_data: &HashMap<String, String>,
) -> SecurityResult<()> {
    // Skip CSRF validation for safe methods
    if matches!(method.to_uppercase().as_str(), "GET" | "HEAD" | "OPTIONS") {
        return Ok(());
    }
    
    // Extract token from request
    let token = extract_token_from_request(headers, form_data)
        .ok_or(SecurityError::CsrfTokenMismatch)?;
    
    // Validate the token
    validate_token(&token, session_id)?;
    
    Ok(())
}

/// Check if CSRF protection is enabled
pub fn is_enabled() -> bool {
    unsafe { CSRF_CONFIG.is_some() }
}

/// Get CSRF configuration
pub fn get_config() -> Option<CsrfConfig> {
    unsafe { CSRF_CONFIG.clone() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_generation() {
        let config = SecurityConfig::default();
        initialize(&config).unwrap();
        
        let session_id = "test_session";
        let token = generate_token(session_id).unwrap();
        
        assert!(!token.is_empty());
        assert_eq!(token.len(), 64); // 32 bytes = 64 hex chars
    }

    #[test]
    fn test_token_validation() {
        let config = SecurityConfig::default();
        initialize(&config).unwrap();
        
        let session_id = "test_session";
        let token = generate_token(session_id).unwrap();
        
        // Valid token should pass
        assert!(validate_token(&token, session_id).is_ok());
        
        // Token should be consumed (one-time use)
        assert!(validate_token(&token, session_id).is_err());
    }

    #[test]
    fn test_token_session_mismatch() {
        let config = SecurityConfig::default();
        initialize(&config).unwrap();
        
        let session_id1 = "session1";
        let session_id2 = "session2";
        let token = generate_token(session_id1).unwrap();
        
        // Token from different session should fail
        assert!(validate_token(&token, session_id2).is_err());
    }

    #[test]
    fn test_token_extraction() {
        let mut headers = HashMap::new();
        headers.insert("X-CSRF-Token".to_string(), "test_token".to_string());
        
        let form_data = HashMap::new();
        
        let token = extract_token_from_request(&headers, &form_data);
        assert_eq!(token, Some("test_token".to_string()));
    }

    #[test]
    fn test_csrf_middleware() {
        let config = SecurityConfig::default();
        initialize(&config).unwrap();
        
        let session_id = "test_session";
        let token = generate_token(session_id).unwrap();
        
        let mut headers = HashMap::new();
        headers.insert("X-CSRF-Token".to_string(), token);
        
        let form_data = HashMap::new();
        
        // POST request with valid token should pass
        assert!(csrf_middleware(session_id, "POST", &headers, &form_data).is_ok());
        
        // GET request should pass without token
        assert!(csrf_middleware(session_id, "GET", &HashMap::new(), &HashMap::new()).is_ok());
    }
}
