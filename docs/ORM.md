# Torch ORM - Laravel Eloquent for Rust

The Torch ORM provides a Laravel Eloquent-inspired Active Record implementation for Rust, offering an intuitive and powerful way to interact with databases.

## Table of Contents

- [Quick Start](#quick-start)
- [Configuration](#configuration)
- [Models](#models)
- [Query Builder](#query-builder)
- [Relationships](#relationships)
- [Migrations](#migrations)
- [Advanced Features](#advanced-features)
- [Laravel Comparison](#laravel-comparison)

## Quick Start

### 1. Enable Database Features

Add the database feature to your `Cargo.toml`:

```toml
[dependencies]
torch-web = { version = "0.2.8", features = ["database"] }
```

### 2. Configure Database

Update your `torch.toml`:

```toml
[database]
driver = "postgres"
host = "127.0.0.1"
port = 5432
database = "torch_app"
username = "postgres"
password = "password"
pool_size = 10
log_queries = true

[database.orm]
enabled = true
timestamps = true
model_events = true
```

### 3. Initialize ORM

```rust
use torch_web::orm::{initialize, OrmConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = OrmConfig {
        database_url: "postgres://user:pass@localhost/mydb".to_string(),
        max_connections: 10,
        log_queries: true,
        ..Default::default()
    };
    
    initialize(config).await?;
    
    // Your application code here
    Ok(())
}
```

### 4. Generate a Model

```bash
torch make model User
```

This generates a model with ORM functionality:

```rust
use serde::{Deserialize, Serialize};
use torch_web::orm::{Model, Timestamps, HasRelationships, impl_model, impl_timestamps, impl_from_row};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Option<i32>,
    pub name: String,
    pub email: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl_model!(User, table = "users", primary_key = "id", primary_key_type = i32);
impl_timestamps!(User);
impl_from_row!(User, { id, name, email, created_at, updated_at });
```

### 5. Use the Model

```rust
// Create a new user
let mut user = User {
    id: None,
    name: "John Doe".to_string(),
    email: "john@example.com".to_string(),
    created_at: None,
    updated_at: None,
};
user.save().await?;

// Find users
let users = User::all().await?;
let user = User::find(1).await?;

// Query with conditions
let active_users = User::query()
    .where_eq("active", true)
    .order_by_desc("created_at")
    .limit(10)
    .get()
    .await?;
```

## Configuration

### Database Configuration

```toml
[database]
# Database driver
driver = "postgres"  # postgres, mysql, sqlite

# Connection settings
host = "127.0.0.1"
port = 5432
database = "torch_app"
username = "postgres"
password = "password"

# Connection pool
pool_size = 10
min_connections = 1
max_connections = 20
timeout = 30

# Development settings
log_queries = true
```

### ORM Configuration

```toml
[database.orm]
# Enable ORM features
enabled = true

# Automatic timestamps
timestamps = true
created_at_column = "created_at"
updated_at_column = "updated_at"

# Soft deletes
soft_deletes = false
deleted_at_column = "deleted_at"

# Naming conventions
table_naming = "snake_case_plural"
primary_key = "id"
foreign_key_suffix = "_id"

# Performance
eager_loading = true
query_cache = true
query_cache_ttl = 300

# Events
model_events = true
default_relationship_loading = "lazy"
```

## Models

### Defining Models

Models represent database tables and provide Active Record functionality:

```rust
use torch_web::orm::{Model, Timestamps, impl_model, impl_timestamps, impl_from_row};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: Option<i32>,
    pub user_id: i32,
    pub title: String,
    pub content: String,
    pub published: bool,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl_model!(Post, table = "posts", primary_key = "id", primary_key_type = i32);
impl_timestamps!(Post);
impl_from_row!(Post, { id, user_id, title, content, published, created_at, updated_at });
```

### Model Methods

#### Creating Records

```rust
// Create and save
let mut post = Post {
    id: None,
    user_id: 1,
    title: "My First Post".to_string(),
    content: "Hello, World!".to_string(),
    published: true,
    created_at: None,
    updated_at: None,
};
post.save().await?;

// Create with attributes
let attributes = HashMap::from([
    ("title".to_string(), json!("My Post")),
    ("content".to_string(), json!("Content here")),
    ("user_id".to_string(), json!(1)),
]);
let post = Post::create(attributes).await?;
```

#### Finding Records

```rust
// Find by ID
let post = Post::find(1).await?;
let post = Post::find_or_fail(1).await?; // Throws error if not found

// Find first
let post = Post::first().await?;

// Get all
let posts = Post::all().await?;

// Count
let count = Post::count().await?;
```

#### Updating Records

```rust
let mut post = Post::find(1).await?.unwrap();
post.title = "Updated Title".to_string();
post.save().await?; // Automatically updates updated_at
```

#### Deleting Records

```rust
let post = Post::find(1).await?.unwrap();
post.delete().await?;
```

### Validation

Add custom validation to your models:

```rust
impl User {
    fn validate(&self) -> torch_web::orm::Result<()> {
        if self.email.is_empty() {
            return Err(torch_web::orm::OrmError::Validation("Email is required".to_string()));
        }
        if !self.email.contains('@') {
            return Err(torch_web::orm::OrmError::Validation("Invalid email format".to_string()));
        }
        Ok(())
    }
}
```

### Model Events

Models support lifecycle events:

```rust
impl User {
    async fn before_create(&mut self) -> torch_web::orm::Result<()> {
        // Hash password, set defaults, etc.
        Ok(())
    }
    
    async fn after_create(&self) -> torch_web::orm::Result<()> {
        // Send welcome email, log creation, etc.
        Ok(())
    }
    
    async fn before_save(&mut self) -> torch_web::orm::Result<()> {
        // Validate, transform data, etc.
        self.validate()?;
        Ok(())
    }
}
```

## Query Builder

The query builder provides a fluent interface for constructing database queries:

### Basic Queries

```rust
// Where clauses
let users = User::query()
    .where_eq("active", true)
    .where_gt("age", 18)
    .where_like("name", "John%")
    .get()
    .await?;

// Multiple conditions
let posts = Post::query()
    .where_eq("published", true)
    .where_in("category", vec!["tech", "science"])
    .where_between("created_at", start_date, end_date)
    .get()
    .await?;

// Null checks
let users = User::query()
    .where_not_null("email_verified_at")
    .where_null("deleted_at")
    .get()
    .await?;
```

### Ordering and Limiting

```rust
let posts = Post::query()
    .order_by_desc("created_at")
    .order_by_asc("title")
    .limit(10)
    .offset(20)
    .get()
    .await?;
```

### Aggregation

```rust
// Count
let count = User::query()
    .where_eq("active", true)
    .count()
    .await?;

// Check existence
let exists = User::query()
    .where_eq("email", "john@example.com")
    .exists()
    .await?;
```

### Pagination

```rust
let paginated = Post::query()
    .where_eq("published", true)
    .paginate(1, 20) // page 1, 20 per page
    .await?;

println!("Total: {}", paginated.total);
println!("Current page: {}", paginated.current_page);
println!("Posts: {:?}", paginated.data);
```

### Raw Queries

```rust
let posts = Post::query()
    .where_raw("created_at > NOW() - INTERVAL '7 days'", vec![])
    .get()
    .await?;
```

## Relationships

Define and query relationships between models:

### One-to-Many (HasMany)

```rust
impl User {
    pub fn posts(&self) -> torch_web::orm::HasMany<Post> {
        self.has_many::<Post>("user_id")
    }
}

// Usage
let user = User::find(1).await?.unwrap();
let posts = user.posts().get().await?;

// With constraints
let recent_posts = user.posts()
    .where_gt("created_at", "2024-01-01")
    .order_by_desc("created_at")
    .limit(5)
    .get()
    .await?;
```

### One-to-One (HasOne)

```rust
impl User {
    pub fn profile(&self) -> torch_web::orm::HasOne<Profile> {
        self.has_one::<Profile>("user_id")
    }
}

// Usage
let user = User::find(1).await?.unwrap();
let profile = user.profile().first().await?;
```

### Inverse Relationships (BelongsTo)

```rust
impl Post {
    pub fn user(&self) -> torch_web::orm::BelongsTo<User> {
        self.belongs_to::<User>("user_id")
    }
}

// Usage
let post = Post::find(1).await?.unwrap();
let author = post.user().first().await?;
```

### Many-to-Many (BelongsToMany)

```rust
impl User {
    pub fn roles(&self) -> torch_web::orm::BelongsToMany<Role> {
        self.belongs_to_many::<Role>("user_roles", "user_id", "role_id")
    }
}

// Usage
let user = User::find(1).await?.unwrap();
let roles = user.roles().get().await?;
```

## Laravel Comparison

| Laravel Eloquent | Torch ORM | Description |
|------------------|-----------|-------------|
| `User::create($data)` | `User::create(attributes)` | Create and save a new model |
| `User::find(1)` | `User::find(1).await?` | Find model by primary key |
| `User::where('active', true)` | `User::query().where_eq("active", true)` | Add where clause |
| `User::orderBy('name')` | `User::query().order_by_asc("name")` | Order results |
| `$user->save()` | `user.save().await?` | Save model to database |
| `$user->delete()` | `user.delete().await?` | Delete model |
| `$user->posts()` | `user.posts()` | Access relationship |
| `User::with('posts')` | `User::query().with("posts")` | Eager load relationships |
| `User::paginate(15)` | `User::query().paginate(1, 15)` | Paginate results |

## Advanced Features

### Custom Query Methods

Add custom query methods to your models:

```rust
impl User {
    pub async fn active() -> torch_web::orm::Result<Vec<User>> {
        User::query()
            .where_eq("active", true)
            .order_by_desc("created_at")
            .get()
            .await
    }
    
    pub async fn by_email_domain(domain: &str) -> torch_web::orm::Result<Vec<User>> {
        User::query()
            .where_like("email", &format!("%@{}", domain))
            .get()
            .await
    }
}
```

### Scopes

Create reusable query scopes:

```rust
impl Post {
    pub fn published(query: QueryBuilder<Post>) -> QueryBuilder<Post> {
        query.where_eq("published", true)
    }
    
    pub fn recent(query: QueryBuilder<Post>) -> QueryBuilder<Post> {
        query.where_gt("created_at", "2024-01-01")
    }
}

// Usage
let posts = Post::query()
    .apply(Post::published)
    .apply(Post::recent)
    .get()
    .await?;
```

### Transactions

Use database transactions for data consistency:

```rust
use torch_web::orm::connection::Transaction;

let mut tx = Transaction::begin().await?;

// Perform multiple operations
let user = User::create(user_data).await?;
let profile = Profile::create(profile_data).await?;

// Commit or rollback
tx.commit().await?;
```

This comprehensive ORM brings Laravel's Eloquent power to Rust with type safety and performance! 🔥
