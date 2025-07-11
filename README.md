# Torch 🔥

**The web framework that doesn't get in your way.**

Torch is a fast, secure, and production-ready web framework for Rust. Built on Tokio and Hyper, it provides everything you need to build modern web applications with minimal configuration.

```rust
use torch_web::{App, Request, Response, main};

#[main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = App::new()
        .get("/", |_req: Request| async {
            Response::ok().body("Hello, World! 🔥")
        })
        .get("/users/:id", |req: Request| async move {
            let id = req.param("id").unwrap_or("Anonymous");
            Response::ok().body(format!("Hello, {}! 🔥", id))
        });

    println!("🔥 Server running at http://localhost:3000");
    app.listen("127.0.0.1:3000").await
}
```
<a href="https://buymeacoffee.com/enigmatikk" target="_blank"><img src="https://www.buymeacoffee.com/assets/img/custom_images/orange_img.png" alt="Buy Me A Coffee" style="height: 41px !important;width: 174px !important;box-shadow: 0px 3px 2px 0px rgba(190, 190, 190, 0.5) !important;-webkit-box-shadow: 0px 3px 2px 0px rgba(190, 190, 190, 0.5) !important;" ></a>

**Why developers choose Torch:**
- 🚀 **Blazing Fast** - Built on Tokio + Hyper for maximum performance
- ⚡ **Compile-Time Routes** - Zero-cost route validation with type-safe extractors
- 🔥 **Ember Templates** - Laravel Blade-inspired templating with inheritance
- 🏗️ **Modular Architecture** - Multi-crate project structure for large applications
- 🛡️ **Secure by Design** - Security features and beautiful error pages included
- 📊 **Production Ready** - Monitoring, caching, and database support
- ⚡ **Real-time Capable** - WebSocket and SSE support out of the box
- 🎯 **Simple & Familiar** - Sinatra-inspired API that just works
- 😄 **Fun Error Pages** - Beautiful 404 pages with rotating flame-themed messages

## ✨ Features

### ⚡ **Compile-Time Route Registration**
- **Zero-cost abstractions** - Routes validated at compile time
- **Type-safe parameter extraction** - `Path<T>`, `Query<T>`, `Json<T>`
- **IDE support** - Full autocomplete and error checking
- **No runtime overhead** - All validation happens at build time

```rust
use torch_web::{routes, get, Path, Query};

routes! {
    #[get("/users/{id}")]
    async fn get_user(Path(id): Path<u32>) -> Response {
        Response::ok().json(format!("User {}", id))
    }

    #[get("/users")]
    async fn list_users(Query(params): Query<UserQuery>) -> Response {
        // Type-safe query parameter extraction
        Response::ok().json(params)
    }
}
```

### 🔥 **Ember Template Engine**
- **Laravel Blade-inspired syntax** - `@extends`, `@section`, `@foreach`
- **Template inheritance** for consistent layouts across pages
- **Component system** with `@include` for reusable templates
- **Automatic XSS protection** and input escaping
- **Hot reloading** in development, intelligent caching in production

```rust
use torch_web::{ember::*, main};

#[main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = App::new()
        .get("/", |_req| async {
            let data = EmberData::new()
                .with("title", "Welcome to Torch")
                .with("users", vec!["Alice", "Bob", "Charlie"]);

            ember("home", data).await
        });

    app.listen("127.0.0.1:3000").await
}
```

**Template file (`templates/home.ember`):**
```html
@extends('layout')

@section('content')
    <h1>{{ $title }}</h1>

    @if(count($users) > 0)
        <ul>
        @foreach($users as $user)
            <li>🔥 {{ $user }}</li>
        @endforeach
        </ul>
    @else
        <p>No users found.</p>
    @endif
@endsection
```

### 🏗️ **Modular Architecture**
- **Multi-crate project structure** - Prevent large monolithic crates
- **Workspace support** - Organize code across focused crates
- **Clear separation of concerns** - Core, Web, Auth, Database layers
- **Team scalability** - Multiple teams can work in parallel

```
my-torch-app/
├── crates/
│   ├── core/          # Business logic
│   ├── web/           # Torch web application
│   ├── auth/          # Authentication
│   ├── database/      # Data layer
│   └── api/           # External integrations
```

### 🎯 **Type-Safe Extractors**
- **Path parameters** - Extract `:id`, `:name` with automatic type conversion
- **Query strings** - Parse `?key=value` into structs or HashMaps
- **JSON bodies** - Deserialize request bodies with serde
- **Headers** - Access any HTTP header with type safety
- **Application state** - Share data across handlers with dependency injection
- **Multiple extractors** - Combine any extractors in a single handler

### 🚀 **High Performance**
- Built on **Tokio + Hyper** for maximum async performance
- Handles thousands of concurrent connections efficiently
- Zero-copy parsing and minimal allocations
- HTTP/1.1 and HTTP/2 support

### 🛡️ **Security First**
- **Input validation** and sanitization
- **HMAC request signing** for API security
- **IP whitelisting** and rate limiting
- **Security headers** and CSRF protection

### 📊 **Production Ready**
- **Structured logging** with tracing support
- **Metrics collection** for monitoring
- **Health checks** and graceful shutdown
- **Configuration management** via TOML and environment variables

### ⚡ **Real-time Support**
- **WebSocket** support for real-time applications
- **Server-Sent Events** (SSE) for live updates
- Connection management and broadcasting

### 🗄️ **Database & Caching**
- **PostgreSQL** support with connection pooling
- **Redis** caching integration
- **Query builder** for safe database operations
- **Migration runner** for schema management

### � **Beautiful Error Pages**
- **Stunning default error pages** with Torch branding
- **Sinatra-inspired 404 messages** with flame themes
- **Fully customizable** error page templates
- **Responsive design** that works on all devices

### 🔥 **Ember Template Engine**
- **Laravel Blade-inspired syntax** - familiar and powerful
- **Template inheritance** with `@extends` and `@section`
- **Component system** for reusable templates
- **Automatic XSS protection** and input escaping
- **Hot reloading** and intelligent caching
- **Zero-config setup** - just create `.ember` files

### 🔧 **Developer Experience**
- **Sinatra-inspired API** - familiar and intuitive
- **Type-safe** request/response handling
- **Middleware system** for composable functionality
- **Hot reloading** in development mode

## 🛠️ Developer Tools

- **[VS Code Extension](https://github.com/Enigmatikk/torch-vscode)** - Syntax highlighting and IntelliSense for Ember templates
- **[Marketplace](https://marketplace.visualstudio.com/search?term=torch%20ember&target=VSCode)** - Install from VS Code Extensions

```bash
# Install VS Code extension for .ember template support
code --install-extension enigmatikk.torch-ember
```

## 🚀 Quick Start

### Installation

**For full-featured applications with templates:**
```bash
cargo add torch-web --features templates,json,database
```

**For API-only applications:**
```bash
cargo add torch-web --features json
```

**For maximum features (production apps):**
```toml
[dependencies]
torch-web = {
    version = "0.2.2",
    features = ["templates", "json", "database", "cache", "websocket"]
}
```

**Available Features:**
- `templates` - Ember templating engine with Laravel Blade-like syntax
- `json` - JSON request/response handling with serde
- `database` - PostgreSQL support with SQLx
- `cache` - Redis caching integration
- `websocket` - WebSocket support for real-time apps
- `api` - Enhanced API development tools

### Hello World

```rust
use torch_web::{App, Request, Response, main};

#[main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = App::new()
        .get("/", |_req: Request| async {
            Response::ok().body("Hello, World! 🔥")
        })
        .get("/hello/:name", |req: Request| async move {
            let name = req.param("name").unwrap_or("Anonymous");
            Response::ok().body(format!("Hello, {}! 🔥", name))
        })
        .get("/json", |_req: Request| async {
            #[cfg(feature = "json")]
            {
                use serde_json::json;
                Response::ok()
                    .json(&json!({
                        "message": "Hello from Torch!",
                        "framework": "torch",
                        "version": "0.1.0"
                    }))
                    .unwrap()
            }
            #[cfg(not(feature = "json"))]
            {
                Response::ok()
                    .content_type("application/json")
                    .body(r#"{"message": "Hello from Torch!", "framework": "torch"}"#)
            }
        });

    println!("🔥 Starting Torch Hello World example...");
    app.listen("127.0.0.1:3000").await
}
```

### Try the Example

```bash
# Clone the repository
git clone https://github.com/Enigmatikk/torch.git
cd torch

# Run the hello world example
cargo run --example hello_world

# Visit http://localhost:3000 to see it in action!
```

## 🔥 Ember Template Engine

Torch includes **Ember**, a powerful templating engine inspired by Laravel's Blade but built for Rust performance:

```rust
use torch_web::{App, ember::*, main};

#[main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = App::new()
        .get("/", |_req| async {
            let data = EmberData::new()
                .with("title", "Welcome to Torch")
                .with("users", vec!["Alice", "Bob", "Charlie"]);

            ember("home", data).await
        });

    app.listen("127.0.0.1:3000").await
}
```

### Template Syntax

Create `templates/home.ember`:

```html
@extends('layout')

@section('content')
    <h1>{{ $title }}</h1>

    @if(count($users) > 0)
        <ul>
        @foreach($users as $user)
            <li>🔥 {{ $user }}</li>
        @endforeach
        </ul>
    @else
        <p>No users found.</p>
    @endif
@endsection
```

Create `templates/layout.ember`:

```html
<!DOCTYPE html>
<html>
<head>
    <title>{{ $title }} - My App</title>
</head>
<body>
    <div class="container">
        @section('content')
            <p>Default content</p>
        @endsection
    </div>
</body>
</html>
```

### Ember Features

- **🎨 Familiar Syntax**: Laravel Blade-inspired directives
- **🏗️ Template Inheritance**: `@extends`, `@section`, `@endsection`
- **🔄 Loops & Conditionals**: `@foreach`, `@if`, `@else`, `@endif`
- **📦 Components**: `@include('partial')` for reusable templates
- **🔒 Auto-Escaping**: XSS protection built-in
- **⚡ Performance**: Compiled templates with intelligent caching
- **🔥 Hot Reload**: Templates update automatically in development

## 🚀 Real-World Example: Multi-Step Registration Wizard

Here's a comprehensive example showing how Torch's features work together in a production-ready application:

```rust
use torch_web::{App, Request, Response, main, ember::*};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// Step 1: Basic Information
#[derive(Debug, Deserialize, Serialize, Clone)]
struct BasicInfo {
    first_name: String,
    last_name: String,
    email: String,
    phone: String,
}

// Registration wizard state
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
struct RegistrationData {
    basic_info: Option<BasicInfo>,
    current_step: u8,
}

// Session store (use Redis in production)
type SessionStore = std::sync::Arc<std::sync::Mutex<HashMap<String, RegistrationData>>>;

#[main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let sessions: SessionStore = std::sync::Arc::new(std::sync::Mutex::new(HashMap::new()));
    let sessions_clone = sessions.clone();

    let app = App::new()
        // Home page with beautiful template
        .get::<_, ()>("/", |_req: Request| async {
            let data = EmberData::new()
                .with("title", "Welcome to Torch")
                .with("page_title", "Multi-Step Registration Wizard")
                .with("description", "Experience the power of Torch with Ember templating");

            ember("home", data).await
        })

        // Registration wizard - Step 1
        .get::<_, ()>("/register", move |req: Request| {
            let sessions = sessions_clone.clone();
            async move {
                let session_id = get_or_create_session(&req);
                let registration_data = get_session_data(&sessions, &session_id);

                let data = EmberData::new()
                    .with("title", "Registration - Step 1")
                    .with("step", 1)
                    .with("step_title", "Basic Information")
                    .with("progress", 33)
                    .with("first_name", registration_data.basic_info
                        .as_ref().map(|b| b.first_name.clone()).unwrap_or_default());

                ember("registration/step1", data).await
            }
        })

        // Handle form submission with session state
        .post::<_, ()>("/register/step1", move |req: Request| {
            let sessions = sessions.clone();
            async move {
                let session_id = get_or_create_session(&req);

                // Parse form data (simplified for example)
                let basic_info = BasicInfo {
                    first_name: "John".to_string(),
                    last_name: "Doe".to_string(),
                    email: "john.doe@example.com".to_string(),
                    phone: "+1-555-0123".to_string(),
                };

                // Update session state
                update_session_data(&sessions, &session_id, |data| {
                    data.basic_info = Some(basic_info);
                    data.current_step = 2;
                });

                // Redirect to next step
                Response::redirect_found("/register/step2")
            }
        });

    println!("🔥 Torch Registration Wizard starting...");
    println!("🌐 Visit http://localhost:3000 to see the demo");

    app.listen("127.0.0.1:3000").await
}

// Helper functions for session management
fn get_or_create_session(req: &Request) -> String {
    req.header("x-session-id").unwrap_or("demo-session").to_string()
}

fn get_session_data(sessions: &SessionStore, session_id: &str) -> RegistrationData {
    let sessions = sessions.lock().unwrap();
    sessions.get(session_id).cloned().unwrap_or_default()
}

fn update_session_data<F>(sessions: &SessionStore, session_id: &str, updater: F)
where F: FnOnce(&mut RegistrationData)
{
    let mut sessions = sessions.lock().unwrap();
    let mut data = sessions.get(session_id).cloned().unwrap_or_default();
    updater(&mut data);
    sessions.insert(session_id.to_string(), data);
}
```

**Template Structure:**
```
templates/
├── layouts/
│   └── main.ember          # Base layout with CSS, header, footer
├── components/
│   ├── header.ember        # Navigation component
│   ├── footer.ember        # Footer component
│   └── progress_bar.ember  # Reusable progress indicator
└── registration/
    ├── step1.ember         # Extends main layout
    ├── step2.ember         # Extends main layout
    └── step3.ember         # Extends main layout
```

**Layout Template (`templates/layouts/main.ember`):**
```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>{{ $title }} - Torch Demo</title>
    <style>
        body { font-family: 'Segoe UI', sans-serif; margin: 0; }
        .container { max-width: 1200px; margin: 0 auto; padding: 2rem; }
        /* Beautiful responsive CSS... */
    </style>
</head>
<body>
    @include('components/header')

    <main class="container">
        @section('content')
            <p>Default content</p>
        @endsection
    </main>

    @include('components/footer')
</body>
</html>
```

**Step Template (`templates/registration/step1.ember`):**
```html
@extends('layouts/main')

@section('content')
    @include('components/progress_bar')

    <div class="form-container">
        <h2>{{ $step_title }}</h2>

        <form method="POST" action="/register/step1">
            <div class="form-group">
                <label>First Name</label>
                <input type="text" name="first_name" value="{{ $first_name }}" required>
            </div>

            <div class="form-group">
                <label>Last Name</label>
                <input type="text" name="last_name" value="{{ $last_name }}" required>
            </div>

            <button type="submit">Continue to Step 2 →</button>
        </form>
    </div>
@endsection
```

This example demonstrates:
- **🔥 Template inheritance** for consistent layouts
- **📦 Component reuse** with `@include` directives
- **🔄 Session state management** across multiple steps
- **📝 Form handling** with validation and redirects
- **🎨 Beautiful responsive design** with consistent theming
- **🏗️ Modular structure** ready for team development

## 🎯 Type-Safe Extractors

Torch features a powerful extractors system that makes handling requests type-safe and ergonomic:

```rust
use torch_web::{App, main, extractors::*};
use std::collections::HashMap;

#[derive(Clone)]
struct AppState {
    counter: std::sync::Arc<tokio::sync::Mutex<u64>>,
}

#[main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let state = AppState {
        counter: std::sync::Arc::new(tokio::sync::Mutex::new(0)),
    };

    let app = App::new()
        .with_state(state)

        // Path parameters
        .get("/users/:id", |Path(user_id): Path<u32>| async move {
            format!("User ID: {}", user_id)
        })

        // Query parameters
        .get("/search", |Query(params): Query<HashMap<String, String>>| async move {
            let query = params.get("q").unwrap_or(&"*".to_string());
            format!("Searching for: {}", query)
        })

        // JSON body (with json feature)
        .post("/users", |Json(user): Json<serde_json::Value>| async move {
            format!("Creating user: {}", user)
        })

        // Headers
        .get("/info", |Headers(headers): Headers| async move {
            let user_agent = headers.get("user-agent")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("Unknown");
            format!("Your browser: {}", user_agent)
        })

        // Application state
        .post("/increment", |State(state): State<AppState>| async move {
            let mut counter = state.counter.lock().await;
            *counter += 1;
            format!("Counter: {}", *counter)
        })

        // Multiple extractors
        .get("/api/:version/search", |
            Path(version): Path<String>,
            Query(params): Query<HashMap<String, String>>,
            State(state): State<AppState>,
        | async move {
            let counter = state.counter.lock().await;
            let query = params.get("q").unwrap_or(&"*".to_string());
            format!("API v{}: Searching '{}' (requests: {})", version, query, *counter)
        });

    app.listen("127.0.0.1:3000").await
}
```

### Available Extractors

- **`Path<T>`** - Extract path parameters (`:id`, `:name`, etc.)
- **`Query<T>`** - Extract query string parameters (`?key=value`)
- **`Json<T>`** - Extract and deserialize JSON request bodies
- **`Headers`** - Access request headers
- **`State<T>`** - Access shared application state
- **Multiple extractors** - Combine any extractors in a single handler

## 🎨 Beautiful Error Pages

One of Torch's standout features is its beautiful, Sinatra-inspired error pages:

### Fun 404 Messages
Torch includes rotating 404 messages with flame themes:
- *"🔥 Torch doesn't know this ditty, but it's got plenty of other hot tracks!"*
- *"🔥 This path hasn't been lit by the Torch yet."*
- *"🔥 Even the brightest flame can't illuminate this missing page."*

### Stunning Design
- **Modern dark theme** with professional gradients
- **Torch branding** with beautiful SVG flame logo
- **Fully responsive** - works on desktop, tablet, and mobile
- **Smooth animations** and hover effects

### Customizable
```rust
use torch_web::ErrorPages;

let custom_pages = ErrorPages::new()
    .custom_404("Your custom 404 HTML here")
    .custom_500("Your custom 500 HTML here");

let app = App::new()
    .error_pages(custom_pages);
```

## 🔧 Feature Flags

Torch uses feature flags to keep your binary size small:

- **`default`** - Includes JSON support
- **`json`** - JSON serialization with serde
- **`production`** - All production features (monitoring, security, etc.)
- **`security`** - Security middleware and utilities
- **`websocket`** - WebSocket and real-time features
- **`database`** - PostgreSQL support with connection pooling
- **`cache`** - Redis caching integration
- **`api`** - API documentation generation
- **`config`** - TOML configuration support
- **`monitoring`** - Metrics and structured logging

## 🏗️ Architecture

Torch is built on proven Rust technologies:

- **[Tokio](https://tokio.rs/)** - Async runtime for high performance
- **[Hyper](https://hyper.rs/)** - Fast HTTP implementation
- **[Tower](https://github.com/tower-rs/tower)** - Middleware and service abstractions
- **[Serde](https://serde.rs/)** - Serialization framework
- **[Tracing](https://tracing.rs/)** - Structured logging and diagnostics

## 🔧 Configuration

Torch supports configuration through TOML files and environment variables.

### Configuration File

Create a `torch.toml` file in your project root:

```toml
[server]
host = "0.0.0.0"
port = 8080
max_connections = 10000
request_timeout_secs = 30

[security]
enable_cors = true
enable_security_headers = true
enable_rate_limiting = true
per_ip_rps_limit = 100

[monitoring]
enable_metrics = true
enable_request_logging = true
log_level = "info"

[database]
url = "postgresql://user:pass@localhost/db"
max_connections = 10

[cache]
redis_url = "redis://localhost:6379"
default_ttl_secs = 3600
```

### Environment Variables

```bash
export TORCH_HOST=0.0.0.0
export TORCH_PORT=8080
export TORCH_DATABASE_URL=postgresql://user:pass@localhost/db
export TORCH_REDIS_URL=redis://localhost:6379
```

## 🛡️ Security Features

```rust
use torch_web::security::*;

// Input validation and sanitization
let app = App::new()
    .middleware(InputValidator::new())
    .middleware(SecurityHeaders::new())
    .middleware(RateLimiter::new(100)); // 100 requests per second

// HMAC request signing
let signing = RequestSigning::new("your-secret-key");
let app = app.middleware(signing);

// IP whitelisting
let whitelist = IpWhitelist::new()
    .allow_ip("192.168.1.1")
    .allow_range("10.0.0.0/8");
let app = app.middleware(whitelist);
```

## 📊 Production Features

```rust
use torch_web::production::*;

// Metrics and monitoring
let app = App::new()
    .middleware(MetricsCollector::new())
    .middleware(PerformanceMonitor::new())
    .middleware(RequestLogger::new());

// Health check endpoint
let app = app.get("/health", |_req| async {
    Response::ok().json(&serde_json::json!({
        "status": "healthy",
        "uptime": "24h",
        "version": "0.1.0"
    })).unwrap()
});
```

## ⚡ Advanced Features

### 🏗️ Modular Project Structure

Torch supports organizing large applications across multiple crates to prevent any single crate from becoming too large:

```toml
# Workspace Cargo.toml
[workspace]
members = [
    "crates/core",      # Business logic
    "crates/web",       # Torch web application
    "crates/auth",      # Authentication
    "crates/database",  # Data layer
    "crates/api",       # External integrations
]

[workspace.dependencies]
torch-web = { version = "0.2.2", features = ["templates", "json"] }
```

**Benefits:**
- ✅ **Faster builds** - Only changed crates are recompiled
- ✅ **Parallel compilation** - Crates can be compiled in parallel
- ✅ **Clear dependencies** - Dependency graph is explicit
- ✅ **Team scalability** - Multiple teams can work simultaneously
- ✅ **Code reuse** - Share components across different applications

### ⚡ Compile-Time Route Validation

Leverage Rust's fantastic compiler for zero-cost route registration:

```rust
use torch_web::{routes, get, post, Path, Query, Json};

// Compile-time validated routes with type-safe extractors
routes! {
    #[get("/users/{id}")]
    async fn get_user(Path(id): Path<u32>) -> Response {
        // id is guaranteed to be a valid u32 at compile time
        Response::ok().json(format!("User {}", id))
    }

    #[get("/users")]
    async fn list_users(Query(params): Query<UserQuery>) -> Response {
        // params is type-checked at compile time
        Response::ok().json(params)
    }

    #[post("/users")]
    async fn create_user(Json(user): Json<CreateUserRequest>) -> Response {
        // JSON deserialization is validated at compile time
        Response::created().json("User created")
    }
}
```

**Compile-Time Benefits:**
- ✅ **Zero runtime overhead** - All validation happens at build time
- ✅ **Type safety** - Parameters are type-checked by the compiler
- ✅ **IDE support** - Full autocomplete and error checking
- ✅ **Early error detection** - Catch route issues before deployment

### 🎨 Consistent Theming System

Build applications with consistent headers, footers, and menus across multiple pages:

```rust
// Shared layout with navigation
// templates/layouts/app.ember
```html
<!DOCTYPE html>
<html>
<head>
    <title>{{ $title }} - My App</title>
    <link rel="stylesheet" href="/css/app.css">
</head>
<body>
    @include('components/header')
    @include('components/navigation')

    <main class="content">
        @section('content')
            <p>Default content</p>
        @endsection
    </main>

    @include('components/footer')
</body>
</html>
```

```rust
// All pages inherit the same layout
// templates/users/index.ember
```html
@extends('layouts/app')

@section('content')
    <h1>{{ $page_title }}</h1>
    @foreach($users as $user)
        <div class="user-card">{{ $user.name }}</div>
    @endforeach
@endsection
```

**Theming Benefits:**
- ✅ **Consistent design** - Shared layouts ensure visual consistency
- ✅ **Component reuse** - Headers, footers, menus defined once
- ✅ **Easy maintenance** - Update navigation in one place
- ✅ **Responsive design** - CSS and JavaScript shared across pages

## 🔄 Middleware System

Torch provides a powerful and flexible middleware system:

```rust
use torch_web::middleware::*;

// Built-in middleware
let app = App::new()
    .middleware(Logger::new())
    .middleware(Cors::permissive())
    .middleware(SecurityHeaders::new())
    .middleware(Compression::new());

// Custom middleware
let app = app.middleware(|req: Request, next| {
    Box::pin(async move {
        let start = std::time::Instant::now();
        let response = next(req).await;
        let duration = start.elapsed();
        println!("Request took: {:?}", duration);
        response
    })
});
```

## 🎯 Use Cases

### Web APIs
- **REST APIs** with JSON serialization
- **GraphQL** endpoints
- **Microservices** architecture
- **Real-time applications** with WebSockets

### Production Applications
- **High-traffic websites** with caching
- **Enterprise applications** with security
- **Data processing** pipelines
- **Integration services** with monitoring

## 📈 Performance

Torch is built for speed and efficiency:

**Why Torch is fast:**
- **Zero-copy parsing** - Minimal allocations in hot paths
- **Async all the way down** - Built on Tokio's proven runtime
- **Smart defaults** - Optimized configurations out of the box
- **Efficient routing** - Fast path matching with minimal overhead

**Benchmark it yourself:**
```bash
# Clone the repository
git clone https://github.com/Enigmatikk/torch.git
cd torch

# Run the hello world example
cargo run --example hello_world --release

# Test with your favorite load testing tool
wrk -t12 -c400 -d30s http://localhost:3000/
```

## 🧪 Try It Now

```bash
# Clone and run in 30 seconds
git clone https://github.com/Enigmatikk/torch.git
cd torch

# Run the hello world example
cargo run --example hello_world

# Visit http://localhost:3000 to see:
# - Hello World endpoint
# - Path parameters (/hello/:name)
# - JSON responses (/json)
# - Beautiful 404 pages (try /nonexistent)
```

## 🚀 Requirements

- **Rust 1.75+** (uses latest async features)
- **Tokio runtime** (included with Torch)

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/Enigmatikk/torch.git
cd torch

# Run tests
cargo test --all-features

# Run examples
cargo run --example hello_world                    # Basic routing
cargo run --example registration_wizard --features templates,json  # Multi-step wizard
cargo run --example ember_demo --features templates               # Template showcase
cargo run --example enhanced_extractors            # Type-safe extractors

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy --all-features
```

## 📄 License

This project is licensed under the **MIT OR Apache-2.0** license.

## 🙏 Acknowledgments

- **[Sinatra](http://sinatrarb.com/)** - Inspired our simple, intuitive API design
- **[Axum](https://github.com/tokio-rs/axum)** - Architectural inspiration for middleware
- **Rust Community** - For building an amazing ecosystem

## 🚀 What's Next?

1. **⭐ Star this repo** if Torch looks useful to you
2. **🧪 Try the registration wizard** - `cargo run --example registration_wizard --features templates,json`
3. **🔥 Explore Ember templates** - See how template inheritance and components work
4. **🏗️ Build a modular app** - Use the multi-crate structure for large projects
5. **⚡ Leverage compile-time routes** - Get type safety and zero-cost abstractions
6. **🤝 Contribute** - we'd love your help making Torch even better

### Join the Community

- 🐛 **Found a bug?** [Open an issue](https://github.com/Enigmatikk/torch/issues)
- 💡 **Have an idea?** [Start a discussion](https://github.com/Enigmatikk/torch/discussions)
- 🤝 **Want to contribute?** Check out [CONTRIBUTING.md](CONTRIBUTING.md)
- 📢 **Using Torch?** We'd love to hear your story!

---

**Built with ❤️ and 🔥 for developers who ship fast**

*Torch - The web framework that doesn't get in your way* 🔥
