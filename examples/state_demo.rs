use torch_web::{App, main, extractors::*};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;

// Application state structures
#[derive(Clone)]
struct AppState {
    counter: Arc<Mutex<u64>>,
    message: String,
}

#[derive(Clone)]
struct DatabasePool {
    connections: Arc<Mutex<Vec<String>>>,
}

impl DatabasePool {
    fn new() -> Self {
        Self {
            connections: Arc::new(Mutex::new(vec![
                "connection_1".to_string(),
                "connection_2".to_string(),
                "connection_3".to_string(),
            ])),
        }
    }

    async fn get_connection(&self) -> Option<String> {
        let mut connections = self.connections.lock().await;
        connections.pop()
    }

    async fn return_connection(&self, conn: String) {
        let mut connections = self.connections.lock().await;
        connections.push(conn);
    }
}

#[main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Create application state
    let app_state = AppState {
        counter: Arc::new(Mutex::new(0)),
        message: "🔥 Welcome to Torch State Management!".to_string(),
    };

    let db_pool = DatabasePool::new();

    let app = App::new()
        // Add multiple types of state
        .with_state(app_state)
        .with_state(db_pool)
        
        // Basic state access
        .get("/", |State(state): State<AppState>| async move {
            format!("{} (Server started)", state.message)
        })
        
        // Counter endpoint - demonstrates mutable state
        .get("/counter", |State(state): State<AppState>| async move {
            let counter = state.counter.lock().await;
            format!("🔥 Current counter: {}", *counter)
        })
        
        .post("/counter/increment", |State(state): State<AppState>| async move {
            let mut counter = state.counter.lock().await;
            *counter += 1;
            format!("🔥 Counter incremented to: {}", *counter)
        })
        
        .post("/counter/reset", |State(state): State<AppState>| async move {
            let mut counter = state.counter.lock().await;
            *counter = 0;
            format!("🔥 Counter reset to: {}", *counter)
        })
        
        // Database pool example - demonstrates multiple state types
        .get("/db/status", |State(db): State<DatabasePool>| async move {
            let connections = db.connections.lock().await;
            format!("🔥 Available database connections: {}", connections.len())
        })
        
        .get("/db/connection", |State(db): State<DatabasePool>| async move {
            match db.get_connection().await {
                Some(conn) => {
                    // Simulate using the connection
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    db.return_connection(conn.clone()).await;
                    format!("🔥 Used connection: {}", conn)
                }
                None => "🔥 No connections available".to_string(),
            }
        })
        
        // Multiple extractors with state
        .get("/api/:version/stats", |
            Path(version): Path<String>,
            Query(params): Query<HashMap<String, String>>,
            State(app_state): State<AppState>,
            State(db): State<DatabasePool>,
        | async move {
            let counter = app_state.counter.lock().await;
            let db_connections = db.connections.lock().await;
            let include_details = params.get("details").map(|v| v == "true").unwrap_or(false);
            
            if include_details {
                format!(
                    "🔥 API v{}: Counter={}, DB Connections={}, Message='{}'",
                    version, *counter, db_connections.len(), app_state.message
                )
            } else {
                format!(
                    "🔥 API v{}: Counter={}, DB Connections={}",
                    version, *counter, db_connections.len()
                )
            }
        });

    println!("🔥 Starting Torch State Management Demo...");
    println!("🔥 Try these endpoints:");
    println!("   GET  /                           - Basic state access");
    println!("   GET  /counter                    - View counter");
    println!("   POST /counter/increment          - Increment counter");
    println!("   POST /counter/reset              - Reset counter");
    println!("   GET  /db/status                  - Database pool status");
    println!("   GET  /db/connection              - Use a database connection");
    println!("   GET  /api/v1/stats               - Multiple state types");
    println!("   GET  /api/v1/stats?details=true  - Detailed stats");
    println!();
    
    app.listen("127.0.0.1:3001").await
}
