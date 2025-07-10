//! Query parameter extraction
//!
//! Extract and deserialize query parameters from the URL.

use std::collections::HashMap;
use std::pin::Pin;
use std::future::Future;
use crate::{Request, extractors::{FromRequestParts, ExtractionError}};

/// Extract query parameters from the request URL
///
/// # Example
///
/// ```rust,no_run
/// use torch_web::extractors::Query;
/// use std::collections::HashMap;
/// use serde::Deserialize;
///
/// // Extract as HashMap
/// async fn search(Query(params): Query<HashMap<String, String>>) {
///     // params contains all query parameters
/// }
///
/// // Extract into a custom struct
/// #[derive(Deserialize)]
/// struct SearchParams {
///     q: String,
///     page: Option<u32>,
///     limit: Option<u32>,
/// }
///
/// async fn search_typed(Query(params): Query<SearchParams>) {
///     // Automatically deserializes and validates query parameters
/// }
/// ```
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
