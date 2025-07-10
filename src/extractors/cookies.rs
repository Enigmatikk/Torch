//! Cookie extraction
//!
//! Extract and parse HTTP cookies from requests.

use std::pin::Pin;
use std::future::Future;
use std::collections::HashMap;
use crate::{Request, extractors::{FromRequestParts, ExtractionError}};

/// Extract cookies from the request
///
/// # Example
///
/// ```rust,no_run
/// use torch_web::extractors::Cookies;
/// use std::collections::HashMap;
///
/// async fn handler(Cookies(cookies): Cookies) {
///     if let Some(session_id) = cookies.get("session_id") {
///         // Handle session
///     }
/// }
/// ```
pub struct Cookies(pub HashMap<String, String>);

impl FromRequestParts for Cookies {
    type Error = ExtractionError;

    fn from_request_parts(
        req: &mut Request,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Self::Error>> + Send + 'static>> {
        let cookie_header = req.headers()
            .get("cookie")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        
        Box::pin(async move {
            let cookies = parse_cookies(&cookie_header)?;
            Ok(Cookies(cookies))
        })
    }
}

/// Extract a specific cookie by name using a helper function
///
/// # Example
///
/// ```rust,no_run
/// use torch_web::extractors::{Cookies, get_cookie};
///
/// async fn handler(Cookies(cookies): Cookies) {
///     match get_cookie(&cookies, "session_id") {
///         Some(id) => {
///             // Handle session
///         }
///         None => {
///             // No session cookie
///         }
///     }
/// }
/// ```

/// Helper function to get a specific cookie by name
pub fn get_cookie<'a>(cookies: &'a std::collections::HashMap<String, String>, name: &str) -> Option<&'a String> {
    cookies.get(name)
}

/// Helper function to get a required cookie by name
pub fn get_required_cookie<'a>(cookies: &'a std::collections::HashMap<String, String>, name: &str) -> Result<&'a String, ExtractionError> {
    cookies.get(name).ok_or_else(|| ExtractionError::MissingHeader(
        format!("Required cookie '{}' not found", name)
    ))
}

/// Convenience extractors for common cookies
pub struct SessionCookie(pub Option<String>);

impl FromRequestParts for SessionCookie {
    type Error = ExtractionError;

    fn from_request_parts(
        req: &mut Request,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Self::Error>> + Send + 'static>> {
        let cookie_header = req.headers()
            .get("cookie")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        
        Box::pin(async move {
            let cookies = parse_cookies(&cookie_header)?;
            let session = cookies.get("session_id")
                .or_else(|| cookies.get("sessionid"))
                .or_else(|| cookies.get("SESSIONID"))
                .cloned();
            Ok(SessionCookie(session))
        })
    }
}

/// Parse cookie header string into a HashMap
fn parse_cookies(cookie_header: &str) -> Result<HashMap<String, String>, ExtractionError> {
    let mut cookies = HashMap::new();
    
    if cookie_header.is_empty() {
        return Ok(cookies);
    }

    for cookie_pair in cookie_header.split(';') {
        let cookie_pair = cookie_pair.trim();
        if let Some((name, value)) = cookie_pair.split_once('=') {
            let name = name.trim().to_string();
            let value = value.trim().to_string();
            
            // Basic URL decoding for cookie values
            let decoded_value = urlencoding::decode(&value)
                .map_err(|e| ExtractionError::InvalidHeader(format!("Invalid cookie encoding: {}", e)))?
                .into_owned();
            
            cookies.insert(name, decoded_value);
        } else {
            // Handle cookies without values (rare but possible)
            let name = cookie_pair.to_string();
            cookies.insert(name, String::new());
        }
    }

    Ok(cookies)
}

/// Cookie builder for creating Set-Cookie headers
#[derive(Debug, Clone)]
pub struct CookieBuilder {
    name: String,
    value: String,
    domain: Option<String>,
    path: Option<String>,
    max_age: Option<i64>,
    expires: Option<String>,
    secure: bool,
    http_only: bool,
    same_site: Option<SameSite>,
}

#[derive(Debug, Clone)]
pub enum SameSite {
    Strict,
    Lax,
    None,
}

impl CookieBuilder {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            domain: None,
            path: None,
            max_age: None,
            expires: None,
            secure: false,
            http_only: false,
            same_site: None,
        }
    }

    pub fn domain(mut self, domain: impl Into<String>) -> Self {
        self.domain = Some(domain.into());
        self
    }

    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    pub fn max_age(mut self, seconds: i64) -> Self {
        self.max_age = Some(seconds);
        self
    }

    pub fn expires(mut self, expires: impl Into<String>) -> Self {
        self.expires = Some(expires.into());
        self
    }

    pub fn secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }

    pub fn http_only(mut self, http_only: bool) -> Self {
        self.http_only = http_only;
        self
    }

    pub fn same_site(mut self, same_site: SameSite) -> Self {
        self.same_site = Some(same_site);
        self
    }

    pub fn build(self) -> String {
        let mut cookie = format!("{}={}", self.name, urlencoding::encode(&self.value));

        if let Some(domain) = self.domain {
            cookie.push_str(&format!("; Domain={}", domain));
        }

        if let Some(path) = self.path {
            cookie.push_str(&format!("; Path={}", path));
        }

        if let Some(max_age) = self.max_age {
            cookie.push_str(&format!("; Max-Age={}", max_age));
        }

        if let Some(expires) = self.expires {
            cookie.push_str(&format!("; Expires={}", expires));
        }

        if self.secure {
            cookie.push_str("; Secure");
        }

        if self.http_only {
            cookie.push_str("; HttpOnly");
        }

        if let Some(same_site) = self.same_site {
            let same_site_str = match same_site {
                SameSite::Strict => "Strict",
                SameSite::Lax => "Lax",
                SameSite::None => "None",
            };
            cookie.push_str(&format!("; SameSite={}", same_site_str));
        }

        cookie
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_cookies() {
        let result = parse_cookies("");
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_parse_single_cookie() {
        let result = parse_cookies("session_id=abc123");
        let cookies = result.unwrap();
        assert_eq!(cookies.get("session_id"), Some(&"abc123".to_string()));
    }

    #[test]
    fn test_parse_multiple_cookies() {
        let result = parse_cookies("session_id=abc123; user_id=456; theme=dark");
        let cookies = result.unwrap();
        assert_eq!(cookies.get("session_id"), Some(&"abc123".to_string()));
        assert_eq!(cookies.get("user_id"), Some(&"456".to_string()));
        assert_eq!(cookies.get("theme"), Some(&"dark".to_string()));
    }

    #[test]
    fn test_parse_cookies_with_spaces() {
        let result = parse_cookies(" session_id = abc123 ; user_id = 456 ");
        let cookies = result.unwrap();
        assert_eq!(cookies.get("session_id"), Some(&"abc123".to_string()));
        assert_eq!(cookies.get("user_id"), Some(&"456".to_string()));
    }

    #[test]
    fn test_cookie_builder() {
        let cookie = CookieBuilder::new("session_id", "abc123")
            .domain("example.com")
            .path("/")
            .max_age(3600)
            .secure(true)
            .http_only(true)
            .same_site(SameSite::Lax)
            .build();

        assert!(cookie.contains("session_id=abc123"));
        assert!(cookie.contains("Domain=example.com"));
        assert!(cookie.contains("Path=/"));
        assert!(cookie.contains("Max-Age=3600"));
        assert!(cookie.contains("Secure"));
        assert!(cookie.contains("HttpOnly"));
        assert!(cookie.contains("SameSite=Lax"));
    }
}
