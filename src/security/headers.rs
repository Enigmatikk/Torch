//! # Security Headers
//!
//! This module provides security headers to protect against various attacks.

use std::collections::HashMap;
use crate::security::{SecurityConfig, SecurityResult};

/// Initialize security headers
pub fn initialize(_config: &SecurityConfig) -> SecurityResult<()> {
    println!("🛡️ Security headers initialized");
    Ok(())
}

/// Get default security headers
pub fn get_security_headers() -> HashMap<String, String> {
    let mut headers = HashMap::new();
    
    // Prevent XSS attacks
    headers.insert("X-XSS-Protection".to_string(), "1; mode=block".to_string());
    
    // Prevent MIME type sniffing
    headers.insert("X-Content-Type-Options".to_string(), "nosniff".to_string());
    
    // Prevent clickjacking
    headers.insert("X-Frame-Options".to_string(), "DENY".to_string());
    
    // Referrer policy
    headers.insert("Referrer-Policy".to_string(), "strict-origin-when-cross-origin".to_string());
    
    // Content Security Policy
    headers.insert("Content-Security-Policy".to_string(), 
        "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'".to_string());
    
    // HSTS (only for HTTPS)
    headers.insert("Strict-Transport-Security".to_string(), 
        "max-age=31536000; includeSubDomains".to_string());
    
    headers
}
