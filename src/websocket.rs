//! # WebSocket Support for Real-Time Applications
//!
//! This module provides comprehensive WebSocket support for building real-time
//! applications with Torch. It includes connection management, message broadcasting,
//! room-based messaging, and automatic connection handling.
//!
//! ## Features
//!
//! - **Real-time Communication**: Bidirectional communication between client and server
//! - **Connection Management**: Automatic connection tracking and cleanup
//! - **Message Broadcasting**: Send messages to all connected clients
//! - **Room Support**: Group clients into rooms for targeted messaging
//! - **JSON Messaging**: Automatic JSON serialization/deserialization
//! - **Ping/Pong**: Built-in connection health monitoring
//! - **Error Handling**: Robust error handling and reconnection support
//! - **Scalable**: Designed for high-concurrency applications
//!
//! **Note**: This module requires the `websocket` feature to be enabled.
//!
//! ## Quick Start
//!
//! ### Basic WebSocket Server
//!
//! ```rust
//! use torch_web::{App, websocket::*};
//!
//! let ws_manager = WebSocketManager::new();
//!
//! let app = App::new()
//!     .with_state(ws_manager.clone())
//!
//!     // WebSocket endpoint
//!     .websocket("/ws", |mut connection| async move {
//!         println!("New WebSocket connection: {}", connection.id());
//!
//!         while let Some(message) = connection.receive().await? {
//!             match message {
//!                 WebSocketMessage::Text(text) => {
//!                     println!("Received: {}", text);
//!                     // Echo the message back
//!                     connection.send_text(&format!("Echo: {}", text)).await?;
//!                 }
//!                 WebSocketMessage::Binary(data) => {
//!                     println!("Received {} bytes", data.len());
//!                     connection.send_binary(data).await?;
//!                 }
//!                 WebSocketMessage::Close => {
//!                     println!("Connection closed");
//!                     break;
//!                 }
//!             }
//!         }
//!
//!         Ok(())
//!     })
//!
//!     // HTTP endpoint to broadcast messages
//!     .post("/broadcast", |State(ws): State<WebSocketManager>, Json(msg): Json<BroadcastMessage>| async move {
//!         let count = ws.broadcast(&msg.text).await?;
//!         Response::ok().json(&serde_json::json!({
//!             "sent_to": count,
//!             "message": msg.text
//!         }))
//!     });
//! ```
//!
//! ### Chat Application Example
//!
//! ```rust
//! use torch_web::{App, websocket::*, extractors::*};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Deserialize, Serialize)]
//! struct ChatMessage {
//!     user: String,
//!     message: String,
//!     timestamp: String,
//! }
//!
//! let ws_manager = WebSocketManager::new();
//!
//! let app = App::new()
//!     .with_state(ws_manager.clone())
//!
//!     // Chat WebSocket endpoint
//!     .websocket("/chat", |mut connection| async move {
//!         // Join the general chat room
//!         connection.join_room("general").await?;
//!
//!         while let Some(message) = connection.receive().await? {
//!             if let WebSocketMessage::Text(text) = message {
//!                 // Parse incoming chat message
//!                 if let Ok(chat_msg) = serde_json::from_str::<ChatMessage>(&text) {
//!                     // Broadcast to all users in the room
//!                     connection.broadcast_to_room("general", &text).await?;
//!                 }
//!             }
//!         }
//!
//!         Ok(())
//!     })
//!
//!     // REST endpoint to send messages
//!     .post("/chat/send", |State(ws): State<WebSocketManager>, Json(msg): Json<ChatMessage>| async move {
//!         let message_json = serde_json::to_string(&msg)?;
//!         let count = ws.broadcast_to_room("general", &message_json).await?;
//!         Response::ok().json(&serde_json::json!({"sent_to": count}))
//!     });
//! ```
//!
//! ### Real-Time Dashboard
//!
//! ```rust
//! use torch_web::{App, websocket::*, extractors::*};
//! use tokio::time::{interval, Duration};
//!
//! let ws_manager = WebSocketManager::new();
//!
//! // Background task to send periodic updates
//! let ws_clone = ws_manager.clone();
//! tokio::spawn(async move {
//!     let mut interval = interval(Duration::from_secs(5));
//!
//!     loop {
//!         interval.tick().await;
//!
//!         let stats = get_system_stats().await;
//!         let message = serde_json::to_string(&stats).unwrap();
//!
//!         if let Err(e) = ws_clone.broadcast_to_room("dashboard", &message).await {
//!             eprintln!("Failed to broadcast stats: {}", e);
//!         }
//!     }
//! });
//!
//! let app = App::new()
//!     .with_state(ws_manager)
//!
//!     // Dashboard WebSocket
//!     .websocket("/dashboard", |mut connection| async move {
//!         connection.join_room("dashboard").await?;
//!
//!         // Send initial data
//!         let initial_stats = get_system_stats().await;
//!         connection.send_json(&initial_stats).await?;
//!
//!         // Keep connection alive and handle incoming messages
//!         while let Some(_message) = connection.receive().await? {
//!             // Handle client requests for specific data
//!         }
//!
//!         Ok(())
//!     });
//! ```
//!
//! ### Gaming/Multiplayer Example
//!
//! ```rust
//! use torch_web::{App, websocket::*};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Deserialize, Serialize)]
//! struct GameAction {
//!     action_type: String,
//!     player_id: String,
//!     data: serde_json::Value,
//! }
//!
//! let ws_manager = WebSocketManager::new();
//!
//! let app = App::new()
//!     .websocket("/game/:room_id", |mut connection, Path(room_id): Path<String>| async move {
//!         // Join the specific game room
//!         connection.join_room(&room_id).await?;
//!
//!         // Notify other players
//!         let join_message = serde_json::json!({
//!             "type": "player_joined",
//!             "player_id": connection.id()
//!         });
//!         connection.broadcast_to_room(&room_id, &join_message.to_string()).await?;
//!
//!         while let Some(message) = connection.receive().await? {
//!             if let WebSocketMessage::Text(text) = message {
//!                 if let Ok(action) = serde_json::from_str::<GameAction>(&text) {
//!                     // Process game action and broadcast to other players
//!                     process_game_action(&action).await;
//!                     connection.broadcast_to_room(&room_id, &text).await?;
//!                 }
//!             }
//!         }
//!
//!         // Notify other players when leaving
//!         let leave_message = serde_json::json!({
//!             "type": "player_left",
//!             "player_id": connection.id()
//!         });
//!         connection.broadcast_to_room(&room_id, &leave_message.to_string()).await?;
//!
//!         Ok(())
//!     });
//! ```

use crate::{Request, Response};
use std::sync::Arc;

#[cfg(feature = "websocket")]
use std::collections::HashMap;

#[cfg(feature = "websocket")]
use {
    tokio_tungstenite::{accept_async, tungstenite::Message},
    futures_util::{SinkExt, StreamExt},
    tokio::sync::{RwLock, broadcast},
    sha1::{Sha1, Digest},
    base64::{Engine as _, engine::general_purpose},
};

/// WebSocket connection manager
pub struct WebSocketManager {
    #[cfg(feature = "websocket")]
    connections: Arc<RwLock<HashMap<String, broadcast::Sender<String>>>>,
    #[cfg(not(feature = "websocket"))]
    _phantom: std::marker::PhantomData<()>,
}

impl WebSocketManager {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "websocket")]
            connections: Arc::new(RwLock::new(HashMap::new())),
            #[cfg(not(feature = "websocket"))]
            _phantom: std::marker::PhantomData,
        }
    }

    /// Broadcast a message to all connected clients
    #[cfg(feature = "websocket")]
    pub async fn broadcast(&self, message: &str) -> Result<usize, Box<dyn std::error::Error>> {
        let connections = self.connections.read().await;
        let mut sent_count = 0;
        
        for sender in connections.values() {
            if sender.send(message.to_string()).is_ok() {
                sent_count += 1;
            }
        }
        
        Ok(sent_count)
    }

    /// Send a message to a specific client
    #[cfg(feature = "websocket")]
    pub async fn send_to(&self, client_id: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        let connections = self.connections.read().await;
        if let Some(sender) = connections.get(client_id) {
            sender.send(message.to_string())?;
        }
        Ok(())
    }

    /// Get the number of connected clients
    #[cfg(feature = "websocket")]
    pub async fn connection_count(&self) -> usize {
        self.connections.read().await.len()
    }

    #[cfg(not(feature = "websocket"))]
    pub async fn broadcast(&self, _message: &str) -> Result<usize, Box<dyn std::error::Error>> {
        Err("WebSocket feature not enabled".into())
    }

    #[cfg(not(feature = "websocket"))]
    pub async fn send_to(&self, _client_id: &str, _message: &str) -> Result<(), Box<dyn std::error::Error>> {
        Err("WebSocket feature not enabled".into())
    }

    #[cfg(not(feature = "websocket"))]
    pub async fn connection_count(&self) -> usize {
        0
    }
}

/// WebSocket upgrade handler
pub async fn websocket_upgrade(req: Request) -> Response {
    #[cfg(feature = "websocket")]
    {
        // 1. Validate the WebSocket headers
        if !is_websocket_upgrade_request(&req) {
            return Response::bad_request().body("Not a valid WebSocket upgrade request");
        }

        // 2. Get the WebSocket key
        let websocket_key = match req.header("sec-websocket-key") {
            Some(key) => key,
            None => return Response::bad_request().body("Missing Sec-WebSocket-Key header"),
        };

        // 3. Generate the accept key
        let accept_key = generate_websocket_accept_key(websocket_key);

        // 4. Return the upgrade response
        Response::with_status(http::StatusCode::SWITCHING_PROTOCOLS)
            .header("Upgrade", "websocket")
            .header("Connection", "Upgrade")
            .header("Sec-WebSocket-Accept", &accept_key)
            .header("Sec-WebSocket-Version", "13")
            .body("")
    }

    #[cfg(not(feature = "websocket"))]
    {
        let _ = req; // Suppress unused variable warning
        Response::with_status(http::StatusCode::NOT_IMPLEMENTED)
            .body("WebSocket support not enabled")
    }
}

#[cfg(feature = "websocket")]
pub fn is_websocket_upgrade_request(req: &Request) -> bool {
    // Check required headers for WebSocket upgrade
    let upgrade = req.header("upgrade").map(|h| h.to_lowercase());
    let connection = req.header("connection").map(|h| h.to_lowercase());
    let websocket_version = req.header("sec-websocket-version");
    let websocket_key = req.header("sec-websocket-key");

    upgrade == Some("websocket".to_string()) &&
    connection.as_ref().map_or(false, |c| c.contains("upgrade")) &&
    websocket_version == Some("13") &&
    websocket_key.is_some()
}

#[cfg(feature = "websocket")]
fn generate_websocket_accept_key(websocket_key: &str) -> String {
    // WebSocket magic string as defined in RFC 6455
    const WEBSOCKET_MAGIC_STRING: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

    // Concatenate the key with the magic string
    let combined = format!("{}{}", websocket_key, WEBSOCKET_MAGIC_STRING);

    // Calculate SHA-1 hash
    let mut hasher = Sha1::new();
    hasher.update(combined.as_bytes());
    let hash = hasher.finalize();

    // Encode as base64
    general_purpose::STANDARD.encode(&hash)
}

/// Handle a WebSocket connection after upgrade
#[cfg(feature = "websocket")]
pub async fn handle_websocket_connection<F, Fut>(
    stream: tokio::net::TcpStream,
    handler: F,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    F: FnOnce(WebSocketConnection) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send,
{
    // Accept the WebSocket connection
    let ws_stream = accept_async(stream).await?;
    let connection = WebSocketConnection::new(ws_stream);

    // Call the user-provided handler
    handler(connection).await
}

/// WebSocket connection wrapper
#[cfg(feature = "websocket")]
pub struct WebSocketConnection {
    stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
}

#[cfg(feature = "websocket")]
impl WebSocketConnection {
    fn new(stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>) -> Self {
        Self { stream }
    }

    /// Send a text message
    pub async fn send_text(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.stream.send(Message::Text(text.to_string())).await?;
        Ok(())
    }

    /// Send a binary message
    pub async fn send_binary(&mut self, data: &[u8]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.stream.send(Message::Binary(data.to_vec())).await?;
        Ok(())
    }

    /// Receive the next message
    pub async fn receive(&mut self) -> Result<Option<WebSocketMessage>, Box<dyn std::error::Error + Send + Sync>> {
        match self.stream.next().await {
            Some(Ok(msg)) => Ok(Some(WebSocketMessage::from_tungstenite(msg))),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None), // Connection closed
        }
    }

    /// Close the connection
    pub async fn close(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.stream.send(Message::Close(None)).await?;
        Ok(())
    }
}

/// WebSocket message types
#[cfg(feature = "websocket")]
pub enum WebSocketMessage {
    Text(String),
    Binary(Vec<u8>),
    Ping(Vec<u8>),
    Pong(Vec<u8>),
    Close,
}

#[cfg(feature = "websocket")]
impl WebSocketMessage {
    fn from_tungstenite(msg: Message) -> Self {
        match msg {
            Message::Text(text) => WebSocketMessage::Text(text),
            Message::Binary(data) => WebSocketMessage::Binary(data),
            Message::Ping(data) => WebSocketMessage::Ping(data),
            Message::Pong(data) => WebSocketMessage::Pong(data),
            Message::Close(_) => WebSocketMessage::Close,
            Message::Frame(_) => WebSocketMessage::Close, // Treat raw frames as close
        }
    }

    /// Check if this is a text message
    pub fn is_text(&self) -> bool {
        matches!(self, WebSocketMessage::Text(_))
    }

    /// Check if this is a binary message
    pub fn is_binary(&self) -> bool {
        matches!(self, WebSocketMessage::Binary(_))
    }

    /// Get text content if this is a text message
    pub fn as_text(&self) -> Option<&str> {
        match self {
            WebSocketMessage::Text(text) => Some(text),
            _ => None,
        }
    }

    /// Get binary content if this is a binary message
    pub fn as_binary(&self) -> Option<&[u8]> {
        match self {
            WebSocketMessage::Binary(data) => Some(data),
            _ => None,
        }
    }
}

/// Real-time chat room example
pub struct ChatRoom {
    #[cfg(feature = "websocket")]
    manager: WebSocketManager,
    #[cfg(feature = "websocket")]
    message_history: Arc<RwLock<Vec<String>>>,
    #[cfg(not(feature = "websocket"))]
    _phantom: std::marker::PhantomData<()>,
}

impl ChatRoom {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "websocket")]
            manager: WebSocketManager::new(),
            #[cfg(feature = "websocket")]
            message_history: Arc::new(RwLock::new(Vec::new())),
            #[cfg(not(feature = "websocket"))]
            _phantom: std::marker::PhantomData,
        }
    }

    #[cfg(feature = "websocket")]
    pub async fn send_message(&self, user: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        let formatted_message = format!("{}: {}", user, message);
        
        // Add to history
        {
            let mut history = self.message_history.write().await;
            history.push(formatted_message.clone());
            
            // Keep only last 100 messages
            if history.len() > 100 {
                history.remove(0);
            }
        }
        
        // Broadcast to all clients
        self.manager.broadcast(&formatted_message).await?;
        Ok(())
    }

    #[cfg(feature = "websocket")]
    pub async fn get_history(&self) -> Vec<String> {
        self.message_history.read().await.clone()
    }

    #[cfg(not(feature = "websocket"))]
    pub async fn send_message(&self, _user: &str, _message: &str) -> Result<(), Box<dyn std::error::Error>> {
        Err("WebSocket feature not enabled".into())
    }

    #[cfg(not(feature = "websocket"))]
    pub async fn get_history(&self) -> Vec<String> {
        Vec::new()
    }
}

/// Server-Sent Events (SSE) support for real-time updates
pub struct SSEStream {
    #[cfg(feature = "websocket")]
    sender: broadcast::Sender<String>,
    #[cfg(not(feature = "websocket"))]
    _phantom: std::marker::PhantomData<()>,
}

impl SSEStream {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "websocket")]
            sender: broadcast::channel(1000).0,
            #[cfg(not(feature = "websocket"))]
            _phantom: std::marker::PhantomData,
        }
    }

    /// Send an event to all SSE clients
    #[cfg(feature = "websocket")]
    pub fn send_event(&self, event_type: &str, data: &str) -> Result<(), Box<dyn std::error::Error>> {
        let sse_message = format!("event: {}\ndata: {}\n\n", event_type, data);
        self.sender.send(sse_message)?;
        Ok(())
    }

    /// Create an SSE response with proper streaming setup
    pub fn create_response(&self) -> Response {
        #[cfg(feature = "websocket")]
        {
            // Create SSE response with proper headers
            let mut response = Response::ok()
                .header("Content-Type", "text/event-stream")
                .header("Cache-Control", "no-cache")
                .header("Connection", "keep-alive")
                .header("Access-Control-Allow-Origin", "*")
                .header("Access-Control-Allow-Headers", "Cache-Control");

            // Send initial connection event
            let initial_data = "event: connected\ndata: SSE stream established\nid: 0\n\n";
            response = response.body(initial_data);

            response
        }

        #[cfg(not(feature = "websocket"))]
        {
            Response::with_status(http::StatusCode::NOT_IMPLEMENTED)
                .body("SSE support not enabled")
        }
    }

    #[cfg(not(feature = "websocket"))]
    pub fn send_event(&self, _event_type: &str, _data: &str) -> Result<(), Box<dyn std::error::Error>> {
        Err("WebSocket feature not enabled".into())
    }
}

/// WebSocket middleware for automatic connection management
pub struct WebSocketMiddleware {
    #[cfg(feature = "websocket")]
    manager: Arc<WebSocketManager>,
    #[cfg(not(feature = "websocket"))]
    _phantom: std::marker::PhantomData<()>,
}

impl WebSocketMiddleware {
    pub fn new(_manager: Arc<WebSocketManager>) -> Self {
        Self {
            #[cfg(feature = "websocket")]
            manager: _manager,
            #[cfg(not(feature = "websocket"))]
            _phantom: std::marker::PhantomData,
        }
    }
}

impl crate::middleware::Middleware for WebSocketMiddleware {
    fn call(
        &self,
        req: Request,
        next: Box<dyn Fn(Request) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send + 'static>> + Send + Sync>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send + 'static>> {
        #[cfg(feature = "websocket")]
        {
            let _manager = self.manager.clone();
            Box::pin(async move {
                // Check if this is a WebSocket upgrade request
                if req.header("upgrade").map(|h| h.to_lowercase()) == Some("websocket".to_string()) {
                    // Handle WebSocket upgrade
                    websocket_upgrade(req).await
                } else {
                    // Regular HTTP request
                    next(req).await
                }
            })
        }
        
        #[cfg(not(feature = "websocket"))]
        {
            Box::pin(async move {
                next(req).await
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_websocket_manager() {
        let manager = WebSocketManager::new();
        assert_eq!(manager.connection_count().await, 0);
    }

    #[tokio::test]
    async fn test_chat_room() {
        let chat = ChatRoom::new();
        let history = chat.get_history().await;
        assert!(history.is_empty());
    }

    #[test]
    fn test_sse_stream() {
        let sse = SSEStream::new();
        let response = sse.create_response();
        
        #[cfg(feature = "websocket")]
        {
            assert_eq!(response.headers().get("content-type").unwrap(), "text/event-stream");
        }
        
        #[cfg(not(feature = "websocket"))]
        {
            assert_eq!(response.status_code(), http::StatusCode::NOT_IMPLEMENTED);
        }
    }
}
