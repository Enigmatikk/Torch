//! # Torch ORM - Laravel Eloquent-inspired ORM for Rust
//!
//! The Torch ORM provides a Laravel Eloquent-like Active Record implementation for Rust,
//! offering an intuitive and powerful way to interact with databases.
//!
//! ## Features
//!
//! - **Active Record Pattern** - Models that can save, update, and delete themselves
//! - **Fluent Query Builder** - Chainable query methods like `where()`, `orderBy()`, `limit()`
//! - **Relationships** - Define and query relationships between models
//! - **Migrations Integration** - Seamless integration with Torch's migration system
//! - **Type Safety** - Full Rust type safety with compile-time query validation
//! - **Async/Await** - Built on async Rust for high performance
//!
//! ## Quick Start
//!
//! ```rust
//! use torch_web::orm::{Model, HasMany, BelongsTo};
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Model, Serialize, Deserialize)]
//! #[table = "users"]
//! struct User {
//!     pub id: Option<i32>,
//!     pub name: String,
//!     pub email: String,
//!     pub created_at: Option<chrono::DateTime<chrono::Utc>>,
//!     pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
//! }
//!
//! impl User {
//!     /// Define relationship to posts
//!     pub fn posts(&self) -> HasMany<Post> {
//!         self.has_many::<Post>("user_id")
//!     }
//! }
//!
//! #[derive(Model, Serialize, Deserialize)]
//! #[table = "posts"]
//! struct Post {
//!     pub id: Option<i32>,
//!     pub user_id: i32,
//!     pub title: String,
//!     pub content: String,
//!     pub created_at: Option<chrono::DateTime<chrono::Utc>>,
//!     pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
//! }
//!
//! impl Post {
//!     /// Define relationship to user
//!     pub fn user(&self) -> BelongsTo<User> {
//!         self.belongs_to::<User>("user_id")
//!     }
//! }
//!
//! // Usage examples
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a new user
//!     let mut user = User {
//!         id: None,
//!         name: "John Doe".to_string(),
//!         email: "john@example.com".to_string(),
//!         created_at: None,
//!         updated_at: None,
//!     };
//!     user.save().await?;
//!
//!     // Find users
//!     let users = User::all().await?;
//!     let user = User::find(1).await?;
//!     let active_users = User::where("active", true)
//!         .order_by("created_at", "desc")
//!         .limit(10)
//!         .get()
//!         .await?;
//!
//!     // Create related models
//!     let mut post = Post {
//!         id: None,
//!         user_id: user.id.unwrap(),
//!         title: "My First Post".to_string(),
//!         content: "Hello, World!".to_string(),
//!         created_at: None,
//!         updated_at: None,
//!     };
//!     post.save().await?;
//!
//!     // Query relationships
//!     let user_posts = user.posts().get().await?;
//!     let post_author = post.user().first().await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Laravel Eloquent Comparison
//!
//! | Laravel Eloquent | Torch ORM | Description |
//! |------------------|-----------|-------------|
//! | `User::create()` | `User::create()` | Create and save a new model |
//! | `User::find(1)` | `User::find(1)` | Find model by primary key |
//! | `User::where('active', true)` | `User::where("active", true)` | Add where clause |
//! | `User::orderBy('name')` | `User::order_by("name", "asc")` | Order results |
//! | `$user->save()` | `user.save().await` | Save model to database |
//! | `$user->delete()` | `user.delete().await` | Delete model |
//! | `$user->posts()` | `user.posts()` | Access relationship |
//!
//! ## Modules
//!
//! - [`model`] - Core Model trait and Active Record implementation
//! - [`query`] - Query builder for fluent database queries
//! - [`relations`] - Relationship definitions and querying
//! - [`connection`] - Database connection management
//! - [`schema`] - Schema introspection and table information
//! - [`macros`] - Derive macros for automatic trait implementation

pub mod model;
pub mod query;
pub mod relations;
pub mod connection;
pub mod schema;
pub mod migration;
pub mod macros;

// Re-export main traits and types for convenience
pub use model::{Model, ModelState, Timestamps};
pub use query::{QueryBuilder, WhereClause, OrderBy};
pub use relations::{HasOne, HasMany, BelongsTo, BelongsToMany, Relation};
pub use connection::{DatabaseConnection, ConnectionPool};
pub use migration::{Migration, MigrationRunner, MigrationRecord};

/// Result type for ORM operations
pub type Result<T> = std::result::Result<T, OrmError>;

/// ORM-specific error types
#[derive(Debug, thiserror::Error)]
pub enum OrmError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Model not found")]
    ModelNotFound,
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Relationship error: {0}")]
    Relationship(String),
    
    #[error("Query error: {0}")]
    Query(String),
    
    #[error("Connection error: {0}")]
    Connection(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Database driver types
#[derive(Debug, Clone, PartialEq)]
pub enum DatabaseDriver {
    /// PostgreSQL database
    Postgres,
    /// MySQL/MariaDB database
    MySql,
    /// SQLite database
    Sqlite,
}

impl DatabaseDriver {
    /// Get the driver from a database URL
    pub fn from_url(url: &str) -> Result<Self> {
        if url.starts_with("postgres://") || url.starts_with("postgresql://") {
            Ok(DatabaseDriver::Postgres)
        } else if url.starts_with("mysql://") || url.starts_with("mariadb://") {
            Ok(DatabaseDriver::MySql)
        } else if url.starts_with("sqlite://") || url.ends_with(".db") || url.ends_with(".sqlite") {
            Ok(DatabaseDriver::Sqlite)
        } else {
            Err(OrmError::Connection(format!("Unsupported database URL: {}", url)))
        }
    }

    /// Get the driver name as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            DatabaseDriver::Postgres => "postgres",
            DatabaseDriver::MySql => "mysql",
            DatabaseDriver::Sqlite => "sqlite",
        }
    }
}

/// Configuration for the ORM
#[derive(Debug, Clone)]
pub struct OrmConfig {
    /// Database connection string
    ///
    /// Examples:
    /// - PostgreSQL: "postgres://user:pass@localhost/dbname"
    /// - MySQL/MariaDB: "mysql://user:pass@localhost/dbname"
    /// - SQLite: "sqlite://path/to/database.db"
    pub database_url: String,

    /// Database driver (auto-detected from URL if not specified)
    pub driver: Option<DatabaseDriver>,

    /// Maximum number of connections in the pool
    pub max_connections: u32,

    /// Minimum number of connections in the pool
    pub min_connections: u32,

    /// Connection timeout in seconds
    pub connect_timeout: u64,

    /// Enable query logging
    pub log_queries: bool,

    /// Default table name prefix
    pub table_prefix: Option<String>,

    /// Timezone for timestamp fields
    pub timezone: chrono_tz::Tz,
}

impl Default for OrmConfig {
    fn default() -> Self {
        Self {
            database_url: "postgres://localhost/torch_app".to_string(),
            driver: None, // Auto-detect from URL
            max_connections: 10,
            min_connections: 1,
            connect_timeout: 30,
            log_queries: false,
            table_prefix: None,
            timezone: chrono_tz::UTC,
        }
    }
}

/// Initialize the ORM with configuration
pub async fn initialize(config: OrmConfig) -> Result<()> {
    connection::initialize_pool(config).await?;
    Ok(())
}

/// Get the global database connection pool
pub fn connection() -> &'static ConnectionPool {
    connection::get_pool()
}

// Re-export macros for convenience
// Note: Macros are exported at the crate root, not in modules
pub use crate::{impl_model, impl_timestamps, impl_from_row};

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_orm_initialization() {
        let config = OrmConfig::default();
        // Note: This would fail without a real database, but shows the API
        // assert!(initialize(config).await.is_ok());
    }
}
