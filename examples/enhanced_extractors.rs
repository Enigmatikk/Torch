use torch_web::{App, main, extractors::*};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// Custom structs for demonstration
#[derive(Deserialize, Serialize, Debug)]
struct User {
    name: String,
    email: String,
    age: Option<u32>,
}

#[derive(Deserialize, Debug)]
struct SearchParams {
    q: String,
    page: Option<u32>,
    limit: Option<u32>,
    sort: Option<String>,
}

#[derive(Deserialize, Debug)]
struct LoginForm {
    username: String,
    password: String,
    remember_me: Option<bool>,
}

#[derive(Clone)]
struct AppState {
    request_count: std::sync::Arc<tokio::sync::Mutex<u64>>,
}

#[main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let state = AppState {
        request_count: std::sync::Arc::new(tokio::sync::Mutex::new(0)),
    };

    let app = App::new()
        .with_state(state)
        
        // Enhanced Path Extractors
        .get("/users/:id", |Path(user_id): Path<u32>| async move {
            format!("🔥 User ID: {} (type: u32)", user_id)
        })
        
        .get("/posts/:id", |Path(post_id): Path<String>| async move {
            format!("🔥 Post ID: {} (type: String)", post_id)
        })
        
        .get("/api/:version/users/:user_id", |Path((version, user_id)): Path<(String, u32)>| async move {
            format!("🔥 API v{}, User: {}", version, user_id)
        })
        
        // Enhanced Query Extractors
        .get("/search", |Query(params): Query<HashMap<String, String>>| async move {
            format!("🔥 Search params: {:?}", params)
        })
        
        .get("/items", |Query(params): Query<HashMap<String, String>>| async move {
            format!("🔥 Query params: {:?}", params)
        })
        
        // Form Data Extractor (manual extraction for now)
        .post("/login", |req: torch_web::Request| async move {
            // Manual form extraction example
            let body = std::str::from_utf8(req.body_bytes()).unwrap_or("");
            format!("🔥 Login form data: {}", body)
        })

        .post("/contact", |req: torch_web::Request| async move {
            // Manual form extraction example
            let body = std::str::from_utf8(req.body_bytes()).unwrap_or("");
            format!("🔥 Contact form data: {}", body)
        })
        
        // Cookie Extractors
        .get("/profile", |
            SessionCookie(session): SessionCookie,
            Cookies(all_cookies): Cookies,
        | async move {
            match session {
                Some(session_id) => format!("🔥 Welcome back! Session: {}", session_id),
                None => format!("🔥 No session found. All cookies: {:?}", all_cookies),
            }
        })
        
        // JSON with enhanced error handling (manual extraction for now)
        .post("/users", |req: torch_web::Request| async move {
            let body = std::str::from_utf8(req.body_bytes()).unwrap_or("{}");
            format!("🔥 Creating user from JSON: {}", body)
        })
        
        // Multiple extractors with state
        .get("/dashboard/:user_id", |
            Path(user_id): Path<u32>,
            Query(params): Query<HashMap<String, String>>,
            SessionCookie(session): SessionCookie,
            State(state): State<AppState>,
        | async move {
            let mut count = state.request_count.lock().await;
            *count += 1;
            
            let theme = params.get("theme").unwrap_or(&"default".to_string()).clone();
            
            match session {
                Some(session_id) => {
                    format!("🔥 Dashboard for user {} (session: {}, theme: {}, requests: {})", 
                           user_id, session_id, theme, *count)
                }
                None => {
                    format!("🔥 Dashboard for user {} (no session, theme: {}, requests: {})", 
                           user_id, theme, *count)
                }
            }
        })
        
        // Error demonstration endpoints
        .get("/error/path/:invalid", |Path(num): Path<u32>| async move {
            format!("This won't work if :invalid is not a number: {}", num)
        })

        .post("/error/json", |req: torch_web::Request| async move {
            let body = std::str::from_utf8(req.body_bytes()).unwrap_or("{}");
            format!("This requires valid JSON: {}", body)
        })
        
        // Cookie setting example
        .get("/set-session", || async {
            let cookie = CookieBuilder::new("session_id", "abc123")
                .path("/")
                .max_age(3600)
                .http_only(true)
                .same_site(SameSite::Lax)
                .build();
            
            format!("🔥 Session set! Cookie: {}", cookie)
        });

    println!("🔥 Starting Enhanced Extractors Demo...");
    println!("🔥 Try these endpoints:");
    println!();
    println!("📍 Path Extractors:");
    println!("   GET  /users/123                     - u32 path param");
    println!("   GET  /posts/hello-world             - String path param");
    println!("   GET  /api/v1/users/456              - Multiple path params");
    println!();
    println!("🔍 Query Extractors:");
    println!("   GET  /search?q=rust&page=1&limit=10 - Structured query params");
    println!("   GET  /items?category=tech&sort=date - HashMap query params");
    println!();
    println!("📝 Form Data:");
    println!("   POST /login                         - Form data (username, password)");
    println!("   POST /contact                       - Raw form data");
    println!();
    println!("🍪 Cookies:");
    println!("   GET  /profile                       - Session cookie extraction");
    println!("   GET  /set-session                   - Set a session cookie");
    println!();
    println!("📊 JSON:");
    println!("   POST /users                         - JSON user creation");
    println!();
    println!("🎛️  Complex:");
    println!("   GET  /dashboard/123?theme=dark      - Multiple extractors + state");
    println!();
    println!("❌ Error Examples:");
    println!("   GET  /error/path/notanumber         - Path parsing error");
    println!("   POST /error/json                    - JSON parsing error");
    println!();
    
    app.listen("127.0.0.1:3002").await
}
