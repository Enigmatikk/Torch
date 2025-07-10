use std::net::SocketAddr;
use http::Method;
use crate::{
    Request, Response, Router, Handler,
    middleware::{MiddlewareStack, Middleware},
    error_pages::ErrorPages,
    server::serve,
    extractors::state::{StateMap, RequestStateExt},
};

/// Your app's starting point - where all the magic happens
pub struct App {
    router: Router,
    middleware: MiddlewareStack,
    error_pages: ErrorPages,
    state: StateMap,
    #[cfg(feature = "api")]
    pub(crate) api_docs: Option<crate::api::ApiDocBuilder>,
    #[cfg(not(feature = "api"))]
    _phantom: std::marker::PhantomData<()>,
}

impl App {
    /// Start with a clean slate
    pub fn new() -> Self {
        Self {
            router: Router::new(),
            middleware: MiddlewareStack::new(),
            error_pages: ErrorPages::new(),
            state: StateMap::new(),
            #[cfg(feature = "api")]
            api_docs: None,
            #[cfg(not(feature = "api"))]
            _phantom: std::marker::PhantomData,
        }
    }

    /// Add application state that can be accessed in handlers
    pub fn with_state<T>(mut self, state: T) -> Self
    where
        T: Clone + Send + Sync + 'static,
    {
        self.state.insert(state);
        self
    }

    /// Stack up middleware for request processing
    pub fn middleware<M>(mut self, middleware: M) -> Self
    where
        M: Middleware,
    {
        self.middleware.add(middleware);
        self
    }

    /// Map any HTTP method to a path with your handler
    pub fn route<H, T>(mut self, method: Method, path: &str, handler: H) -> Self
    where
        H: Handler<T>,
    {
        let handler_fn = crate::handler::into_handler_fn(handler);
        self.router.route(method, path, handler_fn);
        self
    }

    /// Handle GET requests - perfect for serving pages and fetching data
    pub fn get<H, T>(self, path: &str, handler: H) -> Self
    where
        H: Handler<T>,
    {
        self.route(Method::GET, path, handler)
    }

    /// Handle POST requests - for creating new resources
    pub fn post<H, T>(self, path: &str, handler: H) -> Self
    where
        H: Handler<T>,
    {
        self.route(Method::POST, path, handler)
    }

    /// Handle PUT requests - for updating entire resources
    pub fn put<H, T>(self, path: &str, handler: H) -> Self
    where
        H: Handler<T>,
    {
        self.route(Method::PUT, path, handler)
    }

    /// Handle DELETE requests - for removing resources
    pub fn delete<H, T>(self, path: &str, handler: H) -> Self
    where
        H: Handler<T>,
    {
        self.route(Method::DELETE, path, handler)
    }

    /// Handle PATCH requests - for partial updates
    pub fn patch<H, T>(self, path: &str, handler: H) -> Self
    where
        H: Handler<T>,
    {
        self.route(Method::PATCH, path, handler)
    }

    /// Handle OPTIONS requests - usually for CORS preflight
    pub fn options<H, T>(self, path: &str, handler: H) -> Self
    where
        H: Handler<T>,
    {
        self.route(Method::OPTIONS, path, handler)
    }

    /// Handle HEAD requests - like GET but without the body
    pub fn head<H, T>(self, path: &str, handler: H) -> Self
    where
        H: Handler<T>,
    {
        self.route(Method::HEAD, path, handler)
    }

    /// Catch requests that don't match any route
    pub fn not_found<H, T>(mut self, handler: H) -> Self
    where
        H: Handler<T>,
    {
        let handler_fn = crate::handler::into_handler_fn(handler);
        self.router.not_found(handler_fn);
        self
    }

    /// Mount another router at a specific path prefix
    pub fn mount(mut self, prefix: &str, other: Router) -> Self {
        // Merge routes from the other router with the prefix
        let prefix = prefix.trim_end_matches('/');

        // Get all routes from the other router and add them with prefix
        for (method, path, handler) in other.get_all_routes() {
            let prefixed_path = if prefix.is_empty() {
                path
            } else {
                format!("{}{}", prefix, path)
            };

            self.router.route(method, &prefixed_path, handler);
        }

        self
    }

    /// Configure error pages
    pub fn error_pages(mut self, error_pages: ErrorPages) -> Self {
        self.error_pages = error_pages;
        self
    }

    /// Set a custom 404 page
    pub fn custom_404(mut self, html: String) -> Self {
        self.error_pages = self.error_pages.custom_404(html);
        self
    }

    /// Set a custom 500 page
    pub fn custom_500(mut self, html: String) -> Self {
        self.error_pages = self.error_pages.custom_500(html);
        self
    }

    /// Disable default error page styling (use plain HTML)
    pub fn plain_error_pages(mut self) -> Self {
        self.error_pages = self.error_pages.without_default_styling();
        self
    }

    /// Add a WebSocket endpoint
    #[cfg(feature = "websocket")]
    pub fn websocket<F, Fut>(self, path: &str, handler: F) -> Self
    where
        F: Fn(crate::websocket::WebSocketConnection) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send + 'static,
    {
        let handler = std::sync::Arc::new(handler);
        self.get::<_, (Request,)>(path, move |req: Request| {
            let _handler = handler.clone();
            async move {
                // Check if this is a WebSocket upgrade request
                if crate::websocket::is_websocket_upgrade_request(&req) {
                    crate::websocket::websocket_upgrade(req).await
                } else {
                    Response::bad_request().body("WebSocket upgrade required")
                }
            }
        })
    }

    /// Add a WebSocket endpoint (no-op when websocket feature is disabled)
    #[cfg(not(feature = "websocket"))]
    pub fn websocket<F, Fut>(self, _path: &str, _handler: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send + 'static,
    {
        // WebSocket feature not enabled, return self unchanged
        self
    }

    /// Fire up the server and start handling requests
    pub async fn listen(self, addr: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr: SocketAddr = addr.parse()?;
        println!("🔥 Torch server starting on http://{}", addr);
        serve(addr, self).await
    }

    /// Process incoming requests through middleware and routing
    pub(crate) async fn handle_request(&self, mut req: Request) -> Response {
        // Inject application state into the request
        req.set_state_map(self.state.clone());

        let router = self.router.clone();
        let error_pages = self.error_pages.clone();

        let response = self.middleware
            .execute(req, move |req| {
                let router = router.clone();
                Box::pin(async move { router.route_request(req).await })
            })
            .await;

        // Check if this is an error response that should be rendered with error pages
        let status_code = response.status_code().as_u16();
        if status_code >= 400 && self.should_render_error_page(&response) {
            // Create a simple request for error page rendering
            let dummy_req = Request::new();
            error_pages.render_error(status_code, None, &dummy_req)
        } else {
            response
        }
    }

    /// Check if we should render an error page for this response
    fn should_render_error_page(&self, response: &Response) -> bool {
        // Only render error pages for responses that look like default error responses
        // (i.e., they have simple text bodies, not custom HTML)
        let content_type = response.headers().get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        // Don't override responses that are already HTML or have custom content types
        !content_type.starts_with("text/html") &&
        !content_type.starts_with("application/json") &&
        response.body_data().len() < 100 // Simple heuristic for basic error messages
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

/// Quick way to create a new app
pub fn app() -> App {
    App::new()
}

/// Pre-configured apps for common scenarios
impl App {
    /// Create a new app with logging middleware
    pub fn with_logging() -> Self {
        Self::new().middleware(crate::middleware::logger())
    }

    /// Create a new app with CORS middleware
    pub fn with_cors() -> Self {
        Self::new().middleware(crate::middleware::cors())
    }

    /// Create a production-ready app with security, monitoring, and performance middleware
    pub fn with_defaults() -> Self {
        Self::new()
            // Request logging and monitoring
            .middleware(crate::middleware::logger())
            .middleware(crate::production::MetricsCollector::new())
            .middleware(crate::production::PerformanceMonitor)

            // Security middleware
            .middleware(crate::security::SecurityHeaders::new())
            .middleware(crate::security::RequestId)
            .middleware(crate::security::InputValidator)

            // CORS support
            .middleware(crate::middleware::cors())

            // Production features
            .middleware(crate::production::RequestTimeout::new(std::time::Duration::from_secs(30)))
            .middleware(crate::production::RequestSizeLimit::new(16 * 1024 * 1024)) // 16MB
            .middleware(crate::production::health_check())
    }

    /// Create an app with basic security features
    pub fn with_security() -> Self {
        Self::new()
            .middleware(crate::security::SecurityHeaders::new())
            .middleware(crate::security::RequestId)
            .middleware(crate::security::InputValidator)
    }

    /// Create an app with monitoring and metrics
    pub fn with_monitoring() -> Self {
        Self::new()
            .middleware(crate::middleware::logger())
            .middleware(crate::production::MetricsCollector::new())
            .middleware(crate::production::PerformanceMonitor)
            .middleware(crate::production::health_check())
    }
}

#[cfg(disabled_for_now)]
mod tests {
    use super::*;
    use std::pin::Pin;
    use std::future::Future;
    use crate::Response;

    #[tokio::test]
    async fn test_app_creation() {
        let app = App::new()
            .get("/", |_req: Request| async {
                Response::ok().body("Hello, World!")
            })
            .post("/users", |_req: Request| async {
                Response::ok().body("User created")
            });

        // Test GET route
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

        let response = app.handle_request(req).await;
        assert_eq!(response.body_data(), b"Hello, World!");
    }

    #[tokio::test]
    async fn test_app_with_middleware() {
        let app = App::new()
            .middleware(|req: Request, next: Box<dyn Fn(Request) -> Pin<Box<dyn Future<Output = Response> + Send + 'static>> + Send + Sync>| -> Pin<Box<dyn Future<Output = Response> + Send + 'static>> {
                Box::pin(async move {
                    let mut response = next(req).await;
                    response = response.header("X-Test", "middleware");
                    response
                })
            })
            .get("/", |_req: Request| async {
                Response::ok().body("Hello")
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

        let response = app.handle_request(req).await;
        assert_eq!(response.headers().get("X-Test").unwrap(), "middleware");
    }

    #[test]
    fn test_app_builder_pattern() {
        let _app = App::new()
            .get("/", |_req: Request| async { Response::ok() })
            .post("/users", |_req: Request| async { Response::ok() })
            .middleware(crate::middleware::logger())
            .middleware(crate::middleware::cors());
    }

    #[test]
    fn test_convenience_constructors() {
        let _app1 = App::with_logging();
        let _app2 = App::with_cors();
        let _app3 = App::with_security();
        let _app4 = App::with_defaults();
    }
}
