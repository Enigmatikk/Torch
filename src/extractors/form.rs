//! Form data extraction
//!
//! Extract and deserialize form data from request bodies.

use std::pin::Pin;
use std::future::Future;
use std::collections::HashMap;
use crate::{Request, extractors::{FromRequest, ExtractionError}};

/// Extract form data from application/x-www-form-urlencoded request bodies
///
/// # Example
///
/// ```rust,no_run
/// use torch_web::extractors::Form;
/// use std::collections::HashMap;
/// use serde::Deserialize;
///
/// // Extract as HashMap
/// async fn handle_form(Form(data): Form<HashMap<String, String>>) {
///     // data contains all form fields
/// }
///
/// // Extract into a custom struct
/// #[derive(Deserialize)]
/// struct LoginForm {
///     username: String,
///     password: String,
///     remember_me: Option<bool>,
/// }
///
/// async fn login(Form(form): Form<LoginForm>) {
///     // Automatically deserializes and validates form data
/// }
/// ```
pub struct Form<T>(pub T);

impl<T> FromRequest for Form<T>
where
    T: DeserializeFromForm,
{
    type Error = ExtractionError;

    fn from_request(
        req: Request,
    ) -> Pin<Box<dyn Future<Output = Result<(Self, Request), Self::Error>> + Send + 'static>> {
        Box::pin(async move {
            // Check content type
            let content_type = req.headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");

            if !content_type.starts_with("application/x-www-form-urlencoded") {
                return Err(ExtractionError::InvalidQuery(
                    format!("Expected application/x-www-form-urlencoded content type, got: {}", content_type)
                ));
            }

            // Get the request body
            let body_bytes = req.body_bytes();
            
            if body_bytes.is_empty() {
                return Err(ExtractionError::InvalidQuery(
                    "Request body is empty".to_string()
                ));
            }

            // Convert body to string
            let body_str = std::str::from_utf8(body_bytes)
                .map_err(|e| ExtractionError::InvalidQuery(format!("Invalid UTF-8 in form data: {}", e)))?;

            // Deserialize the form data
            let value = T::deserialize_from_form(body_str)?;

            Ok((Form(value), req))
        })
    }
}

/// Trait for types that can be deserialized from form data
pub trait DeserializeFromForm: Sized {
    fn deserialize_from_form(form_data: &str) -> Result<Self, ExtractionError>;
}

// Implement for HashMap<String, String> to get all form fields
impl DeserializeFromForm for HashMap<String, String> {
    fn deserialize_from_form(form_data: &str) -> Result<Self, ExtractionError> {
        let mut fields = HashMap::new();
        
        if form_data.is_empty() {
            return Ok(fields);
        }

        for pair in form_data.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                let key = urlencoding::decode(key)
                    .map_err(|e| ExtractionError::InvalidQuery(format!("Invalid key encoding: {}", e)))?
                    .into_owned();
                let value = urlencoding::decode(value)
                    .map_err(|e| ExtractionError::InvalidQuery(format!("Invalid value encoding: {}", e)))?
                    .into_owned();
                fields.insert(key, value);
            } else {
                // Handle keys without values (e.g., checkboxes)
                let key = urlencoding::decode(pair)
                    .map_err(|e| ExtractionError::InvalidQuery(format!("Invalid key encoding: {}", e)))?
                    .into_owned();
                fields.insert(key, String::new());
            }
        }

        Ok(fields)
    }
}

// Implement for Vec<(String, String)> to preserve order and duplicates
impl DeserializeFromForm for Vec<(String, String)> {
    fn deserialize_from_form(form_data: &str) -> Result<Self, ExtractionError> {
        let mut fields = Vec::new();
        
        if form_data.is_empty() {
            return Ok(fields);
        }

        for pair in form_data.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                let key = urlencoding::decode(key)
                    .map_err(|e| ExtractionError::InvalidQuery(format!("Invalid key encoding: {}", e)))?
                    .into_owned();
                let value = urlencoding::decode(value)
                    .map_err(|e| ExtractionError::InvalidQuery(format!("Invalid value encoding: {}", e)))?
                    .into_owned();
                fields.push((key, value));
            } else {
                let key = urlencoding::decode(pair)
                    .map_err(|e| ExtractionError::InvalidQuery(format!("Invalid key encoding: {}", e)))?
                    .into_owned();
                fields.push((key, String::new()));
            }
        }

        Ok(fields)
    }
}

/// Serde-based form data extractor for custom types
#[cfg(feature = "json")]
pub struct SerdeForm<T>(pub T);

#[cfg(feature = "json")]
impl<T> FromRequest for SerdeForm<T>
where
    T: serde::de::DeserializeOwned,
{
    type Error = ExtractionError;

    fn from_request(
        req: Request,
    ) -> Pin<Box<dyn Future<Output = Result<(Self, Request), Self::Error>> + Send + 'static>> {
        Box::pin(async move {
            // Check content type
            let content_type = req.headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");

            if !content_type.starts_with("application/x-www-form-urlencoded") {
                return Err(ExtractionError::InvalidForm(
                    format!("Expected application/x-www-form-urlencoded content type, got: {}", content_type)
                ));
            }

            // Get the request body
            let body_bytes = req.body_bytes();

            if body_bytes.is_empty() {
                return Err(ExtractionError::InvalidForm(
                    "Request body is empty".to_string()
                ));
            }

            // Convert body to string
            let body_str = std::str::from_utf8(body_bytes)
                .map_err(|e| ExtractionError::InvalidForm(format!("Invalid UTF-8 in form data: {}", e)))?;

            // Deserialize the form data
            let value = deserialize_form_with_serde(body_str)?;

            Ok((SerdeForm(value), req))
        })
    }
}

#[cfg(feature = "json")]
fn deserialize_form_with_serde<T: serde::de::DeserializeOwned>(form_data: &str) -> Result<T, ExtractionError> {

    // First parse into HashMap
    let fields: HashMap<String, String> = DeserializeFromForm::deserialize_from_form(form_data)?;

    // Convert to serde_json::Value for deserialization
    let mut json_map = serde_json::Map::new();
    for (key, value) in fields {
        // Handle form-specific parsing
        let json_value = if value.is_empty() {
            // Empty values could be checkboxes or empty strings
            serde_json::Value::Bool(true)
        } else if value == "on" || value == "true" {
            serde_json::Value::Bool(true)
        } else if value == "off" || value == "false" {
            serde_json::Value::Bool(false)
        } else if let Ok(num) = value.parse::<i64>() {
            serde_json::Value::Number(serde_json::Number::from(num))
        } else if let Ok(float) = value.parse::<f64>() {
            if let Some(num) = serde_json::Number::from_f64(float) {
                serde_json::Value::Number(num)
            } else {
                serde_json::Value::String(value)
            }
        } else {
            serde_json::Value::String(value)
        };
        json_map.insert(key, json_value);
    }

    let json_value = serde_json::Value::Object(json_map);
    serde_json::from_value(json_value).map_err(|e| {
        ExtractionError::InvalidForm(format!("Failed to deserialize form data: {}", e))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_form() {
        let result: Result<HashMap<String, String>, _> = 
            DeserializeFromForm::deserialize_from_form("");
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_simple_form() {
        let result: Result<HashMap<String, String>, _> = 
            DeserializeFromForm::deserialize_from_form("name=john&age=30");
        
        let fields = result.unwrap();
        assert_eq!(fields.get("name"), Some(&"john".to_string()));
        assert_eq!(fields.get("age"), Some(&"30".to_string()));
    }

    #[test]
    fn test_url_encoded_form() {
        let result: Result<HashMap<String, String>, _> = 
            DeserializeFromForm::deserialize_from_form("name=John%20Doe&city=New%20York");
        
        let fields = result.unwrap();
        assert_eq!(fields.get("name"), Some(&"John Doe".to_string()));
        assert_eq!(fields.get("city"), Some(&"New York".to_string()));
    }

    #[test]
    fn test_checkbox_fields() {
        let result: Result<HashMap<String, String>, _> = 
            DeserializeFromForm::deserialize_from_form("subscribe&newsletter=on&name=test");
        
        let fields = result.unwrap();
        assert_eq!(fields.get("subscribe"), Some(&"".to_string()));
        assert_eq!(fields.get("newsletter"), Some(&"on".to_string()));
        assert_eq!(fields.get("name"), Some(&"test".to_string()));
    }

    #[test]
    fn test_vec_preserves_order() {
        let result: Result<Vec<(String, String)>, _> = 
            DeserializeFromForm::deserialize_from_form("a=1&b=2&a=3");
        
        let fields = result.unwrap();
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0], ("a".to_string(), "1".to_string()));
        assert_eq!(fields[1], ("b".to_string(), "2".to_string()));
        assert_eq!(fields[2], ("a".to_string(), "3".to_string()));
    }
}
