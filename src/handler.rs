use std::future::Future;
use std::pin::Pin;
use crate::{Request, Response};

/// A trait for handling HTTP requests
pub trait Handler<T>: Clone + Send + Sync + 'static {
    /// The future type returned by the handler
    type Future: Future<Output = Response> + Send + 'static;

    /// Handle the request and return a response
    fn call(&self, req: Request) -> Self::Future;
}

/// A boxed handler function type for dynamic dispatch
pub type HandlerFn = std::sync::Arc<
    dyn Fn(Request) -> Pin<Box<dyn Future<Output = Response> + Send + 'static>>
        + Send
        + Sync
        + 'static,
>;

/// Implement Handler for async functions that take Request and return Response
impl<F, Fut> Handler<()> for F
where
    F: Fn(Request) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Response> + Send + 'static,
{
    type Future = Fut;

    fn call(&self, req: Request) -> Self::Future {
        self(req)
    }
}

/// Implement Handler for functions that return Response directly (sync handlers)
impl<F> Handler<((),)> for F
where
    F: Fn(Request) -> Response + Clone + Send + Sync + 'static,
{
    type Future = Pin<Box<dyn Future<Output = Response> + Send + 'static>>;

    fn call(&self, req: Request) -> Self::Future {
        let response = self(req);
        Box::pin(async move { response })
    }
}

/// Convert a handler into a boxed handler function
pub fn into_handler_fn<H, T>(handler: H) -> HandlerFn
where
    H: Handler<T>,
{
    std::sync::Arc::new(move |req| Box::pin(handler.call(req)))
}

/// A convenience macro for creating simple handlers
#[macro_export]
macro_rules! handler {
    ($body:expr) => {
        |_req: $crate::Request| async move { $body }
    };
    ($req:ident => $body:expr) => {
        |$req: $crate::Request| async move { $body }
    };
}

#[cfg(disabled_for_now)]
mod tests {
    use super::*;
    use crate::Response;

    #[tokio::test]
    async fn test_async_handler() {
        let handler = |_req: Request| async {
            Response::ok().body("Hello from async handler")
        };

        let req = Request::from_hyper(
            http::Request::builder()
                .method("GET")
                .uri("/")
                .body(())
                .unwrap()
                .into_parts()
                .0,
            Vec::new(),
        )
        .await
        .unwrap();

        let response = handler.call(req).await;
        assert_eq!(response.body_data(), b"Hello from async handler");
    }

    #[tokio::test]
    async fn test_sync_handler() {
        let handler = |_req: Request| Response::ok().body("Hello from sync handler");

        let req = Request::from_hyper(
            http::Request::builder()
                .method("GET")
                .uri("/")
                .body(())
                .unwrap()
                .into_parts()
                .0,
            Vec::new(),
        )
        .await
        .unwrap();

        let response = handler.call(req).await;
        assert_eq!(response.body_data(), b"Hello from sync handler");
    }

    #[test]
    fn test_handler_macro() {
        let _handler1 = handler!(Response::ok().body("Simple response"));
        let _handler2 = handler!(req => {
            Response::ok().body(format!("Path: {}", req.path()))
        });
    }
}
