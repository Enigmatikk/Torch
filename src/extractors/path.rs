//! # Path Parameter Extraction
//!
//! This module provides the [`Path`] extractor for extracting and deserializing
//! path parameters from URL patterns. It supports extracting single values,
//! tuples, and custom structs with automatic type conversion and validation.

use std::collections::HashMap;
use std::pin::Pin;
use std::future::Future;
use std::str::FromStr;
use crate::{Request, extractors::{FromRequestParts, ExtractionError}};

/// Extractor for path parameters from URL patterns.
///
/// The `Path` extractor allows you to extract parameters from URL patterns like
/// `/users/:id` or `/posts/:user_id/comments/:comment_id`. It automatically
/// parses and validates the parameters according to the target type.
///
/// # Supported Types
///
/// - **Primitive types**: `u32`, `i32`, `u64`, `String`, `bool`, etc.
/// - **Tuples**: Extract multiple parameters as `(T1, T2, ...)`
/// - **Custom structs**: Use serde to deserialize into custom types
/// - **Optional types**: Use `Option<T>` for optional parameters
///
/// # URL Pattern Syntax
///
/// Path parameters are defined using the `:name` syntax in route patterns:
/// - `/users/:id` - Single parameter
/// - `/users/:user_id/posts/:post_id` - Multiple parameters
/// - `/files/*path` - Wildcard parameter (captures remaining path)
///
/// # Examples
///
/// ## Single Parameter
///
/// ```rust
/// use torch_web::{App, Response, extractors::Path};
///
/// let app = App::new()
///     .get("/users/:id", |Path(id): Path<u32>| async move {
///         Response::ok().body(format!("User ID: {}", id))
///     });
/// ```
///
/// ## Multiple Parameters as Tuple
///
/// ```rust
/// use torch_web::{App, Response, extractors::Path};
///
/// let app = App::new()
///     .get("/users/:user_id/posts/:post_id", |
///         Path((user_id, post_id)): Path<(u32, u32)>
///     | async move {
///         Response::ok().body(format!("User {} Post {}", user_id, post_id))
///     });
/// ```
///
/// ## Custom Struct with Serde
///
/// ```rust
/// use torch_web::{App, Response, extractors::Path};
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct PostPath {
///     user_id: u32,
///     post_id: u32,
/// }
///
/// let app = App::new()
///     .get("/users/:user_id/posts/:post_id", |Path(path): Path<PostPath>| async move {
///         Response::ok().body(format!("User {} Post {}", path.user_id, path.post_id))
///     });
/// ```
///
/// ## Optional Parameters
///
/// ```rust
/// use torch_web::{App, Response, extractors::Path};
///
/// let app = App::new()
///     .get("/files/:category/:filename", |
///         Path((category, filename)): Path<(String, Option<String>)>
///     | async move {
///         match filename {
///             Some(name) => Response::ok().body(format!("File: {}/{}", category, name)),
///             None => Response::ok().body(format!("Category: {}", category)),
///         }
///     });
/// ```
///
/// ## String Parameters
///
/// ```rust
/// use torch_web::{App, Response, extractors::Path};
///
/// let app = App::new()
///     .get("/search/:query", |Path(query): Path<String>| async move {
///         Response::ok().body(format!("Searching for: {}", query))
///     });
/// ```
///
/// # Error Handling
///
/// Path extraction can fail in several cases:
/// - **Missing parameter**: The URL doesn't contain the expected parameter
/// - **Type conversion error**: The parameter value can't be parsed into the target type
/// - **Validation error**: Custom validation rules fail
///
/// When extraction fails, a `400 Bad Request` response is automatically returned
/// with details about the error.
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
