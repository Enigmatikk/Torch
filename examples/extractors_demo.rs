use torch_web::{App, main, extractors::*};
use std::collections::HashMap;

#[main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = App::new()
        // Basic handler with no extractors
        .get("/", || async {
            "🔥 Welcome to Torch Extractors Demo!"
        })
        
        // Path parameter extraction
        .get("/users/:id", |Path(user_id): Path<u32>| async move {
            format!("🔥 User ID: {}", user_id)
        })
        
        // Multiple path parameters
        .get("/users/:user_id/posts/:post_id", |Path((user_id, post_id)): Path<(String, String)>| async move {
            format!("🔥 User: {}, Post: {}", user_id, post_id)
        })
        
        // Query parameter extraction
        .get("/search", |Query(params): Query<HashMap<String, String>>| async move {
            if let Some(q) = params.get("q") {
                format!("🔥 Searching for: {}", q)
            } else {
                "🔥 No search query provided".to_string()
            }
        })
        
        // Headers extraction
        .get("/headers", |Headers(headers): Headers| async move {
            let user_agent = headers.get("user-agent")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("Unknown");
            format!("🔥 Your User-Agent: {}", user_agent)
        });

    #[cfg(feature = "json")]
    let app = app.post("/users", |Json(user): Json<serde_json::Value>| async move {
        format!("🔥 Creating user: {}", user)
    });

    let app = app
        
        // Multiple extractors combined
        .get("/api/:version/search", |
            Path(version): Path<String>,
            Query(params): Query<HashMap<String, String>>,
            Headers(headers): Headers,
        | async move {
            let query = params.get("q").unwrap_or(&"*".to_string()).clone();
            let user_agent = headers.get("user-agent")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("Unknown");
            
            format!(
                "🔥 API v{}: Searching '{}' from {}",
                version, query, user_agent
            )
        });

    println!("🔥 Starting Torch Extractors Demo...");
    println!("🔥 Try these endpoints:");
    println!("   GET  /                           - Basic handler");
    println!("   GET  /users/123                  - Path parameter");
    println!("   GET  /users/john/posts/hello     - Multiple path params");
    println!("   GET  /search?q=rust              - Query parameters");
    println!("   GET  /headers                    - Headers extraction");
    println!("   POST /users                      - JSON body (send JSON)");
    println!("   GET  /api/v1/search?q=torch      - Multiple extractors");
    println!();
    
    app.listen("127.0.0.1:3000").await
}
