use http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use http_body_util::Full;
use hyper::body::Bytes;

/// HTTP Response builder that provides a fluent API for creating responses
#[derive(Debug)]
pub struct Response {
    status: StatusCode,
    headers: HeaderMap,
    body: Vec<u8>,
}

impl Response {
    /// Create a new response with 200 OK status
    pub fn new() -> Self {
        Self {
            status: StatusCode::OK,
            headers: HeaderMap::new(),
            body: Vec::new(),
        }
    }

    /// Create a response with 200 OK status (alias for new)
    pub fn ok() -> Self {
        Self::new()
    }

    /// Create a response with a specific status code
    pub fn with_status(status: StatusCode) -> Self {
        Self {
            status,
            headers: HeaderMap::new(),
            body: Vec::new(),
        }
    }

    /// Create a 404 Not Found response
    pub fn not_found() -> Self {
        Self::with_status(StatusCode::NOT_FOUND)
            .body("Not Found")
    }

    /// Create a 500 Internal Server Error response
    pub fn internal_error() -> Self {
        Self::with_status(StatusCode::INTERNAL_SERVER_ERROR)
            .body("Internal Server Error")
    }

    /// Create a 400 Bad Request response
    pub fn bad_request() -> Self {
        Self::with_status(StatusCode::BAD_REQUEST)
            .body("Bad Request")
    }

    /// Create a 201 Created response
    pub fn created() -> Self {
        Self::with_status(StatusCode::CREATED)
    }

    /// Create a 204 No Content response
    pub fn no_content() -> Self {
        Self::with_status(StatusCode::NO_CONTENT)
    }

    /// Create a 401 Unauthorized response
    pub fn unauthorized() -> Self {
        Self::with_status(StatusCode::UNAUTHORIZED)
            .body("Unauthorized")
    }

    /// Create a 403 Forbidden response
    pub fn forbidden() -> Self {
        Self::with_status(StatusCode::FORBIDDEN)
            .body("Forbidden")
    }

    /// Create a 422 Unprocessable Entity response
    pub fn unprocessable_entity() -> Self {
        Self::with_status(StatusCode::UNPROCESSABLE_ENTITY)
            .body("Unprocessable Entity")
    }

    /// Create a 429 Too Many Requests response
    pub fn too_many_requests() -> Self {
        Self::with_status(StatusCode::TOO_MANY_REQUESTS)
            .body("Too Many Requests")
    }

    /// Set the status code
    pub fn status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    /// Set the response body from a string
    pub fn body<T: Into<Vec<u8>>>(mut self, body: T) -> Self {
        self.body = body.into();
        self
    }

    /// Set the response body from bytes
    pub fn body_from_bytes(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self
    }

    /// Set a header
    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: TryInto<HeaderName>,
        V: TryInto<HeaderValue>,
        K::Error: std::fmt::Debug,
        V::Error: std::fmt::Debug,
    {
        let key = key.try_into().expect("Invalid header name");
        let value = value.try_into().expect("Invalid header value");
        self.headers.insert(key, value);
        self
    }

    /// Set the Content-Type header
    pub fn content_type(self, content_type: &str) -> Self {
        self.header("content-type", content_type)
    }

    /// Set response as JSON and serialize the value (requires "json" feature)
    #[cfg(feature = "json")]
    pub fn json<T: serde::Serialize>(self, value: &T) -> Result<Self, serde_json::Error> {
        let json_string = serde_json::to_string(value)?;
        Ok(self
            .content_type("application/json")
            .body(json_string))
    }

    /// Set response as HTML
    pub fn html<T: Into<Vec<u8>>>(self, html: T) -> Self {
        self.content_type("text/html; charset=utf-8")
            .body(html)
    }

    /// Set response as plain text
    pub fn text<T: Into<Vec<u8>>>(self, text: T) -> Self {
        self.content_type("text/plain; charset=utf-8")
            .body(text)
    }

    /// Redirect to another URL
    pub fn redirect(status: StatusCode, location: &str) -> Self {
        Self::with_status(status)
            .header("location", location)
    }

    /// Redirect with 302 Found status
    pub fn redirect_found(location: &str) -> Self {
        Self::redirect(StatusCode::FOUND, location)
    }

    /// Redirect with 301 Moved Permanently status
    pub fn redirect_permanent(location: &str) -> Self {
        Self::redirect(StatusCode::MOVED_PERMANENTLY, location)
    }

    /// Get the status code
    pub fn status_code(&self) -> StatusCode {
        self.status
    }

    /// Get a mutable reference to the status code
    pub fn status_code_mut(&mut self) -> &mut StatusCode {
        &mut self.status
    }

    /// Get the headers
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// Get the body as bytes
    pub fn body_data(&self) -> &[u8] {
        &self.body
    }

    /// Get the body as bytes (alias for body_data)
    pub fn body_bytes(&self) -> &[u8] {
        &self.body
    }

    /// Convert to hyper Response
    pub fn into_hyper_response(self) -> hyper::Response<Full<Bytes>> {
        let mut response = hyper::Response::builder()
            .status(self.status);

        // Add headers
        for (key, value) in self.headers.iter() {
            response = response.header(key, value);
        }

        response
            .body(Full::new(Bytes::from(self.body)))
            .expect("Failed to build response")
    }
}

impl Default for Response {
    fn default() -> Self {
        Self::new()
    }
}

impl From<&str> for Response {
    fn from(body: &str) -> Self {
        Response::ok().body(body)
    }
}

impl From<String> for Response {
    fn from(body: String) -> Self {
        Response::ok().body(body)
    }
}

impl From<StatusCode> for Response {
    fn from(status: StatusCode) -> Self {
        Response::with_status(status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_creation() {
        let response = Response::ok().body("Hello, World!");
        assert_eq!(response.status_code(), StatusCode::OK);
        assert_eq!(response.body_data(), b"Hello, World!");
    }

    #[test]
    fn test_response_with_headers() {
        let response = Response::ok()
            .header("x-custom", "value")
            .content_type("text/plain")
            .body("test");
        
        assert_eq!(response.headers().get("x-custom").unwrap(), "value");
        assert_eq!(response.headers().get("content-type").unwrap(), "text/plain");
    }

    #[test]
    fn test_redirect() {
        let response = Response::redirect_found("/new-path");
        assert_eq!(response.status_code(), StatusCode::FOUND);
        assert_eq!(response.headers().get("location").unwrap(), "/new-path");
    }

    #[cfg(feature = "json")]
    #[test]
    fn test_json_response() {
        use serde_json::json;
        
        let data = json!({"message": "Hello, World!"});
        let response = Response::ok().json(&data).unwrap();
        
        assert_eq!(response.headers().get("content-type").unwrap(), "application/json");
        assert_eq!(response.body_data(), br#"{"message":"Hello, World!"}"#);
    }
}
