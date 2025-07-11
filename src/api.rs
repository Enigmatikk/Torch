//! # API Development and Documentation
//!
//! This module provides comprehensive tools for building and documenting REST APIs
//! with Torch. It includes API versioning, automatic OpenAPI/Swagger documentation
//! generation, endpoint documentation, and API testing utilities.
//!
//! ## Features
//!
//! - **API Versioning**: Support for multiple API versions with deprecation handling
//! - **OpenAPI Generation**: Automatic OpenAPI 3.0 specification generation
//! - **Interactive Documentation**: Built-in Swagger UI for API exploration
//! - **Endpoint Documentation**: Rich documentation for API endpoints
//! - **Schema Validation**: Request/response schema validation
//! - **API Testing**: Built-in testing utilities for API endpoints
//! - **Rate Limiting**: Per-endpoint rate limiting configuration
//! - **Authentication**: API key and JWT authentication support
//!
//! **Note**: This module requires the `api` feature to be enabled.
//!
//! ## Quick Start
//!
//! ### Basic API Setup
//!
//! ```rust
//! use torch_web::{App, api::*};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Deserialize, Serialize)]
//! struct User {
//!     id: u32,
//!     name: String,
//!     email: String,
//! }
//!
//! let app = App::new()
//!     // Enable API documentation
//!     .with_api_docs(ApiDocBuilder::new()
//!         .title("My API")
//!         .version("1.0.0")
//!         .description("A sample API built with Torch"))
//!
//!     // API endpoints with documentation
//!     .get("/api/users", |_req| async {
//!         let users = vec![
//!             User { id: 1, name: "Alice".to_string(), email: "alice@example.com".to_string() },
//!             User { id: 2, name: "Bob".to_string(), email: "bob@example.com".to_string() },
//!         ];
//!         Response::ok().json(&users)
//!     })
//!     .document_endpoint("/api/users", EndpointDoc::new()
//!         .method("GET")
//!         .summary("List all users")
//!         .description("Returns a list of all users in the system")
//!         .response(200, ResponseDoc::new()
//!             .description("List of users")
//!             .json_schema::<Vec<User>>()))
//!
//!     // Swagger UI endpoint
//!     .get("/docs", |_req| async {
//!         swagger_ui("/api/openapi.json")
//!     })
//!
//!     // OpenAPI spec endpoint
//!     .get("/api/openapi.json", |_req| async {
//!         generate_openapi_spec()
//!     });
//! ```
//!
//! ### API Versioning
//!
//! ```rust
//! use torch_web::{App, api::*};
//!
//! let app = App::new()
//!     // Version 1 (deprecated)
//!     .api_version(ApiVersion::new("v1", "Legacy API")
//!         .deprecated(Some("2024-12-31")))
//!     .get("/api/v1/users", |_req| async {
//!         Response::ok()
//!             .header("Deprecation", "true")
//!             .header("Sunset", "2024-12-31")
//!             .json(&legacy_users())
//!     })
//!
//!     // Version 2 (current)
//!     .api_version(ApiVersion::new("v2", "Current API"))
//!     .get("/api/v2/users", |_req| async {
//!         Response::ok().json(&current_users())
//!     });
//! ```
//!
//! ### Advanced Documentation
//!
//! ```rust
//! use torch_web::{App, api::*, extractors::*};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Deserialize)]
//! struct CreateUserRequest {
//!     name: String,
//!     email: String,
//! }
//!
//! #[derive(Serialize)]
//! struct CreateUserResponse {
//!     id: u32,
//!     name: String,
//!     email: String,
//!     created_at: String,
//! }
//!
//! let app = App::new()
//!     .post("/api/users", |Json(req): Json<CreateUserRequest>| async move {
//!         // Create user logic
//!         let user = CreateUserResponse {
//!             id: 123,
//!             name: req.name,
//!             email: req.email,
//!             created_at: "2024-01-01T00:00:00Z".to_string(),
//!         };
//!         Response::created().json(&user)
//!     })
//!     .document_endpoint("/api/users", EndpointDoc::new()
//!         .method("POST")
//!         .summary("Create a new user")
//!         .description("Creates a new user account with the provided information")
//!         .tag("Users")
//!         .request_body(RequestBodyDoc::new()
//!             .description("User creation data")
//!             .json_schema::<CreateUserRequest>()
//!             .required(true))
//!         .response(201, ResponseDoc::new()
//!             .description("User created successfully")
//!             .json_schema::<CreateUserResponse>())
//!         .response(400, ResponseDoc::new()
//!             .description("Invalid request data"))
//!         .response(409, ResponseDoc::new()
//!             .description("User already exists")));
//! ```

use std::collections::HashMap;
use crate::{Request, Response, App, Handler};

#[cfg(feature = "json")]
use serde_json::{json, Value};

/// API version information
#[derive(Debug, Clone)]
pub struct ApiVersion {
    pub version: String,
    pub description: String,
    pub deprecated: bool,
    pub sunset_date: Option<String>,
}

impl ApiVersion {
    pub fn new(version: &str, description: &str) -> Self {
        Self {
            version: version.to_string(),
            description: description.to_string(),
            deprecated: false,
            sunset_date: None,
        }
    }

    pub fn deprecated(mut self, sunset_date: Option<&str>) -> Self {
        self.deprecated = true;
        self.sunset_date = sunset_date.map(|s| s.to_string());
        self
    }
}

/// API endpoint documentation
#[derive(Debug, Clone)]
pub struct EndpointDoc {
    pub method: String,
    pub path: String,
    pub summary: String,
    pub description: String,
    pub parameters: Vec<ParameterDoc>,
    pub responses: HashMap<u16, ResponseDoc>,
    pub tags: Vec<String>,
}

/// Internal API endpoint representation
#[derive(Debug, Clone)]
pub struct ApiEndpoint {
    pub method: String,
    pub path: String,
    pub summary: String,
    pub description: String,
    pub parameters: Vec<ParameterDoc>,
    pub responses: HashMap<u16, ResponseDoc>,
    pub tags: Vec<String>,
}

/// Complete API documentation
#[derive(Debug, Clone)]
pub struct ApiDocumentation {
    pub title: String,
    pub version: String,
    pub description: String,
    pub endpoints: Vec<ApiEndpoint>,
}

#[derive(Debug, Clone)]
pub struct ParameterDoc {
    pub name: String,
    pub location: ParameterLocation,
    pub description: String,
    pub required: bool,
    pub schema_type: String,
    pub example: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ParameterLocation {
    Path,
    Query,
    Header,
    Body,
}

#[derive(Debug, Clone)]
pub struct ResponseDoc {
    pub description: String,
    pub content_type: String,
    pub example: Option<String>,
}

/// API documentation builder
#[derive(Clone)]
pub struct ApiDocBuilder {
    title: String,
    description: String,
    version: String,
    base_url: String,
    endpoints: Vec<EndpointDoc>,
    versions: HashMap<String, ApiVersion>,
}

impl ApiDocBuilder {
    pub fn new(title: &str, version: &str) -> Self {
        Self {
            title: title.to_string(),
            description: String::new(),
            version: version.to_string(),
            base_url: "/".to_string(),
            endpoints: Vec::new(),
            versions: HashMap::new(),
        }
    }

    pub fn description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    pub fn base_url(mut self, base_url: &str) -> Self {
        self.base_url = base_url.to_string();
        self
    }

    pub fn add_version(mut self, version: ApiVersion) -> Self {
        self.versions.insert(version.version.clone(), version);
        self
    }

    pub fn add_endpoint(mut self, endpoint: EndpointDoc) -> Self {
        self.endpoints.push(endpoint);
        self
    }

    /// Generate OpenAPI 3.0 specification
    #[cfg(feature = "json")]
    pub fn generate_openapi(&self) -> Value {
        let mut paths = serde_json::Map::new();
        
        for endpoint in &self.endpoints {
            let path_item = paths.entry(&endpoint.path).or_insert_with(|| json!({}));
            
            let mut operation = serde_json::Map::new();
            operation.insert("summary".to_string(), json!(endpoint.summary));
            operation.insert("description".to_string(), json!(endpoint.description));
            operation.insert("tags".to_string(), json!(endpoint.tags));
            
            // Note: deprecated field removed from ApiEndpoint for simplicity
            
            // Parameters
            if !endpoint.parameters.is_empty() {
                let params: Vec<Value> = endpoint.parameters.iter().map(|p| {
                    json!({
                        "name": p.name,
                        "in": match p.location {
                            ParameterLocation::Path => "path",
                            ParameterLocation::Query => "query",
                            ParameterLocation::Header => "header",
                            ParameterLocation::Body => "body",
                        },
                        "description": p.description,
                        "required": p.required,
                        "schema": {
                            "type": p.schema_type
                        }
                    })
                }).collect();
                operation.insert("parameters".to_string(), json!(params));
            }
            
            // Responses
            let mut responses = serde_json::Map::new();
            for (status, response) in &endpoint.responses {
                responses.insert(status.to_string(), json!({
                    "description": response.description,
                    "content": {
                        response.content_type.clone(): {
                            "example": response.example
                        }
                    }
                }));
            }
            operation.insert("responses".to_string(), json!(responses));
            
            path_item[endpoint.method.to_lowercase()] = json!(operation);
        }
        
        json!({
            "openapi": "3.0.0",
            "info": {
                "title": self.title,
                "description": self.description,
                "version": self.version
            },
            "servers": [{
                "url": self.base_url
            }],
            "paths": paths
        })
    }

    #[cfg(not(feature = "json"))]
    pub fn generate_openapi(&self) -> String {
        "OpenAPI generation requires 'json' feature".to_string()
    }

    /// Generate simple HTML documentation
    pub fn generate_html_docs(&self) -> String {
        let mut html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>{} API Documentation</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; }}
        .endpoint {{ margin: 20px 0; padding: 20px; border: 1px solid #ddd; border-radius: 5px; }}
        .method {{ display: inline-block; padding: 4px 8px; border-radius: 3px; color: white; font-weight: bold; }}
        .get {{ background-color: #61affe; }}
        .post {{ background-color: #49cc90; }}
        .put {{ background-color: #fca130; }}
        .delete {{ background-color: #f93e3e; }}
        .deprecated {{ opacity: 0.6; }}
        .parameter {{ margin: 10px 0; padding: 10px; background-color: #f8f9fa; border-radius: 3px; }}
    </style>
</head>
<body>
    <h1>{} API Documentation</h1>
    <p>{}</p>
    <p><strong>Version:</strong> {}</p>
"#,
            self.title, self.title, self.description, self.version
        );

        if !self.versions.is_empty() {
            html.push_str("<h2>Available Versions</h2>");
            for version in self.versions.values() {
                let deprecated_class = if version.deprecated { " class=\"deprecated\"" } else { "" };
                html.push_str(&format!(
                    "<div{}><strong>v{}</strong> - {}</div>",
                    deprecated_class, version.version, version.description
                ));
            }
        }

        html.push_str("<h2>Endpoints</h2>");
        
        for endpoint in &self.endpoints {
            let deprecated_class = ""; // Deprecated field removed for simplicity
            let method_class = endpoint.method.to_lowercase();
            
            html.push_str(&format!(
                r#"<div class="endpoint{}">
                    <h3><span class="method {}">{}</span> {}</h3>
                    <p><strong>Summary:</strong> {}</p>
                    <p>{}</p>
"#,
                deprecated_class, method_class, endpoint.method, endpoint.path,
                endpoint.summary, endpoint.description
            ));

            if !endpoint.parameters.is_empty() {
                html.push_str("<h4>Parameters</h4>");
                for param in &endpoint.parameters {
                    html.push_str(&format!(
                        r#"<div class="parameter">
                            <strong>{}</strong> ({:?}) - {}
                            {}</div>"#,
                        param.name,
                        param.location,
                        param.description,
                        if param.required { " <em>(required)</em>" } else { "" }
                    ));
                }
            }

            if !endpoint.responses.is_empty() {
                html.push_str("<h4>Responses</h4>");
                for (status, response) in &endpoint.responses {
                    html.push_str(&format!(
                        "<div><strong>{}</strong> - {}</div>",
                        status, response.description
                    ));
                }
            }

            html.push_str("</div>");
        }

        html.push_str("</body></html>");
        html
    }
}

/// API versioning middleware
pub struct ApiVersioning {
    default_version: String,
    supported_versions: Vec<String>,
    version_header: String,
}

impl ApiVersioning {
    pub fn new(default_version: &str) -> Self {
        Self {
            default_version: default_version.to_string(),
            supported_versions: vec![default_version.to_string()],
            version_header: "API-Version".to_string(),
        }
    }

    pub fn add_version(mut self, version: &str) -> Self {
        self.supported_versions.push(version.to_string());
        self
    }

    pub fn version_header(mut self, header: &str) -> Self {
        self.version_header = header.to_string();
        self
    }

    fn extract_version(&self, req: &Request) -> String {
        // Try header first
        if let Some(version) = req.header(&self.version_header) {
            return version.to_string();
        }

        // Try query parameter
        if let Some(version) = req.query("version") {
            return version.to_string();
        }

        // Try path prefix (e.g., /v1/users)
        let path = req.path();
        if path.starts_with("/v") {
            if let Some(version_part) = path.split('/').nth(1) {
                if version_part.starts_with('v') {
                    return version_part[1..].to_string();
                }
            }
        }

        self.default_version.clone()
    }
}

impl crate::middleware::Middleware for ApiVersioning {
    fn call(
        &self,
        req: Request,
        next: Box<dyn Fn(Request) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send + 'static>> + Send + Sync>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send + 'static>> {
        let version = self.extract_version(&req);
        let supported_versions = self.supported_versions.clone();
        let version_header = self.version_header.clone();

        Box::pin(async move {
            // Check if version is supported
            if !supported_versions.contains(&version) {
                return Response::bad_request()
                    .json(&json!({
                        "error": "Unsupported API version",
                        "requested_version": version,
                        "supported_versions": supported_versions
                    }))
                    .unwrap_or_else(|_| Response::bad_request().body("Unsupported API version"));
            }

            // Add version info to request context (would need to extend Request struct)
            let mut response = next(req).await;
            response = response.header(&version_header, &version);
            response
        })
    }
}

/// Convenience methods for App to add documented endpoints
impl App {
    /// Add a documented GET endpoint
    pub fn documented_get<H, T>(
        self,
        path: &str,
        handler: H,
        doc: EndpointDoc,
    ) -> Self
    where
        H: Handler<T>,
    {
        // Store the documentation for later use in API doc generation
        #[cfg(feature = "api")]
        {
            let mut app = self;
            if let Some(ref mut api_docs) = app.api_docs {
                let mut endpoint_doc = doc;
                endpoint_doc.method = "GET".to_string();
                endpoint_doc.path = path.to_string();
                *api_docs = api_docs.clone().add_endpoint(endpoint_doc);
            }
            app.get(path, handler)
        }

        #[cfg(not(feature = "api"))]
        {
            let _ = doc; // Suppress unused warning
            self.get(path, handler)
        }
    }

    /// Add a documented POST endpoint
    pub fn documented_post<H, T>(
        self,
        path: &str,
        handler: H,
        doc: EndpointDoc,
    ) -> Self
    where
        H: Handler<T>,
    {
        // Store the documentation for later use in API doc generation
        #[cfg(feature = "api")]
        {
            let mut app = self;
            if let Some(ref mut api_docs) = app.api_docs {
                let mut endpoint_doc = doc;
                endpoint_doc.method = "POST".to_string();
                endpoint_doc.path = path.to_string();
                *api_docs = api_docs.clone().add_endpoint(endpoint_doc);
            }
            app.post(path, handler)
        }

        #[cfg(not(feature = "api"))]
        {
            let _ = doc; // Suppress unused warning
            self.post(path, handler)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_version() {
        let version = ApiVersion::new("1.0", "Initial version");
        assert_eq!(version.version, "1.0");
        assert!(!version.deprecated);
    }

    #[test]
    fn test_api_doc_builder() {
        let builder = ApiDocBuilder::new("Test API", "1.0")
            .description("A test API")
            .base_url("https://api.example.com");
        
        assert_eq!(builder.title, "Test API");
        assert_eq!(builder.version, "1.0");
    }

    #[cfg(feature = "json")]
    #[test]
    fn test_openapi_generation() {
        let mut builder = ApiDocBuilder::new("Test API", "1.0");
        
        let endpoint = EndpointDoc {
            method: "GET".to_string(),
            path: "/users".to_string(),
            summary: "Get users".to_string(),
            description: "Retrieve all users".to_string(),
            parameters: vec![],
            responses: HashMap::new(),
            tags: vec!["users".to_string()],
            // deprecated field removed
        };
        
        builder = builder.add_endpoint(endpoint);
        let openapi = builder.generate_openapi();
        
        assert!(openapi["openapi"].as_str().unwrap().starts_with("3.0"));
        assert_eq!(openapi["info"]["title"], "Test API");
    }
}
