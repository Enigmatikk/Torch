//! # High-Performance Caching
//!
//! This module provides comprehensive caching solutions for Torch applications,
//! including both in-memory and Redis-based caching. It supports response caching,
//! data caching, and cache middleware for automatic request/response caching.
//!
//! ## Features
//!
//! - **In-Memory Cache**: Fast, local caching with TTL support
//! - **Redis Cache**: Distributed caching with Redis backend
//! - **Response Caching**: Automatic HTTP response caching middleware
//! - **TTL Support**: Time-to-live expiration for cache entries
//! - **Cache Invalidation**: Manual and automatic cache invalidation
//! - **Serialization**: JSON serialization for complex data types
//!
//! ## Cache Types
//!
//! ### In-Memory Cache
//! Fast, local caching that stores data in application memory. Best for:
//! - Single-instance applications
//! - Frequently accessed data
//! - Low-latency requirements
//!
//! ### Redis Cache
//! Distributed caching using Redis as the backend. Best for:
//! - Multi-instance applications
//! - Shared cache across services
//! - Persistent caching
//! - Large cache sizes
//!
//! ## Examples
//!
//! ### Basic In-Memory Caching
//!
//! ```rust
//! use torch_web::{App, cache::MemoryCache};
//! use std::time::Duration;
//!
//! let cache = MemoryCache::new(Some(Duration::from_secs(300))); // 5 minute TTL
//!
//! let app = App::new()
//!     .with_state(cache)
//!     .get("/data/:id", |Path(id): Path<u32>, State(cache): State<MemoryCache>| async move {
//!         let cache_key = format!("data:{}", id);
//!
//!         // Try to get from cache first
//!         if let Some(cached_data) = cache.get(&cache_key).await {
//!             return Response::ok()
//!                 .header("X-Cache", "HIT")
//!                 .body(cached_data);
//!         }
//!
//!         // Fetch data (expensive operation)
//!         let data = fetch_data_from_database(id).await;
//!
//!         // Cache the result
//!         cache.set(&cache_key, &data, None).await;
//!
//!         Response::ok()
//!             .header("X-Cache", "MISS")
//!             .body(data)
//!     });
//! ```
//!
//! ### Redis Caching
//!
//! ```rust
//! use torch_web::{App, cache::RedisCache};
//!
//! let cache = RedisCache::new("redis://localhost:6379").await?;
//!
//! let app = App::new()
//!     .with_state(cache)
//!     .get("/users/:id", |Path(id): Path<u32>, State(cache): State<RedisCache>| async move {
//!         let cache_key = format!("user:{}", id);
//!
//!         if let Some(user_json) = cache.get(&cache_key).await? {
//!             return Response::ok()
//!                 .header("Content-Type", "application/json")
//!                 .header("X-Cache", "HIT")
//!                 .body(user_json);
//!         }
//!
//!         let user = get_user_from_db(id).await?;
//!         let user_json = serde_json::to_string(&user)?;
//!
//!         // Cache for 1 hour
//!         cache.set(&cache_key, &user_json, Some(Duration::from_secs(3600))).await?;
//!
//!         Response::ok()
//!             .header("Content-Type", "application/json")
//!             .header("X-Cache", "MISS")
//!             .body(user_json)
//!     });
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use crate::{Request, Response, middleware::Middleware};

#[cfg(feature = "cache")]
use redis::{Client, Commands};

#[cfg(feature = "json")]
use serde::{Serialize, Deserialize};

/// Cached response structure for serialization
#[cfg(feature = "json")]
#[derive(Serialize, Deserialize)]
struct CachedResponse {
    status_code: u16,
    headers: HashMap<String, String>,
    body: String,
}

/// Internal cache entry with expiration tracking.
///
/// This struct represents a single cache entry with its value and optional
/// expiration time. It's used internally by the cache implementations.
#[derive(Debug, Clone)]
struct CacheEntry {
    value: String,
    expires_at: Option<Instant>,
}

impl CacheEntry {
    /// Creates a new cache entry with optional TTL.
    fn new(value: String, ttl: Option<Duration>) -> Self {
        Self {
            value,
            expires_at: ttl.map(|duration| Instant::now() + duration),
        }
    }

    /// Checks if this cache entry has expired.
    fn is_expired(&self) -> bool {
        self.expires_at.map_or(false, |expires_at| Instant::now() > expires_at)
    }
}

/// High-performance in-memory cache implementation.
///
/// `MemoryCache` provides fast, local caching with automatic expiration support.
/// It's ideal for single-instance applications or when you need very low latency
/// cache access. The cache is thread-safe and supports concurrent reads and writes.
///
/// # Features
///
/// - **Thread-safe**: Safe for concurrent access from multiple threads
/// - **TTL Support**: Automatic expiration of cache entries
/// - **Memory efficient**: Expired entries are cleaned up automatically
/// - **Fast access**: O(1) average case for get/set operations
/// - **Flexible TTL**: Per-entry TTL or default TTL for all entries
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use torch_web::cache::MemoryCache;
/// use std::time::Duration;
///
/// let cache = MemoryCache::new(Some(Duration::from_secs(300))); // 5 minute default TTL
///
/// // Set a value with default TTL
/// cache.set("user:123", "John Doe", None).await;
///
/// // Set a value with custom TTL
/// cache.set("session:abc", "active", Some(Duration::from_secs(3600))).await;
///
/// // Get a value
/// if let Some(name) = cache.get("user:123").await {
///     println!("User name: {}", name);
/// }
/// ```
///
/// ## With JSON Serialization
///
/// ```rust
/// use torch_web::cache::MemoryCache;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct User {
///     id: u32,
///     name: String,
///     email: String,
/// }
///
/// let cache = MemoryCache::new(None); // No default TTL
///
/// let user = User {
///     id: 123,
///     name: "John Doe".to_string(),
///     email: "john@example.com".to_string(),
/// };
///
/// // Serialize and cache
/// let user_json = serde_json::to_string(&user)?;
/// cache.set("user:123", &user_json, Some(Duration::from_secs(3600))).await;
///
/// // Retrieve and deserialize
/// if let Some(cached_json) = cache.get("user:123").await {
///     let cached_user: User = serde_json::from_str(&cached_json)?;
///     println!("Cached user: {}", cached_user.name);
/// }
/// ```
///
/// ## Cache Invalidation
///
/// ```rust
/// use torch_web::cache::MemoryCache;
///
/// let cache = MemoryCache::new(None);
///
/// // Set some values
/// cache.set("key1", "value1", None).await;
/// cache.set("key2", "value2", None).await;
///
/// // Remove a specific key
/// cache.delete("key1").await;
///
/// // Clear all entries
/// cache.clear().await;
/// ```
pub struct MemoryCache {
    store: Arc<RwLock<HashMap<String, CacheEntry>>>,
    default_ttl: Option<Duration>,
}

impl MemoryCache {
    pub fn new(default_ttl: Option<Duration>) -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            default_ttl,
        }
    }

    pub async fn get(&self, key: &str) -> Option<String> {
        let store = self.store.read().await;
        if let Some(entry) = store.get(key) {
            if !entry.is_expired() {
                return Some(entry.value.clone());
            }
        }
        None
    }

    pub async fn set(&self, key: &str, value: &str, ttl: Option<Duration>) -> Result<(), Box<dyn std::error::Error>> {
        let mut store = self.store.write().await;
        let ttl = ttl.or(self.default_ttl);
        store.insert(key.to_string(), CacheEntry::new(value.to_string(), ttl));
        Ok(())
    }

    pub async fn delete(&self, key: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let mut store = self.store.write().await;
        Ok(store.remove(key).is_some())
    }

    pub async fn clear(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut store = self.store.write().await;
        store.clear();
        Ok(())
    }

    pub async fn cleanup_expired(&self) -> Result<usize, Box<dyn std::error::Error>> {
        let mut store = self.store.write().await;
        let initial_size = store.len();
        store.retain(|_, entry| !entry.is_expired());
        Ok(initial_size - store.len())
    }

    pub async fn size(&self) -> usize {
        self.store.read().await.len()
    }
}

/// Redis cache implementation
pub struct RedisCache {
    #[cfg(feature = "cache")]
    client: Client,
    #[allow(dead_code)]
    default_ttl: Option<Duration>,
    #[cfg(not(feature = "cache"))]
    _phantom: std::marker::PhantomData<()>,
}

impl RedisCache {
    #[cfg(feature = "cache")]
    pub fn new(redis_url: &str, default_ttl: Option<Duration>) -> Result<Self, redis::RedisError> {
        let client = Client::open(redis_url)?;
        Ok(Self {
            client,
            default_ttl,
        })
    }

    #[cfg(not(feature = "cache"))]
    pub fn new(_redis_url: &str, default_ttl: Option<Duration>) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            default_ttl,
            _phantom: std::marker::PhantomData,
        })
    }

    #[cfg(feature = "cache")]
    pub async fn get(&self, key: &str) -> Result<Option<String>, redis::RedisError> {
        let mut conn = self.client.get_connection()?;
        let result: Option<String> = conn.get(key)?;
        Ok(result)
    }

    #[cfg(not(feature = "cache"))]
    pub async fn get(&self, _key: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
        Err("Redis cache feature not enabled".into())
    }

    #[cfg(feature = "cache")]
    pub async fn set(&self, key: &str, value: &str, ttl: Option<Duration>) -> Result<(), redis::RedisError> {
        let mut conn = self.client.get_connection()?;
        if let Some(ttl) = ttl.or(self.default_ttl) {
            conn.set_ex::<_, _, ()>(key, value, ttl.as_secs())?;
        } else {
            conn.set::<_, _, ()>(key, value)?;
        }
        Ok(())
    }

    #[cfg(not(feature = "cache"))]
    pub async fn set(&self, _key: &str, _value: &str, _ttl: Option<Duration>) -> Result<(), Box<dyn std::error::Error>> {
        Err("Redis cache feature not enabled".into())
    }

    #[cfg(feature = "cache")]
    pub async fn delete(&self, key: &str) -> Result<bool, redis::RedisError> {
        let mut conn = self.client.get_connection()?;
        let result: i32 = conn.del(key)?;
        Ok(result > 0)
    }

    #[cfg(not(feature = "cache"))]
    pub async fn delete(&self, _key: &str) -> Result<bool, Box<dyn std::error::Error>> {
        Err("Redis cache feature not enabled".into())
    }
}

/// Cache trait for unified interface
pub trait Cache: Send + Sync {
    fn get(&self, key: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Option<String>> + Send + '_>>;
    fn set(&self, key: &str, value: &str, ttl: Option<Duration>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), Box<dyn std::error::Error>>> + Send + '_>>;
    fn delete(&self, key: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool, Box<dyn std::error::Error>>> + Send + '_>>;
}

impl Cache for MemoryCache {
    fn get(&self, key: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Option<String>> + Send + '_>> {
        let key = key.to_string();
        Box::pin(async move { self.get(&key).await })
    }

    fn set(&self, key: &str, value: &str, ttl: Option<Duration>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), Box<dyn std::error::Error>>> + Send + '_>> {
        let key = key.to_string();
        let value = value.to_string();
        Box::pin(async move { self.set(&key, &value, ttl).await })
    }

    fn delete(&self, key: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool, Box<dyn std::error::Error>>> + Send + '_>> {
        let key = key.to_string();
        Box::pin(async move { self.delete(&key).await })
    }
}

/// Response caching middleware
pub struct CacheMiddleware {
    cache: Arc<dyn Cache>,
    cache_duration: Duration,
    cache_key_prefix: String,
}

impl CacheMiddleware {
    pub fn new(cache: Arc<dyn Cache>, cache_duration: Duration) -> Self {
        Self {
            cache,
            cache_duration,
            cache_key_prefix: "torch_cache:".to_string(),
        }
    }

    pub fn with_prefix(mut self, prefix: &str) -> Self {
        self.cache_key_prefix = prefix.to_string();
        self
    }

    fn generate_cache_key(&self, req: &Request) -> String {
        format!("{}{}:{}", self.cache_key_prefix, req.method(), req.path())
    }
}

impl Middleware for CacheMiddleware {
    fn call(
        &self,
        req: Request,
        next: Box<dyn Fn(Request) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send + 'static>> + Send + Sync>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send + 'static>> {
        let cache = self.cache.clone();
        let cache_duration = self.cache_duration;
        let cache_key = self.generate_cache_key(&req);

        Box::pin(async move {
            let is_get_request = req.method() == &http::Method::GET;

            // Only cache GET requests
            if is_get_request {
                // Try to get from cache first
                if let Some(cached_data) = cache.get(&cache_key).await {
                    #[cfg(feature = "json")]
                    {
                        // Parse cached response data
                        if let Ok(cached_response) = serde_json::from_str::<CachedResponse>(&cached_data) {
                            let mut response = Response::with_status(
                                http::StatusCode::from_u16(cached_response.status_code).unwrap_or(http::StatusCode::OK)
                            ).body(cached_response.body);

                            // Restore headers
                            for (name, value) in cached_response.headers {
                                response = response.header(&name, &value);
                            }

                            return response.header("X-Cache", "HIT");
                        }
                    }

                    #[cfg(not(feature = "json"))]
                    {
                        // Simple string caching when JSON feature is not available
                        return Response::ok()
                            .header("X-Cache", "HIT")
                            .body(cached_data);
                    }
                }
            }

            // Execute the request
            let response = next(req).await;

            // Cache successful GET responses
            if is_get_request && response.status_code().is_success() {
                #[cfg(feature = "json")]
                {
                    let cached_response = CachedResponse {
                        status_code: response.status_code().as_u16(),
                        headers: response.headers().iter()
                            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                            .collect(),
                        body: String::from_utf8_lossy(response.body_data()).to_string(),
                    };

                    if let Ok(serialized) = serde_json::to_string(&cached_response) {
                        if let Err(e) = cache.set(&cache_key, &serialized, Some(cache_duration)).await {
                            eprintln!("Failed to cache response: {}", e);
                        }
                    }
                }

                #[cfg(not(feature = "json"))]
                {
                    // Simple string caching when JSON feature is not available
                    let response_body = String::from_utf8_lossy(response.body_data());
                    if let Err(e) = cache.set(&cache_key, &response_body, Some(cache_duration)).await {
                        eprintln!("Failed to cache response: {}", e);
                    }
                }
            }

            response.header("X-Cache", "MISS")
        })
    }
}

/// Cache warming utility
pub struct CacheWarmer {
    cache: Arc<dyn Cache>,
}

impl CacheWarmer {
    pub fn new(cache: Arc<dyn Cache>) -> Self {
        Self { cache }
    }

    /// Warm the cache with predefined data
    pub async fn warm_cache(&self, data: HashMap<String, String>) -> Result<usize, Box<dyn std::error::Error>> {
        let mut warmed_count = 0;
        
        for (key, value) in data {
            if let Err(e) = self.cache.set(&key, &value, None).await {
                eprintln!("Failed to warm cache for key {}: {}", key, e);
            } else {
                warmed_count += 1;
            }
        }
        
        Ok(warmed_count)
    }

    /// Preload cache from database or external source
    pub async fn preload_from_source<F, Fut>(&self, loader: F) -> Result<usize, Box<dyn std::error::Error>>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<HashMap<String, String>, Box<dyn std::error::Error>>>,
    {
        let data = loader().await?;
        self.warm_cache(data).await
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub sets: u64,
    pub deletes: u64,
    pub errors: u64,
}

impl CacheStats {
    pub fn new() -> Self {
        Self {
            hits: 0,
            misses: 0,
            sets: 0,
            deletes: 0,
            errors: 0,
        }
    }

    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_cache() {
        let cache = MemoryCache::new(Some(Duration::from_secs(60)));
        
        // Test set and get
        cache.set("key1", "value1", None).await.unwrap();
        assert_eq!(cache.get("key1").await, Some("value1".to_string()));
        
        // Test non-existent key
        assert_eq!(cache.get("nonexistent").await, None);
        
        // Test delete
        assert!(cache.delete("key1").await.unwrap());
        assert_eq!(cache.get("key1").await, None);
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let cache = MemoryCache::new(None);
        
        // Set with short TTL
        cache.set("key1", "value1", Some(Duration::from_millis(10))).await.unwrap();
        assert_eq!(cache.get("key1").await, Some("value1".to_string()));
        
        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert_eq!(cache.get("key1").await, None);
    }

    #[tokio::test]
    async fn test_cache_cleanup() {
        let cache = MemoryCache::new(None);
        
        // Add expired entries
        cache.set("key1", "value1", Some(Duration::from_millis(1))).await.unwrap();
        cache.set("key2", "value2", Some(Duration::from_millis(1))).await.unwrap();
        cache.set("key3", "value3", None).await.unwrap(); // No expiration
        
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        let cleaned = cache.cleanup_expired().await.unwrap();
        assert_eq!(cleaned, 2);
        assert_eq!(cache.size().await, 1);
    }

    #[test]
    fn test_cache_stats() {
        let mut stats = CacheStats::new();
        stats.hits = 80;
        stats.misses = 20;
        
        assert_eq!(stats.hit_rate(), 0.8);
    }
}
