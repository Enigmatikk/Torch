//! # Model Relationships - Laravel Eloquent-style Relationships
//!
//! This module provides relationship definitions and querying capabilities
//! similar to Laravel's Eloquent relationships.
//!
//! ## Supported Relationships
//!
//! - **HasOne** - One-to-one relationship (e.g., User has one Profile)
//! - **HasMany** - One-to-many relationship (e.g., User has many Posts)
//! - **BelongsTo** - Inverse of HasOne/HasMany (e.g., Post belongs to User)
//! - **BelongsToMany** - Many-to-many relationship (e.g., User belongs to many Roles)
//!
//! ## Usage
//!
//! ```rust
//! use torch_web::orm::{Model, HasMany, BelongsTo};
//!
//! #[derive(Model)]
//! struct User {
//!     pub id: Option<i32>,
//!     pub name: String,
//!     pub email: String,
//! }
//!
//! impl User {
//!     /// User has many posts
//!     pub fn posts(&self) -> HasMany<Post> {
//!         self.has_many::<Post>("user_id")
//!     }
//!     
//!     /// User has one profile
//!     pub fn profile(&self) -> HasOne<Profile> {
//!         self.has_one::<Profile>("user_id")
//!     }
//!     
//!     /// User belongs to many roles (many-to-many)
//!     pub fn roles(&self) -> BelongsToMany<Role> {
//!         self.belongs_to_many::<Role>("user_roles", "user_id", "role_id")
//!     }
//! }
//!
//! #[derive(Model)]
//! struct Post {
//!     pub id: Option<i32>,
//!     pub user_id: i32,
//!     pub title: String,
//!     pub content: String,
//! }
//!
//! impl Post {
//!     /// Post belongs to user
//!     pub fn user(&self) -> BelongsTo<User> {
//!         self.belongs_to::<User>("user_id")
//!     }
//! }
//!
//! // Usage examples
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     let user = User::find(1).await?.unwrap();
//!     
//!     // Get all posts for the user
//!     let posts = user.posts().get().await?;
//!     
//!     // Get posts with additional constraints
//!     let recent_posts = user.posts()
//!         .where_gt("created_at", "2024-01-01")
//!         .order_by_desc("created_at")
//!         .limit(10)
//!         .get()
//!         .await?;
//!     
//!     // Get the user's profile
//!     let profile = user.profile().first().await?;
//!     
//!     // Get user's roles
//!     let roles = user.roles().get().await?;
//!     
//!     Ok(())
//! }
//! ```

use async_trait::async_trait;
use std::marker::PhantomData;

use crate::orm::{Model, QueryBuilder, Result};

/// Base trait for all relationship types
#[async_trait]
pub trait Relation<T: Model> {
    /// Get the query builder for this relationship
    fn query(&self) -> QueryBuilder<T>;
    
    /// Execute the relationship query and return all results
    async fn get(&self) -> Result<Vec<T>> {
        self.query().get().await
    }
    
    /// Execute the relationship query and return the first result
    async fn first(&self) -> Result<Option<T>> {
        self.query().first().await
    }
    
    /// Count the number of related models
    async fn count(&self) -> Result<i64> {
        self.query().count().await
    }
    
    /// Check if any related models exist
    async fn exists(&self) -> Result<bool> {
        Ok(self.count().await? > 0)
    }
}

/// HasOne relationship - represents a one-to-one relationship
/// 
/// Example: User has one Profile
#[derive(Debug, Clone)]
pub struct HasOne<T: Model> {
    #[allow(dead_code)]
    parent_table: String,
    #[allow(dead_code)]
    parent_key: String,
    foreign_key: String,
    local_key_value: Option<serde_json::Value>,
    _phantom: PhantomData<T>,
}

impl<T: Model> HasOne<T> {
    /// Create a new HasOne relationship
    pub fn new(
        parent_table: &str,
        parent_key: &str,
        foreign_key: &str,
        local_key_value: Option<serde_json::Value>,
    ) -> Self {
        Self {
            parent_table: parent_table.to_string(),
            parent_key: parent_key.to_string(),
            foreign_key: foreign_key.to_string(),
            local_key_value,
            _phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<T: Model> Relation<T> for HasOne<T> {
    fn query(&self) -> QueryBuilder<T> {
        let mut query = T::query();
        
        if let Some(ref value) = self.local_key_value {
            query = query.where_eq(&self.foreign_key, value.clone());
        }
        
        query.limit(1)
    }
}

/// HasMany relationship - represents a one-to-many relationship
/// 
/// Example: User has many Posts
#[derive(Debug, Clone)]
pub struct HasMany<T: Model> {
    #[allow(dead_code)]
    parent_table: String,
    #[allow(dead_code)]
    parent_key: String,
    foreign_key: String,
    local_key_value: Option<serde_json::Value>,
    _phantom: PhantomData<T>,
}

impl<T: Model> HasMany<T> {
    /// Create a new HasMany relationship
    pub fn new(
        parent_table: &str,
        parent_key: &str,
        foreign_key: &str,
        local_key_value: Option<serde_json::Value>,
    ) -> Self {
        Self {
            parent_table: parent_table.to_string(),
            parent_key: parent_key.to_string(),
            foreign_key: foreign_key.to_string(),
            local_key_value,
            _phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<T: Model> Relation<T> for HasMany<T> {
    fn query(&self) -> QueryBuilder<T> {
        let mut query = T::query();
        
        if let Some(ref value) = self.local_key_value {
            query = query.where_eq(&self.foreign_key, value.clone());
        }
        
        query
    }
}

/// BelongsTo relationship - represents the inverse of HasOne/HasMany
/// 
/// Example: Post belongs to User
#[derive(Debug, Clone)]
pub struct BelongsTo<T: Model> {
    #[allow(dead_code)]
    child_table: String,
    #[allow(dead_code)]
    foreign_key: String,
    owner_key: String,
    foreign_key_value: Option<serde_json::Value>,
    _phantom: PhantomData<T>,
}

impl<T: Model> BelongsTo<T> {
    /// Create a new BelongsTo relationship
    pub fn new(
        child_table: &str,
        foreign_key: &str,
        owner_key: &str,
        foreign_key_value: Option<serde_json::Value>,
    ) -> Self {
        Self {
            child_table: child_table.to_string(),
            foreign_key: foreign_key.to_string(),
            owner_key: owner_key.to_string(),
            foreign_key_value,
            _phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<T: Model> Relation<T> for BelongsTo<T> {
    fn query(&self) -> QueryBuilder<T> {
        let mut query = T::query();
        
        if let Some(ref value) = self.foreign_key_value {
            query = query.where_eq(&self.owner_key, value.clone());
        }
        
        query.limit(1)
    }
}

/// BelongsToMany relationship - represents a many-to-many relationship
/// 
/// Example: User belongs to many Roles (through user_roles pivot table)
#[derive(Debug, Clone)]
pub struct BelongsToMany<T: Model> {
    #[allow(dead_code)]
    parent_table: String,
    #[allow(dead_code)]
    pivot_table: String,
    #[allow(dead_code)]
    foreign_pivot_key: String,
    related_pivot_key: String,
    #[allow(dead_code)]
    parent_key: String,
    related_key: String,
    parent_key_value: Option<serde_json::Value>,
    _phantom: PhantomData<T>,
}

impl<T: Model> BelongsToMany<T> {
    /// Create a new BelongsToMany relationship
    pub fn new(
        parent_table: &str,
        pivot_table: &str,
        foreign_pivot_key: &str,
        related_pivot_key: &str,
        parent_key: &str,
        related_key: &str,
        parent_key_value: Option<serde_json::Value>,
    ) -> Self {
        Self {
            parent_table: parent_table.to_string(),
            pivot_table: pivot_table.to_string(),
            foreign_pivot_key: foreign_pivot_key.to_string(),
            related_pivot_key: related_pivot_key.to_string(),
            parent_key: parent_key.to_string(),
            related_key: related_key.to_string(),
            parent_key_value,
            _phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<T: Model> Relation<T> for BelongsToMany<T> {
    fn query(&self) -> QueryBuilder<T> {
        // For many-to-many relationships, we need to join through the pivot table
        // This is a simplified implementation - a full implementation would need
        // more sophisticated JOIN support in the QueryBuilder
        let mut query = T::query();
        
        if let Some(ref value) = self.parent_key_value {
            // This would need to be implemented with proper JOIN support
            // For now, we'll use a subquery approach
            query = query.where_raw(
                &format!(
                    "{} IN (SELECT {} FROM {} WHERE {} = ?)",
                    self.related_key,
                    self.related_pivot_key,
                    self.pivot_table,
                    self.foreign_pivot_key
                ),
                vec![value.clone()]
            );
        }
        
        query
    }
}

/// Extension trait to add relationship methods to models
pub trait HasRelationships: Model {
    /// Define a HasOne relationship
    fn has_one<T: Model>(&self, foreign_key: &str) -> HasOne<T> {
        let local_key_value = self.id().map(|id| serde_json::to_value(id).unwrap());
        HasOne::new(
            Self::table_name(),
            Self::primary_key(),
            foreign_key,
            local_key_value,
        )
    }
    
    /// Define a HasMany relationship
    fn has_many<T: Model>(&self, foreign_key: &str) -> HasMany<T> {
        let local_key_value = self.id().map(|id| serde_json::to_value(id).unwrap());
        HasMany::new(
            Self::table_name(),
            Self::primary_key(),
            foreign_key,
            local_key_value,
        )
    }
    
    /// Define a BelongsTo relationship
    fn belongs_to<T: Model>(&self, foreign_key: &str) -> BelongsTo<T> {
        // Extract the foreign key value from the model
        let foreign_key_value = self.to_attributes()
            .ok()
            .and_then(|attrs| attrs.get(foreign_key).cloned());
        
        BelongsTo::new(
            Self::table_name(),
            foreign_key,
            T::primary_key(),
            foreign_key_value,
        )
    }
    
    /// Define a BelongsToMany relationship
    fn belongs_to_many<T: Model>(
        &self,
        pivot_table: &str,
        foreign_pivot_key: &str,
        related_pivot_key: &str,
    ) -> BelongsToMany<T> {
        let parent_key_value = self.id().map(|id| serde_json::to_value(id).unwrap());
        BelongsToMany::new(
            Self::table_name(),
            pivot_table,
            foreign_pivot_key,
            related_pivot_key,
            Self::primary_key(),
            T::primary_key(),
            parent_key_value,
        )
    }
}

// Implement HasRelationships for all models
impl<T: Model> HasRelationships for T {}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Note: These tests would require actual model implementations
    // and database setup to run properly
    
    #[test]
    fn test_relationship_creation() {
        // Test that relationships can be created
        // This would need actual model instances to test properly
    }
}
