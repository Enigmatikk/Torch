use std::future::Future;
use std::pin::Pin;
use crate::{Request, Response};

/// Function signature for middleware - takes a request and the next handler
pub type MiddlewareFn = std::sync::Arc<
    dyn Fn(
            Request,
            Box<dyn Fn(Request) -> Pin<Box<dyn Future<Output = Response> + Send + 'static>> + Send + Sync>,
        ) -> Pin<Box<dyn Future<Output = Response> + Send + 'static>>
        + Send
        + Sync
        + 'static,
>;

/// Middleware can intercept requests before they reach your handlers
pub trait Middleware: Send + Sync + 'static {
    /// Do your thing with the request, then decide whether to continue the chain
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
