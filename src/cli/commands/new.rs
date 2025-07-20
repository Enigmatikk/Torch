//! Create new Torch applications

use colored::*;
use std::fs;
use std::path::Path;

/// Create a new Torch project
pub fn create_project(name: &str, minimal: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("{} Creating new Torch application: {}", "🔥".yellow(), name.cyan().bold());
    
    let project_path = Path::new(name);
    
    if project_path.exists() {
        return Err(format!("Directory '{}' already exists", name).into());
    }
    
    // Create project directory
    fs::create_dir_all(project_path)?;
    
    // Create project structure
    create_project_structure(project_path, minimal)?;
    
    println!("{} Project created successfully!", "✅".green());
    println!();
    println!("{}", "Next steps:".bold());
    println!("  {} {}", "cd".cyan(), name);
    println!("  {} {}", "torch".cyan(), "serve --hot".yellow());
    println!();
    println!("{}", "Available commands:".bold());
    println!("  {} {}          - Start development server with hot reload", "torch".cyan(), "serve --hot".yellow());
    println!("  {} {}           - Generate controllers, models, etc.", "torch".cyan(), "make".yellow());
    println!("  {} {}          - Build for production", "torch".cyan(), "build --release".yellow());
    println!("  {} {}           - Run database migrations", "torch".cyan(), "migrate".yellow());
    println!("  {} {}            - Interactive REPL", "torch".cyan(), "tinker".yellow());
    
    Ok(())
}

fn create_project_structure(path: &Path, minimal: bool) -> Result<(), Box<dyn std::error::Error>> {
    // Create core directories
    fs::create_dir_all(path.join("src"))?;
    fs::create_dir_all(path.join("src/controllers"))?;
    fs::create_dir_all(path.join("src/models"))?;
    fs::create_dir_all(path.join("src/middleware"))?;
    fs::create_dir_all(path.join("templates"))?;
    fs::create_dir_all(path.join("static/css"))?;
    fs::create_dir_all(path.join("static/js"))?;
    fs::create_dir_all(path.join("static/images"))?;
    fs::create_dir_all(path.join("config"))?;
    fs::create_dir_all(path.join("migrations"))?;
    fs::create_dir_all(path.join("storage/logs"))?;
    fs::create_dir_all(path.join("storage/framework"))?;

    if !minimal {
        fs::create_dir_all(path.join("examples"))?;
        fs::create_dir_all(path.join("tests"))?;
        fs::create_dir_all(path.join("src/seeders"))?;
        fs::create_dir_all(path.join("src/factories"))?;
        fs::create_dir_all(path.join("src/policies"))?;
        fs::create_dir_all(path.join("src/events"))?;
        fs::create_dir_all(path.join("src/listeners"))?;
        fs::create_dir_all(path.join("src/jobs"))?;
        fs::create_dir_all(path.join("src/notifications"))?;
    }
    
    // Create Cargo.toml
    let cargo_toml = if minimal {
        create_minimal_cargo_toml(path.file_name().unwrap().to_str().unwrap())
    } else {
        create_full_cargo_toml(path.file_name().unwrap().to_str().unwrap())
    };
    
    fs::write(path.join("Cargo.toml"), cargo_toml)?;
    
    // Create main.rs
    let main_rs = if minimal {
        r#"use torch_web::{App, Request, Response};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = App::new()
        .get("/", |_req: Request| async {
            Response::ok().body("🔥 Welcome to Torch!")
        })
        .get("/hello/:name", |req: Request| async move {
            let name = req.param("name").unwrap_or("World");
            Response::ok().body(format!("Hello, {}!", name))
        });

    println!("🔥 Torch server starting on http://127.0.0.1:3000");
    app.listen("127.0.0.1:3000").await
}
"#
    } else {
        r#"use torch_web::{App, Request, Response};
use tracing::{info, Level};
use tracing_subscriber;

mod controllers;
mod models;
mod middleware;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("🔥 Starting Torch application");

    let app = App::new()
        .get("/", home_handler)
        .get("/hello/:name", hello_handler)
        .get("/health", health_handler);

    let host = "127.0.0.1";
    let port = 3000;

    info!("🔥 Torch server starting on http://{}:{}", host, port);
    app.listen(&format!("{}:{}", host, port)).await
}

async fn home_handler(_req: Request) -> Response {
    Response::ok().body("🔥 Welcome to Torch!")
}

async fn hello_handler(req: Request) -> Response {
    let name = req.param("name").unwrap_or("World");
    Response::ok().body(format!("Hello, {}!", name))
}

async fn health_handler(_req: Request) -> Response {
    Response::ok().json(&serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })).unwrap()
}
"#
    };
    
    fs::write(path.join("src/main.rs"), main_rs)?;

    // Create torch.toml configuration
    let torch_config = create_torch_config(minimal);
    fs::write(path.join("torch.toml"), torch_config)?;

    // Create basic template
    let layout_template = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>@yield('title', 'Torch App')</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        .container { max-width: 800px; margin: 0 auto; }
        .flame { color: #FF6B35; }
    </style>
</head>
<body>
    <div class="container">
        @yield('content')
    </div>
</body>
</html>
"#;
    
    fs::write(path.join("templates/layout.ember"), layout_template)?;
    
    let welcome_template = r#"@extends('layout')

@section('title', 'Welcome to Torch')

@section('content')
    <h1>🔥 Welcome to Torch!</h1>
    <p>Your Torch application is ready to ignite!</p>
    
    <h2>Quick Links:</h2>
    <ul>
        <li><a href="/hello/Torch">Say Hello</a></li>
        <li><a href="/about">About</a></li>
    </ul>
@endsection
"#;
    
    fs::write(path.join("templates/welcome.ember"), welcome_template)?;
    
    // Create README
    let readme = format!(r#"# {}

A Torch web application.

## Getting Started

```bash
# Run the application
cargo run

# Or use the Torch CLI
torch serve --hot
```

## Project Structure

- `src/` - Application source code
- `templates/` - Ember templates
- `static/` - Static assets (CSS, JS, images)
- `config/` - Configuration files

## Learn More

- [Torch Documentation](https://docs.rs/torch-web)
- [GitHub Repository](https://github.com/Enigmatikk/torch)
"#, path.file_name().unwrap().to_str().unwrap());

    fs::write(path.join("README.md"), readme)?;

    // Create additional files for non-minimal projects
    if !minimal {
        create_additional_files(path)?;
    }

    // Create configuration files
    create_config_files(path)?;

    // Create gitignore
    create_gitignore(path)?;

    Ok(())
}

/// Create additional files for full project setup
fn create_additional_files(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Create example controller
    let user_controller = r#"//! User controller - Example controller

use torch_web::{Request, Response, extractors::*};
use serde::{Deserialize, Serialize};

pub struct UserController {}

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub id: u32,
    pub name: String,
    pub email: String,
    pub created_at: String,
}

impl UserController {
    /// GET /users - List all users
    pub async fn index(_req: Request) -> Response {
        let users = vec![
            UserResponse {
                id: 1,
                name: "John Doe".to_string(),
                email: "john@example.com".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
            }
        ];

        Response::ok().json(&serde_json::json!({
            "users": users
        })).unwrap()
    }

    /// GET /users/:id - Show specific user
    pub async fn show(Path(id): Path<u32>) -> Response {
        let user = UserResponse {
            id,
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        Response::ok().json(&user).unwrap()
    }

    /// POST /users - Create new user
    pub async fn create(Json(req): Json<CreateUserRequest>) -> Response {
        let user = UserResponse {
            id: 1,
            name: req.name,
            email: req.email,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        Response::created().json(&user).unwrap()
    }
}
"#;

    fs::write(path.join("src/controllers/user_controller.rs"), user_controller)?;

    // Create controllers mod.rs
    let controllers_mod = r#"//! Controllers module

pub mod user_controller;

pub use user_controller::UserController;
"#;

    fs::write(path.join("src/controllers/mod.rs"), controllers_mod)?;

    // Create example model
    let user_model = r#"//! User model - Example model

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Option<u32>,
    pub name: String,
    pub email: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl User {
    pub fn new(name: String, email: String) -> Self {
        Self {
            id: None,
            name,
            email,
            created_at: Some(chrono::Utc::now()),
            updated_at: Some(chrono::Utc::now()),
        }
    }

    /// Find all users
    pub async fn all() -> Result<Vec<Self>, Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Implement database query
        Ok(vec![])
    }

    /// Find user by ID
    pub async fn find(id: u32) -> Result<Option<Self>, Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Implement database query
        Ok(None)
    }

    /// Save user to database
    pub async fn save(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Implement database save
        Ok(())
    }
}
"#;

    fs::write(path.join("src/models/user.rs"), user_model)?;

    // Create models mod.rs
    let models_mod = r#"//! Models module

pub mod user;

pub use user::User;
"#;

    fs::write(path.join("src/models/mod.rs"), models_mod)?;

    // Create example middleware
    let auth_middleware = r#"//! Authentication middleware - Example middleware

use torch_web::{Request, Response, middleware::Middleware};
use std::pin::Pin;
use std::future::Future;

pub struct AuthMiddleware {}

impl AuthMiddleware {
    pub fn new() -> Self {
        Self {}
    }
}

impl Middleware for AuthMiddleware {
    fn call(
        &self,
        req: Request,
        next: Box<dyn Fn(Request) -> Pin<Box<dyn Future<Output = Response> + Send + 'static>> + Send + Sync>,
    ) -> Pin<Box<dyn Future<Output = Response> + Send + 'static>> {
        Box::pin(async move {
            // TODO: Implement authentication logic
            // Check for authorization header, validate tokens, etc.

            // For now, just pass through
            next(req).await
        })
    }
}
"#;

    fs::write(path.join("src/middleware/auth.rs"), auth_middleware)?;

    // Create middleware mod.rs
    let middleware_mod = r#"//! Middleware module

pub mod auth;

pub use auth::AuthMiddleware;
"#;

    fs::write(path.join("src/middleware/mod.rs"), middleware_mod)?;

    Ok(())
}

/// Create configuration files
fn create_config_files(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Create app.toml
    let app_config = r#"[app]
name = "Torch App"
env = "development"
debug = true
url = "http://localhost:3000"
timezone = "UTC"

[server]
host = "127.0.0.1"
port = 3000
workers = 4

[database]
default = "postgres"

[database.connections.postgres]
driver = "postgres"
host = "localhost"
port = 5432
database = "torch_app"
username = "postgres"
password = "password"

[cache]
default = "memory"

[cache.stores.memory]
driver = "memory"

[cache.stores.redis]
driver = "redis"
host = "localhost"
port = 6379

[session]
driver = "memory"
lifetime = 3600
encrypt = false

[logging]
level = "info"
channels = ["console", "file"]

[logging.channels.console]
driver = "console"

[logging.channels.file]
driver = "file"
path = "storage/logs/app.log"
"#;

    fs::write(path.join("config/app.toml"), app_config)?;

    // Create database.toml
    let database_config = r#"[default]
connection = "postgres"

[connections.postgres]
driver = "postgres"
host = "localhost"
port = 5432
database = "torch_app"
username = "postgres"
password = "password"
pool_size = 10
timeout = 30

[connections.sqlite]
driver = "sqlite"
database = "storage/database.sqlite"
pool_size = 5

[migrations]
table = "migrations"
directory = "migrations"
"#;

    fs::write(path.join("config/database.toml"), database_config)?;

    Ok(())
}

/// Create .gitignore file
fn create_gitignore(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let gitignore_content = r#"# Rust
/target/
**/*.rs.bk
Cargo.lock

# IDE
.vscode/
.idea/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db

# Logs
storage/logs/*.log
*.log

# Environment
.env
.env.local
.env.production

# Database
*.sqlite
*.db

# Cache
storage/framework/cache/
storage/framework/sessions/
storage/framework/views/

# Temporary files
*.tmp
*.temp

# Build artifacts
dist/
build/
"#;

    fs::write(path.join(".gitignore"), gitignore_content)?;

    Ok(())
}

/// Get the current torch-web version from the root Cargo.toml
fn get_torch_version() -> String {
    // Try to read the version from the current package's Cargo.toml
    if let Ok(cargo_toml) = std::fs::read_to_string("Cargo.toml") {
        for line in cargo_toml.lines() {
            if line.starts_with("version = ") {
                if let Some(version) = line.split('"').nth(1) {
                    return version.to_string();
                }
            }
        }
    }

    // Fallback to a reasonable default if we can't read the version
    env!("CARGO_PKG_VERSION").to_string()
}

/// Create a minimal Cargo.toml for basic applications
fn create_minimal_cargo_toml(project_name: &str) -> String {
    let torch_version = get_torch_version();
    format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]
description = "A Torch web application"

[dependencies]
torch-web = {{ version = "{}", features = ["json"] }}
tokio = {{ version = "1.0", features = ["full"] }}
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"

[[bin]]
name = "server"
path = "src/main.rs"
"#, project_name, torch_version)
}

/// Create a full-featured Cargo.toml for production applications
fn create_full_cargo_toml(project_name: &str) -> String {
    let torch_version = get_torch_version();
    format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]
description = "A Torch web application"

[dependencies]
# Core Torch framework with all features
torch-web = {{ version = "{}", features = ["full"] }}

# Async runtime
tokio = {{ version = "1.0", features = ["full"] }}

# Serialization
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"

# Date/time handling
chrono = {{ version = "0.4", features = ["serde"] }}

# UUID generation
uuid = {{ version = "1.0", features = ["v4", "serde"] }}

# Database (PostgreSQL)
sqlx = {{ version = "0.8", features = ["postgres", "runtime-tokio-rustls", "chrono", "uuid"] }}

# Caching (Redis)
redis = {{ version = "0.24", features = ["tokio-comp"] }}

# Logging and tracing
tracing = "0.1"
tracing-subscriber = {{ version = "0.3", features = ["env-filter"] }}

# Configuration
toml = "0.8"

# Environment variables
dotenv = "0.15"

# Error handling
anyhow = "1.0"

[[bin]]
name = "server"
path = "src/main.rs"
"#, project_name, torch_version)
}

/// Create torch.toml configuration file
fn create_torch_config(minimal: bool) -> String {
    if minimal {
        create_minimal_torch_config()
    } else {
        create_full_torch_config()
    }
}

/// Create minimal torch.toml for basic applications
pub fn create_minimal_torch_config() -> String {
    r#"# Torch Configuration File
# This file contains all the configuration for your Torch application
# Similar to Laravel's config files, but in TOML format

[app]
# Application name
name = "Torch App"

# Application environment (local, development, staging, production)
env = "local"

# Debug mode - shows detailed error pages in development
debug = true

# Application URL
url = "http://127.0.0.1:3000"

# Timezone for the application
timezone = "UTC"

[server]
# Server host and port
host = "127.0.0.1"
port = 3000

# Request timeout in seconds
timeout = 30

# Maximum request body size in MB
max_body_size = 16

# Enable hot reload in development
hot_reload = true

[logging]
# Log level: trace, debug, info, warn, error
level = "info"

# Log format: json, pretty
format = "pretty"

# Log to file (optional)
# file = "logs/torch.log"

# Uncomment to enable database support
# [database]
# driver = "postgres"
# host = "127.0.0.1"
# port = 5432
# database = "torch_app"
# username = "postgres"
# password = "password"
# pool_size = 10
# timeout = 30

# Uncomment to enable Redis caching
# [cache]
# driver = "redis"
# host = "127.0.0.1"
# port = 6379
# database = 0
# password = ""
# prefix = "torch_cache"

# Uncomment to enable session management
# [session]
# driver = "cookie"  # cookie, redis, database
# lifetime = 120     # minutes
# encrypt = true
# secure = false     # set to true in production with HTTPS
# same_site = "lax"  # strict, lax, none

# Uncomment to enable CORS
# [cors]
# allowed_origins = ["*"]
# allowed_methods = ["GET", "POST", "PUT", "DELETE", "OPTIONS"]
# allowed_headers = ["*"]
# expose_headers = []
# max_age = 86400
# credentials = false
"#.to_string()
}

/// Create full torch.toml for production applications
pub fn create_full_torch_config() -> String {
    r#"# Torch Configuration File
# This file contains all the configuration for your Torch application
# Similar to Laravel's config files, but in TOML format

[app]
# Application name
name = "Torch App"

# Application environment (local, development, staging, production)
env = "local"

# Debug mode - shows detailed error pages in development
debug = true

# Application URL
url = "http://127.0.0.1:3000"

# Timezone for the application
timezone = "UTC"

# Application key for encryption (generate with: torch key:generate)
# key = ""

[server]
# Server host and port
host = "127.0.0.1"
port = 3000

# Request timeout in seconds
timeout = 30

# Maximum request body size in MB
max_body_size = 16

# Enable hot reload in development
hot_reload = true

# Number of worker threads (0 = auto-detect)
workers = 0

# Enable HTTP/2 support
http2 = true

# TLS configuration for HTTPS
# [server.tls]
# cert = "certs/server.crt"
# key = "certs/server.key"

[logging]
# Log level: trace, debug, info, warn, error
level = "info"

# Log format: json, pretty
format = "pretty"

# Log to file
file = "logs/torch.log"

# Rotate log files
rotate = true
max_size = "100MB"
max_files = 10

[database]
# Database driver: postgres, mysql, sqlite
driver = "postgres"

# Connection details
host = "127.0.0.1"
port = 5432
database = "torch_app"
username = "postgres"
password = "password"

# Connection pool settings
pool_size = 10
min_connections = 1
max_connections = 20
timeout = 30

# Enable query logging in development
log_queries = true

# Migration settings
[database.migrations]
table = "migrations"
path = "migrations"

# ORM Configuration
[database.orm]
# Enable ORM features
enabled = true

# Automatic timestamp management
timestamps = true

# Default timestamp column names
created_at_column = "created_at"
updated_at_column = "updated_at"

# Soft deletes
soft_deletes = false
deleted_at_column = "deleted_at"

# Model conventions
table_naming = "snake_case_plural"  # snake_case_plural, snake_case, custom
primary_key = "id"
foreign_key_suffix = "_id"

# Query optimization
eager_loading = true
query_cache = true
query_cache_ttl = 300  # seconds

# Model events
model_events = true

# Relationship loading strategy
default_relationship_loading = "lazy"  # lazy, eager

[cache]
# Cache driver: redis, memory, file
driver = "redis"

# Redis connection
host = "127.0.0.1"
port = 6379
database = 0
password = ""

# Cache key prefix
prefix = "torch_cache"

# Default TTL in seconds
default_ttl = 3600

[session]
# Session driver: cookie, redis, database
driver = "cookie"

# Session lifetime in minutes
lifetime = 120

# Encrypt session data
encrypt = true

# Cookie settings
secure = false      # set to true in production with HTTPS
http_only = true
same_site = "lax"   # strict, lax, none
path = "/"
domain = ""

# Session table name (for database driver)
table = "sessions"

[cors]
# CORS configuration
enabled = true
allowed_origins = ["http://localhost:3000", "http://127.0.0.1:3000"]
allowed_methods = ["GET", "POST", "PUT", "DELETE", "OPTIONS", "PATCH"]
allowed_headers = ["*"]
expose_headers = ["X-Request-ID"]
max_age = 86400
credentials = true

[security]
# Security headers
[security.headers]
# Content Security Policy
csp = "default-src 'self'"

# HTTP Strict Transport Security (HSTS)
hsts = "max-age=31536000; includeSubDomains"

# X-Frame-Options
frame_options = "DENY"

# X-Content-Type-Options
content_type_options = "nosniff"

# X-XSS-Protection
xss_protection = "1; mode=block"

# Referrer Policy
referrer_policy = "strict-origin-when-cross-origin"

[mail]
# Mail driver: smtp, sendmail, log
driver = "log"

# SMTP settings
[mail.smtp]
host = "smtp.mailtrap.io"
port = 587
username = ""
password = ""
encryption = "tls"  # tls, ssl, none

# Default from address
from_address = "noreply@torchapp.com"
from_name = "Torch App"

[queue]
# Queue driver: redis, database, sync
driver = "sync"

# Default queue name
default = "default"

# Queue connection (for redis driver)
connection = "default"

# Failed job settings
[queue.failed]
driver = "database"
table = "failed_jobs"

[filesystem]
# Default disk
default = "local"

# Disk configurations
[filesystem.disks.local]
driver = "local"
root = "storage/app"

[filesystem.disks.public]
driver = "local"
root = "storage/app/public"
url = "/storage"

# Uncomment for S3 support
# [filesystem.disks.s3]
# driver = "s3"
# bucket = "your-bucket"
# region = "us-east-1"
# key = ""
# secret = ""

[broadcasting]
# Broadcasting driver: redis, log, null
driver = "log"

# Pusher settings (for real-time features)
# [broadcasting.pusher]
# app_id = ""
# key = ""
# secret = ""
# cluster = "mt1"

[monitoring]
# Enable application monitoring
enabled = true

# Metrics collection
collect_metrics = true

# Health check endpoint
health_check = "/health"

# Prometheus metrics endpoint
metrics_endpoint = "/metrics"

[api]
# API configuration
prefix = "api"
version = "v1"

# Rate limiting
rate_limit = 60  # requests per minute
rate_limit_by = "ip"  # ip, user, api_key

# API documentation
docs_enabled = true
docs_path = "/docs"

[templates]
# Template engine settings
engine = "ember"  # Torch's built-in templating engine

# Template caching
cache = true

# Template directories
paths = ["resources/views"]

# Template file extension
extension = "ember"

[localization]
# Default locale
default = "en"

# Available locales
available = ["en", "es", "fr", "de"]

# Locale detection method
detection = "header"  # header, session, query

# Fallback locale
fallback = "en"
"#.to_string()
}
