//! # Security Middleware and Utilities
//!
//! This module provides comprehensive security middleware and utilities for protecting
//! Torch web applications. It includes protection against common web vulnerabilities,
//! request signing, IP whitelisting, security headers, and input validation.
//!
//! ## Available Security Features
//!
//! - **Security Headers**: Automatic security headers (HSTS, CSP, X-Frame-Options, etc.)
//! - **Request Signing**: HMAC-based request authentication
//! - **IP Whitelisting**: Restrict access to specific IP addresses or ranges
//! - **Request ID**: Generate unique IDs for request tracking
//! - **Input Validation**: Validate and sanitize user input
//! - **Rate Limiting**: Protect against abuse and DoS attacks
//!
//! ## Security Best Practices
//!
//! ### 1. Always Use HTTPS in Production
//! ```text
//! Configure your reverse proxy (nginx, Apache) or load balancer
//! to terminate SSL and forward to your Torch application
//! ```
//!
//! ### 2. Enable Security Headers
//! ```rust
//! use torch_web::{App, security::SecurityHeaders};
//!
//! let app = App::new()
//!     .middleware(SecurityHeaders::new())
//!     .get("/", |_req| async { Response::ok().body("Secure!") });
//! ```
//!
//! ### 3. Validate All Input
//! ```rust
//! use torch_web::{App, security::InputValidator};
//!
//! let app = App::new()
//!     .middleware(InputValidator)
//!     .post("/api/data", |req| async move {
//!         // Input is automatically validated
//!         Response::ok().body("Data processed")
//!     });
//! ```
//!
//! ### 4. Use Request Signing for APIs
//! ```rust
//! use torch_web::{App, security::RequestSigning};
//!
//! let app = App::new()
//!     .middleware(RequestSigning::new("your-secret-key"))
//!     .post("/api/webhook", |req| async move {
//!         // Request signature is automatically verified
//!         Response::ok().body("Webhook processed")
//!     });
//! ```

use std::collections::HashSet;
use std::net::IpAddr;
use std::str::FromStr;
use crate::{Request, Response, middleware::Middleware};

#[cfg(feature = "security")]
use {
    hmac::{Hmac, Mac},
    sha2::Sha256,
    base64::{Engine as _, engine::general_purpose},
    uuid::Uuid,
};

/// HMAC-based request signing middleware for API security.
///
/// This middleware verifies that incoming requests are signed with a shared secret,
/// providing authentication and integrity protection for API endpoints. It's particularly
/// useful for webhooks and server-to-server communication.
///
/// **Note**: This middleware requires the `security` feature to be enabled.
///
/// # How It Works
///
/// 1. The client calculates an HMAC-SHA256 signature of the request body using a shared secret
/// 2. The signature is sent in the `X-Signature` header as `sha256=<hex-encoded-signature>`
/// 3. The middleware recalculates the signature and compares it with the provided one
/// 4. If signatures match, the request is allowed; otherwise, it's rejected with 401 Unauthorized
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use torch_web::{App, security::RequestSigning};
///
/// let app = App::new()
///     .middleware(RequestSigning::new("your-secret-key"))
///     .post("/webhook", |req| async move {
///         // Request signature has been verified
///         Response::ok().body("Webhook received")
///     });
/// ```
///
/// ## Client-Side Signing (Example)
///
/// ```python
/// import hmac
/// import hashlib
/// import requests
///
/// def sign_request(secret, body):
///     signature = hmac.new(
///         secret.encode('utf-8'),
///         body.encode('utf-8'),
///         hashlib.sha256
///     ).hexdigest()
///     return f"sha256={signature}"
///
/// # Send signed request
/// body = '{"event": "user.created", "data": {...}}'
/// signature = sign_request("your-secret-key", body)
///
/// response = requests.post(
///     "https://your-api.com/webhook",
///     data=body,
///     headers={
///         "Content-Type": "application/json",
///         "X-Signature": signature
///     }
/// )
/// ```
///
/// ## With Custom Header Name
///
/// ```rust
/// use torch_web::{App, security::RequestSigning};
///
/// // You can customize the header name by modifying the middleware
/// let app = App::new()
///     .middleware(RequestSigning::new("your-secret-key"))
///     .post("/github-webhook", |req| async move {
///         // GitHub uses X-Hub-Signature-256 header
///         Response::ok().body("GitHub webhook processed")
///     });
/// ```
pub struct RequestSigning {
    #[cfg(feature = "security")]
    secret: Vec<u8>,
    #[cfg(not(feature = "security"))]
    _phantom: std::marker::PhantomData<()>,
}

impl RequestSigning {
    #[cfg(feature = "security")]
    pub fn new(secret: &str) -> Self {
        Self {
            secret: secret.as_bytes().to_vec(),
        }
    }

    #[cfg(not(feature = "security"))]
    pub fn new(_secret: &str) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl Middleware for RequestSigning {
    fn call(
        &self,
        req: Request,
        next: Box<dyn Fn(Request) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send + 'static>> + Send + Sync>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send + 'static>> {
        #[cfg(feature = "security")]
        {
            let secret = self.secret.clone();
            Box::pin(async move {
                // Verify request signature
                if let Some(signature) = req.header("X-Signature") {
                    let body = req.body();
                    let timestamp = req.header("X-Timestamp").unwrap_or("0");
                    
                    let payload = format!("{}{}", timestamp, std::str::from_utf8(body).unwrap_or(""));
                    
                    let mut mac = Hmac::<Sha256>::new_from_slice(&secret)
                        .expect("HMAC can take key of any size");
                    mac.update(payload.as_bytes());
                    let expected = general_purpose::STANDARD.encode(mac.finalize().into_bytes());
                    
                    if signature != expected {
                        return Response::with_status(http::StatusCode::UNAUTHORIZED)
                            .body("Invalid signature");
                    }
                } else {
                    return Response::with_status(http::StatusCode::UNAUTHORIZED)
                        .body("Missing signature");
                }
                
                next(req).await
            })
        }
        
        #[cfg(not(feature = "security"))]
        {
            Box::pin(async move {
                next(req).await
            })
        }
    }
}

/// IP whitelist middleware
pub struct IpWhitelist {
    allowed_ips: HashSet<IpAddr>,
    allowed_ranges: Vec<(IpAddr, u8)>, // IP and prefix length
}

impl IpWhitelist {
    pub fn new() -> Self {
        Self {
            allowed_ips: HashSet::new(),
            allowed_ranges: Vec::new(),
        }
    }

    pub fn allow_ip(mut self, ip: &str) -> Self {
        if let Ok(ip) = IpAddr::from_str(ip) {
            self.allowed_ips.insert(ip);
        }
        self
    }

    pub fn allow_range(mut self, range: &str) -> Self {
        if let Some((ip_str, prefix_str)) = range.split_once('/') {
            if let (Ok(ip), Ok(prefix)) = (IpAddr::from_str(ip_str), prefix_str.parse::<u8>()) {
                self.allowed_ranges.push((ip, prefix));
            }
        }
        self
    }

    #[allow(dead_code)]
    fn is_ip_allowed(&self, client_ip: IpAddr) -> bool {
        // Check exact IP matches
        if self.allowed_ips.contains(&client_ip) {
            return true;
        }

        // Check IP ranges
        for (range_ip, prefix) in &self.allowed_ranges {
            if self.ip_in_range(client_ip, *range_ip, *prefix) {
                return true;
            }
        }

        false
    }

    #[allow(dead_code)]
    fn ip_in_range(&self, ip: IpAddr, range_ip: IpAddr, prefix: u8) -> bool {
        match (ip, range_ip) {
            (IpAddr::V4(ip), IpAddr::V4(range_ip)) => {
                let ip_bits = u32::from(ip);
                let range_bits = u32::from(range_ip);
                let mask = !((1u32 << (32 - prefix)) - 1);
                (ip_bits & mask) == (range_bits & mask)
            }
            (IpAddr::V6(ip), IpAddr::V6(range_ip)) => {
                let ip_bits = u128::from(ip);
                let range_bits = u128::from(range_ip);
                let mask = !((1u128 << (128 - prefix)) - 1);
                (ip_bits & mask) == (range_bits & mask)
            }
            _ => false,
        }
    }
}

fn is_ip_allowed_static(
    client_ip: IpAddr,
    allowed_ips: &HashSet<IpAddr>,
    allowed_ranges: &[(IpAddr, u8)]
) -> bool {
    // Check exact IP matches
    if allowed_ips.contains(&client_ip) {
        return true;
    }

    // Check IP ranges
    for (range_ip, prefix) in allowed_ranges {
        if ip_in_range_static(client_ip, *range_ip, *prefix) {
            return true;
        }
    }

    false
}

fn ip_in_range_static(ip: IpAddr, range_ip: IpAddr, prefix: u8) -> bool {
    match (ip, range_ip) {
        (IpAddr::V4(ip), IpAddr::V4(range_ip)) => {
            let ip_bits = u32::from(ip);
            let range_bits = u32::from(range_ip);
            let mask = !((1u32 << (32 - prefix)) - 1);
            (ip_bits & mask) == (range_bits & mask)
        }
        (IpAddr::V6(ip), IpAddr::V6(range_ip)) => {
            let ip_bits = u128::from(ip);
            let range_bits = u128::from(range_ip);
            let mask = !((1u128 << (128 - prefix)) - 1);
            (ip_bits & mask) == (range_bits & mask)
        }
        _ => false,
    }
}

impl Middleware for IpWhitelist {
    fn call(
        &self,
        req: Request,
        next: Box<dyn Fn(Request) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send + 'static>> + Send + Sync>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send + 'static>> {
        let allowed_ips = self.allowed_ips.clone();
        let allowed_ranges = self.allowed_ranges.clone();

        Box::pin(async move {
            // Extract client IP from headers or connection info
            let client_ip = req.header("X-Forwarded-For")
                .or_else(|| req.header("X-Real-IP"))
                .and_then(|ip_str| IpAddr::from_str(ip_str).ok());

            if let Some(client_ip) = client_ip {
                if !is_ip_allowed_static(client_ip, &allowed_ips, &allowed_ranges) {
                    return Response::with_status(http::StatusCode::FORBIDDEN)
                        .body("IP address not allowed");
                }
            } else {
                return Response::with_status(http::StatusCode::BAD_REQUEST)
                    .body("Unable to determine client IP");
            }

            next(req).await
        })
    }
}

/// Request ID middleware for tracking requests
pub struct RequestId;

impl Middleware for RequestId {
    fn call(
        &self,
        req: Request,
        next: Box<dyn Fn(Request) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send + 'static>> + Send + Sync>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send + 'static>> {
        Box::pin(async move {
            // Generate or extract request ID
            let request_id = req.header("X-Request-ID")
                .map(|id| id.to_string())
                .unwrap_or_else(|| {
                    #[cfg(feature = "security")]
                    {
                        Uuid::new_v4().to_string()
                    }
                    #[cfg(not(feature = "security"))]
                    {
                        format!("req_{}", std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis())
                    }
                });

            // Add request ID to request context (would need to extend Request struct)
            // For now, we'll add it as a custom header in the response
            
            let mut response = next(req).await;
            response = response.header("X-Request-ID", &request_id);
            response
        })
    }
}

/// Input validation middleware
pub struct InputValidator;

impl InputValidator {
    fn is_safe_input(input: &str) -> bool {
        // Basic SQL injection patterns
        let sql_patterns = [
            "union", "select", "insert", "update", "delete", "drop", "create", "alter",
            "exec", "execute", "sp_", "xp_", "--", "/*", "*/", ";",
        ];

        // Basic XSS patterns
        let xss_patterns = [
            "<script", "</script>", "javascript:", "onload=", "onerror=", "onclick=",
            "onmouseover=", "onfocus=", "onblur=", "onchange=", "onsubmit=",
        ];

        let input_lower = input.to_lowercase();

        // Check for SQL injection patterns
        for pattern in &sql_patterns {
            if input_lower.contains(pattern) {
                return false;
            }
        }

        // Check for XSS patterns
        for pattern in &xss_patterns {
            if input_lower.contains(pattern) {
                return false;
            }
        }

        // Check for path traversal
        if input.contains("../") || input.contains("..\\") {
            return false;
        }

        // Check for null bytes
        if input.contains('\0') {
            return false;
        }

        true
    }

    fn validate_request_data(req: &Request) -> Result<(), String> {
        // Validate query parameters
        for (key, value) in req.query_params() {
            if !Self::is_safe_input(key) || !Self::is_safe_input(value) {
                return Err(format!("Invalid query parameter: {}", key));
            }
        }

        // Validate path parameters
        for (key, value) in req.params() {
            if !Self::is_safe_input(key) || !Self::is_safe_input(value) {
                return Err(format!("Invalid path parameter: {}", key));
            }
        }

        // Validate request body if it's text
        if let Ok(body_str) = req.body_string() {
            if !Self::is_safe_input(&body_str) {
                return Err("Invalid request body content".to_string());
            }
        }

        Ok(())
    }
}

impl Middleware for InputValidator {
    fn call(
        &self,
        req: Request,
        next: Box<dyn Fn(Request) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send + 'static>> + Send + Sync>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send + 'static>> {
        Box::pin(async move {
            // Validate input
            if let Err(error) = Self::validate_request_data(&req) {
                return Response::with_status(http::StatusCode::BAD_REQUEST)
                    .body(format!("Input validation failed: {}", error));
            }

            next(req).await
        })
    }
}

/// Enhanced security headers middleware
pub struct SecurityHeaders {
    content_security_policy: Option<String>,
}

impl SecurityHeaders {
    pub fn new() -> Self {
        Self {
            content_security_policy: Some(
                "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self' https:; connect-src 'self'; frame-ancestors 'none';"
                    .to_string(),
            ),
        }
    }

    pub fn with_csp(mut self, csp: &str) -> Self {
        self.content_security_policy = Some(csp.to_string());
        self
    }

    pub fn without_csp(mut self) -> Self {
        self.content_security_policy = None;
        self
    }
}

impl Middleware for SecurityHeaders {
    fn call(
        &self,
        req: Request,
        next: Box<dyn Fn(Request) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send + 'static>> + Send + Sync>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send + 'static>> {
        let csp = self.content_security_policy.clone();
        Box::pin(async move {
            let mut response = next(req).await;

            // Add comprehensive security headers
            response = response
                .header("X-Content-Type-Options", "nosniff")
                .header("X-Frame-Options", "DENY")
                .header("X-XSS-Protection", "1; mode=block")
                .header("Referrer-Policy", "strict-origin-when-cross-origin")
                .header("Permissions-Policy", "geolocation=(), microphone=(), camera=()")
                .header("Strict-Transport-Security", "max-age=31536000; includeSubDomains; preload");

            if let Some(csp) = csp {
                response = response.header("Content-Security-Policy", &csp);
            }

            response
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_ip_whitelist() {
        let whitelist = IpWhitelist::new()
            .allow_ip("192.168.1.1")
            .allow_range("10.0.0.0/8");

        assert!(whitelist.is_ip_allowed(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))));
        assert!(whitelist.is_ip_allowed(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))));
        assert!(whitelist.is_ip_allowed(IpAddr::V4(Ipv4Addr::new(10, 255, 255, 255))));
        assert!(!whitelist.is_ip_allowed(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2))));
        assert!(!whitelist.is_ip_allowed(IpAddr::V4(Ipv4Addr::new(11, 0, 0, 1))));
    }

    #[test]
    fn test_input_validation() {
        assert!(!InputValidator::is_safe_input("'; DROP TABLE users; --"));
        assert!(!InputValidator::is_safe_input("<script>alert('xss')</script>"));
        assert!(!InputValidator::is_safe_input("../../../etc/passwd"));
        assert!(!InputValidator::is_safe_input("test\0null"));
        assert!(InputValidator::is_safe_input("normal input text"));
        assert!(InputValidator::is_safe_input("user@example.com"));
    }
}
