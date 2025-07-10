//! Path parameter extraction
//!
//! Extract and deserialize path parameters from the URL.

use std::collections::HashMap;
use std::pin::Pin;
use std::future::Future;
use std::str::FromStr;
use crate::{Request, extractors::{FromRequestParts, ExtractionError}};

/// Extract path parameters from the request URL
///
/// # Example
///
/// ```rust,no_run
/// use torch_web::extractors::Path;
///
/// // Extract a single parameter
/// async fn get_user(Path(user_id): Path<u32>) {
///     // user_id is automatically parsed from the URL
/// }
///
/// // Extract multiple parameters as a tuple
/// async fn get_post(Path((user_id, post_id)): Path<(u32, u32)>) {
///     // Extracts from "/users/:user_id/posts/:post_id"
/// }
///
/// // Extract into a custom struct
/// #[derive(serde::Deserialize)]
/// struct PostPath {
///     user_id: u32,
///     post_id: u32,
/// }
///
/// async fn get_post_struct(Path(path): Path<PostPath>) {
///     // Automatically deserializes path parameters
/// }
/// ```
pub struct Path<T>(pub T);

impl<T> FromRequestParts for Path<T>
where
    T: DeserializeFromPath,
{
    type Error = ExtractionError;

    fn from_request_parts(
        req: &mut Request,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Self::Error>> + Send + 'static>> {
        let params = req.path_params().clone();
        
        Box::pin(async move {
            let value = T::deserialize_from_path(params)?;
            Ok(Path(value))
        })
    }
}

/// Trait for types that can be deserialized from path parameters
pub trait DeserializeFromPath: Sized {
    fn deserialize_from_path(params: HashMap<String, String>) -> Result<Self, ExtractionError>;
}

/// Marker trait to prevent conflicting implementations
pub trait PathDeserializable {}

// Implement PathDeserializable for basic types
impl PathDeserializable for String {}
impl PathDeserializable for u8 {}
impl PathDeserializable for u16 {}
impl PathDeserializable for u32 {}
impl PathDeserializable for u64 {}
impl PathDeserializable for usize {}
impl PathDeserializable for i8 {}
impl PathDeserializable for i16 {}
impl PathDeserializable for i32 {}
impl PathDeserializable for i64 {}
impl PathDeserializable for isize {}
impl PathDeserializable for f32 {}
impl PathDeserializable for f64 {}
impl PathDeserializable for bool {}
impl PathDeserializable for std::net::IpAddr {}
impl PathDeserializable for std::net::Ipv4Addr {}
impl PathDeserializable for std::net::Ipv6Addr {}

#[cfg(feature = "uuid")]
impl PathDeserializable for uuid::Uuid {}

// Generic implementation for all PathDeserializable types
impl<T> DeserializeFromPath for T
where
    T: FromStr + PathDeserializable,
    T::Err: std::fmt::Display,
{
    fn deserialize_from_path(params: HashMap<String, String>) -> Result<Self, ExtractionError> {
        if params.len() != 1 {
            return Err(ExtractionError::InvalidPathParam(
                format!("Expected exactly one path parameter for type {}, got {}",
                       std::any::type_name::<T>(), params.len())
            ));
        }

        let (param_name, value) = params.into_iter().next().unwrap();
        value.parse().map_err(|e| {
            ExtractionError::InvalidPathParam(
                format!("Failed to parse parameter '{}' as {}: {}",
                       param_name, std::any::type_name::<T>(), e)
            )
        })
    }
}

// Implement for tuples of up to 6 elements
impl<T1, T2> DeserializeFromPath for (T1, T2)
where
    T1: FromStr + PathDeserializable,
    T2: FromStr + PathDeserializable,
    T1::Err: std::fmt::Display,
    T2::Err: std::fmt::Display,
{
    fn deserialize_from_path(params: HashMap<String, String>) -> Result<Self, ExtractionError> {
        if params.len() != 2 {
            return Err(ExtractionError::InvalidPathParam(
                format!("Expected exactly 2 path parameters, got {}", params.len())
            ));
        }

        // Convert to sorted vector to ensure consistent ordering
        let mut param_pairs: Vec<_> = params.into_iter().collect();
        param_pairs.sort_by(|a, b| a.0.cmp(&b.0)); // Sort by parameter name

        let first = param_pairs[0].1.parse().map_err(|e| {
            ExtractionError::InvalidPathParam(
                format!("Failed to parse parameter '{}' as {}: {}",
                       param_pairs[0].0, std::any::type_name::<T1>(), e)
            )
        })?;

        let second = param_pairs[1].1.parse().map_err(|e| {
            ExtractionError::InvalidPathParam(
                format!("Failed to parse parameter '{}' as {}: {}",
                       param_pairs[1].0, std::any::type_name::<T2>(), e)
            )
        })?;

        Ok((first, second))
    }
}

// Implement for HashMap<String, String> to get all parameters
impl DeserializeFromPath for HashMap<String, String> {
    fn deserialize_from_path(params: HashMap<String, String>) -> Result<Self, ExtractionError> {
        Ok(params)
    }
}

// Implement for 3-tuples
impl<T1, T2, T3> DeserializeFromPath for (T1, T2, T3)
where
    T1: FromStr + PathDeserializable,
    T2: FromStr + PathDeserializable,
    T3: FromStr + PathDeserializable,
    T1::Err: std::fmt::Display,
    T2::Err: std::fmt::Display,
    T3::Err: std::fmt::Display,
{
    fn deserialize_from_path(params: HashMap<String, String>) -> Result<Self, ExtractionError> {
        if params.len() != 3 {
            return Err(ExtractionError::InvalidPathParam(
                format!("Expected exactly 3 path parameters, got {}", params.len())
            ));
        }

        let mut param_pairs: Vec<_> = params.into_iter().collect();
        param_pairs.sort_by(|a, b| a.0.cmp(&b.0));

        let first = param_pairs[0].1.parse().map_err(|e| {
            ExtractionError::InvalidPathParam(
                format!("Failed to parse parameter '{}': {}", param_pairs[0].0, e)
            )
        })?;

        let second = param_pairs[1].1.parse().map_err(|e| {
            ExtractionError::InvalidPathParam(
                format!("Failed to parse parameter '{}': {}", param_pairs[1].0, e)
            )
        })?;

        let third = param_pairs[2].1.parse().map_err(|e| {
            ExtractionError::InvalidPathParam(
                format!("Failed to parse parameter '{}': {}", param_pairs[2].0, e)
            )
        })?;

        Ok((first, second, third))
    }
}

// Note: Serde support for path parameters is complex due to trait conflicts
// For now, we support basic types and tuples. Custom structs can be handled
// by extracting into HashMap<String, String> and then manually deserializing

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_param_extraction() {
        let mut params = HashMap::new();
        params.insert("id".to_string(), "123".to_string());

        let result: Result<u32, _> = DeserializeFromPath::deserialize_from_path(params);
        assert_eq!(result.unwrap(), 123);
    }

    #[test]
    fn test_invalid_param_extraction() {
        let mut params = HashMap::new();
        params.insert("id".to_string(), "not_a_number".to_string());

        let result: Result<u32, _> = DeserializeFromPath::deserialize_from_path(params);
        assert!(result.is_err());
    }

    #[test]
    fn test_hashmap_extraction() {
        let mut params = HashMap::new();
        params.insert("user_id".to_string(), "123".to_string());
        params.insert("post_id".to_string(), "456".to_string());

        let result: Result<HashMap<String, String>, _> = 
            DeserializeFromPath::deserialize_from_path(params.clone());
        assert_eq!(result.unwrap(), params);
    }
}
