//! Header extraction
//!
//! Extract headers from the HTTP request.

use std::pin::Pin;
use std::future::Future;
use crate::{Request, extractors::{FromRequestParts, ExtractionError}};
use http::HeaderMap;

/// Extract headers from the request
///
/// # Example
///
/// ```rust,no_run
/// use torch_web::extractors::Headers;
/// use http::HeaderMap;
///
/// async fn handler(Headers(headers): Headers) {
///     if let Some(auth) = headers.get("authorization") {
///         // Handle authorization header
///     }
/// }
/// ```
pub struct Headers(pub HeaderMap);

impl FromRequestParts for Headers {
    type Error = ExtractionError;

    fn from_request_parts(
        req: &mut Request,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Self::Error>> + Send + 'static>> {
        let headers = req.headers().clone();
        
        Box::pin(async move {
            Ok(Headers(headers))
        })
    }
}

/// Extract a specific header by name (simplified version without const generics)
///
/// # Example
///
/// ```rust,no_run
/// use torch_web::extractors::HeaderExtractor;
///
/// async fn handler(header: HeaderExtractor) {
///     // Use header.get() to access the header value
/// }
/// ```
pub struct HeaderExtractor {
    name: String,
    value: Option<String>,
}

impl HeaderExtractor {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            value: None,
        }
    }

    pub fn get(&self) -> Option<&str> {
        self.value.as_deref()
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl FromRequestParts for HeaderExtractor {
    type Error = ExtractionError;

    fn from_request_parts(
        req: &mut Request,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Self::Error>> + Send + 'static>> {
        let headers = req.headers().clone();

        Box::pin(async move {
            // This is a placeholder - in practice, you'd need to specify which header to extract
            // For now, we'll extract the first header as an example
            let (name, value) = if let Some((name, value)) = headers.iter().next() {
                (name.as_str().to_string(), value.to_str().unwrap_or("").to_string())
            } else {
                ("".to_string(), "".to_string())
            };

            Ok(HeaderExtractor {
                name,
                value: if value.is_empty() { None } else { Some(value) },
            })
        })
    }
}

// Note: Const generic header extractors removed due to current Rust limitations
// In a production implementation, you'd use macros or a different approach

/// Extract the User-Agent header
///
/// # Example
///
/// ```rust,no_run
/// use torch_web::extractors::UserAgent;
///
/// async fn handler(UserAgent(user_agent): UserAgent) {
///     // user_agent contains the User-Agent header value
/// }
/// ```
pub struct UserAgent(pub Option<String>);

impl FromRequestParts for UserAgent {
    type Error = ExtractionError;

    fn from_request_parts(
        req: &mut Request,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Self::Error>> + Send + 'static>> {
        let user_agent = req.headers()
            .get("user-agent")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        
        Box::pin(async move {
            Ok(UserAgent(user_agent))
        })
    }
}

/// Extract the Authorization header
///
/// # Example
///
/// ```rust,no_run
/// use torch_web::extractors::Authorization;
///
/// async fn handler(Authorization(auth): Authorization) {
///     match auth {
///         Some(token) => {
///             // Handle authorization
///         }
///         None => {
///             // No authorization provided
///         }
///     }
/// }
/// ```
pub struct Authorization(pub Option<String>);

impl FromRequestParts for Authorization {
    type Error = ExtractionError;

    fn from_request_parts(
        req: &mut Request,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Self::Error>> + Send + 'static>> {
        let auth = req.headers()
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        
        Box::pin(async move {
            Ok(Authorization(auth))
        })
    }
}

/// Extract the Content-Type header
///
/// # Example
///
/// ```rust,no_run
/// use torch_web::extractors::ContentType;
///
/// async fn handler(ContentType(content_type): ContentType) {
///     match content_type.as_deref() {
///         Some("application/json") => {
///             // Handle JSON content
///         }
///         Some("application/xml") => {
///             // Handle XML content
///         }
///         _ => {
///             // Handle other or missing content type
///         }
///     }
/// }
/// ```
pub struct ContentType(pub Option<String>);

impl FromRequestParts for ContentType {
    type Error = ExtractionError;

    fn from_request_parts(
        req: &mut Request,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Self::Error>> + Send + 'static>> {
        let content_type = req.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        
        Box::pin(async move {
            Ok(ContentType(content_type))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Request;

    #[tokio::test]
    async fn test_headers_extraction() {
        let mut req = Request::new();
        req.headers_mut().insert("x-custom", "test-value".parse().unwrap());
        
        let result = Headers::from_request_parts(&mut req).await;
        assert!(result.is_ok());
        
        let Headers(headers) = result.unwrap();
        assert_eq!(headers.get("x-custom").unwrap(), "test-value");
    }

    #[tokio::test]
    async fn test_user_agent_extraction() {
        let mut req = Request::new();
        req.headers_mut().insert("user-agent", "Mozilla/5.0".parse().unwrap());
        
        let result = UserAgent::from_request_parts(&mut req).await;
        assert!(result.is_ok());
        
        let UserAgent(user_agent) = result.unwrap();
        assert_eq!(user_agent, Some("Mozilla/5.0".to_string()));
    }

    #[tokio::test]
    async fn test_missing_user_agent() {
        let mut req = Request::new();
        
        let result = UserAgent::from_request_parts(&mut req).await;
        assert!(result.is_ok());
        
        let UserAgent(user_agent) = result.unwrap();
        assert_eq!(user_agent, None);
    }

    // Note: RequiredHeader tests removed due to const generic limitations
    // In a production implementation, you'd use a different approach for typed headers
}
