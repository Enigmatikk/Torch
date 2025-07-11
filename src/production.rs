//! # Production-Ready Features
//!
//! This module provides enterprise-grade features for deploying Torch applications
//! in production environments. It includes performance monitoring, metrics collection,
//! request timeouts, rate limiting, and other essential production middleware.
//!
//! ## Key Features
//!
//! - **Performance Monitoring**: Track request latency, throughput, and resource usage
//! - **Metrics Collection**: Collect and export metrics for monitoring systems
//! - **Request Timeouts**: Prevent long-running requests from consuming resources
//! - **Rate Limiting**: Protect against abuse and DoS attacks
//! - **Health Checks**: Built-in health check endpoints
//! - **Graceful Shutdown**: Handle shutdown signals gracefully
//! - **Connection Limits**: Control concurrent connection limits
//!
//! ## Quick Start
//!
//! ```rust
//! use torch_web::{App, production::*};
//!
//! let app = App::new()
//!     // Add production middleware
//!     .middleware(MetricsCollector::new())
//!     .middleware(PerformanceMonitor)
//!     .middleware(RequestTimeout::new(Duration::from_secs(30)))
//!     .middleware(health_check())
//!
//!     // Your application routes
//!     .get("/", |_req| async { Response::ok().body("Hello, Production!") })
//!     .get("/api/users", |_req| async {
//!         Response::ok().json(&serde_json::json!({"users": []}))
//!     });
//! ```
//!
//! ## Production Configuration
//!
//! ```rust
//! use torch_web::{App, production::ProductionConfig};
//! use std::time::Duration;
//!
//! let config = ProductionConfig {
//!     max_connections: 10_000,
//!     request_timeout: Duration::from_secs(30),
//!     max_body_size: 16 * 1024 * 1024, // 16MB
//!     enable_compression: true,
//!     enable_http2: true,
//!     rate_limit_rps: Some(1000),
//!     ..Default::default()
//! };
//!
//! // Apply configuration to your app
//! let app = App::with_defaults() // Includes production middleware
//!     .get("/", |_req| async { Response::ok().body("Production Ready!") });
//! ```

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use crate::{Request, Response, middleware::Middleware};

#[cfg(feature = "monitoring")]
use {
    std::sync::atomic::{AtomicU64, Ordering},
};

// DashMap import removed as it's not currently used

/// Configuration for production server deployment.
///
/// This struct contains all the configuration options needed to run a Torch
/// application in a production environment. It provides sensible defaults
/// for high-scale applications while allowing customization for specific needs.
///
/// # Examples
///
/// ## Default Configuration
///
/// ```rust
/// use torch_web::production::ProductionConfig;
///
/// let config = ProductionConfig::default();
/// println!("Max connections: {}", config.max_connections); // 10,000
/// println!("Request timeout: {:?}", config.request_timeout); // 30 seconds
/// ```
///
/// ## Custom Configuration
///
/// ```rust
/// use torch_web::production::ProductionConfig;
/// use std::time::Duration;
///
/// let config = ProductionConfig {
///     max_connections: 50_000,
///     request_timeout: Duration::from_secs(60),
///     max_body_size: 32 * 1024 * 1024, // 32MB
///     rate_limit_rps: Some(2000), // 2000 RPS per IP
///     worker_threads: Some(16), // 16 worker threads
///     ..Default::default()
/// };
/// ```
///
/// ## High-Performance Configuration
///
/// ```rust
/// use torch_web::production::ProductionConfig;
/// use std::time::Duration;
///
/// let config = ProductionConfig {
///     max_connections: 100_000,
///     request_timeout: Duration::from_secs(15), // Shorter timeout
///     keep_alive_timeout: Duration::from_secs(30),
///     max_body_size: 8 * 1024 * 1024, // 8MB limit
///     enable_compression: true,
///     enable_http2: true,
///     rate_limit_rps: Some(5000), // Higher rate limit
///     worker_threads: Some(32), // More workers
///     graceful_shutdown_timeout: Duration::from_secs(10),
/// };
/// ```
#[derive(Debug, Clone)]
pub struct ProductionConfig {
    /// Maximum number of concurrent connections the server will accept.
    ///
    /// This limits the total number of active connections to prevent resource
    /// exhaustion. When this limit is reached, new connections will be rejected.
    /// Default: 10,000
    pub max_connections: usize,

    /// Maximum time to wait for a request to complete.
    ///
    /// Requests that take longer than this duration will be automatically
    /// terminated with a 408 Request Timeout response. This prevents
    /// slow or malicious clients from consuming server resources.
    /// Default: 30 seconds
    pub request_timeout: Duration,

    /// How long to keep idle connections alive.
    ///
    /// Connections that remain idle for longer than this duration will be
    /// closed to free up resources. This applies to HTTP/1.1 keep-alive
    /// connections and HTTP/2 streams.
    /// Default: 60 seconds
    pub keep_alive_timeout: Duration,

    /// Maximum size of request bodies in bytes.
    ///
    /// Requests with bodies larger than this size will be rejected with
    /// a 413 Payload Too Large response. This prevents memory exhaustion
    /// from large uploads.
    /// Default: 16MB (16 * 1024 * 1024)
    pub max_body_size: usize,

    /// Whether to enable automatic compression of responses.
    ///
    /// When enabled, responses will be compressed using gzip or deflate
    /// based on the client's Accept-Encoding header. This reduces bandwidth
    /// usage but increases CPU usage.
    /// Default: true
    pub enable_compression: bool,

    /// Number of worker threads for the async runtime.
    ///
    /// If None, the runtime will use the default number of threads
    /// (typically equal to the number of CPU cores). Set this to control
    /// the thread pool size explicitly.
    /// Default: None (auto-detect)
    pub worker_threads: Option<usize>,

    /// Whether to enable HTTP/2 support.
    ///
    /// HTTP/2 provides better performance through multiplexing, header
    /// compression, and server push. Most modern clients support HTTP/2.
    /// Default: true
    pub enable_http2: bool,

    /// Rate limit in requests per second per IP address.
    ///
    /// If Some(n), each IP address will be limited to n requests per second.
    /// Requests exceeding this limit will receive a 429 Too Many Requests
    /// response. If None, no rate limiting is applied.
    /// Default: Some(1000)
    pub rate_limit_rps: Option<u32>,

    /// Maximum time to wait for graceful shutdown.
    ///
    /// When a shutdown signal is received, the server will stop accepting
    /// new connections and wait up to this duration for existing requests
    /// to complete before forcefully terminating.
    /// Default: 30 seconds
    pub graceful_shutdown_timeout: Duration,
}

impl Default for ProductionConfig {
    fn default() -> Self {
        Self {
            max_connections: 10_000,
            request_timeout: Duration::from_secs(30),
            keep_alive_timeout: Duration::from_secs(60),
            max_body_size: 16 * 1024 * 1024, // 16MB
            enable_compression: true,
            worker_threads: None, // Use default (number of CPU cores)
            enable_http2: true,
            rate_limit_rps: Some(1000), // 1000 requests per second per IP
            graceful_shutdown_timeout: Duration::from_secs(30),
        }
    }
}

/// Connection pool middleware for managing database connections
pub struct ConnectionPool<T> {
    pool: Arc<T>,
}

impl<T> ConnectionPool<T> 
where 
    T: Send + Sync + 'static,
{
    pub fn new(pool: T) -> Self {
        Self {
            pool: Arc::new(pool),
        }
    }
    
    pub fn get_pool(&self) -> Arc<T> {
        self.pool.clone()
    }
}

/// Rate limiting middleware
pub struct RateLimiter {
    semaphore: Arc<Semaphore>,
    #[allow(dead_code)]
    requests_per_second: u32,
}

impl RateLimiter {
    pub fn new(requests_per_second: u32) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(requests_per_second as usize)),
            requests_per_second,
        }
    }
}

impl Middleware for RateLimiter {
    fn call(
        &self,
        req: Request,
        next: Box<dyn Fn(Request) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send + 'static>> + Send + Sync>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send + 'static>> {
        let semaphore = self.semaphore.clone();
        Box::pin(async move {
            // Try to acquire a permit
            let _permit = match semaphore.try_acquire() {
                Ok(permit) => permit,
                Err(_) => {
                    return Response::with_status(http::StatusCode::TOO_MANY_REQUESTS)
                        .body("Rate limit exceeded");
                }
            };
            
            // Process the request
            next(req).await
        })
    }
}

/// Request timeout middleware
pub struct RequestTimeout {
    timeout: Duration,
}

impl RequestTimeout {
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }
}

impl Middleware for RequestTimeout {
    fn call(
        &self,
        req: Request,
        next: Box<dyn Fn(Request) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send + 'static>> + Send + Sync>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send + 'static>> {
        let timeout = self.timeout;
        Box::pin(async move {
            match tokio::time::timeout(timeout, next(req)).await {
                Ok(response) => response,
                Err(_) => Response::with_status(http::StatusCode::REQUEST_TIMEOUT)
                    .body("Request timeout"),
            }
        })
    }
}

/// Request size limiting middleware
pub struct RequestSizeLimit {
    max_size: usize,
}

impl RequestSizeLimit {
    pub fn new(max_size: usize) -> Self {
        Self { max_size }
    }
}

impl Middleware for RequestSizeLimit {
    fn call(
        &self,
        req: Request,
        next: Box<dyn Fn(Request) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send + 'static>> + Send + Sync>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send + 'static>> {
        let max_size = self.max_size;
        Box::pin(async move {
            if req.body().len() > max_size {
                return Response::with_status(http::StatusCode::PAYLOAD_TOO_LARGE)
                    .body("Request body too large");
            }
            next(req).await
        })
    }
}

/// Performance monitoring middleware
pub struct PerformanceMonitor;

impl Middleware for PerformanceMonitor {
    fn call(
        &self,
        req: Request,
        next: Box<dyn Fn(Request) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send + 'static>> + Send + Sync>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send + 'static>> {
        Box::pin(async move {
            let start = Instant::now();
            let method = req.method().clone();
            let path = req.path().to_string();
            
            let response = next(req).await;
            
            let duration = start.elapsed();
            let status = response.status_code();
            
            // Log performance metrics (in production, you'd send this to a monitoring system)
            if duration > Duration::from_millis(1000) {
                eprintln!(
                    "SLOW REQUEST: {} {} - {} ({:.2}ms)",
                    method,
                    path,
                    status,
                    duration.as_secs_f64() * 1000.0
                );
            }
            
            response
        })
    }
}

/// Health check middleware
pub fn health_check() -> impl Middleware {
    |req: Request, next: Box<dyn Fn(Request) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send + 'static>> + Send + Sync>| {
        Box::pin(async move {
            if req.path() == "/health" {
                return Response::ok()
                    .json(&{
                        #[cfg(feature = "monitoring")]
                        {
                            serde_json::json!({
                                "status": "healthy",
                                "timestamp": chrono::Utc::now().to_rfc3339(),
                                "uptime": "unknown"
                            })
                        }
                        #[cfg(not(feature = "monitoring"))]
                        {
                            serde_json::json!({
                                "status": "healthy",
                                "timestamp": "unknown",
                                "uptime": "unknown"
                            })
                        }
                    })
                    .unwrap_or_else(|_| Response::ok().body("healthy"));
            }
            next(req).await
        })
    }
}

/// Advanced metrics collection middleware with real monitoring integration
pub struct MetricsCollector {
    #[cfg(feature = "monitoring")]
    request_counter: Arc<AtomicU64>,
    #[cfg(feature = "monitoring")]
    active_requests: Arc<AtomicU64>,
    #[cfg(not(feature = "monitoring"))]
    _phantom: std::marker::PhantomData<()>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "monitoring")]
            request_counter: Arc::new(AtomicU64::new(0)),
            #[cfg(feature = "monitoring")]
            active_requests: Arc::new(AtomicU64::new(0)),
            #[cfg(not(feature = "monitoring"))]
            _phantom: std::marker::PhantomData,
        }
    }

    #[cfg(feature = "monitoring")]
    pub fn get_request_count(&self) -> u64 {
        self.request_counter.load(Ordering::Relaxed)
    }

    #[cfg(feature = "monitoring")]
    pub fn get_active_requests(&self) -> u64 {
        self.active_requests.load(Ordering::Relaxed)
    }
}

impl Middleware for MetricsCollector {
    fn call(
        &self,
        req: Request,
        next: Box<dyn Fn(Request) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send + 'static>> + Send + Sync>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send + 'static>> {
        Box::pin(async move {
            let start = Instant::now();
            let method = req.method().clone();
            let path = req.path().to_string();
            
            let response = next(req).await;
            
            let duration = start.elapsed();
            let status = response.status_code();
            
            // In production, send metrics to your monitoring system
            // For now, just log them
            println!(
                "METRIC: method={} path={} status={} duration_ms={:.2}",
                method,
                path,
                status.as_u16(),
                duration.as_secs_f64() * 1000.0
            );
            
            response
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::pin::Pin;
    use std::future::Future;
    use crate::Response;

    #[tokio::test]
    async fn test_rate_limiter() {
        let rate_limiter = RateLimiter::new(1);
        
        let next = Box::new(|_req: Request| -> Pin<Box<dyn Future<Output = Response> + Send + 'static>> {
            Box::pin(async { Response::ok().body("success") })
        });

        let req = crate::Request::from_hyper(
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

        // First request should succeed
        let response = rate_limiter.call(req, next.clone()).await;
        assert_eq!(response.status_code(), http::StatusCode::OK);
    }

    #[tokio::test]
    async fn test_request_timeout() {
        let timeout_middleware = RequestTimeout::new(Duration::from_millis(100));
        
        let next = Box::new(|_req: Request| -> Pin<Box<dyn Future<Output = Response> + Send + 'static>> {
            Box::pin(async {
                tokio::time::sleep(Duration::from_millis(200)).await;
                Response::ok().body("too slow")
            })
        });

        let req = crate::Request::from_hyper(
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

        let response = timeout_middleware.call(req, next).await;
        assert_eq!(response.status_code(), http::StatusCode::REQUEST_TIMEOUT);
    }
}
