use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request as HyperRequest, Response as HyperResponse};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use crate::{App, Request};

/// Start the HTTP server
pub async fn serve(
    addr: SocketAddr,
    app: App,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = Arc::new(app);
    let listener = TcpListener::bind(addr).await?;

    println!("🔥 Torch server listening on http://{}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let app = app.clone();

        tokio::task::spawn(async move {
            let service = service_fn(move |req| {
                let app = app.clone();
                async move { handle_request(req, app).await }
            });

            if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

/// Handle a single HTTP request
async fn handle_request(
    hyper_req: HyperRequest<hyper::body::Incoming>,
    app: Arc<App>,
) -> Result<HyperResponse<http_body_util::Full<hyper::body::Bytes>>, Infallible> {
    let (parts, body) = hyper_req.into_parts();

    // Convert hyper request to our Request type
    let request = match Request::from_hyper(parts, body).await {
        Ok(req) => req,
        Err(err) => {
            eprintln!("Error parsing request: {:?}", err);
            return Ok(create_error_response(500, "Internal Server Error"));
        }
    };

    // Handle the request with our app
    let response = app.handle_request(request).await;

    // Convert our Response back to hyper Response
    Ok(response.into_hyper_response())
}

/// Create an error response
fn create_error_response(status: u16, message: &str) -> HyperResponse<http_body_util::Full<hyper::body::Bytes>> {
    use http_body_util::Full;
    use hyper::body::Bytes;

    HyperResponse::builder()
        .status(status)
        .header("content-type", "text/plain")
        .body(Full::new(Bytes::from(message.to_string())))
        .unwrap()
}

/// Configuration for the server
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Maximum number of concurrent connections
    pub max_connections: Option<usize>,
    /// Request timeout in seconds
    pub request_timeout: Option<u64>,
    /// Keep-alive timeout in seconds
    pub keep_alive_timeout: Option<u64>,
    /// Maximum request body size in bytes
    pub max_body_size: Option<usize>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            max_connections: None,
            request_timeout: Some(30),
            keep_alive_timeout: Some(60),
            max_body_size: Some(1024 * 1024), // 1MB
        }
    }
}

/// A more advanced server builder with configuration options
pub struct Server {
    app: App,
    config: ServerConfig,
}

impl Server {
    /// Create a new server with the given app
    pub fn new(app: App) -> Self {
        Self {
            app,
            config: ServerConfig::default(),
        }
    }

    /// Set the server configuration
    pub fn config(mut self, config: ServerConfig) -> Self {
        self.config = config;
        self
    }

    /// Set maximum number of concurrent connections
    pub fn max_connections(mut self, max: usize) -> Self {
        self.config.max_connections = Some(max);
        self
    }

    /// Set request timeout
    pub fn request_timeout(mut self, timeout_secs: u64) -> Self {
        self.config.request_timeout = Some(timeout_secs);
        self
    }

    /// Set keep-alive timeout
    pub fn keep_alive_timeout(mut self, timeout_secs: u64) -> Self {
        self.config.keep_alive_timeout = Some(timeout_secs);
        self
    }

    /// Set maximum request body size
    pub fn max_body_size(mut self, size: usize) -> Self {
        self.config.max_body_size = Some(size);
        self
    }

    /// Start the server
    pub async fn listen(
        self,
        addr: SocketAddr,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        serve(addr, self.app).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{App, Response};

    #[test]
    fn test_server_config() {
        let config = ServerConfig::default();
        assert_eq!(config.request_timeout, Some(30));
        assert_eq!(config.keep_alive_timeout, Some(60));
        assert_eq!(config.max_body_size, Some(1024 * 1024));
    }

    #[test]
    fn test_server_builder() {
        let app = App::new().get::<_, (crate::Request,)>("/", |_req: crate::Request| async { Response::ok() });
        
        let _server = Server::new(app)
            .max_connections(1000)
            .request_timeout(60)
            .keep_alive_timeout(120)
            .max_body_size(2 * 1024 * 1024);
    }
}
