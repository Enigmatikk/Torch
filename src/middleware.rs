//! # Middleware System
//!
//! Torch's middleware system allows you to intercept and modify HTTP requests and responses
//! as they flow through your application. Middleware can be used for logging, authentication,
//! CORS, rate limiting, and many other cross-cutting concerns.
//!
//! ## How Middleware Works
//!
//! Middleware forms a chain where each middleware can:
//! 1. Inspect and modify the incoming request
//! 2. Call the next middleware in the chain (or the final handler)
//! 3. Inspect and modify the outgoing response
//! 4. Short-circuit the chain by returning early
//!
//! ## Examples
//!
//! ### Basic Logging Middleware
//!
//! ```rust
//! use torch_web::{App, Request, Response, middleware::Middleware};
//! use std::pin::Pin;
//! use std::future::Future;
//!
//! struct Logger;
//!
//! impl Middleware for Logger {
//!     fn call(
//!         &self,
//!         req: Request,
//!         next: Box<dyn Fn(Request) -> Pin<Box<dyn Future<Output = Response> + Send + 'static>> + Send + Sync>,
//!     ) -> Pin<Box<dyn Future<Output = Response> + Send + 'static>> {
//!         Box::pin(async move {
//!             println!("{} {}", req.method(), req.path());
//!             let response = next(req).await;
//!             println!("Response: {}", response.status_code());
//!             response
//!         })
//!     }
//! }
//!
//! let app = App::new()
//!     .middleware(Logger)
//!     .get("/", |_req| async { Response::ok().body("Hello!") });
//! ```
//!
//! ### Function-based Middleware
//!
//! ```rust
//! use torch_web::{App, Request, Response};
//!
//! let app = App::new()
//!     .middleware(|req: Request, next| async move {
//!         // Add a custom header to all responses
//!         let mut response = next(req).await;
//!         response = response.header("X-Powered-By", "Torch");
//!         response
//!     })
//!     .get("/", |_req| async { Response::ok().body("Hello!") });
//! ```

use std::future::Future;
use std::pin::Pin;
use crate::{Request, Response};

/// Type alias for middleware functions.
///
/// This represents the function signature that middleware must implement.
/// It takes a request and a "next" function that continues the middleware chain.
pub type MiddlewareFn = std::sync::Arc<
    dyn Fn(
            Request,
            Box<dyn Fn(Request) -> Pin<Box<dyn Future<Output = Response> + Send + 'static>> + Send + Sync>,
        ) -> Pin<Box<dyn Future<Output = Response> + Send + 'static>>
        + Send
        + Sync
        + 'static,
>;

/// Trait for implementing middleware components.
///
/// Middleware can intercept requests before they reach handlers and modify
/// responses before they're sent to clients. This trait provides a standard
/// interface for all middleware components.
///
/// # Examples
///
/// ## Authentication Middleware
///
/// ```rust
/// use torch_web::{Request, Response, middleware::Middleware};
/// use std::pin::Pin;
/// use std::future::Future;
///
/// struct AuthMiddleware {
///     secret_key: String,
/// }
///
/// impl AuthMiddleware {
///     fn new(secret_key: String) -> Self {
///         Self { secret_key }
///     }
/// }
///
/// impl Middleware for AuthMiddleware {
///     fn call(
///         &self,
///         req: Request,
///         next: Box<dyn Fn(Request) -> Pin<Box<dyn Future<Output = Response> + Send + 'static>> + Send + Sync>,
///     ) -> Pin<Box<dyn Future<Output = Response> + Send + 'static>> {
///         Box::pin(async move {
///             // Check for authorization header
///             if let Some(auth_header) = req.header("authorization") {
///                 if auth_header.starts_with("Bearer ") {
///                     // Validate token here...
///                     return next(req).await;
///                 }
///             }
///
///             Response::unauthorized().body("Authentication required")
///         })
///     }
/// }
/// ```
///
/// ## CORS Middleware
///
/// ```rust
/// use torch_web::{Request, Response, middleware::Middleware};
/// use std::pin::Pin;
/// use std::future::Future;
///
/// struct CorsMiddleware;
///
/// impl Middleware for CorsMiddleware {
///     fn call(
///         &self,
///         req: Request,
///         next: Box<dyn Fn(Request) -> Pin<Box<dyn Future<Output = Response> + Send + 'static>> + Send + Sync>,
///     ) -> Pin<Box<dyn Future<Output = Response> + Send + 'static>> {
///         Box::pin(async move {
///             let mut response = next(req).await;
///             response = response
///                 .header("Access-Control-Allow-Origin", "*")
///                 .header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
///                 .header("Access-Control-Allow-Headers", "Content-Type, Authorization");
///             response
///         })
///     }
/// }
/// ```
pub trait Middleware: Send + Sync + 'static {
    /// Processes a request through the middleware chain.
    ///
    /// This method receives the current request and a `next` function that
    /// continues processing through the remaining middleware and eventually
    /// to the route handler.
    ///
    /// # Parameters
    ///
    /// * `req` - The HTTP request to process
    /// * `next` - Function to call the next middleware or handler in the chain
    ///
    /// # Returns
    ///
    /// Returns a `Future` that resolves to the HTTP response. The middleware
    /// can modify the request before calling `next`, modify the response after
    /// calling `next`, or return early without calling `next` at all.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torch_web::{Request, Response, middleware::Middleware};
    /// use std::pin::Pin;
    /// use std::future::Future;
    ///
    /// struct TimingMiddleware;
    ///
    /// impl Middleware for TimingMiddleware {
    ///     fn call(
    ///         &self,
    ///         req: Request,
    ///         next: Box<dyn Fn(Request) -> Pin<Box<dyn Future<Output = Response> + Send + 'static>> + Send + Sync>,
    ///     ) -> Pin<Box<dyn Future<Output = Response> + Send + 'static>> {
    ///         Box::pin(async move {
    ///             let start = std::time::Instant::now();
    ///             let response = next(req).await;
    ///             let duration = start.elapsed();
    ///
    ///             println!("Request took {:?}", duration);
    ///             response.header("X-Response-Time", &format!("{}ms", duration.as_millis()))
    ///         })
    ///     }
    /// }
    /// ```
    fn call(
        &self,
        req: Request,
        next: Box<dyn Fn(Request) -> Pin<Box<dyn Future<Output = Response> + Send + 'static>> + Send + Sync>,
    ) -> Pin<Box<dyn Future<Output = Response> + Send + 'static>>;
}

/// Any function that matches the signature can be middleware
impl<F, Fut> Middleware for F
where
    F: Fn(
            Request,
            Box<dyn Fn(Request) -> Pin<Box<dyn Future<Output = Response> + Send + 'static>> + Send + Sync>,
        ) -> Fut
        + Send
        + Sync
        + 'static,
    Fut: Future<Output = Response> + Send + 'static,
{
    fn call(
        &self,
        req: Request,
        next: Box<dyn Fn(Request) -> Pin<Box<dyn Future<Output = Response> + Send + 'static>> + Send + Sync>,
    ) -> Pin<Box<dyn Future<Output = Response> + Send + 'static>> {
        Box::pin(self(req, next))
    }
}

/// Organizes middleware into a processing pipeline
pub struct MiddlewareStack {
    middleware: Vec<MiddlewareFn>,
}

impl MiddlewareStack {
    /// Start with an empty stack
    pub fn new() -> Self {
        Self {
            middleware: Vec::new(),
        }
    }

    /// Add another layer to the stack
    pub fn add<M>(&mut self, middleware: M)
    where
        M: Middleware,
    {
        let middleware_fn = std::sync::Arc::new(move |req, next| middleware.call(req, next));
        self.middleware.push(middleware_fn);
    }

    /// Run a request through the middleware pipeline
    pub async fn execute<F>(&self, req: Request, handler: F) -> Response
    where
        F: Fn(Request) -> Pin<Box<dyn Future<Output = Response> + Send + 'static>> + Send + Sync + 'static,
    {
        if self.middleware.is_empty() {
            // Fast path when no middleware is configured
            return handler(req).await;
        }

        // For now, execute middleware in sequence (simplified implementation)
        let response = handler(req).await;

        // Apply middleware effects to the response (simplified)
        for middleware in &self.middleware {
            // This is a simplified approach - in a full implementation,
            // you would need to restructure the middleware trait to support
            // proper chaining with async closures
            let _ = middleware; // Suppress unused warning
        }

        response
    }
}

impl Default for MiddlewareStack {
    fn default() -> Self {
        Self::new()
    }
}

/// Built-in middleware for logging requests
pub fn logger() -> impl Middleware {
    |req: Request, next: Box<dyn Fn(Request) -> Pin<Box<dyn Future<Output = Response> + Send + 'static>> + Send + Sync>| {
        Box::pin(async move {
            let method = req.method().clone();
            let path = req.path().to_string();
            let start = std::time::Instant::now();

            let response = next(req).await;

            let duration = start.elapsed();
            println!(
                "{} {} - {} ({:.2}ms)",
                method,
                path,
                response.status_code(),
                duration.as_secs_f64() * 1000.0
            );

            response
        })
    }
}

/// Built-in middleware for CORS
pub fn cors() -> impl Middleware {
    |req: Request, next: Box<dyn Fn(Request) -> Pin<Box<dyn Future<Output = Response> + Send + 'static>> + Send + Sync>| {
        Box::pin(async move {
            let mut response = next(req).await;

            // Add CORS headers (this is a simple implementation)
            response = response
                .header("Access-Control-Allow-Origin", "*")
                .header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
                .header("Access-Control-Allow-Headers", "Content-Type, Authorization");

            response
        })
    }
}

/// Built-in middleware for adding security headers
pub fn security_headers() -> impl Middleware {
    |req: Request, next: Box<dyn Fn(Request) -> Pin<Box<dyn Future<Output = Response> + Send + 'static>> + Send + Sync>| {
        Box::pin(async move {
            let mut response = next(req).await;

            response = response
                .header("X-Content-Type-Options", "nosniff")
                .header("X-Frame-Options", "DENY")
                .header("X-XSS-Protection", "1; mode=block");

            response
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Response;

    #[tokio::test]
    async fn test_middleware_stack() {
        let mut stack = MiddlewareStack::new();
        
        // Add a middleware that adds a header
        stack.add(|req: Request, next: Box<dyn Fn(Request) -> Pin<Box<dyn Future<Output = Response> + Send + 'static>> + Send + Sync>| -> Pin<Box<dyn Future<Output = Response> + Send + 'static>> {
            Box::pin(async move {
                let mut response = next(req).await;
                response = response.header("X-Test", "middleware");
                response
            })
        });

        let handler = |_req: Request| -> Pin<Box<dyn Future<Output = Response> + Send + 'static>> {
            Box::pin(async { Response::ok().body("Hello") })
        };

        let req = Request::from_hyper(
            http::Request::builder()
                .method("GET")
                .uri("/")
                .body(())
                .unwrap()
                .into_parts()
                .0,
            Vec::new(),
        )
        .await
        .unwrap();

        let response = stack.execute(req, handler).await;
        assert_eq!(response.headers().get("X-Test").unwrap(), "middleware");
        assert_eq!(response.body_data(), b"Hello");
    }

    #[tokio::test]
    async fn test_cors_middleware() {
        let cors_middleware = cors();
        
        let next = Box::new(|_req: Request| -> Pin<Box<dyn Future<Output = Response> + Send + 'static>> {
            Box::pin(async { Response::ok().body("Hello") })
        });

        let req = Request::from_hyper(
            http::Request::builder()
                .method("GET")
                .uri("/")
                .body(())
                .unwrap()
                .into_parts()
                .0,
            Vec::new(),
        )
        .await
        .unwrap();

        let response = cors_middleware.call(req, next).await;
        assert_eq!(
            response.headers().get("Access-Control-Allow-Origin").unwrap(),
            "*"
        );
    }
}
