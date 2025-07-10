# Torch 🔥

**The web framework that doesn't get in your way.**

Torch is a fast, secure, and production-ready web framework for Rust. Built on Tokio and Hyper, it provides everything you need to build modern web applications with minimal configuration.

```rust
use torch::{App, Request, Response};

#[tokio::main]
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
- 🛡️ **Secure by Design** - Security features and beautiful error pages included
- 📊 **Production Ready** - Monitoring, caching, and database support
- ⚡ **Real-time Capable** - WebSocket and SSE support out of the box
- 🎯 **Simple & Familiar** - Sinatra-inspired API that just works
- 😄 **Fun Error Pages** - Beautiful 404 pages with rotating flame-themed messages

## ✨ Features

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

### 🔧 **Developer Experience**
- **Sinatra-inspired API** - familiar and intuitive
- **Type-safe** request/response handling
- **Middleware system** for composable functionality
- **Hot reloading** in development mode

## 🚀 Quick Start

### Installation

Add Torch to your `Cargo.toml`:

```toml
[dependencies]
torch = "0.1.0"
tokio = { version = "1.0", features = ["full"] }

# For JSON support (recommended)
torch = { version = "0.1.0", features = ["json"] }

# For production features
torch = { version = "0.1.0", features = ["production"] }

# All features
torch = { version = "0.1.0", features = ["production", "websocket", "database", "cache"] }
```

### Hello World

```rust
use torch::{App, Request, Response};

#[tokio::main]
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
use torch::ErrorPages;

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
use torch::security::*;

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
use torch::production::*;

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

## 🔄 Middleware System

Torch provides a powerful and flexible middleware system:

```rust
use torch::middleware::*;

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

# Run the example
cargo run --example hello_world

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
2. **🧪 Try the example** to see how it feels
3. **🔥 Build something awesome** and let us know about it
4. **🤝 Contribute** - we'd love your help making Torch even better

### Join the Community

- 🐛 **Found a bug?** [Open an issue](https://github.com/Enigmatikk/torch/issues)
- 💡 **Have an idea?** [Start a discussion](https://github.com/Enigmatikk/torch/discussions)
- 🤝 **Want to contribute?** Check out [CONTRIBUTING.md](CONTRIBUTING.md)
- 📢 **Using Torch?** We'd love to hear your story!

---

**Built with ❤️ and 🔥 for developers who ship fast**

*Torch - The web framework that doesn't get in your way* 🔥
