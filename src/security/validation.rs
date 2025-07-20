//! # Input Validation and Sanitization
//!
//! This module provides comprehensive input validation and sanitization to prevent
//! various security vulnerabilities including XSS, SQL injection, and malicious input.

use regex::Regex;
use std::collections::HashMap;
use once_cell::sync::Lazy;

use crate::security::{SecurityError, SecurityResult};

/// Common validation patterns
static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap()
});

static URL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^https?://[a-zA-Z0-9.-]+(?:\.[a-zA-Z]{2,})+(?:/[^\s]*)?$").unwrap()
});

static PHONE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\+?[1-9]\d{1,14}$").unwrap()
});

static ALPHANUMERIC_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9]+$").unwrap()
});

static SQL_INJECTION_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"(?i)(union|select|insert|update|delete|drop|create|alter|exec|execute)").unwrap(),
        Regex::new(r"(?i)(script|javascript|vbscript|onload|onerror|onclick)").unwrap(),
        Regex::new(r#"['"`;]"#).unwrap(),
        Regex::new(r"--").unwrap(),
        Regex::new(r"/\*.*\*/").unwrap(),
    ]
});

static XSS_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"(?i)<script[^>]*>.*?</script>").unwrap(),
        Regex::new(r"(?i)<iframe[^>]*>.*?</iframe>").unwrap(),
        Regex::new(r"(?i)<object[^>]*>.*?</object>").unwrap(),
        Regex::new(r"(?i)<embed[^>]*>").unwrap(),
        Regex::new(r"(?i)javascript:").unwrap(),
        Regex::new(r"(?i)vbscript:").unwrap(),
        Regex::new(r"(?i)data:").unwrap(),
        Regex::new(r"(?i)on[a-zA-Z]+\s*=").unwrap(),
    ]
});

/// HTML entities for escaping
static HTML_ENTITIES: Lazy<HashMap<char, &'static str>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert('<', "&lt;");
    map.insert('>', "&gt;");
    map.insert('&', "&amp;");
    map.insert('"', "&quot;");
    map.insert('\'', "&#x27;");
    map.insert('/', "&#x2F;");
    map
});

/// Validation rules for different input types
#[derive(Debug, Clone)]
pub enum ValidationRule {
    /// Email address validation
    Email,
    /// URL validation
    Url,
    /// Phone number validation
    Phone,
    /// Alphanumeric only
    Alphanumeric,
    /// Minimum length
    MinLength(usize),
    /// Maximum length
    MaxLength(usize),
    /// Custom regex pattern
    Pattern(String),
    /// No SQL injection patterns
    NoSqlInjection,
    /// No XSS patterns
    NoXss,
    /// Required field (not empty)
    Required,
}

/// Input sanitization options
#[derive(Debug, Clone)]
pub struct SanitizationOptions {
    /// Remove HTML tags
    pub strip_html: bool,
    /// Escape HTML entities
    pub escape_html: bool,
    /// Remove SQL injection patterns
    pub remove_sql_patterns: bool,
    /// Remove XSS patterns
    pub remove_xss_patterns: bool,
    /// Trim whitespace
    pub trim_whitespace: bool,
    /// Convert to lowercase
    pub to_lowercase: bool,
    /// Remove non-printable characters
    pub remove_non_printable: bool,
}

impl Default for SanitizationOptions {
    fn default() -> Self {
        Self {
            strip_html: false,
            escape_html: true,
            remove_sql_patterns: true,
            remove_xss_patterns: true,
            trim_whitespace: true,
            to_lowercase: false,
            remove_non_printable: true,
        }
    }
}

/// Validate input against a set of rules
pub fn validate(input: &str, rules: &[ValidationRule]) -> SecurityResult<()> {
    for rule in rules {
        match rule {
            ValidationRule::Email => {
                if !EMAIL_REGEX.is_match(input) {
                    return Err(SecurityError::InvalidInput("Invalid email format".to_string()));
                }
            }
            ValidationRule::Url => {
                if !URL_REGEX.is_match(input) {
                    return Err(SecurityError::InvalidInput("Invalid URL format".to_string()));
                }
            }
            ValidationRule::Phone => {
                if !PHONE_REGEX.is_match(input) {
                    return Err(SecurityError::InvalidInput("Invalid phone number format".to_string()));
                }
            }
            ValidationRule::Alphanumeric => {
                if !ALPHANUMERIC_REGEX.is_match(input) {
                    return Err(SecurityError::InvalidInput("Input must be alphanumeric".to_string()));
                }
            }
            ValidationRule::MinLength(min) => {
                if input.len() < *min {
                    return Err(SecurityError::InvalidInput(format!("Input must be at least {} characters", min)));
                }
            }
            ValidationRule::MaxLength(max) => {
                if input.len() > *max {
                    return Err(SecurityError::InvalidInput(format!("Input must be at most {} characters", max)));
                }
            }
            ValidationRule::Pattern(pattern) => {
                let regex = Regex::new(pattern).map_err(|_| {
                    SecurityError::InvalidInput("Invalid regex pattern".to_string())
                })?;
                if !regex.is_match(input) {
                    return Err(SecurityError::InvalidInput("Input does not match required pattern".to_string()));
                }
            }
            ValidationRule::NoSqlInjection => {
                for pattern in SQL_INJECTION_PATTERNS.iter() {
                    if pattern.is_match(input) {
                        return Err(SecurityError::InvalidInput("Potential SQL injection detected".to_string()));
                    }
                }
            }
            ValidationRule::NoXss => {
                for pattern in XSS_PATTERNS.iter() {
                    if pattern.is_match(input) {
                        return Err(SecurityError::InvalidInput("Potential XSS attack detected".to_string()));
                    }
                }
            }
            ValidationRule::Required => {
                if input.trim().is_empty() {
                    return Err(SecurityError::InvalidInput("Field is required".to_string()));
                }
            }
        }
    }
    Ok(())
}

/// Sanitize input according to options
pub fn sanitize(input: &str, options: &SanitizationOptions) -> String {
    let mut result = input.to_string();
    
    // Trim whitespace
    if options.trim_whitespace {
        result = result.trim().to_string();
    }
    
    // Remove non-printable characters
    if options.remove_non_printable {
        result = result.chars().filter(|c| c.is_ascii_graphic() || c.is_whitespace()).collect();
    }
    
    // Remove SQL injection patterns
    if options.remove_sql_patterns {
        for pattern in SQL_INJECTION_PATTERNS.iter() {
            result = pattern.replace_all(&result, "").to_string();
        }
    }
    
    // Remove XSS patterns
    if options.remove_xss_patterns {
        for pattern in XSS_PATTERNS.iter() {
            result = pattern.replace_all(&result, "").to_string();
        }
    }
    
    // Strip HTML tags
    if options.strip_html {
        let html_regex = Regex::new(r"<[^>]*>").unwrap();
        result = html_regex.replace_all(&result, "").to_string();
    }
    
    // Escape HTML entities
    if options.escape_html {
        result = escape_html(&result);
    }
    
    // Convert to lowercase
    if options.to_lowercase {
        result = result.to_lowercase();
    }
    
    result
}

/// Validate and sanitize input with default security settings
pub fn validate_and_sanitize(input: &str) -> SecurityResult<String> {
    // Apply basic validation rules
    let rules = vec![
        ValidationRule::NoSqlInjection,
        ValidationRule::NoXss,
        ValidationRule::MaxLength(10000), // Prevent extremely long inputs
    ];
    
    validate(input, &rules)?;
    
    // Apply default sanitization
    let options = SanitizationOptions::default();
    Ok(sanitize(input, &options))
}

/// Escape HTML entities to prevent XSS
pub fn escape_html(input: &str) -> String {
    input.chars().map(|c| {
        HTML_ENTITIES.get(&c).unwrap_or(&c.to_string().as_str()).to_string()
    }).collect()
}

/// Sanitize HTML content while preserving safe tags
pub fn sanitize_html(input: &str) -> String {
    // For now, we'll escape all HTML
    // In a full implementation, this would use a proper HTML sanitizer
    // that allows safe tags while removing dangerous ones
    escape_html(input)
}

/// Validate email address
pub fn validate_email(email: &str) -> SecurityResult<()> {
    validate(email, &[ValidationRule::Email, ValidationRule::MaxLength(254)])
}

/// Validate URL
pub fn validate_url(url: &str) -> SecurityResult<()> {
    validate(url, &[ValidationRule::Url, ValidationRule::MaxLength(2048)])
}

/// Validate username (alphanumeric, 3-30 characters)
pub fn validate_username(username: &str) -> SecurityResult<()> {
    validate(username, &[
        ValidationRule::Required,
        ValidationRule::Alphanumeric,
        ValidationRule::MinLength(3),
        ValidationRule::MaxLength(30),
    ])
}

/// Validate file upload
pub fn validate_file_upload(filename: &str, content_type: &str, size: usize, allowed_types: &[String], max_size: usize) -> SecurityResult<()> {
    // Check file size
    if size > max_size {
        return Err(SecurityError::FileUploadNotAllowed(format!("File too large: {} bytes", size)));
    }
    
    // Check content type
    if !allowed_types.contains(&content_type.to_string()) {
        return Err(SecurityError::FileUploadNotAllowed(format!("File type not allowed: {}", content_type)));
    }
    
    // Validate filename
    let filename_rules = vec![
        ValidationRule::Required,
        ValidationRule::MaxLength(255),
        ValidationRule::NoXss,
    ];
    validate(filename, &filename_rules)?;
    
    // Check for dangerous file extensions
    let dangerous_extensions = vec![".exe", ".bat", ".cmd", ".com", ".pif", ".scr", ".vbs", ".js", ".jar", ".php", ".asp", ".jsp"];
    let filename_lower = filename.to_lowercase();
    for ext in dangerous_extensions {
        if filename_lower.ends_with(ext) {
            return Err(SecurityError::FileUploadNotAllowed(format!("Dangerous file extension: {}", ext)));
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        assert!(validate_email("test@example.com").is_ok());
        assert!(validate_email("invalid-email").is_err());
    }

    #[test]
    fn test_sql_injection_detection() {
        let malicious_input = "'; DROP TABLE users; --";
        assert!(validate(malicious_input, &[ValidationRule::NoSqlInjection]).is_err());
    }

    #[test]
    fn test_xss_detection() {
        let malicious_input = "<script>alert('xss')</script>";
        assert!(validate(malicious_input, &[ValidationRule::NoXss]).is_err());
    }

    #[test]
    fn test_html_escaping() {
        let input = "<script>alert('test')</script>";
        let escaped = escape_html(input);
        assert!(!escaped.contains("<script>"));
        assert!(escaped.contains("&lt;script&gt;"));
    }

    #[test]
    fn test_sanitization() {
        let input = "  <script>alert('test')</script>  ";
        let options = SanitizationOptions::default();
        let sanitized = sanitize(input, &options);
        assert!(!sanitized.contains("<script>"));
        assert!(!sanitized.starts_with(' '));
        assert!(!sanitized.ends_with(' '));
    }
}
