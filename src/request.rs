//! # HTTP Request Handling
//!
//! This module provides the [`Request`] struct, which wraps HTTP requests and provides
//! convenient methods for accessing request data like headers, body, path parameters,
//! and query parameters.

use std::collections::HashMap;
use std::any::{Any, TypeId};
use std::sync::Arc;
use http::{HeaderMap, Method, Uri, Version};
use http_body_util::BodyExt;
use hyper::body::Incoming;
use crate::extractors::state::StateMap;

/// HTTP request wrapper that provides convenient access to request data.
///
/// The `Request` struct encapsulates all the information about an incoming HTTP request,
/// including the method, URI, headers, body, and extracted path parameters. It provides
/// a high-level API for accessing this data in handlers.
///
/// # Examples
///
/// ## Basic Usage in Handlers
///
/// ```rust
/// use torch_web::{App, Request, Response};
///
/// let app = App::new()
///     .get("/", |req: Request| async move {
///         println!("Method: {}", req.method());
///         println!("Path: {}", req.path());
///         Response::ok().body("Hello!")
///     });
/// ```
///
/// ## Accessing Headers
///
/// ```rust
/// use torch_web::{App, Request, Response};
///
/// let app = App::new()
///     .post("/api/data", |req: Request| async move {
///         if let Some(content_type) = req.header("content-type") {
///             println!("Content-Type: {}", content_type);
///         }
///
///         if let Some(auth) = req.header("authorization") {
///             println!("Authorization: {}", auth);
///         }
///
///         Response::ok().body("Received")
///     });
/// ```
///
/// ## Working with Request Body
///
/// ```rust
/// use torch_web::{App, Request, Response};
///
/// let app = App::new()
///     .post("/upload", |req: Request| async move {
///         let body_bytes = req.body();
///         println!("Received {} bytes", body_bytes.len());
///
///         if let Ok(body_text) = req.body_string() {
///             println!("Body: {}", body_text);
///         }
///
///         Response::ok().body("Upload complete")
///     });
/// ```
///
/// ## Path Parameters
///
/// ```rust
/// use torch_web::{App, Request, Response};
///
/// let app = App::new()
///     .get("/users/:id/posts/:post_id", |req: Request| async move {
///         let user_id = req.param("id").unwrap_or("unknown");
///         let post_id = req.param("post_id").unwrap_or("unknown");
///
///         Response::ok().body(format!("User {} Post {}", user_id, post_id))
///     });
/// ```
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
    /// Creates a new empty request with default values.
    ///
    /// This is primarily used for testing and internal purposes. In normal operation,
    /// requests are created from incoming HTTP requests via `from_hyper`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torch_web::Request;
    ///
    /// let req = Request::new();
    /// assert_eq!(req.path(), "/");
    /// assert_eq!(req.method().as_str(), "GET");
    /// ```
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

    /// Creates a new Request from Hyper's request parts and body.
    ///
    /// This is an internal method used by the framework to convert incoming
    /// Hyper requests into Torch Request objects. It reads the entire request
    /// body into memory and parses query parameters.
    ///
    /// # Parameters
    ///
    /// * `parts` - The HTTP request parts (method, URI, headers, etc.)
    /// * `body` - The request body stream
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the constructed `Request` or an error if
    /// the body cannot be read or the request is malformed.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use torch_web::Request;
    /// use hyper::body::Incoming;
    /// use http::request::Parts;
    ///
    /// async fn handle_hyper_request(parts: Parts, body: Incoming) {
    ///     match Request::from_hyper(parts, body).await {
    ///         Ok(req) => {
    ///             println!("Received request to {}", req.path());
    ///         }
    ///         Err(e) => {
    ///             eprintln!("Failed to parse request: {}", e);
    ///         }
    ///     }
    /// }
    /// ```
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

    /// Returns the HTTP method of the request.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torch_web::{App, Request, Response, Method};
    ///
    /// let app = App::new()
    ///     .route(Method::POST, "/api/data", |req: Request| async move {
    ///         match req.method() {
    ///             &Method::POST => Response::ok().body("POST request"),
    ///             &Method::GET => Response::ok().body("GET request"),
    ///             _ => Response::method_not_allowed().body("Method not allowed"),
    ///         }
    ///     });
    /// ```
    pub fn method(&self) -> &Method {
        &self.method
    }

    /// Returns the complete URI of the request.
    ///
    /// This includes the path, query string, and fragment (if present).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torch_web::{App, Request, Response};
    ///
    /// let app = App::new()
    ///     .get("/debug", |req: Request| async move {
    ///         let uri = req.uri();
    ///         Response::ok().body(format!("Full URI: {}", uri))
    ///     });
    /// ```
    pub fn uri(&self) -> &Uri {
        &self.uri
    }

    /// Returns the path portion of the request URI.
    ///
    /// This excludes the query string and fragment, returning only the path component.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torch_web::{App, Request, Response};
    ///
    /// let app = App::new()
    ///     .get("/users/:id", |req: Request| async move {
    ///         println!("Request path: {}", req.path()); // "/users/123"
    ///         Response::ok().body("User page")
    ///     });
    /// ```
    pub fn path(&self) -> &str {
        self.uri.path()
    }

    /// Returns the HTTP version of the request.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torch_web::{App, Request, Response};
    /// use http::Version;
    ///
    /// let app = App::new()
    ///     .get("/version", |req: Request| async move {
    ///         let version = match req.version() {
    ///             Version::HTTP_09 => "HTTP/0.9",
    ///             Version::HTTP_10 => "HTTP/1.0",
    ///             Version::HTTP_11 => "HTTP/1.1",
    ///             Version::HTTP_2 => "HTTP/2.0",
    ///             Version::HTTP_3 => "HTTP/3.0",
    ///             _ => "Unknown",
    ///         };
    ///         Response::ok().body(format!("HTTP Version: {}", version))
    ///     });
    /// ```
    pub fn version(&self) -> Version {
        self.version
    }

    /// Returns a reference to the request headers.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torch_web::{App, Request, Response};
    ///
    /// let app = App::new()
    ///     .post("/api/data", |req: Request| async move {
    ///         let headers = req.headers();
    ///
    ///         for (name, value) in headers.iter() {
    ///             println!("{}: {:?}", name, value);
    ///         }
    ///
    ///         Response::ok().body("Headers logged")
    ///     });
    /// ```
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// Returns the value of a specific header.
    ///
    /// This is a convenience method that looks up a header by name and converts
    /// it to a string. Returns `None` if the header doesn't exist or contains
    /// invalid UTF-8.
    ///
    /// # Parameters
    ///
    /// * `name` - The header name to look up (case-insensitive)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torch_web::{App, Request, Response};
    ///
    /// let app = App::new()
    ///     .post("/api/upload", |req: Request| async move {
    ///         if let Some(content_type) = req.header("content-type") {
    ///             println!("Content-Type: {}", content_type);
    ///
    ///             if content_type.starts_with("application/json") {
    ///                 return Response::ok().body("JSON data received");
    ///             }
    ///         }
    ///
    ///         Response::bad_request().body("Content-Type required")
    ///     });
    /// ```
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name)?.to_str().ok()
    }

    /// Returns the request body as a byte slice.
    ///
    /// The entire request body is read into memory when the request is created,
    /// so this method provides immediate access to the raw bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torch_web::{App, Request, Response};
    ///
    /// let app = App::new()
    ///     .post("/upload", |req: Request| async move {
    ///         let body_bytes = req.body();
    ///         println!("Received {} bytes", body_bytes.len());
    ///
    ///         // Process binary data
    ///         if body_bytes.starts_with(b"PNG") {
    ///             Response::ok().body("PNG image received")
    ///         } else {
    ///             Response::ok().body("Data received")
    ///         }
    ///     });
    /// ```
    pub fn body(&self) -> &[u8] {
        &self.body
    }

    /// Returns the request body as a UTF-8 string.
    ///
    /// This method attempts to convert the request body bytes into a valid UTF-8 string.
    /// Returns an error if the body contains invalid UTF-8 sequences.
    ///
    /// # Returns
    ///
    /// Returns `Ok(String)` if the body is valid UTF-8, or `Err(FromUtf8Error)` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torch_web::{App, Request, Response};
    ///
    /// let app = App::new()
    ///     .post("/text", |req: Request| async move {
    ///         match req.body_string() {
    ///             Ok(text) => {
    ///                 println!("Received text: {}", text);
    ///                 Response::ok().body(format!("Echo: {}", text))
    ///             }
    ///             Err(_) => {
    ///                 Response::bad_request().body("Invalid UTF-8 in request body")
    ///             }
    ///         }
    ///     });
    /// ```
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

    /// Get all path parameters (for extractors)
    pub fn path_params(&self) -> &HashMap<String, String> {
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

    /// Get the raw query string
    pub fn query_string(&self) -> Option<&str> {
        self.uri.query()
    }

    /// Get the request body as bytes (for extractors)
    pub fn body_bytes(&self) -> &[u8] {
        &self.body
    }

    /// Set the request body (for testing)
    #[cfg(test)]
    pub fn set_body(&mut self, body: Vec<u8>) {
        self.body = body;
    }

    /// Get mutable access to headers (for extractors)
    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        &mut self.headers
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

/// Implementation of RequestStateExt for Request
impl crate::extractors::state::RequestStateExt for Request {
    fn get_state(&self, type_id: TypeId) -> Option<&Arc<dyn Any + Send + Sync>> {
        // Check if we have a StateMap stored in extensions
        if let Some(state_map_any) = self.extensions.get(&TypeId::of::<StateMap>()) {
            if let Some(state_map) = state_map_any.downcast_ref::<StateMap>() {
                return state_map.get_by_type_id(type_id);
            }
        }
        None
    }

    fn set_state_map(&mut self, state_map: StateMap) {
        self.extensions.insert(TypeId::of::<StateMap>(), Box::new(state_map));
    }

    fn state_map(&self) -> Option<&StateMap> {
        self.extensions
            .get(&TypeId::of::<StateMap>())
            .and_then(|state_map_any| state_map_any.downcast_ref::<StateMap>())
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
