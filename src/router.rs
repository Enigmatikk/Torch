//! # HTTP Router
//!
//! Fast, lightweight HTTP request routing with support for path parameters and wildcards.
//! The router efficiently matches incoming requests to registered handlers based on
//! HTTP method and URL path patterns.

use std::collections::HashMap;
use http::Method;
use crate::{Request, Response, HandlerFn};

/// A fast, lightweight HTTP router that matches requests to handlers.
///
/// The router supports:
/// - Path parameters (`:name` syntax)
/// - Wildcard matching (`*` syntax)
/// - Multiple HTTP methods
/// - Custom 404 handlers
/// - Efficient O(1) method lookup with linear path matching
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use torch_web::{Router, Request, Response, Method};
///
/// let mut router = Router::new();
///
/// router.get("/", |_req: Request| async {
///     Response::ok().body("Home page")
/// });
///
/// router.get("/users/:id", |req: Request| async move {
///     let id = req.param("id").unwrap();
///     Response::ok().body(format!("User: {}", id))
/// });
/// ```
///
/// ## With Parameters and Wildcards
///
/// ```rust
/// use torch_web::{Router, Request, Response};
///
/// let mut router = Router::new();
///
/// // Path parameters
/// router.get("/users/:id/posts/:post_id", |req: Request| async move {
///     let user_id = req.param("id").unwrap();
///     let post_id = req.param("post_id").unwrap();
///     Response::ok().body(format!("User {} Post {}", user_id, post_id))
/// });
///
/// // Wildcard for static files
/// router.get("/static/*", |req: Request| async move {
///     let path = req.path();
///     Response::ok().body(format!("Serving static file: {}", path))
/// });
/// ```
pub struct Router {
    routes: HashMap<Method, Vec<Route>>,
    not_found_handler: Option<HandlerFn>,
}

/// Represents a single route with its pattern and handler.
///
/// This is an internal structure that pairs a route pattern with its handler function.
/// Routes are stored in the router and matched against incoming requests.
#[derive(Clone)]
struct Route {
    pattern: RoutePattern,
    handler: HandlerFn,
}

/// Pattern matching engine for route paths.
///
/// Parses route patterns into segments that can be efficiently matched against
/// incoming request paths. Supports static segments, named parameters, and wildcards.
#[derive(Debug, Clone)]
struct RoutePattern {
    segments: Vec<Segment>,
}

/// A single segment of a route pattern.
///
/// Route patterns are broken down into segments separated by `/`. Each segment
/// can be one of three types:
/// - `Static`: Exact string match (e.g., "users", "api")
/// - `Param`: Named parameter that captures the segment value (e.g., ":id", ":name")
/// - `Wildcard`: Matches any remaining path segments (e.g., "*")
#[derive(Debug, Clone, PartialEq)]
enum Segment {
    /// A static segment that must match exactly
    Static(String),
    /// A parameter segment that captures the value with the given name
    Param(String),
    /// A wildcard that matches any remaining path
    Wildcard,
}

impl Router {
    /// Creates a new empty router.
    ///
    /// The router starts with no routes registered and uses the default 404 handler
    /// for unmatched requests.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torch_web::Router;
    ///
    /// let router = Router::new();
    /// ```
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
            not_found_handler: None,
        }
    }

    /// Registers a route for the specified HTTP method and path pattern.
    ///
    /// This is the core method for route registration. All other route methods
    /// (get, post, etc.) delegate to this method.
    ///
    /// # Parameters
    ///
    /// * `method` - The HTTP method to match (GET, POST, PUT, DELETE, etc.)
    /// * `path` - The path pattern to match, supporting parameters and wildcards
    /// * `handler` - The handler function to execute when the route matches
    ///
    /// # Path Pattern Syntax
    ///
    /// - Static segments: `/users`, `/api/v1`
    /// - Parameters: `/users/:id`, `/posts/:id/comments/:comment_id`
    /// - Wildcards: `/static/*` (matches any remaining path)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torch_web::{Router, Request, Response, Method};
    ///
    /// let mut router = Router::new();
    ///
    /// router.route(Method::GET, "/", |_req: Request| async {
    ///     Response::ok().body("Home")
    /// });
    ///
    /// router.route(Method::POST, "/users", |_req: Request| async {
    ///     Response::created().body("User created")
    /// });
    ///
    /// router.route(Method::GET, "/users/:id", |req: Request| async move {
    ///     let id = req.param("id").unwrap();
    ///     Response::ok().body(format!("User: {}", id))
    /// });
    /// ```
    pub fn route(&mut self, method: Method, path: &str, handler: HandlerFn) {
        let pattern = RoutePattern::parse(path);
        let route = Route { pattern, handler };

        self.routes
            .entry(method)
            .or_insert_with(Vec::new)
            .push(route);
    }

    /// Registers a GET route handler.
    ///
    /// Convenience method for registering GET routes. GET requests should be
    /// idempotent and safe (no side effects).
    ///
    /// # Parameters
    ///
    /// * `path` - The path pattern to match
    /// * `handler` - The handler function to execute
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torch_web::{Router, Request, Response};
    ///
    /// let mut router = Router::new();
    ///
    /// router.get("/", |_req: Request| async {
    ///     Response::ok().body("Welcome!")
    /// });
    ///
    /// router.get("/users/:id", |req: Request| async move {
    ///     let id = req.param("id").unwrap();
    ///     Response::ok().body(format!("User: {}", id))
    /// });
    /// ```
    pub fn get(&mut self, path: &str, handler: HandlerFn) {
        self.route(Method::GET, path, handler);
    }

    /// Registers a POST route handler.
    ///
    /// Convenience method for registering POST routes. POST requests are typically
    /// used for creating resources or submitting data.
    ///
    /// # Parameters
    ///
    /// * `path` - The path pattern to match
    /// * `handler` - The handler function to execute
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torch_web::{Router, Request, Response};
    ///
    /// let mut router = Router::new();
    ///
    /// router.post("/users", |_req: Request| async {
    ///     Response::created().body("User created")
    /// });
    ///
    /// router.post("/login", |_req: Request| async {
    ///     Response::ok().body("Login successful")
    /// });
    /// ```
    pub fn post(&mut self, path: &str, handler: HandlerFn) {
        self.route(Method::POST, path, handler);
    }

    /// Registers a PUT route handler.
    ///
    /// Convenience method for registering PUT routes. PUT requests are typically
    /// used for updating entire resources or creating resources with specific IDs.
    ///
    /// # Parameters
    ///
    /// * `path` - The path pattern to match
    /// * `handler` - The handler function to execute
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torch_web::{Router, Request, Response};
    ///
    /// let mut router = Router::new();
    ///
    /// router.put("/users/:id", |req: Request| async move {
    ///     let id = req.param("id").unwrap();
    ///     Response::ok().body(format!("Updated user: {}", id))
    /// });
    /// ```
    pub fn put(&mut self, path: &str, handler: HandlerFn) {
        self.route(Method::PUT, path, handler);
    }

    /// Registers a DELETE route handler.
    ///
    /// Convenience method for registering DELETE routes. DELETE requests are
    /// used for removing resources.
    ///
    /// # Parameters
    ///
    /// * `path` - The path pattern to match
    /// * `handler` - The handler function to execute
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torch_web::{Router, Request, Response};
    ///
    /// let mut router = Router::new();
    ///
    /// router.delete("/users/:id", |req: Request| async move {
    ///     let id = req.param("id").unwrap();
    ///     Response::ok().body(format!("Deleted user: {}", id))
    /// });
    /// ```
    pub fn delete(&mut self, path: &str, handler: HandlerFn) {
        self.route(Method::DELETE, path, handler);
    }

    /// Registers a PATCH route handler.
    ///
    /// Convenience method for registering PATCH routes. PATCH requests are
    /// used for partial updates to resources.
    ///
    /// # Parameters
    ///
    /// * `path` - The path pattern to match
    /// * `handler` - The handler function to execute
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torch_web::{Router, Request, Response};
    ///
    /// let mut router = Router::new();
    ///
    /// router.patch("/users/:id", |req: Request| async move {
    ///     let id = req.param("id").unwrap();
    ///     Response::ok().body(format!("Patched user: {}", id))
    /// });
    /// ```
    pub fn patch(&mut self, path: &str, handler: HandlerFn) {
        self.route(Method::PATCH, path, handler);
    }

    /// Sets a custom handler for requests that don't match any registered route.
    ///
    /// By default, unmatched requests return a 404 Not Found response. This method
    /// allows you to customize that behavior.
    ///
    /// # Parameters
    ///
    /// * `handler` - The handler function to execute for unmatched routes
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torch_web::{Router, Request, Response};
    ///
    /// let mut router = Router::new();
    ///
    /// router.not_found(|req: Request| async move {
    ///     Response::not_found()
    ///         .body(format!("Sorry, {} was not found", req.path()))
    /// });
    /// ```
    pub fn not_found(&mut self, handler: HandlerFn) {
        self.not_found_handler = Some(handler);
    }

    /// Get all routes for mounting (internal use)
    pub(crate) fn get_all_routes(&self) -> Vec<(Method, String, HandlerFn)> {
        let mut all_routes = Vec::new();

        for (method, routes) in &self.routes {
            for route in routes {
                // Convert the pattern back to a string representation
                let path = route.pattern.to_string();
                all_routes.push((method.clone(), path, route.handler.clone()));
            }
        }

        all_routes
    }

    /// Route a request to the appropriate handler
    pub async fn route_request(&self, mut req: Request) -> Response {
        if let Some(routes) = self.routes.get(req.method()) {
            for route in routes {
                if let Some(params) = route.pattern.matches(req.path()) {
                    // Set path parameters in the request
                    for (name, value) in params {
                        req.set_param(name, value);
                    }
                    return (route.handler)(req).await;
                }
            }
        }

        // No route found, use 404 handler or default
        if let Some(handler) = &self.not_found_handler {
            handler(req).await
        } else {
            Response::not_found()
        }
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Router {
    fn clone(&self) -> Self {
        Self {
            routes: self.routes.clone(),
            not_found_handler: self.not_found_handler.clone(),
        }
    }
}

impl RoutePattern {
    /// Convert pattern back to string representation
    fn to_string(&self) -> String {
        let mut result = String::from("/");
        for segment in &self.segments {
            match segment {
                Segment::Static(s) => {
                    result.push_str(s);
                    result.push('/');
                }
                Segment::Param(name) => {
                    result.push(':');
                    result.push_str(name);
                    result.push('/');
                }
                Segment::Wildcard => {
                    result.push('*');
                    result.push('/');
                }
            }
        }
        // Remove trailing slash unless it's the root path
        if result.len() > 1 && result.ends_with('/') {
            result.pop();
        }
        result
    }

    /// Parse a route pattern string into segments
    fn parse(pattern: &str) -> Self {
        let mut segments = Vec::new();

        for segment in pattern.split('/').filter(|s| !s.is_empty()) {
            if segment.starts_with(':') {
                let param_name = segment[1..].to_string();
                segments.push(Segment::Param(param_name));
            } else if segment == "*" {
                segments.push(Segment::Wildcard);
            } else {
                segments.push(Segment::Static(segment.to_string()));
            }
        }

        Self { segments }
    }

    /// Check if this pattern matches the given path and extract parameters
    fn matches(&self, path: &str) -> Option<HashMap<String, String>> {
        let path_segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        
        // Handle root path
        if path == "/" && self.segments.is_empty() {
            return Some(HashMap::new());
        }

        let mut params = HashMap::new();
        let mut path_idx = 0;
        let mut pattern_idx = 0;

        while pattern_idx < self.segments.len() && path_idx < path_segments.len() {
            match &self.segments[pattern_idx] {
                Segment::Static(expected) => {
                    if path_segments[path_idx] != expected {
                        return None;
                    }
                    path_idx += 1;
                    pattern_idx += 1;
                }
                Segment::Param(name) => {
                    params.insert(name.clone(), path_segments[path_idx].to_string());
                    path_idx += 1;
                    pattern_idx += 1;
                }
                Segment::Wildcard => {
                    // Wildcard matches everything remaining
                    return Some(params);
                }
            }
        }

        // Check if we consumed all segments
        if pattern_idx == self.segments.len() && path_idx == path_segments.len() {
            Some(params)
        } else if pattern_idx < self.segments.len() 
            && matches!(self.segments[pattern_idx], Segment::Wildcard) {
            Some(params)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Response;

    #[test]
    fn test_route_pattern_parsing() {
        let pattern = RoutePattern::parse("/users/:id/posts/:post_id");
        assert_eq!(pattern.segments.len(), 4);
        assert_eq!(pattern.segments[0], Segment::Static("users".to_string()));
        assert_eq!(pattern.segments[1], Segment::Param("id".to_string()));
        assert_eq!(pattern.segments[2], Segment::Static("posts".to_string()));
        assert_eq!(pattern.segments[3], Segment::Param("post_id".to_string()));
    }

    #[test]
    fn test_route_pattern_matching() {
        let pattern = RoutePattern::parse("/users/:id");
        let params = pattern.matches("/users/123").unwrap();
        assert_eq!(params.get("id"), Some(&"123".to_string()));

        assert!(pattern.matches("/users").is_none());
        assert!(pattern.matches("/users/123/extra").is_none());
    }

    #[test]
    fn test_wildcard_matching() {
        let pattern = RoutePattern::parse("/files/*");
        let params = pattern.matches("/files/path/to/file.txt");
        assert!(params.is_some());
    }

    #[tokio::test]
    async fn test_router_basic_routing() {
        let mut router = Router::new();
        
        router.get("/", std::sync::Arc::new(|_| Box::pin(async {
            Response::ok().body("Home")
        })));

        router.get("/users/:id", std::sync::Arc::new(|req| Box::pin(async move {
            let id = req.param("id").unwrap_or("unknown");
            Response::ok().body(format!("User: {}", id))
        })));

        // Test root route
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

        let response = router.route_request(req).await;
        assert_eq!(response.body_bytes(), b"Home");
    }
}
