//! WebSocket support for real-time applications

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
