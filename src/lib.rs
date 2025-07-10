//! # Torch Web Framework
//!
//! A fast, secure web framework that gets out of your way. Built for developers who need
//! production-ready features without the complexity.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use torch::{App, Request, Response};
//!
//! #[tokio::main]
//! async fn main() {
//!     let app = App::new()
//!         .get("/", |_req: Request| async {
//!             Response::ok().body("Hello, World!")
//!         })
//!         .get("/users/:id", |req: Request| async move {
//!             let id = req.param("id").unwrap();
//!             Response::ok().body(format!("User ID: {}", id))
//!         });
//!
//!     app.listen("127.0.0.1:3000").await.unwrap();
//! }
//! ```

pub mod api;
pub mod app;
pub mod cache;
pub mod config;
pub mod database;
pub mod error_pages;
pub mod handler;
pub mod middleware;
pub mod production;
pub mod request;
pub mod response;
pub mod router;
pub mod security;
pub mod server;
pub mod websocket;

// Everything you need to get started
pub use app::App;
pub use error_pages::ErrorPages;
pub use handler::{Handler, HandlerFn};
pub use request::Request;
pub use response::Response;
pub use router::Router;

// HTTP essentials from the http crate
pub use http::{Method, StatusCode, HeaderMap, HeaderName, HeaderValue};

#[cfg(feature = "json")]
pub use serde_json::{json, Value as JsonValue};
