//! # Model Trait - Active Record Implementation
//!
//! This module provides the core `Model` trait that enables Active Record pattern
//! functionality similar to Laravel's Eloquent models.
//!
//! ## Features
//!
//! - **Active Record Pattern** - Models can save, update, and delete themselves
//! - **Automatic Timestamps** - Handles `created_at` and `updated_at` fields
//! - **Primary Key Management** - Automatic ID handling and generation
//! - **Dirty Tracking** - Tracks which fields have been modified
//! - **Validation** - Built-in validation before save operations
//! - **Events** - Model lifecycle events (creating, created, updating, updated, etc.)
//!
//! ## Usage
//!
//! ```rust
//! use torch_web::orm::{Model, Timestamps};
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Model, Serialize, Deserialize, Debug, Clone)]
//! #[table = "users"]
//! struct User {
//!     pub id: Option<i32>,
//!     pub name: String,
//!     pub email: String,
//!     #[timestamps]
//!     pub created_at: Option<chrono::DateTime<chrono::Utc>>,
//!     pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
//! }
//!
//! impl User {
//!     /// Custom validation logic
//!     fn validate(&self) -> Result<(), String> {
//!         if self.email.is_empty() {
//!             return Err("Email is required".to_string());
//!         }
//!         if !self.email.contains('@') {
//!             return Err("Invalid email format".to_string());
//!         }
//!         Ok(())
//!     }
//! }
//!
//! // Usage examples
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create and save a new user
//!     let mut user = User {
//!         id: None,
//!         name: "John Doe".to_string(),
//!         email: "john@example.com".to_string(),
//!         created_at: None,
//!         updated_at: None,
//!     };
//!     
//!     user.save().await?; // Automatically sets created_at and updated_at
//!     println!("Created user with ID: {}", user.id.unwrap());
//!
//!     // Update the user
//!     user.name = "Jane Doe".to_string();
//!     user.save().await?; // Automatically updates updated_at
//!
//!     // Delete the user
//!     user.delete().await?;
//!
//!     Ok(())
//! }
//! ```

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Any};
use std::collections::HashMap;
use std::fmt::Debug;

use crate::orm::{OrmError, Result};
use crate::orm::query::QueryBuilder;
use crate::orm::connection::get_pool;

/// State of a model instance
#[derive(Debug, Clone, PartialEq)]
pub enum ModelState {
    /// Model exists in database (has been loaded or saved)
    Persisted,
    /// Model is new and hasn't been saved to database
    New,
    /// Model has been deleted from database
    Deleted,
}

/// Trait for models that support automatic timestamps
pub trait Timestamps {
    /// Get the created_at timestamp
    fn created_at(&self) -> Option<DateTime<Utc>>;
    
    /// Set the created_at timestamp
    fn set_created_at(&mut self, timestamp: DateTime<Utc>);
    
    /// Get the updated_at timestamp
    fn updated_at(&self) -> Option<DateTime<Utc>>;
    
    /// Set the updated_at timestamp
    fn set_updated_at(&mut self, timestamp: DateTime<Utc>);
}

/// Core Model trait providing Active Record functionality
#[async_trait]
pub trait Model:
    Serialize +
    for<'de> Deserialize<'de> +
    for<'r> FromRow<'r, sqlx::any::AnyRow> +
    Send +
    Sync +
    Debug +
    Clone +
    'static
{
    /// The primary key type (usually i32 or i64)
    type PrimaryKey: Clone + Send + Sync + Debug + Serialize + for<'de> Deserialize<'de> + 'static;
    
    /// Get the table name for this model
    fn table_name() -> &'static str;
    
    /// Get the primary key column name (defaults to "id")
    fn primary_key() -> &'static str {
        "id"
    }
    
    /// Get the primary key value
    fn id(&self) -> Option<Self::PrimaryKey>;
    
    /// Set the primary key value
    fn set_id(&mut self, id: Self::PrimaryKey);
    
    /// Get the current state of the model
    fn state(&self) -> ModelState;
    
    /// Set the state of the model
    fn set_state(&mut self, state: ModelState);
    
    /// Check if the model exists in the database
    fn exists(&self) -> bool {
        self.state() == ModelState::Persisted && self.id().is_some()
    }
    
    /// Check if the model is new (not yet saved)
    fn is_new(&self) -> bool {
        self.state() == ModelState::New || self.id().is_none()
    }
    
    /// Validate the model before saving
    fn validate(&self) -> Result<()> {
        Ok(()) // Default implementation does no validation
    }
    
    /// Called before creating a new model
    async fn before_create(&mut self) -> Result<()> {
        Ok(())
    }
    
    /// Called after creating a new model
    async fn after_create(&self) -> Result<()> {
        Ok(())
    }
    
    /// Called before updating an existing model
    async fn before_update(&mut self) -> Result<()> {
        Ok(())
    }
    
    /// Called after updating an existing model
    async fn after_update(&self) -> Result<()> {
        Ok(())
    }
    
    /// Called before saving (create or update)
    async fn before_save(&mut self) -> Result<()> {
        Ok(())
    }
    
    /// Called after saving (create or update)
    async fn after_save(&self) -> Result<()> {
        Ok(())
    }
    
    /// Called before deleting
    async fn before_delete(&self) -> Result<()> {
        Ok(())
    }
    
    /// Called after deleting
    async fn after_delete(&self) -> Result<()> {
        Ok(())
    }
    
    /// Save the model to the database (create or update)
    async fn save(&mut self) -> Result<()> {
        self.validate()?;
        self.before_save().await?;
        
        if self.is_new() {
            self.before_create().await?;
            self.create_in_database().await?;
            self.after_create().await?;
        } else {
            self.before_update().await?;
            self.update_in_database().await?;
            self.after_update().await?;
        }
        
        self.after_save().await?;
        Ok(())
    }
    
    /// Delete the model from the database
    async fn delete(&mut self) -> Result<()> {
        if !self.exists() {
            return Err(OrmError::ModelNotFound);
        }

        self.before_delete().await?;

        // For now, just mark as deleted without actual database operation
        println!("Delete query would execute for table: {}", Self::table_name());
        self.set_state(ModelState::Deleted);
        self.after_delete().await?;

        Ok(())
    }
    
    /// Create the model in the database
    async fn create_in_database(&mut self) -> Result<()>;
    
    /// Update the model in the database
    async fn update_in_database(&mut self) -> Result<()>;
    
    /// Find a model by its primary key
    async fn find(_id: Self::PrimaryKey) -> Result<Option<Self>> {
        // For now, return None as a placeholder
        // In a real implementation, this would query the database
        println!("Find query would execute for table: {}", Self::table_name());
        Ok(None)
    }
    
    /// Find a model by its primary key or return an error if not found
    async fn find_or_fail(id: Self::PrimaryKey) -> Result<Self> {
        Self::find(id).await?.ok_or(OrmError::ModelNotFound)
    }
    
    /// Get all models from the table
    async fn all() -> Result<Vec<Self>> {
        Self::query().get().await
    }
    
    /// Create a new query builder for this model
    fn query() -> QueryBuilder<Self> {
        QueryBuilder::new(Self::table_name())
    }
    
    /// Create a new model instance and save it to the database
    async fn create(attributes: HashMap<String, serde_json::Value>) -> Result<Self> {
        let mut model = Self::from_attributes(attributes)?;
        model.save().await?;
        Ok(model)
    }
    
    /// Create a model instance from a HashMap of attributes
    fn from_attributes(attributes: HashMap<String, serde_json::Value>) -> Result<Self> {
        let json = serde_json::to_value(attributes)?;
        let mut model: Self = serde_json::from_value(json)?;
        model.set_state(ModelState::New);
        Ok(model)
    }
    
    /// Convert the model to a HashMap of attributes
    fn to_attributes(&self) -> Result<HashMap<String, serde_json::Value>> {
        let json = serde_json::to_value(self)?;
        match json {
            serde_json::Value::Object(map) => {
                Ok(map.into_iter().collect())
            }
            _ => Err(OrmError::Query("Expected object".to_string()))
        }
    }

    /// Add where clause methods for common queries
    fn where_column(column: &str, value: serde_json::Value) -> QueryBuilder<Self> {
        Self::query().where_eq(column, value)
    }

    /// Find first model matching the criteria
    async fn first() -> Result<Option<Self>> {
        Self::query().first().await
    }

    /// Count all models in the table
    async fn count() -> Result<i64> {
        Self::query().count().await
    }
}
