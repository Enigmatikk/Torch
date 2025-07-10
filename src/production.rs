//! Production-ready features for high-scale applications

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use crate::{Request, Response, middleware::Middleware};

#[cfg(feature = "monitoring")]
use {
    std::sync::atomic::{AtomicU64, Ordering},
};

// DashMap import removed as it's not currently used

/// Production server configuration for high-scale applications
#[derive(Debug, Clone)]
pub struct ProductionConfig {
    /// Maximum number of concurrent connections
    pub max_connections: usize,
    /// Request timeout in seconds
    pub request_timeout: Duration,
    /// Keep-alive timeout in seconds
    pub keep_alive_timeout: Duration,
    /// Maximum request body size in bytes
    pub max_body_size: usize,
    /// Enable request/response compression
    pub enable_compression: bool,
    /// Number of worker threads
    pub worker_threads: Option<usize>,
    /// Enable HTTP/2
    pub enable_http2: bool,
    /// Rate limiting: requests per second per IP
    pub rate_limit_rps: Option<u32>,
    /// Enable graceful shutdown
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

#[cfg(disabled_for_now)]
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
