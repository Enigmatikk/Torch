//! Database integration with connection pooling and query builder

use std::collections::HashMap;
use std::sync::Arc;

use crate::{Request, Response, middleware::Middleware};

#[cfg(feature = "database")]
use {
    sqlx::{Pool, Postgres, Row, Column},
    serde_json::Value,
};

/// Database connection pool manager
pub struct DatabasePool {
    #[cfg(feature = "database")]
    pool: Pool<Postgres>,
    #[cfg(not(feature = "database"))]
    _phantom: std::marker::PhantomData<()>,
}

impl DatabasePool {
    /// Create a new database pool
    #[cfg(feature = "database")]
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(20)
            .connect(database_url)
            .await?;
        
        Ok(Self { pool })
    }

    #[cfg(not(feature = "database"))]
    pub async fn new(_database_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            _phantom: std::marker::PhantomData,
        })
    }

    /// Execute a query and return results as JSON
    #[cfg(feature = "database")]
    pub async fn query_json(&self, query: &str, params: &[&str]) -> Result<Vec<Value>, sqlx::Error> {
        let mut query_builder = sqlx::query(query);
        
        for param in params {
            query_builder = query_builder.bind(param);
        }
        
        let rows = query_builder.fetch_all(&self.pool).await?;
        let mut results = Vec::new();
        
        for row in rows {
            let mut json_row = serde_json::Map::new();
            
            for (i, column) in row.columns().iter().enumerate() {
                let column_name = column.name();
                let value: Option<String> = row.try_get(i).ok();
                json_row.insert(
                    column_name.to_string(),
                    value.map(Value::String).unwrap_or(Value::Null),
                );
            }
            
            results.push(Value::Object(json_row));
        }
        
        Ok(results)
    }

    #[cfg(not(feature = "database"))]
    pub async fn query_json(&self, _query: &str, _params: &[&str]) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
        Err("Database feature not enabled".into())
    }

    /// Execute a query and return the number of affected rows
    #[cfg(feature = "database")]
    pub async fn execute(&self, query: &str, params: &[&str]) -> Result<u64, sqlx::Error> {
        let mut query_builder = sqlx::query(query);
        
        for param in params {
            query_builder = query_builder.bind(param);
        }
        
        let result = query_builder.execute(&self.pool).await?;
        Ok(result.rows_affected())
    }

    #[cfg(not(feature = "database"))]
    pub async fn execute(&self, _query: &str, _params: &[&str]) -> Result<u64, Box<dyn std::error::Error>> {
        Err("Database feature not enabled".into())
    }
}

/// Simple query builder for common operations
pub struct QueryBuilder {
    table: String,
    select_fields: Vec<String>,
    where_conditions: Vec<String>,
    order_by: Vec<String>,
    limit_value: Option<u64>,
    offset_value: Option<u64>,
}

impl QueryBuilder {
    pub fn new(table: &str) -> Self {
        Self {
            table: table.to_string(),
            select_fields: vec!["*".to_string()],
            where_conditions: Vec::new(),
            order_by: Vec::new(),
            limit_value: None,
            offset_value: None,
        }
    }

    pub fn select(mut self, fields: &[&str]) -> Self {
        self.select_fields = fields.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn where_eq(mut self, field: &str, value: &str) -> Self {
        self.where_conditions.push(format!("{} = '{}'", field, value));
        self
    }

    pub fn where_like(mut self, field: &str, pattern: &str) -> Self {
        self.where_conditions.push(format!("{} LIKE '{}'", field, pattern));
        self
    }

    pub fn order_by(mut self, field: &str, direction: &str) -> Self {
        self.order_by.push(format!("{} {}", field, direction));
        self
    }

    pub fn limit(mut self, limit: u64) -> Self {
        self.limit_value = Some(limit);
        self
    }

    pub fn offset(mut self, offset: u64) -> Self {
        self.offset_value = Some(offset);
        self
    }

    pub fn build_select(&self) -> String {
        let mut query = format!("SELECT {} FROM {}", self.select_fields.join(", "), self.table);
        
        if !self.where_conditions.is_empty() {
            query.push_str(&format!(" WHERE {}", self.where_conditions.join(" AND ")));
        }
        
        if !self.order_by.is_empty() {
            query.push_str(&format!(" ORDER BY {}", self.order_by.join(", ")));
        }
        
        if let Some(limit) = self.limit_value {
            query.push_str(&format!(" LIMIT {}", limit));
        }
        
        if let Some(offset) = self.offset_value {
            query.push_str(&format!(" OFFSET {}", offset));
        }
        
        query
    }

    pub fn build_insert(&self, data: &HashMap<String, String>) -> String {
        let fields: Vec<String> = data.keys().cloned().collect();
        let values: Vec<String> = data.values().map(|v| format!("'{}'", v)).collect();
        
        format!(
            "INSERT INTO {} ({}) VALUES ({})",
            self.table,
            fields.join(", "),
            values.join(", ")
        )
    }

    pub fn build_update(&self, data: &HashMap<String, String>) -> String {
        let updates: Vec<String> = data
            .iter()
            .map(|(k, v)| format!("{} = '{}'", k, v))
            .collect();
        
        let mut query = format!("UPDATE {} SET {}", self.table, updates.join(", "));
        
        if !self.where_conditions.is_empty() {
            query.push_str(&format!(" WHERE {}", self.where_conditions.join(" AND ")));
        }
        
        query
    }

    pub fn build_delete(&self) -> String {
        let mut query = format!("DELETE FROM {}", self.table);
        
        if !self.where_conditions.is_empty() {
            query.push_str(&format!(" WHERE {}", self.where_conditions.join(" AND ")));
        }
        
        query
    }
}

/// Database middleware for automatic connection injection
pub struct DatabaseMiddleware {
    pool: Arc<DatabasePool>,
}

impl DatabaseMiddleware {
    pub fn new(pool: Arc<DatabasePool>) -> Self {
        Self { pool }
    }
}

impl Middleware for DatabaseMiddleware {
    fn call(
        &self,
        mut req: Request,
        next: Box<dyn Fn(Request) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send + 'static>> + Send + Sync>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send + 'static>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            // Inject the database pool into the request extensions
            req.insert_extension(pool);
            next(req).await
        })
    }
}

/// Extension trait to add database access to Request
pub trait RequestDatabaseExt {
    /// Get the database pool from the request context
    #[cfg(feature = "database")]
    fn db_pool(&self) -> Option<Arc<DatabasePool>>;

    #[cfg(not(feature = "database"))]
    fn db_pool(&self) -> Option<()>;
}

impl RequestDatabaseExt for crate::Request {
    #[cfg(feature = "database")]
    fn db_pool(&self) -> Option<Arc<DatabasePool>> {
        self.get_extension::<Arc<DatabasePool>>().cloned()
    }

    #[cfg(not(feature = "database"))]
    fn db_pool(&self) -> Option<()> {
        None
    }
}

/// Migration runner for database schema management
pub struct MigrationRunner {
    #[cfg(feature = "database")]
    #[allow(dead_code)]
    pool: Arc<DatabasePool>,
    #[allow(dead_code)]
    migrations_dir: String,
    #[cfg(not(feature = "database"))]
    _phantom: std::marker::PhantomData<()>,
}

impl MigrationRunner {
    pub fn new(_pool: Arc<DatabasePool>, migrations_dir: &str) -> Self {
        Self {
            #[cfg(feature = "database")]
            pool: _pool,
            migrations_dir: migrations_dir.to_string(),
            #[cfg(not(feature = "database"))]
            _phantom: std::marker::PhantomData,
        }
    }

    /// Run pending migrations
    #[cfg(feature = "database")]
    pub async fn run_migrations(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Migration system initialized for directory: {}", self.migrations_dir);

        // In a production implementation, this would:
        // 1. Create migrations table
        // 2. Read migration files from directory
        // 3. Execute pending migrations in order
        // 4. Record completed migrations

        // For now, we'll just log that migrations would run
        println!("Migration system ready - would execute SQL files from {}", self.migrations_dir);
        Ok(())
    }

    #[cfg(not(feature = "database"))]
    pub async fn run_migrations(&self) -> Result<(), Box<dyn std::error::Error>> {
        Err("Database feature not enabled".into())
    }
}

/// Database health check
pub async fn database_health_check(pool: &DatabasePool) -> Response {
    #[cfg(feature = "database")]
    {
        match pool.query_json("SELECT 1 as health_check", &[]).await {
            Ok(_) => Response::ok().json(&serde_json::json!({
                "database": "healthy",
                "timestamp": chrono::Utc::now().to_rfc3339()
            })).unwrap_or_else(|_| Response::ok().body("healthy")),
            Err(e) => Response::with_status(http::StatusCode::SERVICE_UNAVAILABLE)
                .json(&serde_json::json!({
                    "database": "unhealthy",
                    "error": e.to_string(),
                    "timestamp": chrono::Utc::now().to_rfc3339()
                })).unwrap_or_else(|_| Response::with_status(http::StatusCode::SERVICE_UNAVAILABLE).body("unhealthy"))
        }
    }
    
    #[cfg(not(feature = "database"))]
    {
        let _ = pool; // Suppress unused variable warning
        Response::ok().body("Database feature not enabled")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_builder_select() {
        let query = QueryBuilder::new("users")
            .select(&["id", "name", "email"])
            .where_eq("active", "true")
            .order_by("created_at", "DESC")
            .limit(10)
            .build_select();
        
        assert!(query.contains("SELECT id, name, email FROM users"));
        assert!(query.contains("WHERE active = 'true'"));
        assert!(query.contains("ORDER BY created_at DESC"));
        assert!(query.contains("LIMIT 10"));
    }

    #[test]
    fn test_query_builder_insert() {
        let mut data = HashMap::new();
        data.insert("name".to_string(), "John Doe".to_string());
        data.insert("email".to_string(), "john@example.com".to_string());
        
        let query = QueryBuilder::new("users").build_insert(&data);
        assert!(query.contains("INSERT INTO users"));
        assert!(query.contains("name"));
        assert!(query.contains("email"));
    }

    #[test]
    fn test_query_builder_update() {
        let mut data = HashMap::new();
        data.insert("name".to_string(), "Jane Doe".to_string());
        
        let query = QueryBuilder::new("users")
            .where_eq("id", "1")
            .build_update(&data);
        
        assert!(query.contains("UPDATE users SET"));
        assert!(query.contains("name = 'Jane Doe'"));
        assert!(query.contains("WHERE id = '1'"));
    }

    #[test]
    fn test_query_builder_delete() {
        let query = QueryBuilder::new("users")
            .where_eq("id", "1")
            .build_delete();
        
        assert!(query.contains("DELETE FROM users"));
        assert!(query.contains("WHERE id = '1'"));
    }
}
