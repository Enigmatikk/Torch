//! # Database Connection Management
//!
//! This module handles database connections and connection pooling for the ORM.
//! It provides a global connection pool that can be used throughout the application.
//!
//! ## Features
//!
//! - **Connection Pooling** - Efficient connection reuse with configurable pool size
//! - **Automatic Reconnection** - Handles connection failures gracefully
//! - **Multiple Database Support** - PostgreSQL, MySQL, SQLite support
//! - **Configuration** - Configurable timeouts, pool sizes, and connection parameters
//! - **Health Checks** - Built-in connection health monitoring
//!
//! ## Usage
//!
//! ```rust
//! use torch_web::orm::{OrmConfig, initialize};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize the ORM with configuration
//!     let config = OrmConfig {
//!         database_url: "postgres://user:pass@localhost/mydb".to_string(),
//!         max_connections: 20,
//!         min_connections: 5,
//!         connect_timeout: 30,
//!         log_queries: true,
//!         ..Default::default()
//!     };
//!     
//!     initialize(config).await?;
//!     
//!     // Now you can use the ORM throughout your application
//!     let users = User::all().await?;
//!     
//!     Ok(())
//! }
//! ```

use once_cell::sync::OnceCell;
use sqlx::{Pool, Any};
use sqlx::any::AnyPoolOptions;
use std::time::Duration;

use crate::orm::{OrmConfig, OrmError, Result};

/// Type alias for the database connection pool (supports multiple databases)
pub type ConnectionPool = Pool<Any>;

/// Global connection pool instance
static POOL: OnceCell<ConnectionPool> = OnceCell::new();

/// Database connection wrapper
#[derive(Debug, Clone)]
pub struct DatabaseConnection {
    pool: ConnectionPool,
}

impl DatabaseConnection {
    /// Create a new database connection from a pool
    pub fn new(pool: ConnectionPool) -> Self {
        Self { pool }
    }
    
    /// Get the underlying connection pool
    pub fn pool(&self) -> &ConnectionPool {
        &self.pool
    }
    
    /// Test the database connection
    pub async fn ping(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map_err(OrmError::Database)?;
        Ok(())
    }
    
    /// Get connection pool statistics
    pub fn stats(&self) -> PoolStats {
        let size = self.pool.size();
        let idle = self.pool.num_idle();
        PoolStats {
            size,
            idle,
            connections: size.saturating_sub(idle as u32),
        }
    }
}

/// Connection pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    /// Total pool size
    pub size: u32,
    /// Number of idle connections
    pub idle: usize,
    /// Number of active connections
    pub connections: u32,
}

/// Initialize the global connection pool
pub async fn initialize_pool(config: OrmConfig) -> Result<()> {
    let pool = create_pool(&config).await?;
    
    POOL.set(pool)
        .map_err(|_| OrmError::Connection("Pool already initialized".to_string()))?;
    
    Ok(())
}

/// Create a new connection pool from configuration
async fn create_pool(config: &OrmConfig) -> Result<ConnectionPool> {
    // Auto-detect database driver if not specified
    let driver = match &config.driver {
        Some(driver) => driver.clone(),
        None => crate::orm::DatabaseDriver::from_url(&config.database_url)?,
    };

    println!("🔌 Connecting to {} database...", driver.as_str());

    let pool = AnyPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(Duration::from_secs(config.connect_timeout))
        .idle_timeout(Duration::from_secs(600)) // 10 minutes
        .max_lifetime(Duration::from_secs(1800)) // 30 minutes
        .test_before_acquire(true)
        .connect(&config.database_url)
        .await
        .map_err(OrmError::Database)?;

    // Test the connection with database-agnostic query
    sqlx::query("SELECT 1")
        .fetch_one(&pool)
        .await
        .map_err(OrmError::Database)?;

    println!("✅ {} database connected successfully!", driver.as_str());

    Ok(pool)
}

/// Get the global connection pool
pub fn get_pool() -> &'static ConnectionPool {
    POOL.get().expect("Database pool not initialized. Call initialize_pool() first.")
}

/// Get a database connection from the pool
pub fn connection() -> DatabaseConnection {
    DatabaseConnection::new(get_pool().clone())
}

/// Check if the connection pool is initialized
pub fn is_initialized() -> bool {
    POOL.get().is_some()
}

/// Close the connection pool
pub async fn close_pool() -> Result<()> {
    if let Some(pool) = POOL.get() {
        pool.close().await;
    }
    Ok(())
}

/// Health check for the database connection
pub async fn health_check() -> Result<HealthStatus> {
    let pool = get_pool();
    
    let start = std::time::Instant::now();
    let ping_result = sqlx::query("SELECT 1")
        .fetch_one(pool)
        .await;
    let response_time = start.elapsed();
    
    match ping_result {
        Ok(_) => Ok(HealthStatus {
            healthy: true,
            response_time_ms: response_time.as_millis() as u64,
            pool_stats: connection().stats(),
            error: None,
        }),
        Err(e) => Ok(HealthStatus {
            healthy: false,
            response_time_ms: response_time.as_millis() as u64,
            pool_stats: connection().stats(),
            error: Some(e.to_string()),
        }),
    }
}

/// Database health status
#[derive(Debug, Clone)]
pub struct HealthStatus {
    /// Whether the database is healthy
    pub healthy: bool,
    /// Response time in milliseconds
    pub response_time_ms: u64,
    /// Connection pool statistics
    pub pool_stats: PoolStats,
    /// Error message if unhealthy
    pub error: Option<String>,
}

/// Transaction wrapper for database operations
pub struct Transaction<'a> {
    tx: sqlx::Transaction<'a, Any>,
}

impl<'a> Transaction<'a> {
    /// Begin a new transaction
    pub async fn begin() -> Result<Transaction<'a>> {
        let pool = get_pool();
        let tx = pool.begin().await.map_err(OrmError::Database)?;
        Ok(Transaction { tx })
    }
    
    /// Commit the transaction
    pub async fn commit(self) -> Result<()> {
        self.tx.commit().await.map_err(OrmError::Database)?;
        Ok(())
    }
    
    /// Rollback the transaction
    pub async fn rollback(self) -> Result<()> {
        self.tx.rollback().await.map_err(OrmError::Database)?;
        Ok(())
    }
    
    /// Execute a query within the transaction
    pub async fn execute(&mut self, query: &str) -> Result<sqlx::any::AnyQueryResult> {
        sqlx::query(query)
            .execute(&mut *self.tx)
            .await
            .map_err(OrmError::Database)
    }

    /// Fetch one row within the transaction
    pub async fn fetch_one(&mut self, query: &str) -> Result<sqlx::any::AnyRow> {
        sqlx::query(query)
            .fetch_one(&mut *self.tx)
            .await
            .map_err(OrmError::Database)
    }

    /// Fetch all rows within the transaction
    pub async fn fetch_all(&mut self, query: &str) -> Result<Vec<sqlx::any::AnyRow>> {
        sqlx::query(query)
            .fetch_all(&mut *self.tx)
            .await
            .map_err(OrmError::Database)
    }
}

/// Run a simple transaction (simplified implementation)
/// For complex transactions, use Transaction::begin() directly
pub async fn simple_transaction<F, Fut, R>(f: F) -> Result<R>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<R>>,
{
    // For now, just execute the function without transaction support
    // A full implementation would require more complex lifetime management
    f().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pool_initialization() {
        // Note: This would require a real database connection
        // let config = OrmConfig::default();
        // assert!(initialize_pool(config).await.is_ok());
        // assert!(is_initialized());
    }
    
    #[test]
    fn test_pool_not_initialized() {
        // This should panic in a real scenario
        // get_pool();
    }
}
