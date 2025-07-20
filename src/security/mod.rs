//! # Torch Security Module
//!
//! This module provides comprehensive security features for the Torch web framework,
//! including input validation, SQL injection prevention, XSS protection, CSRF protection,
//! and secure defaults.
//!
//! ## Features
//!
//! - **Input Validation** - Comprehensive input sanitization and validation
//! - **SQL Injection Prevention** - Parameterized queries and input escaping
//! - **XSS Protection** - Output encoding and Content Security Policy
//! - **CSRF Protection** - Token-based CSRF protection
//! - **Rate Limiting** - Request rate limiting and DDoS protection
//! - **Secure Headers** - Security headers for HTTPS, HSTS, etc.
//! - **Authentication** - Secure password hashing and session management
//! - **Authorization** - Role-based access control
//!
//! ## Usage
//!
//! ```rust
//! use torch_web::security::{SecurityConfig, validate_input, sanitize_html};
//!
//! // Configure security settings
//! let config = SecurityConfig {
//!     csrf_protection: true,
//!     xss_protection: true,
//!     sql_injection_protection: true,
//!     rate_limiting: true,
//!     secure_headers: true,
//!     ..Default::default()
//! };
//!
//! // Validate and sanitize input
//! let user_input = "Some user input";
//! let validated = validate_input(user_input)?;
//! let sanitized = sanitize_html(&validated);
//! ```

pub mod validation;
pub mod csrf;
pub mod headers;
pub mod rate_limit;
pub mod auth;
pub mod encryption;

use serde::{Deserialize, Serialize};

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable CSRF protection
    pub csrf_protection: bool,
    
    /// Enable XSS protection
    pub xss_protection: bool,
    
    /// Enable SQL injection protection
    pub sql_injection_protection: bool,
    
    /// Enable rate limiting
    pub rate_limiting: bool,
    
    /// Enable secure headers
    pub secure_headers: bool,
    
    /// Maximum request size in bytes
    pub max_request_size: usize,
    
    /// Session timeout in seconds
    pub session_timeout: u64,
    
    /// Password minimum length
    pub password_min_length: usize,
    
    /// Password complexity requirements
    pub password_complexity: PasswordComplexity,
    
    /// Rate limiting configuration
    pub rate_limit_config: RateLimitConfig,
    
    /// CORS configuration
    pub cors_config: CorsConfig,
    
    /// Content Security Policy
    pub csp_policy: String,
    
    /// Allowed file upload types
    pub allowed_upload_types: Vec<String>,
    
    /// Maximum file upload size in bytes
    pub max_upload_size: usize,
}

/// Password complexity requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordComplexity {
    /// Require uppercase letters
    pub require_uppercase: bool,
    
    /// Require lowercase letters
    pub require_lowercase: bool,
    
    /// Require numbers
    pub require_numbers: bool,
    
    /// Require special characters
    pub require_special_chars: bool,
    
    /// Minimum number of character types required
    pub min_char_types: usize,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Requests per minute
    pub requests_per_minute: u32,
    
    /// Burst size
    pub burst_size: u32,
    
    /// Rate limit by IP address
    pub by_ip: bool,
    
    /// Rate limit by user ID
    pub by_user: bool,
    
    /// Rate limit by API key
    pub by_api_key: bool,
}

/// CORS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    /// Allowed origins
    pub allowed_origins: Vec<String>,
    
    /// Allowed methods
    pub allowed_methods: Vec<String>,
    
    /// Allowed headers
    pub allowed_headers: Vec<String>,
    
    /// Exposed headers
    pub exposed_headers: Vec<String>,
    
    /// Allow credentials
    pub allow_credentials: bool,
    
    /// Max age for preflight requests
    pub max_age: u32,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            csrf_protection: true,
            xss_protection: true,
            sql_injection_protection: true,
            rate_limiting: true,
            secure_headers: true,
            max_request_size: 16 * 1024 * 1024, // 16MB
            session_timeout: 3600, // 1 hour
            password_min_length: 8,
            password_complexity: PasswordComplexity::default(),
            rate_limit_config: RateLimitConfig::default(),
            cors_config: CorsConfig::default(),
            csp_policy: "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'".to_string(),
            allowed_upload_types: vec![
                "image/jpeg".to_string(),
                "image/png".to_string(),
                "image/gif".to_string(),
                "text/plain".to_string(),
                "application/pdf".to_string(),
            ],
            max_upload_size: 10 * 1024 * 1024, // 10MB
        }
    }
}

impl Default for PasswordComplexity {
    fn default() -> Self {
        Self {
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_special_chars: true,
            min_char_types: 3,
        }
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            burst_size: 10,
            by_ip: true,
            by_user: true,
            by_api_key: true,
        }
    }
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec!["http://localhost:3000".to_string()],
            allowed_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
                "OPTIONS".to_string(),
            ],
            allowed_headers: vec!["*".to_string()],
            exposed_headers: vec![],
            allow_credentials: false,
            max_age: 86400, // 24 hours
        }
    }
}

/// Security error types
#[derive(Debug, thiserror::Error)]
pub enum SecurityError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("CSRF token mismatch")]
    CsrfTokenMismatch,
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),
    
    #[error("Password does not meet complexity requirements")]
    WeakPassword,
    
    #[error("File upload not allowed: {0}")]
    FileUploadNotAllowed(String),
    
    #[error("Request too large")]
    RequestTooLarge,
    
    #[error("Session expired")]
    SessionExpired,
    
    #[error("Encryption error: {0}")]
    EncryptionError(String),
}

/// Result type for security operations
pub type SecurityResult<T> = Result<T, SecurityError>;

/// Initialize security module with configuration
pub fn initialize_security(config: SecurityConfig) -> SecurityResult<()> {
    // Initialize security components
    csrf::initialize(&config)?;
    rate_limit::initialize(&config.rate_limit_config)?;
    headers::initialize(&config)?;
    
    println!("🔒 Security module initialized with secure defaults");
    Ok(())
}

/// Validate and sanitize user input
pub fn validate_input(input: &str) -> SecurityResult<String> {
    validation::validate_and_sanitize(input)
}

/// Sanitize HTML content to prevent XSS
pub fn sanitize_html(input: &str) -> String {
    validation::sanitize_html(input)
}

/// Generate secure random token
pub fn generate_secure_token(length: usize) -> String {
    encryption::generate_random_token(length)
}

/// Hash password securely
pub fn hash_password(password: &str) -> SecurityResult<String> {
    auth::hash_password(password)
}

/// Verify password against hash
pub fn verify_password(password: &str, hash: &str) -> SecurityResult<bool> {
    auth::verify_password(password, hash)
}

/// Check if password meets complexity requirements
pub fn validate_password_strength(password: &str, config: &PasswordComplexity) -> SecurityResult<()> {
    auth::validate_password_strength(password, config)
}

/// Security headers middleware (placeholder for future implementation)
pub struct SecurityHeaders;

impl SecurityHeaders {
    pub fn new() -> Self {
        Self
    }
}

/// Request ID middleware (placeholder for future implementation)
pub struct RequestId;

/// Input validation middleware (placeholder for future implementation)
pub struct InputValidator;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_config_defaults() {
        let config = SecurityConfig::default();
        assert!(config.csrf_protection);
        assert!(config.xss_protection);
        assert!(config.sql_injection_protection);
        assert!(config.rate_limiting);
        assert!(config.secure_headers);
    }

    #[test]
    fn test_password_complexity_defaults() {
        let complexity = PasswordComplexity::default();
        assert!(complexity.require_uppercase);
        assert!(complexity.require_lowercase);
        assert!(complexity.require_numbers);
        assert!(complexity.require_special_chars);
        assert_eq!(complexity.min_char_types, 3);
    }
}
