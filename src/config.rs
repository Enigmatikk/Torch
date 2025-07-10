//! Configuration system for Torch framework

#[cfg(feature = "config")]
use std::path::Path;
use std::time::Duration;

#[cfg(feature = "config")]
use serde::{Deserialize, Serialize};

/// Main configuration structure for Torch applications
#[derive(Debug, Clone)]
#[cfg_attr(feature = "config", derive(Serialize, Deserialize))]
pub struct TorchConfig {
    /// Server configuration
    pub server: ServerConfig,
    /// Security configuration
    pub security: SecurityConfig,
    /// Monitoring and logging configuration
    pub monitoring: MonitoringConfig,
    /// Performance configuration
    pub performance: PerformanceConfig,
    /// Rate limiting configuration
    pub rate_limiting: RateLimitingConfig,
    /// Database configuration
    pub database: Option<DatabaseConfig>,
    /// Custom application settings
    #[cfg_attr(feature = "config", serde(default))]
    pub custom: std::collections::HashMap<String, String>,
}

/// Server configuration
#[derive(Debug, Clone)]
#[cfg_attr(feature = "config", derive(Serialize, Deserialize))]
pub struct ServerConfig {
    /// Server host address
    pub host: String,
    /// Server port
    pub port: u16,
    /// Maximum number of concurrent connections
    pub max_connections: usize,
    /// Request timeout in seconds
    pub request_timeout_secs: u64,
    /// Keep-alive timeout in seconds
    pub keep_alive_timeout_secs: u64,
    /// Maximum request body size in bytes
    pub max_body_size: usize,
    /// Number of worker threads (None = auto-detect)
    pub worker_threads: Option<usize>,
    /// Enable HTTP/2
    pub enable_http2: bool,
    /// Enable TLS/SSL
    pub enable_tls: bool,
    /// TLS certificate file path
    pub tls_cert_path: Option<String>,
    /// TLS private key file path
    pub tls_key_path: Option<String>,
    /// Graceful shutdown timeout in seconds
    pub graceful_shutdown_timeout_secs: u64,
}

/// Security configuration
#[derive(Debug, Clone)]
#[cfg_attr(feature = "config", derive(Serialize, Deserialize))]
pub struct SecurityConfig {
    /// Enable CORS
    pub enable_cors: bool,
    /// CORS allowed origins
    pub cors_allowed_origins: Vec<String>,
    /// CORS allowed methods
    pub cors_allowed_methods: Vec<String>,
    /// CORS allowed headers
    pub cors_allowed_headers: Vec<String>,
    /// Enable security headers
    pub enable_security_headers: bool,
    /// Content Security Policy
    pub content_security_policy: Option<String>,
    /// Enable request signing verification
    pub enable_request_signing: bool,
    /// Secret key for request signing
    pub signing_secret: Option<String>,
    /// Enable IP whitelisting
    pub enable_ip_whitelist: bool,
    /// Whitelisted IP addresses/ranges
    pub ip_whitelist: Vec<String>,
    /// Enable request ID tracking
    pub enable_request_id: bool,
    /// Maximum request size for security
    pub max_request_size: usize,
    /// Enable input validation
    pub enable_input_validation: bool,
    /// SQL injection protection
    pub enable_sql_injection_protection: bool,
    /// XSS protection
    pub enable_xss_protection: bool,
}

/// Monitoring and logging configuration
#[derive(Debug, Clone)]
#[cfg_attr(feature = "config", derive(Serialize, Deserialize))]
pub struct MonitoringConfig {
    /// Enable request logging
    pub enable_request_logging: bool,
    /// Log level (error, warn, info, debug, trace)
    pub log_level: String,
    /// Log format (json, text)
    pub log_format: String,
    /// Log file path (None = stdout)
    pub log_file: Option<String>,
    /// Enable metrics collection
    pub enable_metrics: bool,
    /// Metrics endpoint path
    pub metrics_endpoint: String,
    /// Enable health check endpoint
    pub enable_health_check: bool,
    /// Health check endpoint path
    pub health_check_endpoint: String,
    /// Enable performance monitoring
    pub enable_performance_monitoring: bool,
    /// Slow request threshold in milliseconds
    pub slow_request_threshold_ms: u64,
    /// Enable error tracking
    pub enable_error_tracking: bool,
    /// Error tracking service URL
    pub error_tracking_url: Option<String>,
    /// Enable distributed tracing
    pub enable_distributed_tracing: bool,
    /// Tracing service endpoint
    pub tracing_endpoint: Option<String>,
}

/// Performance configuration
#[derive(Debug, Clone)]
#[cfg_attr(feature = "config", derive(Serialize, Deserialize))]
pub struct PerformanceConfig {
    /// Enable response compression
    pub enable_compression: bool,
    /// Compression level (1-9)
    pub compression_level: u8,
    /// Enable response caching
    pub enable_caching: bool,
    /// Cache TTL in seconds
    pub cache_ttl_secs: u64,
    /// Maximum cache size in MB
    pub max_cache_size_mb: usize,
    /// Enable connection pooling
    pub enable_connection_pooling: bool,
    /// Connection pool size
    pub connection_pool_size: usize,
    /// Connection pool timeout in seconds
    pub connection_pool_timeout_secs: u64,
    /// Enable keep-alive
    pub enable_keep_alive: bool,
    /// Keep-alive timeout in seconds
    pub keep_alive_timeout_secs: u64,
}

/// Rate limiting configuration
#[derive(Debug, Clone)]
#[cfg_attr(feature = "config", derive(Serialize, Deserialize))]
pub struct RateLimitingConfig {
    /// Enable rate limiting
    pub enable_rate_limiting: bool,
    /// Global requests per second limit
    pub global_rps_limit: Option<u32>,
    /// Per-IP requests per second limit
    pub per_ip_rps_limit: Option<u32>,
    /// Per-user requests per second limit
    pub per_user_rps_limit: Option<u32>,
    /// Rate limiting window in seconds
    pub window_secs: u64,
    /// Enable burst allowance
    pub enable_burst: bool,
    /// Burst size
    pub burst_size: u32,
    /// Rate limiting storage backend (memory, redis)
    pub storage_backend: String,
    /// Redis URL for distributed rate limiting
    pub redis_url: Option<String>,
}

/// Database configuration
#[derive(Debug, Clone)]
#[cfg_attr(feature = "config", derive(Serialize, Deserialize))]
pub struct DatabaseConfig {
    /// Database URL
    pub url: String,
    /// Maximum number of connections
    pub max_connections: u32,
    /// Minimum number of connections
    pub min_connections: u32,
    /// Connection timeout in seconds
    pub connect_timeout_secs: u64,
    /// Query timeout in seconds
    pub query_timeout_secs: u64,
    /// Enable connection pooling
    pub enable_pooling: bool,
    /// Enable query logging
    pub enable_query_logging: bool,
    /// Enable migrations
    pub enable_migrations: bool,
    /// Migrations directory
    pub migrations_dir: Option<String>,
}

impl Default for TorchConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            security: SecurityConfig::default(),
            monitoring: MonitoringConfig::default(),
            performance: PerformanceConfig::default(),
            rate_limiting: RateLimitingConfig::default(),
            database: None,
            custom: std::collections::HashMap::new(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
            max_connections: 10_000,
            request_timeout_secs: 30,
            keep_alive_timeout_secs: 60,
            max_body_size: 16 * 1024 * 1024, // 16MB
            worker_threads: None,
            enable_http2: true,
            enable_tls: false,
            tls_cert_path: None,
            tls_key_path: None,
            graceful_shutdown_timeout_secs: 30,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_cors: true,
            cors_allowed_origins: vec!["*".to_string()],
            cors_allowed_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
                "OPTIONS".to_string(),
            ],
            cors_allowed_headers: vec![
                "Content-Type".to_string(),
                "Authorization".to_string(),
                "X-Requested-With".to_string(),
            ],
            enable_security_headers: true,
            content_security_policy: Some(
                "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'"
                    .to_string(),
            ),
            enable_request_signing: false,
            signing_secret: None,
            enable_ip_whitelist: false,
            ip_whitelist: Vec::new(),
            enable_request_id: true,
            max_request_size: 16 * 1024 * 1024, // 16MB
            enable_input_validation: true,
            enable_sql_injection_protection: true,
            enable_xss_protection: true,
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enable_request_logging: true,
            log_level: "info".to_string(),
            log_format: "json".to_string(),
            log_file: None,
            enable_metrics: true,
            metrics_endpoint: "/metrics".to_string(),
            enable_health_check: true,
            health_check_endpoint: "/health".to_string(),
            enable_performance_monitoring: true,
            slow_request_threshold_ms: 1000,
            enable_error_tracking: true,
            error_tracking_url: None,
            enable_distributed_tracing: false,
            tracing_endpoint: None,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enable_compression: true,
            compression_level: 6,
            enable_caching: true,
            cache_ttl_secs: 300, // 5 minutes
            max_cache_size_mb: 100,
            enable_connection_pooling: true,
            connection_pool_size: 100,
            connection_pool_timeout_secs: 30,
            enable_keep_alive: true,
            keep_alive_timeout_secs: 60,
        }
    }
}

impl Default for RateLimitingConfig {
    fn default() -> Self {
        Self {
            enable_rate_limiting: true,
            global_rps_limit: Some(10_000),
            per_ip_rps_limit: Some(100),
            per_user_rps_limit: Some(1000),
            window_secs: 60,
            enable_burst: true,
            burst_size: 10,
            storage_backend: "memory".to_string(),
            redis_url: None,
        }
    }
}

impl TorchConfig {
    /// Load configuration from a TOML file
    #[cfg(feature = "config")]
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: TorchConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to a TOML file
    #[cfg(feature = "config")]
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();
        
        // Server configuration
        if let Ok(host) = std::env::var("TORCH_HOST") {
            config.server.host = host;
        }
        if let Ok(port) = std::env::var("TORCH_PORT") {
            if let Ok(port) = port.parse() {
                config.server.port = port;
            }
        }
        if let Ok(max_conn) = std::env::var("TORCH_MAX_CONNECTIONS") {
            if let Ok(max_conn) = max_conn.parse() {
                config.server.max_connections = max_conn;
            }
        }
        
        // Security configuration
        if let Ok(enable_cors) = std::env::var("TORCH_ENABLE_CORS") {
            config.security.enable_cors = enable_cors.parse().unwrap_or(true);
        }
        if let Ok(secret) = std::env::var("TORCH_SIGNING_SECRET") {
            config.security.signing_secret = Some(secret);
            config.security.enable_request_signing = true;
        }
        
        // Add more environment variable mappings as needed
        
        config
    }

    /// Get server address as string
    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }

    /// Get request timeout as Duration
    pub fn request_timeout(&self) -> Duration {
        Duration::from_secs(self.server.request_timeout_secs)
    }

    /// Get keep-alive timeout as Duration
    pub fn keep_alive_timeout(&self) -> Duration {
        Duration::from_secs(self.server.keep_alive_timeout_secs)
    }

    /// Get graceful shutdown timeout as Duration
    pub fn graceful_shutdown_timeout(&self) -> Duration {
        Duration::from_secs(self.server.graceful_shutdown_timeout_secs)
    }
}
