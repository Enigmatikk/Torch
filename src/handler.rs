use std::future::Future;
use std::pin::Pin;
use crate::{Request, Response};
use crate::extractors::{FromRequestParts, IntoResponse};

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

// Handler implementations for extractors

/// Handler for async functions with no extractors (just returns a response)
impl<F, Fut, Res> Handler<((),)> for F
where
    F: Fn() -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Res> + Send + 'static,
    Res: IntoResponse,
{
    type Future = Pin<Box<dyn Future<Output = Response> + Send + 'static>>;

    fn call(&self, _req: Request) -> Self::Future {
        let fut = self();
        Box::pin(async move {
            let res = fut.await;
            res.into_response()
        })
    }
}

/// Handler for functions with one extractor
impl<F, Fut, Res, E1> Handler<(E1,)> for F
where
    F: Fn(E1) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Res> + Send + 'static,
    Res: IntoResponse,
    E1: FromRequestParts + Send + 'static,
{
    type Future = Pin<Box<dyn Future<Output = Response> + Send + 'static>>;

    fn call(&self, mut req: Request) -> Self::Future {
        let handler = self.clone();
        Box::pin(async move {
            match E1::from_request_parts(&mut req).await {
                Ok(e1) => {
                    let res = handler(e1).await;
                    res.into_response()
                }
                Err(err) => err.into_response(),
            }
        })
    }
}

/// Handler for functions with two extractors
impl<F, Fut, Res, E1, E2> Handler<(E1, E2)> for F
where
    F: Fn(E1, E2) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Res> + Send + 'static,
    Res: IntoResponse,
    E1: FromRequestParts + Send + 'static,
    E2: FromRequestParts + Send + 'static,
{
    type Future = Pin<Box<dyn Future<Output = Response> + Send + 'static>>;

    fn call(&self, mut req: Request) -> Self::Future {
        let handler = self.clone();
        Box::pin(async move {
            let e1 = match E1::from_request_parts(&mut req).await {
                Ok(e1) => e1,
                Err(err) => return err.into_response(),
            };

            let e2 = match E2::from_request_parts(&mut req).await {
                Ok(e2) => e2,
                Err(err) => return err.into_response(),
            };

            let res = handler(e1, e2).await;
            res.into_response()
        })
    }
}

/// Handler for functions with three extractors
impl<F, Fut, Res, E1, E2, E3> Handler<(E1, E2, E3)> for F
where
    F: Fn(E1, E2, E3) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Res> + Send + 'static,
    Res: IntoResponse,
    E1: FromRequestParts + Send + 'static,
    E2: FromRequestParts + Send + 'static,
    E3: FromRequestParts + Send + 'static,
{
    type Future = Pin<Box<dyn Future<Output = Response> + Send + 'static>>;

    fn call(&self, mut req: Request) -> Self::Future {
        let handler = self.clone();
        Box::pin(async move {
            let e1 = match E1::from_request_parts(&mut req).await {
                Ok(e1) => e1,
                Err(err) => return err.into_response(),
            };

            let e2 = match E2::from_request_parts(&mut req).await {
                Ok(e2) => e2,
                Err(err) => return err.into_response(),
            };

            let e3 = match E3::from_request_parts(&mut req).await {
                Ok(e3) => e3,
                Err(err) => return err.into_response(),
            };

            let res = handler(e1, e2, e3).await;
            res.into_response()
        })
    }
}

/// Handler for functions with four extractors
impl<F, Fut, Res, E1, E2, E3, E4> Handler<(E1, E2, E3, E4)> for F
where
    F: Fn(E1, E2, E3, E4) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Res> + Send + 'static,
    Res: IntoResponse,
    E1: FromRequestParts + Send + 'static,
    E2: FromRequestParts + Send + 'static,
    E3: FromRequestParts + Send + 'static,
    E4: FromRequestParts + Send + 'static,
{
    type Future = Pin<Box<dyn Future<Output = Response> + Send + 'static>>;

    fn call(&self, mut req: Request) -> Self::Future {
        let handler = self.clone();
        Box::pin(async move {
            let e1 = match E1::from_request_parts(&mut req).await {
                Ok(e1) => e1,
                Err(err) => return err.into_response(),
            };

            let e2 = match E2::from_request_parts(&mut req).await {
                Ok(e2) => e2,
                Err(err) => return err.into_response(),
            };

            let e3 = match E3::from_request_parts(&mut req).await {
                Ok(e3) => e3,
                Err(err) => return err.into_response(),
            };

            let e4 = match E4::from_request_parts(&mut req).await {
                Ok(e4) => e4,
                Err(err) => return err.into_response(),
            };

            let res = handler(e1, e2, e3, e4).await;
            res.into_response()
        })
    }
}

/// Convert a handler into a boxed handler function
pub fn into_handler_fn<H, T>(handler: H) -> HandlerFn
where
    H: Handler<T>,
{
    std::sync::Arc::new(move |req| Box::pin(handler.call(req)))
}

// Handler implementations for FromRequest extractors (like Json, Form)

/// Handler for async functions that take Request directly
impl<F, Fut, Res> Handler<(Request,)> for F
where
    F: Fn(Request) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Res> + Send + 'static,
    Res: IntoResponse,
{
    type Future = Pin<Box<dyn Future<Output = Response> + Send + 'static>>;

    fn call(&self, req: Request) -> Self::Future {
        let handler = self.clone();
        Box::pin(async move {
            let res = handler(req).await;
            res.into_response()
        })
    }
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

#[cfg(test)]
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
