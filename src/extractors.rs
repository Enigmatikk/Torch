//! # Extractors
//!
//! Extractors allow you to declaratively parse requests and extract the data you need.
//! They provide a type-safe way to access request components like path parameters,
//! query strings, JSON bodies, headers, and application state.
//!
//! ## Example
//!
//! ```rust,no_run
//! use torch_web::{App, extractors::*};
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct User {
//!     name: String,
//!     email: String,
//! }
//!
//! async fn create_user(
//!     Path(user_id): Path<u32>,
//!     Query(params): Query<std::collections::HashMap<String, String>>,
//!     Json(user): Json<User>,
//! ) -> Response {
//!     // user_id, params, and user are all type-safe and validated
//!     Response::ok().json(&user).unwrap()
//! }
//! ```

use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use crate::{Request, Response};
use http::StatusCode;

/// Trait for extracting data from the complete request
pub trait FromRequest: Sized {
    /// The error type returned when extraction fails
    type Error: IntoResponse + Send + Sync;

    /// Extract data from the request
    fn from_request(
        req: Request,
    ) -> Pin<Box<dyn Future<Output = Result<(Self, Request), Self::Error>> + Send + 'static>>;
}

/// Trait for extracting data from request parts (without consuming the body)
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
