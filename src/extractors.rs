//! # Request Extractors
//!
//! Extractors provide a powerful, type-safe way to parse and extract data from HTTP requests.
//! They allow you to declaratively specify what data you need from a request, and the framework
//! will automatically parse and validate it for you.
//!
//! ## Available Extractors
//!
//! - **[`Path<T>`]** - Extract path parameters from the URL
//! - **[`Query<T>`]** - Extract and parse query string parameters
//! - **[`Json<T>`]** - Parse JSON request bodies (requires `json` feature)
//! - **[`Form<T>`]** - Parse form-encoded request bodies
//! - **[`Headers`]** - Access request headers with convenience methods
//! - **[`State<T>`]** - Access application state
//! - **[`Cookies`]** - Access and manage HTTP cookies
//!
//! ## Basic Usage
//!
//! Extractors are used as function parameters in your handlers. The framework automatically
//! calls the appropriate extractor based on the parameter type:
//!
//! ```rust
//! use torch_web::{App, Request, Response, extractors::*};
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct CreateUser {
//!     name: String,
//!     email: String,
//! }
//!
//! #[derive(Deserialize)]
//! struct Pagination {
//!     page: Option<u32>,
//!     limit: Option<u32>,
//! }
//!
//! let app = App::new()
//!     // Extract path parameter
//!     .get("/users/:id", |Path(id): Path<u32>| async move {
//!         Response::ok().body(format!("User ID: {}", id))
//!     })
//!
//!     // Extract query parameters
//!     .get("/users", |Query(pagination): Query<Pagination>| async move {
//!         let page = pagination.page.unwrap_or(1);
//!         let limit = pagination.limit.unwrap_or(10);
//!         Response::ok().body(format!("Page {} with {} items", page, limit))
//!     })
//!
//!     // Extract JSON body
//!     .post("/users", |Json(user): Json<CreateUser>| async move {
//!         Response::created().json(&user)
//!     })
//!
//!     // Combine multiple extractors
//!     .put("/users/:id", |
//!         Path(id): Path<u32>,
//!         Json(user): Json<CreateUser>,
//!         headers: Headers,
//!     | async move {
//!         if let Some(auth) = headers.authorization() {
//!             Response::ok().body(format!("Updated user {} with auth", id))
//!         } else {
//!             Response::unauthorized().body("Authentication required")
//!         }
//!     });
//! ```
//!
//! ## Error Handling
//!
//! Extractors automatically handle parsing errors and return appropriate HTTP error responses:
//!
//! - **Path extraction errors** → 404 Not Found
//! - **Query parsing errors** → 400 Bad Request
//! - **JSON parsing errors** → 400 Bad Request with error details
//! - **Missing required data** → 400 Bad Request
//!
//! ## Custom Extractors
//!
//! You can create custom extractors by implementing the [`FromRequest`] or [`FromRequestParts`] traits:
//!
//! ```rust
//! use torch_web::{Request, Response, extractors::*};
//! use std::pin::Pin;
//! use std::future::Future;
//!
//! struct ApiKey(String);
//!
//! impl FromRequestParts for ApiKey {
//!     type Error = Response;
//!
//!     fn from_request_parts(
//!         req: &Request,
//!     ) -> Pin<Box<dyn Future<Output = Result<Self, Self::Error>> + Send + 'static>> {
//!         let api_key = req.header("x-api-key")
//!             .map(|s| s.to_string())
//!             .ok_or_else(|| Response::unauthorized().body("API key required"));
//!
//!         Box::pin(async move {
//!             api_key.map(ApiKey)
//!         })
//!     }
//! }
//! ```

use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use crate::{Request, Response};
use http::StatusCode;

/// Trait for extracting data from the complete HTTP request.
///
/// This trait is used for extractors that need access to the entire request,
/// including the request body. Examples include JSON body parsing, form data
/// extraction, and any extractor that needs to consume the request body.
///
/// # Type Parameters
///
/// * `Self` - The type that will be extracted from the request
///
/// # Associated Types
///
/// * `Error` - The error type returned when extraction fails. Must implement
///   [`IntoResponse`] so it can be converted to an HTTP response.
///
/// # Examples
///
/// ```rust
/// use torch_web::{Request, Response, extractors::*};
/// use std::pin::Pin;
/// use std::future::Future;
///
/// struct CustomBody(Vec<u8>);
///
/// impl FromRequest for CustomBody {
///     type Error = Response;
///
///     fn from_request(
///         req: Request,
///     ) -> Pin<Box<dyn Future<Output = Result<(Self, Request), Self::Error>> + Send + 'static>> {
///         Box::pin(async move {
///             let body = req.body().to_vec();
///             Ok((CustomBody(body), req))
///         })
///     }
/// }
/// ```
pub trait FromRequest: Sized {
    /// The error type returned when extraction fails
    type Error: IntoResponse + Send + Sync;

    /// Extracts data from the complete request.
    ///
    /// This method receives the entire request and should return both the extracted
    /// data and the (potentially modified) request for further processing.
    ///
    /// # Parameters
    ///
    /// * `req` - The HTTP request to extract data from
    ///
    /// # Returns
    ///
    /// Returns a `Future` that resolves to either:
    /// - `Ok((Self, Request))` - The extracted data and the request
    /// - `Err(Self::Error)` - An error that can be converted to an HTTP response
    fn from_request(
        req: Request,
    ) -> Pin<Box<dyn Future<Output = Result<(Self, Request), Self::Error>> + Send + 'static>>;
}

/// Trait for extracting data from request parts without consuming the body.
///
/// This trait is used for extractors that only need access to request metadata
/// like headers, path parameters, query parameters, or the request method/URI.
/// These extractors don't consume the request body, allowing multiple extractors
/// to be used together.
///
/// # Type Parameters
///
/// * `Self` - The type that will be extracted from the request
///
/// # Associated Types
///
/// * `Error` - The error type returned when extraction fails. Must implement
///   [`IntoResponse`] so it can be converted to an HTTP response.
///
/// # Examples
///
/// ```rust
/// use torch_web::{Request, Response, extractors::*};
/// use std::pin::Pin;
/// use std::future::Future;
///
/// struct UserAgent(String);
///
/// impl FromRequestParts for UserAgent {
///     type Error = Response;
///
///     fn from_request_parts(
///         req: &Request,
///     ) -> Pin<Box<dyn Future<Output = Result<Self, Self::Error>> + Send + 'static>> {
///         let user_agent = req.header("user-agent")
///             .unwrap_or("Unknown")
///             .to_string();
///
///         Box::pin(async move {
///             Ok(UserAgent(user_agent))
///         })
///     }
/// }
/// ```
pub trait FromRequestParts: Sized {
    /// The error type returned when extraction fails
    type Error: IntoResponse + Send + Sync;

    /// Extract data from request parts
    fn from_request_parts(
        req: &mut Request,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Self::Error>> + Send + 'static>>;
}

/// Error type for extraction failures
#[derive(Debug)]
pub enum ExtractionError {
    /// Missing path parameter
    MissingPathParam(String),
    /// Invalid path parameter format
    InvalidPathParam(String),
    /// Invalid query parameter format
    InvalidQuery(String),
    /// Invalid JSON body
    InvalidJson(String),
    /// Missing required header
    MissingHeader(String),
    /// Invalid header value
    InvalidHeader(String),
    /// Missing application state
    MissingState(String),
    /// Invalid form data
    InvalidForm(String),
    /// Invalid cookie
    InvalidCookie(String),
    /// Content too large
    ContentTooLarge(String),
    /// Unsupported media type
    UnsupportedMediaType(String),
    /// Custom error
    Custom(String),
}

impl std::fmt::Display for ExtractionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtractionError::MissingPathParam(msg) => write!(f, "Missing path parameter: {}", msg),
            ExtractionError::InvalidPathParam(msg) => write!(f, "Invalid path parameter: {}", msg),
            ExtractionError::InvalidQuery(msg) => write!(f, "Invalid query parameter: {}", msg),
            ExtractionError::InvalidJson(msg) => write!(f, "Invalid JSON body: {}", msg),
            ExtractionError::MissingHeader(msg) => write!(f, "Missing header: {}", msg),
            ExtractionError::InvalidHeader(msg) => write!(f, "Invalid header: {}", msg),
            ExtractionError::MissingState(msg) => write!(f, "Missing application state: {}", msg),
            ExtractionError::InvalidForm(msg) => write!(f, "Invalid form data: {}", msg),
            ExtractionError::InvalidCookie(msg) => write!(f, "Invalid cookie: {}", msg),
            ExtractionError::ContentTooLarge(msg) => write!(f, "Content too large: {}", msg),
            ExtractionError::UnsupportedMediaType(msg) => write!(f, "Unsupported media type: {}", msg),
            ExtractionError::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for ExtractionError {}

// Ensure ExtractionError is Send + Sync for async compatibility
unsafe impl Send for ExtractionError {}
unsafe impl Sync for ExtractionError {}

/// Convert extraction errors into HTTP responses
pub trait IntoResponse {
    fn into_response(self) -> Response;
}

impl IntoResponse for ExtractionError {
    fn into_response(self) -> Response {
        let status = match self {
            ExtractionError::MissingPathParam(_) | ExtractionError::InvalidPathParam(_) => {
                StatusCode::BAD_REQUEST
            }
            ExtractionError::InvalidQuery(_) => StatusCode::BAD_REQUEST,
            ExtractionError::InvalidJson(_) => StatusCode::BAD_REQUEST,
            ExtractionError::MissingHeader(_) | ExtractionError::InvalidHeader(_) => {
                StatusCode::BAD_REQUEST
            }
            ExtractionError::MissingState(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ExtractionError::InvalidForm(_) => StatusCode::BAD_REQUEST,
            ExtractionError::InvalidCookie(_) => StatusCode::BAD_REQUEST,
            ExtractionError::ContentTooLarge(_) => StatusCode::PAYLOAD_TOO_LARGE,
            ExtractionError::UnsupportedMediaType(_) => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            ExtractionError::Custom(_) => StatusCode::BAD_REQUEST,
        };

        Response::with_status(status).body(self.to_string())
    }
}

impl IntoResponse for Infallible {
    fn into_response(self) -> Response {
        match self {}
    }
}

impl IntoResponse for Response {
    fn into_response(self) -> Response {
        self
    }
}

impl IntoResponse for &'static str {
    fn into_response(self) -> Response {
        Response::ok().body(self)
    }
}

impl IntoResponse for String {
    fn into_response(self) -> Response {
        Response::ok().body(self)
    }
}

impl IntoResponse for StatusCode {
    fn into_response(self) -> Response {
        Response::with_status(self)
    }
}

impl<T> IntoResponse for (StatusCode, T)
where
    T: IntoResponse,
{
    fn into_response(self) -> Response {
        let mut response = self.1.into_response();
        *response.status_code_mut() = self.0;
        response
    }
}

// Re-export common types for convenience
pub use path::Path;
pub use query::{Query, SerdeQuery};
pub use headers::{Headers, HeaderExtractor, UserAgent, Authorization, ContentType};
pub use state::State;
pub use form::{Form, SerdeForm};
pub use cookies::{Cookies, SessionCookie, CookieBuilder, SameSite, get_cookie, get_required_cookie};

#[cfg(feature = "json")]
pub use json::{Json, RawJson, JsonWithLimit};

// Module declarations
mod path;
mod query;
mod headers;
pub mod state;
mod form;
mod cookies;

#[cfg(feature = "json")]
mod json;
