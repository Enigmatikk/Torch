//! JSON body extraction
//!
//! Extract and deserialize JSON request bodies.

use std::pin::Pin;
use std::future::Future;
use crate::{Request, extractors::{FromRequest, ExtractionError}};
use serde::de::DeserializeOwned;

/// Extract and deserialize a JSON request body
///
/// # Example
///
/// ```rust,no_run
/// use torch_web::extractors::Json;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct CreateUser {
///     name: String,
///     email: String,
///     age: Option<u32>,
/// }
///
/// async fn create_user(Json(user): Json<CreateUser>) {
///     // user is automatically deserialized from the JSON body
///     println!("Creating user: {}", user.name);
/// }
/// ```
pub struct Json<T>(pub T);

impl<T> FromRequest for Json<T>
where
    T: DeserializeOwned,
{
    type Error = ExtractionError;

    fn from_request(
        mut req: Request,
    ) -> Pin<Box<dyn Future<Output = Result<(Self, Request), Self::Error>> + Send + 'static>> {
        Box::pin(async move {
            // Check content type
            let content_type = req.headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");

            if !content_type.starts_with("application/json") {
                return Err(ExtractionError::InvalidJson(
                    format!("Expected application/json content type, got: {}", content_type)
                ));
            }

            // Get the request body
            let body_bytes = req.body_bytes();
            
            if body_bytes.is_empty() {
                return Err(ExtractionError::InvalidJson(
                    "Request body is empty".to_string()
                ));
            }

            // Deserialize the JSON
            let value: T = serde_json::from_slice(body_bytes)
                .map_err(|e| ExtractionError::InvalidJson(format!("Failed to parse JSON: {}", e)))?;

            Ok((Json(value), req))
        })
    }
}

/// Extract raw JSON value without type checking
///
/// # Example
///
/// ```rust,no_run
/// use torch_web::extractors::RawJson;
/// use serde_json::Value;
///
/// async fn handle_any_json(RawJson(value): RawJson) {
///     // value is a serde_json::Value that can be any JSON
///     match value {
///         Value::Object(obj) => {
///             // Handle JSON object
///         }
///         Value::Array(arr) => {
///             // Handle JSON array
///         }
///         _ => {
///             // Handle other JSON types
///         }
///     }
/// }
/// ```
pub struct RawJson(pub serde_json::Value);

impl FromRequest for RawJson {
    type Error = ExtractionError;

    fn from_request(
        mut req: Request,
    ) -> Pin<Box<dyn Future<Output = Result<(Self, Request), Self::Error>> + Send + 'static>> {
        Box::pin(async move {
            // Check content type
            let content_type = req.headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");

            if !content_type.starts_with("application/json") {
                return Err(ExtractionError::InvalidJson(
                    format!("Expected application/json content type, got: {}", content_type)
                ));
            }

            // Get the request body
            let body_bytes = req.body_bytes();
            
            if body_bytes.is_empty() {
                return Err(ExtractionError::InvalidJson(
                    "Request body is empty".to_string()
                ));
            }

            // Parse as raw JSON value
            let value: serde_json::Value = serde_json::from_slice(body_bytes)
                .map_err(|e| ExtractionError::InvalidJson(format!("Failed to parse JSON: {}", e)))?;

            Ok((RawJson(value), req))
        })
    }
}

/// Extract JSON with size limits
///
/// # Example
///
/// ```rust,no_run
/// use torch_web::extractors::JsonWithLimit;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct SmallPayload {
///     message: String,
/// }
///
/// async fn handle_small_json(JsonWithLimit(payload): JsonWithLimit<SmallPayload, 1024>) {
///     // payload is limited to 1KB
/// }
/// ```
pub struct JsonWithLimit<T, const LIMIT: usize>(pub T);

impl<T, const LIMIT: usize> FromRequest for JsonWithLimit<T, LIMIT>
where
    T: DeserializeOwned,
{
    type Error = ExtractionError;

    fn from_request(
        mut req: Request,
    ) -> Pin<Box<dyn Future<Output = Result<(Self, Request), Self::Error>> + Send + 'static>> {
        Box::pin(async move {
            // Check content type
            let content_type = req.headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");

            if !content_type.starts_with("application/json") {
                return Err(ExtractionError::InvalidJson(
                    format!("Expected application/json content type, got: {}", content_type)
                ));
            }

            // Get the request body
            let body_bytes = req.body_bytes();
            
            if body_bytes.is_empty() {
                return Err(ExtractionError::InvalidJson(
                    "Request body is empty".to_string()
                ));
            }

            // Check size limit
            if body_bytes.len() > LIMIT {
                return Err(ExtractionError::InvalidJson(
                    format!("Request body too large: {} bytes (limit: {} bytes)", 
                           body_bytes.len(), LIMIT)
                ));
            }

            // Deserialize the JSON
            let value: T = serde_json::from_slice(body_bytes)
                .map_err(|e| ExtractionError::InvalidJson(format!("Failed to parse JSON: {}", e)))?;

            Ok((JsonWithLimit(value), req))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    struct TestUser {
        name: String,
        age: u32,
    }

    #[tokio::test]
    async fn test_json_extraction() {
        let user = TestUser {
            name: "John".to_string(),
            age: 30,
        };
        let json_body = serde_json::to_vec(&user).unwrap();

        let mut req = Request::new();
        req.headers_mut().insert("content-type", "application/json".parse().unwrap());
        req.set_body(json_body);

        let result = Json::<TestUser>::from_request(req).await;
        assert!(result.is_ok());

        let (Json(extracted_user), _) = result.unwrap();
        assert_eq!(extracted_user, user);
    }

    #[tokio::test]
    async fn test_json_wrong_content_type() {
        let mut req = Request::new();
        req.headers_mut().insert("content-type", "text/plain".parse().unwrap());
        req.set_body(b"not json".to_vec());

        let result = Json::<TestUser>::from_request(req).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_json_empty_body() {
        let mut req = Request::new();
        req.headers_mut().insert("content-type", "application/json".parse().unwrap());

        let result = Json::<TestUser>::from_request(req).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_json_invalid_json() {
        let mut req = Request::new();
        req.headers_mut().insert("content-type", "application/json".parse().unwrap());
        req.set_body(b"invalid json".to_vec());

        let result = Json::<TestUser>::from_request(req).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_raw_json_extraction() {
        let json_str = r#"{"name": "John", "age": 30}"#;
        let mut req = Request::new();
        req.headers_mut().insert("content-type", "application/json".parse().unwrap());
        req.set_body(json_str.as_bytes().to_vec());

        let result = RawJson::from_request(req).await;
        assert!(result.is_ok());

        let (RawJson(value), _) = result.unwrap();
        assert_eq!(value["name"], "John");
        assert_eq!(value["age"], 30);
    }

    #[tokio::test]
    async fn test_json_with_limit() {
        let user = TestUser {
            name: "John".to_string(),
            age: 30,
        };
        let json_body = serde_json::to_vec(&user).unwrap();

        let mut req = Request::new();
        req.headers_mut().insert("content-type", "application/json".parse().unwrap());
        req.set_body(json_body);

        // Should succeed with generous limit
        let result = JsonWithLimit::<TestUser, 1024>::from_request(req).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_json_with_limit_exceeded() {
        let large_string = "x".repeat(2000);
        let user = TestUser {
            name: large_string,
            age: 30,
        };
        let json_body = serde_json::to_vec(&user).unwrap();

        let mut req = Request::new();
        req.headers_mut().insert("content-type", "application/json".parse().unwrap());
        req.set_body(json_body);

        // Should fail with small limit
        let result = JsonWithLimit::<TestUser, 100>::from_request(req).await;
        assert!(result.is_err());
    }
}
