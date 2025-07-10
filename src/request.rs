use std::collections::HashMap;
use std::any::{Any, TypeId};
use http::{HeaderMap, Method, Uri, Version};
use http_body_util::BodyExt;
use hyper::body::Incoming;

/// HTTP Request wrapper that provides convenient access to request data
#[derive(Debug)]
pub struct Request {
    method: Method,
    uri: Uri,
    version: Version,
    headers: HeaderMap,
    body: Vec<u8>,
    params: HashMap<String, String>,
    query: HashMap<String, String>,
    extensions: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl Request {
    /// Create a simple empty request (for internal use)
    pub fn new() -> Self {
        Self {
            method: Method::GET,
            uri: "/".parse().unwrap(),
            version: Version::HTTP_11,
            headers: HeaderMap::new(),
            body: Vec::new(),
            params: HashMap::new(),
            query: HashMap::new(),
            extensions: HashMap::new(),
        }
    }

    /// Create a new Request from hyper's request parts and body
    pub async fn from_hyper(
        parts: http::request::Parts,
        body: Incoming,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let body_bytes = body.collect().await?.to_bytes().to_vec();
        
        let query = Self::parse_query_string(parts.uri.query().unwrap_or(""));
        
        Ok(Request {
            method: parts.method,
            uri: parts.uri,
            version: parts.version,
            headers: parts.headers,
            body: body_bytes,
            params: HashMap::new(),
            query,
            extensions: HashMap::new(),
        })
    }

    /// Get the HTTP method
    pub fn method(&self) -> &Method {
        &self.method
    }

    /// Get the URI
    pub fn uri(&self) -> &Uri {
        &self.uri
    }

    /// Get the path from the URI
    pub fn path(&self) -> &str {
        self.uri.path()
    }

    /// Get the HTTP version
    pub fn version(&self) -> Version {
        self.version
    }

    /// Get the headers
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// Get a specific header value
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name)?.to_str().ok()
    }

    /// Get the request body as bytes
    pub fn body(&self) -> &[u8] {
        &self.body
    }

    /// Get the request body as a string
    pub fn body_string(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.body.clone())
    }

    /// Parse the request body as JSON (requires "json" feature)
    #[cfg(feature = "json")]
    pub async fn json<T>(&self) -> Result<T, serde_json::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        serde_json::from_slice(&self.body)
    }

    /// Get a path parameter by name
    pub fn param(&self, name: &str) -> Option<&str> {
        self.params.get(name).map(|s| s.as_str())
    }

    /// Get all path parameters
    pub fn params(&self) -> &HashMap<String, String> {
        &self.params
    }

    /// Set a path parameter (used internally by the router)
    pub(crate) fn set_param(&mut self, name: String, value: String) {
        self.params.insert(name, value);
    }

    /// Get a reference to the request extensions
    pub fn extensions(&self) -> &HashMap<TypeId, Box<dyn Any + Send + Sync>> {
        &self.extensions
    }

    /// Get a mutable reference to the request extensions
    pub fn extensions_mut(&mut self) -> &mut HashMap<TypeId, Box<dyn Any + Send + Sync>> {
        &mut self.extensions
    }

    /// Insert a value into the request extensions
    pub fn insert_extension<T: Send + Sync + 'static>(&mut self, value: T) {
        self.extensions.insert(TypeId::of::<T>(), Box::new(value));
    }

    /// Get a value from the request extensions
    pub fn get_extension<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.extensions
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref())
    }

    /// Get a query parameter by name
    pub fn query(&self, name: &str) -> Option<&str> {
        self.query.get(name).map(|s| s.as_str())
    }

    /// Get all query parameters
    pub fn query_params(&self) -> &HashMap<String, String> {
        &self.query
    }

    /// Parse query string into a HashMap
    fn parse_query_string(query: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        for pair in query.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                let key = urlencoding::decode(key).unwrap_or_else(|_| key.into()).into_owned();
                let value = urlencoding::decode(value).unwrap_or_else(|_| value.into()).into_owned();
                params.insert(key, value);
            } else if !pair.is_empty() {
                let key = urlencoding::decode(pair).unwrap_or_else(|_| pair.into()).into_owned();
                params.insert(key, String::new());
            }
        }
        
        params
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Imports for potential future test use

    #[test]
    fn test_parse_query_string() {
        let query = "name=John&age=30&city=New%20York";
        let params = Request::parse_query_string(query);
        
        assert_eq!(params.get("name"), Some(&"John".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
        assert_eq!(params.get("city"), Some(&"New York".to_string()));
    }

    #[test]
    fn test_parse_empty_query_string() {
        let params = Request::parse_query_string("");
        assert!(params.is_empty());
    }
}
