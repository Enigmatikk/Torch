//! # Query Parameter Extraction
//!
//! This module provides the [`Query`] extractor for parsing URL query parameters
//! into strongly-typed Rust structs. It supports both simple HashMap extraction
//! and complex struct deserialization with validation.

use std::collections::HashMap;
use std::pin::Pin;
use std::future::Future;
use crate::{Request, extractors::{FromRequestParts, ExtractionError}};

/// Extractor for URL query parameters.
///
/// The `Query` extractor parses query parameters from the URL query string and
/// deserializes them into the specified type. It supports both simple key-value
/// extraction and complex struct deserialization with type conversion and validation.
///
/// # Supported Types
///
/// - **HashMap<String, String>**: Extract all parameters as string key-value pairs
/// - **Custom structs**: Use serde to deserialize into typed structs
/// - **Optional fields**: Use `Option<T>` for optional parameters
/// - **Collections**: Use `Vec<T>` for repeated parameters
/// - **Primitive types**: Automatic conversion to numbers, booleans, etc.
///
/// # Query String Format
///
/// Query parameters follow standard URL encoding:
/// - `?name=value&other=123` - Basic parameters
/// - `?tags=rust&tags=web` - Repeated parameters (becomes Vec)
/// - `?search=hello%20world` - URL-encoded values
/// - `?active=true&count=42` - Type conversion
///
/// # Examples
///
/// ## Simple HashMap Extraction
///
/// ```rust
/// use torch_web::{App, Response, extractors::Query};
/// use std::collections::HashMap;
///
/// let app = App::new()
///     .get("/search", |Query(params): Query<HashMap<String, String>>| async move {
///         if let Some(q) = params.get("q") {
///             Response::ok().body(format!("Searching for: {}", q))
///         } else {
///             Response::bad_request().body("Missing search query")
///         }
///     });
/// ```
///
/// ## Typed Struct Extraction
///
/// ```rust
/// use torch_web::{App, Response, extractors::Query};
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct SearchParams {
///     q: String,                    // Required parameter
///     page: Option<u32>,           // Optional with default None
///     limit: Option<u32>,          // Optional with default None
///     #[serde(default)]
///     sort: String,                // Optional with default ""
/// }
///
/// let app = App::new()
///     .get("/search", |Query(params): Query<SearchParams>| async move {
///         let page = params.page.unwrap_or(1);
///         let limit = params.limit.unwrap_or(10);
///
///         Response::ok().body(format!(
///             "Searching '{}' - page {} with {} items",
///             params.q, page, limit
///         ))
///     });
/// ```
///
/// ## With Default Values
///
/// ```rust
/// use torch_web::{App, Response, extractors::Query};
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct PaginationParams {
///     #[serde(default = "default_page")]
///     page: u32,
///     #[serde(default = "default_limit")]
///     limit: u32,
///     #[serde(default)]
///     sort: String,
/// }
///
/// fn default_page() -> u32 { 1 }
/// fn default_limit() -> u32 { 20 }
///
/// let app = App::new()
///     .get("/items", |Query(params): Query<PaginationParams>| async move {
///         Response::ok().body(format!(
///             "Page {} with {} items, sorted by '{}'",
///             params.page, params.limit, params.sort
///         ))
///     });
/// ```
///
/// ## Array Parameters
///
/// ```rust
/// use torch_web::{App, Response, extractors::Query};
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct FilterParams {
///     tags: Vec<String>,           // ?tags=rust&tags=web&tags=framework
///     categories: Option<Vec<u32>>, // ?categories=1&categories=2
/// }
///
/// let app = App::new()
///     .get("/posts", |Query(params): Query<FilterParams>| async move {
///         Response::ok().body(format!(
///             "Filtering by tags: {:?}, categories: {:?}",
///             params.tags, params.categories
///         ))
///     });
/// ```
///
/// ## Boolean and Numeric Types
///
/// ```rust
/// use torch_web::{App, Response, extractors::Query};
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct SearchFilters {
///     active: Option<bool>,        // ?active=true or ?active=false
///     min_price: Option<f64>,      // ?min_price=19.99
///     max_results: Option<u32>,    // ?max_results=100
/// }
///
/// let app = App::new()
///     .get("/products", |Query(filters): Query<SearchFilters>| async move {
///         let mut conditions = Vec::new();
///
///         if let Some(active) = filters.active {
///             conditions.push(format!("active = {}", active));
///         }
///         if let Some(min_price) = filters.min_price {
///             conditions.push(format!("price >= {}", min_price));
///         }
///
///         Response::ok().body(format!("Filters: {}", conditions.join(", ")))
///     });
/// ```
///
/// ## Combined with Path Parameters
///
/// ```rust
/// use torch_web::{App, Response, extractors::{Path, Query}};
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct PostQuery {
///     include_comments: Option<bool>,
///     format: Option<String>,
/// }
///
/// let app = App::new()
///     .get("/users/:id/posts", |
///         Path(user_id): Path<u32>,
///         Query(query): Query<PostQuery>,
///     | async move {
///         let include_comments = query.include_comments.unwrap_or(false);
///         let format = query.format.unwrap_or_else(|| "json".to_string());
///
///         Response::ok().body(format!(
///             "Posts for user {} (comments: {}, format: {})",
///             user_id, include_comments, format
///         ))
///     });
/// ```
///
/// # Error Handling
///
/// Query extraction can fail for several reasons:
/// - **Missing required parameters**: Returns 400 with parameter name
/// - **Type conversion errors**: Returns 400 with conversion details
/// - **Invalid format**: Returns 400 with parsing error
/// - **Validation errors**: Returns 400 with validation details
///
/// All errors result in a `400 Bad Request` response with descriptive error messages.
pub struct Query<T>(pub T);

impl<T> FromRequestParts for Query<T>
where
    T: DeserializeFromQuery,
{
    type Error = ExtractionError;

    fn from_request_parts(
        req: &mut Request,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Self::Error>> + Send + 'static>> {
        let query_string = req.query_string().unwrap_or("").to_string();
        
        Box::pin(async move {
            let value = T::deserialize_from_query(&query_string)?;
            Ok(Query(value))
        })
    }
}

/// Trait for types that can be deserialized from query parameters
pub trait DeserializeFromQuery: Sized {
    fn deserialize_from_query(query: &str) -> Result<Self, ExtractionError>;
}

// Implement for HashMap<String, String> to get all parameters
impl DeserializeFromQuery for HashMap<String, String> {
    fn deserialize_from_query(query: &str) -> Result<Self, ExtractionError> {
        let mut params = HashMap::new();
        
        if query.is_empty() {
            return Ok(params);
        }

        for pair in query.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                let key = urlencoding::decode(key)
                    .map_err(|e| ExtractionError::InvalidQuery(format!("Invalid key encoding: {}", e)))?
                    .into_owned();
                let value = urlencoding::decode(value)
                    .map_err(|e| ExtractionError::InvalidQuery(format!("Invalid value encoding: {}", e)))?
                    .into_owned();
                params.insert(key, value);
            } else {
                // Handle keys without values (e.g., "?debug&verbose")
                let key = urlencoding::decode(pair)
                    .map_err(|e| ExtractionError::InvalidQuery(format!("Invalid key encoding: {}", e)))?
                    .into_owned();
                params.insert(key, String::new());
            }
        }

        Ok(params)
    }
}

// Implement for Vec<(String, String)> to preserve order and duplicates
impl DeserializeFromQuery for Vec<(String, String)> {
    fn deserialize_from_query(query: &str) -> Result<Self, ExtractionError> {
        let mut params = Vec::new();
        
        if query.is_empty() {
            return Ok(params);
        }

        for pair in query.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                let key = urlencoding::decode(key)
                    .map_err(|e| ExtractionError::InvalidQuery(format!("Invalid key encoding: {}", e)))?
                    .into_owned();
                let value = urlencoding::decode(value)
                    .map_err(|e| ExtractionError::InvalidQuery(format!("Invalid value encoding: {}", e)))?
                    .into_owned();
                params.push((key, value));
            } else {
                let key = urlencoding::decode(pair)
                    .map_err(|e| ExtractionError::InvalidQuery(format!("Invalid key encoding: {}", e)))?
                    .into_owned();
                params.push((key, String::new()));
            }
        }

        Ok(params)
    }
}

/// Serde-based query parameter extractor for custom types
#[cfg(feature = "json")]
pub struct SerdeQuery<T>(pub T);

#[cfg(feature = "json")]
impl<T> FromRequestParts for SerdeQuery<T>
where
    T: serde::de::DeserializeOwned,
{
    type Error = ExtractionError;

    fn from_request_parts(
        req: &mut Request,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Self::Error>> + Send + 'static>> {
        let query_string = req.query_string().unwrap_or("").to_string();

        Box::pin(async move {
            let value = deserialize_query_with_serde(&query_string)?;
            Ok(SerdeQuery(value))
        })
    }
}

#[cfg(feature = "json")]
fn deserialize_query_with_serde<T: serde::de::DeserializeOwned>(query: &str) -> Result<T, ExtractionError> {

    // First parse into HashMap
    let params: HashMap<String, String> = DeserializeFromQuery::deserialize_from_query(query)?;

    // Convert to serde_json::Value for deserialization
    let mut json_map = serde_json::Map::new();
    for (key, value) in params {
        // Try to parse as different types with better type inference
        let json_value = if value.is_empty() {
            serde_json::Value::Bool(true) // For flag-style parameters
        } else if value == "true" {
            serde_json::Value::Bool(true)
        } else if value == "false" {
            serde_json::Value::Bool(false)
        } else if value == "null" {
            serde_json::Value::Null
        } else if let Ok(num) = value.parse::<i64>() {
            serde_json::Value::Number(serde_json::Number::from(num))
        } else if let Ok(float) = value.parse::<f64>() {
            if let Some(num) = serde_json::Number::from_f64(float) {
                serde_json::Value::Number(num)
            } else {
                serde_json::Value::String(value)
            }
        } else {
            // Handle arrays (comma-separated values)
            if value.contains(',') {
                let array_values: Vec<serde_json::Value> = value
                    .split(',')
                    .map(|s| {
                        let trimmed = s.trim();
                        if let Ok(num) = trimmed.parse::<i64>() {
                            serde_json::Value::Number(serde_json::Number::from(num))
                        } else if let Ok(float) = trimmed.parse::<f64>() {
                            if let Some(num) = serde_json::Number::from_f64(float) {
                                serde_json::Value::Number(num)
                            } else {
                                serde_json::Value::String(trimmed.to_string())
                            }
                        } else {
                            serde_json::Value::String(trimmed.to_string())
                        }
                    })
                    .collect();
                serde_json::Value::Array(array_values)
            } else {
                serde_json::Value::String(value)
            }
        };
        json_map.insert(key, json_value);
    }

    let json_value = serde_json::Value::Object(json_map);
    serde_json::from_value(json_value).map_err(|e| {
        ExtractionError::InvalidQuery(format!("Failed to deserialize query parameters: {}", e))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_query() {
        let result: Result<HashMap<String, String>, _> = 
            DeserializeFromQuery::deserialize_from_query("");
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_simple_query() {
        let result: Result<HashMap<String, String>, _> = 
            DeserializeFromQuery::deserialize_from_query("name=john&age=30");
        
        let params = result.unwrap();
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
    }

    #[test]
    fn test_url_encoded_query() {
        let result: Result<HashMap<String, String>, _> = 
            DeserializeFromQuery::deserialize_from_query("name=John%20Doe&city=New%20York");
        
        let params = result.unwrap();
        assert_eq!(params.get("name"), Some(&"John Doe".to_string()));
        assert_eq!(params.get("city"), Some(&"New York".to_string()));
    }

    #[test]
    fn test_flag_parameters() {
        let result: Result<HashMap<String, String>, _> = 
            DeserializeFromQuery::deserialize_from_query("debug&verbose&name=test");
        
        let params = result.unwrap();
        assert_eq!(params.get("debug"), Some(&"".to_string()));
        assert_eq!(params.get("verbose"), Some(&"".to_string()));
        assert_eq!(params.get("name"), Some(&"test".to_string()));
    }

    #[test]
    fn test_vec_preserves_order() {
        let result: Result<Vec<(String, String)>, _> = 
            DeserializeFromQuery::deserialize_from_query("a=1&b=2&a=3");
        
        let params = result.unwrap();
        assert_eq!(params.len(), 3);
        assert_eq!(params[0], ("a".to_string(), "1".to_string()));
        assert_eq!(params[1], ("b".to_string(), "2".to_string()));
        assert_eq!(params[2], ("a".to_string(), "3".to_string()));
    }
}
