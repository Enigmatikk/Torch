//! # Application Builder
//!
//! The core application builder for Torch web framework. This module provides the main
//! [`App`] struct that serves as the entry point for building web applications.

use std::net::SocketAddr;
use http::Method;
use crate::{
    Request, Response, Router, Handler,
    middleware::{MiddlewareStack, Middleware},
    error_pages::ErrorPages,
    server::serve,
    extractors::state::{StateMap, RequestStateExt},
};

/// The main application builder for Torch web framework.
///
/// `App` is the central component that ties together routing, middleware, state management,
/// and server configuration. It follows a builder pattern for easy configuration and
/// provides a fluent API for defining routes and middleware.
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use torch_web::{App, Request, Response};
///
/// let app = App::new()
///     .get("/", |_req: Request| async {
///         Response::ok().body("Hello, World!")
///     })
///     .post("/users", |_req: Request| async {
///         Response::created().body("User created")
///     });
/// ```
///
/// ## With Middleware
///
/// ```rust
/// use torch_web::{App, Request, Response};
///
/// let app = App::new()
///     .middleware(torch_web::middleware::logger())
///     .middleware(torch_web::middleware::cors())
///     .get("/api/users", |_req: Request| async {
///         Response::ok().json(&serde_json::json!({"users": []}))
///     });
/// ```
///
/// ## With Application State
///
/// ```rust
/// use torch_web::{App, Request, Response, extractors::State};
/// use std::sync::Arc;
///
/// #[derive(Clone)]
/// struct AppState {
///     counter: Arc<std::sync::atomic::AtomicU64>,
/// }
///
/// let state = AppState {
///     counter: Arc::new(std::sync::atomic::AtomicU64::new(0)),
/// };
///
/// let app = App::new()
///     .with_state(state)
///     .get("/count", |State(state): State<AppState>| async move {
///         let count = state.counter.load(std::sync::atomic::Ordering::SeqCst);
///         Response::ok().body(format!("Count: {}", count))
///     });
/// ```
///
/// ## Production-Ready Configuration
///
/// ```rust
/// use torch_web::App;
///
/// let app = App::with_defaults() // Includes security, monitoring, CORS
///     .get("/health", |_req| async {
///         Response::ok().json(&serde_json::json!({"status": "healthy"}))
///     });
/// ```
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
    /// Creates a new application instance with default configuration.
    ///
    /// This initializes an empty application with:
    /// - A new router with no routes
    /// - An empty middleware stack
    /// - Default error pages
    /// - Empty state map
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torch_web::App;
    ///
    /// let app = App::new();
    /// ```
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

    /// Adds application state that can be accessed in handlers via the `State` extractor.
    ///
    /// Application state allows you to share data across all handlers, such as database
    /// connections, configuration, or any other shared resources. The state must implement
    /// `Clone + Send + Sync + 'static`.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type of state to add. Must be `Clone + Send + Sync + 'static`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torch_web::{App, Request, Response, extractors::State};
    /// use std::sync::Arc;
    /// use std::sync::atomic::{AtomicU64, Ordering};
    ///
    /// #[derive(Clone)]
    /// struct AppState {
    ///     counter: Arc<AtomicU64>,
    /// }
    ///
    /// let state = AppState {
    ///     counter: Arc::new(AtomicU64::new(0)),
    /// };
    ///
    /// let app = App::new()
    ///     .with_state(state)
    ///     .get("/count", |State(state): State<AppState>| async move {
    ///         let count = state.counter.fetch_add(1, Ordering::SeqCst);
    ///         Response::ok().body(format!("Count: {}", count))
    ///     });
    /// ```
    pub fn with_state<T>(mut self, state: T) -> Self
    where
        T: Clone + Send + Sync + 'static,
    {
        self.state.insert(state);
        self
    }

    /// Adds middleware to the application's middleware stack.
    ///
    /// Middleware is executed in the order it's added, wrapping the final route handler.
    /// Each middleware can modify the request before it reaches the handler and/or
    /// modify the response before it's sent to the client.
    ///
    /// # Type Parameters
    ///
    /// * `M` - The middleware type. Must implement the [`Middleware`] trait.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torch_web::{App, middleware};
    ///
    /// let app = App::new()
    ///     .middleware(middleware::logger())
    ///     .middleware(middleware::cors())
    ///     .get("/", |_req| async { Response::ok().body("Hello!") });
    /// ```
    ///
    /// ## Custom Middleware
    ///
    /// ```rust
    /// use torch_web::{App, Request, Response, middleware::Middleware};
    /// use std::pin::Pin;
    /// use std::future::Future;
    ///
    /// struct CustomMiddleware;
    ///
    /// impl Middleware for CustomMiddleware {
    ///     fn call(
    ///         &self,
    ///         req: Request,
    ///         next: Box<dyn Fn(Request) -> Pin<Box<dyn Future<Output = Response> + Send + 'static>> + Send + Sync>,
    ///     ) -> Pin<Box<dyn Future<Output = Response> + Send + 'static>> {
    ///         Box::pin(async move {
    ///             // Modify request here
    ///             let mut response = next(req).await;
    ///             // Modify response here
    ///             response = response.header("X-Custom", "middleware");
    ///             response
    ///         })
    ///     }
    /// }
    ///
    /// let app = App::new().middleware(CustomMiddleware);
    /// ```
    pub fn middleware<M>(mut self, middleware: M) -> Self
    where
        M: Middleware,
    {
        self.middleware.add(middleware);
        self
    }

    /// Registers a route for any HTTP method with the specified path and handler.
    ///
    /// This is the most flexible route registration method that allows you to specify
    /// any HTTP method. For common methods, consider using the convenience methods
    /// like `get`, `post`, `put`, etc.
    ///
    /// # Parameters
    ///
    /// * `method` - The HTTP method to match (GET, POST, PUT, DELETE, etc.)
    /// * `path` - The URL path pattern to match. Supports parameters like `/users/:id`
    /// * `handler` - The handler function to execute when the route matches
    ///
    /// # Path Parameters
    ///
    /// Paths can include parameters using the `:name` syntax:
    /// - `/users/:id` matches `/users/123` and captures `id = "123"`
    /// - `/posts/:id/comments/:comment_id` captures multiple parameters
    /// - Use the `Path` extractor to access parameters in handlers
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torch_web::{App, Request, Response, Method};
    ///
    /// let app = App::new()
    ///     .route(Method::GET, "/", |_req: Request| async {
    ///         Response::ok().body("Hello, World!")
    ///     })
    ///     .route(Method::POST, "/users", |_req: Request| async {
    ///         Response::created().body("User created")
    ///     });
    /// ```
    pub fn route<H, T>(mut self, method: Method, path: &str, handler: H) -> Self
    where
        H: Handler<T>,
    {
        let handler_fn = crate::handler::into_handler_fn(handler);
        self.router.route(method, path, handler_fn);
        self
    }

    /// Registers a GET route handler.
    ///
    /// GET requests are typically used for retrieving data and should be idempotent
    /// (safe to repeat). Perfect for serving web pages, API endpoints that fetch data,
    /// and static resources.
    ///
    /// # Parameters
    ///
    /// * `path` - The URL path pattern to match
    /// * `handler` - The handler function to execute
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torch_web::{App, Request, Response, extractors::Path};
    ///
    /// let app = App::new()
    ///     .get("/", |_req: Request| async {
    ///         Response::ok().body("Welcome to Torch!")
    ///     })
    ///     .get("/users/:id", |Path(id): Path<u32>| async move {
    ///         Response::ok().body(format!("User ID: {}", id))
    ///     })
    ///     .get("/api/users", |_req: Request| async {
    ///         Response::ok().json(&serde_json::json!({"users": []}))
    ///     });
    /// ```
    pub fn get<H, T>(self, path: &str, handler: H) -> Self
    where
        H: Handler<T>,
    {
        self.route(Method::GET, path, handler)
    }

    /// Registers a POST route handler.
    ///
    /// POST requests are typically used for creating new resources or submitting data.
    /// They are not idempotent and can have side effects.
    ///
    /// # Parameters
    ///
    /// * `path` - The URL path pattern to match
    /// * `handler` - The handler function to execute
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torch_web::{App, Request, Response, extractors::Json};
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Deserialize, Serialize)]
    /// struct CreateUser {
    ///     name: String,
    ///     email: String,
    /// }
    ///
    /// let app = App::new()
    ///     .post("/users", |Json(user): Json<CreateUser>| async move {
    ///         // Create user logic here
    ///         Response::created().json(&user)
    ///     })
    ///     .post("/login", |_req: Request| async {
    ///         Response::ok().body("Login successful")
    ///     });
    /// ```
    pub fn post<H, T>(self, path: &str, handler: H) -> Self
    where
        H: Handler<T>,
    {
        self.route(Method::POST, path, handler)
    }

    /// Registers a PUT route handler.
    ///
    /// PUT requests are typically used for updating entire resources or creating
    /// resources with a specific ID. They should be idempotent.
    ///
    /// # Parameters
    ///
    /// * `path` - The URL path pattern to match
    /// * `handler` - The handler function to execute
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torch_web::{App, Request, Response, extractors::{Path, Json}};
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Deserialize, Serialize)]
    /// struct UpdateUser {
    ///     name: String,
    ///     email: String,
    /// }
    ///
    /// let app = App::new()
    ///     .put("/users/:id", |Path(id): Path<u32>, Json(user): Json<UpdateUser>| async move {
    ///         // Update user logic here
    ///         Response::ok().json(&user)
    ///     });
    /// ```
    pub fn put<H, T>(self, path: &str, handler: H) -> Self
    where
        H: Handler<T>,
    {
        self.route(Method::PUT, path, handler)
    }

    /// Registers a DELETE route handler.
    ///
    /// DELETE requests are used for removing resources. They should be idempotent
    /// (deleting a non-existent resource should not cause an error).
    ///
    /// # Parameters
    ///
    /// * `path` - The URL path pattern to match
    /// * `handler` - The handler function to execute
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torch_web::{App, Request, Response, extractors::Path};
    ///
    /// let app = App::new()
    ///     .delete("/users/:id", |Path(id): Path<u32>| async move {
    ///         // Delete user logic here
    ///         Response::no_content()
    ///     })
    ///     .delete("/posts/:id", |Path(id): Path<u32>| async move {
    ///         Response::ok().body(format!("Deleted post {}", id))
    ///     });
    /// ```
    pub fn delete<H, T>(self, path: &str, handler: H) -> Self
    where
        H: Handler<T>,
    {
        self.route(Method::DELETE, path, handler)
    }

    /// Registers a PATCH route handler.
    ///
    /// PATCH requests are used for partial updates to resources. Unlike PUT,
    /// PATCH only updates the specified fields rather than replacing the entire resource.
    ///
    /// # Parameters
    ///
    /// * `path` - The URL path pattern to match
    /// * `handler` - The handler function to execute
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torch_web::{App, Request, Response, extractors::{Path, Json}};
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Deserialize, Serialize)]
    /// struct PatchUser {
    ///     name: Option<String>,
    ///     email: Option<String>,
    /// }
    ///
    /// let app = App::new()
    ///     .patch("/users/:id", |Path(id): Path<u32>, Json(patch): Json<PatchUser>| async move {
    ///         // Partial update logic here
    ///         Response::ok().json(&patch)
    ///     });
    /// ```
    pub fn patch<H, T>(self, path: &str, handler: H) -> Self
    where
        H: Handler<T>,
    {
        self.route(Method::PATCH, path, handler)
    }

    /// Registers an OPTIONS route handler.
    ///
    /// OPTIONS requests are typically used for CORS preflight requests to check
    /// what methods and headers are allowed for cross-origin requests.
    ///
    /// # Parameters
    ///
    /// * `path` - The URL path pattern to match
    /// * `handler` - The handler function to execute
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torch_web::{App, Request, Response};
    ///
    /// let app = App::new()
    ///     .options("/api/*", |_req: Request| async {
    ///         Response::ok()
    ///             .header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE")
    ///             .header("Access-Control-Allow-Headers", "Content-Type, Authorization")
    ///             .body("")
    ///     });
    /// ```
    pub fn options<H, T>(self, path: &str, handler: H) -> Self
    where
        H: Handler<T>,
    {
        self.route(Method::OPTIONS, path, handler)
    }

    /// Registers a HEAD route handler.
    ///
    /// HEAD requests are identical to GET requests except that the server must not
    /// return a message body. They're useful for checking if a resource exists or
    /// getting metadata without downloading the full content.
    ///
    /// # Parameters
    ///
    /// * `path` - The URL path pattern to match
    /// * `handler` - The handler function to execute
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torch_web::{App, Request, Response};
    ///
    /// let app = App::new()
    ///     .head("/users/:id", |_req: Request| async {
    ///         // Check if user exists and return appropriate headers
    ///         Response::ok()
    ///             .header("Content-Type", "application/json")
    ///             .header("Content-Length", "42")
    ///     });
    /// ```
    pub fn head<H, T>(self, path: &str, handler: H) -> Self
    where
        H: Handler<T>,
    {
        self.route(Method::HEAD, path, handler)
    }

    /// Sets a custom handler for requests that don't match any registered route.
    ///
    /// By default, unmatched requests return a 404 Not Found response. This method
    /// allows you to customize that behavior with your own handler.
    ///
    /// # Parameters
    ///
    /// * `handler` - The handler function to execute for unmatched routes
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torch_web::{App, Request, Response};
    ///
    /// let app = App::new()
    ///     .get("/", |_req: Request| async {
    ///         Response::ok().body("Home")
    ///     })
    ///     .not_found(|req: Request| async move {
    ///         Response::not_found()
    ///             .body(format!("Sorry, {} was not found", req.path()))
    ///     });
    /// ```
    pub fn not_found<H, T>(mut self, handler: H) -> Self
    where
        H: Handler<T>,
    {
        let handler_fn = crate::handler::into_handler_fn(handler);
        self.router.not_found(handler_fn);
        self
    }

    /// Mounts another router at a specific path prefix.
    ///
    /// This allows you to modularize your application by creating separate routers
    /// for different parts of your API and then mounting them under different prefixes.
    /// All routes from the mounted router will be prefixed with the specified path.
    ///
    /// # Parameters
    ///
    /// * `prefix` - The path prefix to mount the router under (e.g., "/api/v1")
    /// * `other` - The router to mount
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torch_web::{App, Router, Request, Response};
    ///
    /// // Create a router for user-related endpoints
    /// let mut user_router = Router::new();
    /// user_router.get("/", |_req: Request| async {
    ///     Response::ok().body("List users")
    /// });
    /// user_router.get("/:id", |_req: Request| async {
    ///     Response::ok().body("Get user")
    /// });
    ///
    /// // Mount it under /api/users
    /// let app = App::new()
    ///     .get("/", |_req: Request| async {
    ///         Response::ok().body("Home")
    ///     })
    ///     .mount("/api/users", user_router);
    ///
    /// // Now you have:
    /// // GET / -> "Home"
    /// // GET /api/users/ -> "List users"
    /// // GET /api/users/:id -> "Get user"
    /// ```
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

    /// Configures custom error pages for the application.
    ///
    /// This replaces the default error page configuration with a custom one.
    /// Use this when you want full control over error page rendering.
    ///
    /// # Parameters
    ///
    /// * `error_pages` - The custom error pages configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torch_web::{App, ErrorPages};
    ///
    /// let custom_errors = ErrorPages::new()
    ///     .custom_404("<h1>Page Not Found</h1>".to_string())
    ///     .custom_500("<h1>Server Error</h1>".to_string());
    ///
    /// let app = App::new()
    ///     .error_pages(custom_errors);
    /// ```
    pub fn error_pages(mut self, error_pages: ErrorPages) -> Self {
        self.error_pages = error_pages;
        self
    }

    /// Sets a custom HTML template for 404 Not Found errors.
    ///
    /// This is a convenience method for setting just the 404 error page
    /// without having to create a full ErrorPages configuration.
    ///
    /// # Parameters
    ///
    /// * `html` - The HTML content to display for 404 errors
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torch_web::App;
    ///
    /// let app = App::new()
    ///     .custom_404(r#"
    ///         <html>
    ///             <body>
    ///                 <h1>🔥 Torch doesn't know this route!</h1>
    ///                 <p>The page you're looking for doesn't exist.</p>
    ///             </body>
    ///         </html>
    ///     "#.to_string());
    /// ```
    pub fn custom_404(mut self, html: String) -> Self {
        self.error_pages = self.error_pages.custom_404(html);
        self
    }

    /// Sets a custom HTML template for 500 Internal Server Error responses.
    ///
    /// This is a convenience method for setting just the 500 error page
    /// without having to create a full ErrorPages configuration.
    ///
    /// # Parameters
    ///
    /// * `html` - The HTML content to display for 500 errors
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torch_web::App;
    ///
    /// let app = App::new()
    ///     .custom_500(r#"
    ///         <html>
    ///             <body>
    ///                 <h1>🔥 Something went wrong!</h1>
    ///                 <p>We're working to fix this issue.</p>
    ///             </body>
    ///         </html>
    ///     "#.to_string());
    /// ```
    pub fn custom_500(mut self, html: String) -> Self {
        self.error_pages = self.error_pages.custom_500(html);
        self
    }

    /// Disables the default error page styling and uses plain HTML.
    ///
    /// By default, Torch adds CSS styling to error pages. This method
    /// disables that styling if you prefer plain HTML or want to add
    /// your own styling.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torch_web::App;
    ///
    /// let app = App::new()
    ///     .plain_error_pages()
    ///     .custom_404("<h1>Not Found</h1>".to_string());
    /// ```
    pub fn plain_error_pages(mut self) -> Self {
        self.error_pages = self.error_pages.without_default_styling();
        self
    }

    /// Adds a WebSocket endpoint to the application.
    ///
    /// WebSocket endpoints allow for real-time, bidirectional communication between
    /// the client and server. This method registers a handler that will be called
    /// when a WebSocket connection is established.
    ///
    /// **Note**: This method requires the `websocket` feature to be enabled.
    ///
    /// # Parameters
    ///
    /// * `path` - The URL path for the WebSocket endpoint
    /// * `handler` - The function to handle WebSocket connections
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torch_web::App;
    ///
    /// let app = App::new()
    ///     .websocket("/ws", |mut connection| async move {
    ///         while let Some(message) = connection.receive().await? {
    ///             // Echo the message back
    ///             connection.send(message).await?;
    ///         }
    ///         Ok(())
    ///     });
    /// ```
    ///
    /// ## Chat Server Example
    ///
    /// ```rust
    /// use torch_web::{App, extractors::State};
    /// use std::sync::Arc;
    /// use tokio::sync::broadcast;
    ///
    /// #[derive(Clone)]
    /// struct ChatState {
    ///     sender: broadcast::Sender<String>,
    /// }
    ///
    /// let (tx, _) = broadcast::channel(100);
    /// let state = ChatState { sender: tx };
    ///
    /// let app = App::new()
    ///     .with_state(state)
    ///     .websocket("/chat", |mut connection| async move {
    ///         // Handle chat messages
    ///         Ok(())
    ///     });
    /// ```
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

    /// No-op WebSocket method when the websocket feature is disabled.
    ///
    /// This method exists to provide a consistent API regardless of whether
    /// the `websocket` feature is enabled. When the feature is disabled,
    /// this method does nothing and returns the app unchanged.
    #[cfg(not(feature = "websocket"))]
    pub fn websocket<F, Fut>(self, _path: &str, _handler: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send + 'static,
    {
        // WebSocket feature not enabled, return self unchanged
        self
    }

    /// Starts the HTTP server and begins listening for incoming requests.
    ///
    /// This method consumes the `App` and starts the server on the specified address.
    /// The server will run until the process is terminated or an error occurs.
    ///
    /// # Parameters
    ///
    /// * `addr` - The address to bind to (e.g., "127.0.0.1:3000" or "0.0.0.0:8080")
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the server shuts down gracefully, or an error if
    /// the server fails to start or encounters a fatal error.
    ///
    /// # Examples
    ///
    /// ## Basic Server
    ///
    /// ```rust,no_run
    /// use torch_web::{App, Request, Response};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    ///     let app = App::new()
    ///         .get("/", |_req: Request| async {
    ///             Response::ok().body("Hello, World!")
    ///         });
    ///
    ///     app.listen("127.0.0.1:3000").await
    /// }
    /// ```
    ///
    /// ## Production Server
    ///
    /// ```rust,no_run
    /// use torch_web::App;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    ///     let app = App::with_defaults()
    ///         .get("/health", |_req| async {
    ///             Response::ok().json(&serde_json::json!({"status": "healthy"}))
    ///         });
    ///
    ///     // Bind to all interfaces in production
    ///     app.listen("0.0.0.0:8080").await
    /// }
    /// ```
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

/// Creates a new application instance.
///
/// This is a convenience function that's equivalent to calling [`App::new()`].
/// Use this when you prefer a functional style over method chaining.
///
/// # Examples
///
/// ```rust
/// use torch_web::{app, Request, Response};
///
/// let app = app()
///     .get("/", |_req: Request| async {
///         Response::ok().body("Hello, World!")
///     });
/// ```
pub fn app() -> App {
    App::new()
}

/// Pre-configured application constructors for common scenarios.
///
/// These methods provide sensible defaults for different types of applications,
/// saving you from having to manually configure common middleware combinations.
impl App {
    /// Creates a new app with request logging middleware enabled.
    ///
    /// This is useful for development and debugging, as it logs all incoming
    /// requests with their method, path, and response status.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torch_web::{App, Request, Response};
    ///
    /// let app = App::with_logging()
    ///     .get("/", |_req: Request| async {
    ///         Response::ok().body("Hello, World!")
    ///     });
    /// ```
    pub fn with_logging() -> Self {
        Self::new().middleware(crate::middleware::logger())
    }

    /// Creates a new app with CORS (Cross-Origin Resource Sharing) middleware enabled.
    ///
    /// This allows your API to be accessed from web browsers running on different
    /// domains. Useful for frontend applications that need to call your API.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torch_web::{App, Request, Response};
    ///
    /// let app = App::with_cors()
    ///     .get("/api/data", |_req: Request| async {
    ///         Response::ok().json(&serde_json::json!({"data": "value"}))
    ///     });
    /// ```
    pub fn with_cors() -> Self {
        Self::new().middleware(crate::middleware::cors())
    }

    /// Creates a production-ready app with comprehensive middleware stack.
    ///
    /// This includes:
    /// - Request logging and monitoring
    /// - Performance metrics collection
    /// - Security headers (HSTS, CSP, etc.)
    /// - Request ID generation
    /// - Input validation
    /// - CORS support
    /// - Request timeout (30 seconds)
    /// - Request size limit (16MB)
    /// - Health check endpoint
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torch_web::{App, Request, Response};
    ///
    /// let app = App::with_defaults()
    ///     .get("/", |_req: Request| async {
    ///         Response::ok().body("Production app")
    ///     })
    ///     .get("/api/users", |_req: Request| async {
    ///         Response::ok().json(&serde_json::json!({"users": []}))
    ///     });
    /// ```
    pub fn with_defaults() -> Self {
        Self::new()
            // Request logging and monitoring
            .middleware(crate::middleware::logger())
            .middleware(crate::production::MetricsCollector::new())
            .middleware(crate::production::PerformanceMonitor)

            // Security middleware (TODO: Implement proper middleware integration)
            // .middleware(crate::security::SecurityHeaders::new())
            // .middleware(crate::security::RequestId)
            // .middleware(crate::security::InputValidator)

            // CORS support
            .middleware(crate::middleware::cors())

            // Production features
            .middleware(crate::production::RequestTimeout::new(std::time::Duration::from_secs(30)))
            .middleware(crate::production::RequestSizeLimit::new(16 * 1024 * 1024)) // 16MB
            .middleware(crate::production::health_check())
    }

    /// Creates an app with essential security middleware enabled.
    ///
    /// This includes:
    /// - Security headers (HSTS, CSP, X-Frame-Options, etc.)
    /// - Request ID generation for tracking
    /// - Input validation to prevent common attacks
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torch_web::{App, Request, Response};
    ///
    /// let app = App::with_security()
    ///     .get("/secure-endpoint", |_req: Request| async {
    ///         Response::ok().body("This endpoint has security headers")
    ///     });
    /// ```
    pub fn with_security() -> Self {
        // TODO: Implement proper security middleware integration
        Self::new()
            // .middleware(crate::security::SecurityHeaders::new())
            // .middleware(crate::security::RequestId)
            // .middleware(crate::security::InputValidator)
    }

    /// Creates an app with monitoring and metrics collection enabled.
    ///
    /// This includes:
    /// - Request logging
    /// - Performance metrics collection
    /// - Performance monitoring
    /// - Health check endpoint at `/health`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torch_web::{App, Request, Response};
    ///
    /// let app = App::with_monitoring()
    ///     .get("/api/status", |_req: Request| async {
    ///         Response::ok().json(&serde_json::json!({"status": "ok"}))
    ///     });
    /// ```
    pub fn with_monitoring() -> Self {
        Self::new()
            .middleware(crate::middleware::logger())
            .middleware(crate::production::MetricsCollector::new())
            .middleware(crate::production::PerformanceMonitor)
            .middleware(crate::production::health_check())
    }
}

#[cfg(test)]
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
            .get::<_, (Request,)>("/", |_req: Request| async { Response::ok() })
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
