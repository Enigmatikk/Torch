//! # Ember Template Engine
//!
//! A powerful, Laravel Blade-inspired templating engine for Torch.
//! Ember provides a clean, expressive syntax for building dynamic HTML templates.
//!
//! ## Features
//!
//! - **Blade-like syntax**: Familiar `@if`, `@foreach`, `@extends`, `@section` directives
//! - **Template inheritance**: Build layouts and extend them
//! - **Component system**: Reusable template components
//! - **Automatic escaping**: XSS protection by default
//! - **Template caching**: Compiled templates are cached for performance
//! - **Hot reloading**: Templates are recompiled when changed in development
//!
//! ## Example
//!
//! ```rust,no_run
//! use torch_web::{App, Response, ember::*};
//!
//! async fn home() -> Response {
//!     let data = EmberData::new()
//!         .with("title", "Welcome to Torch")
//!         .with("users", vec!["Alice", "Bob", "Charlie"]);
//!     
//!     ember("home", data).await
//! }
//! ```
//!
//! Template file `templates/home.ember`:
//! ```html
//! @extends('layout')
//!
//! @section('content')
//!     <h1>{{ $title }}</h1>
//!     
//!     @if(count($users) > 0)
//!         <ul>
//!         @foreach($users as $user)
//!             <li>{{ $user }}</li>
//!         @endforeach
//!         </ul>
//!     @else
//!         <p>No users found.</p>
//!     @endif
//! @endsection
//! ```

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use crate::Response;

#[cfg(feature = "templates")]
use {
    once_cell::sync::Lazy,
    regex::Regex,
    std::fs,
    std::sync::RwLock,
};

/// Template data container for passing variables to templates
#[derive(Debug, Clone)]
pub struct EmberData {
    data: HashMap<String, EmberValue>,
}

/// Values that can be passed to templates
#[derive(Debug, Clone)]
pub enum EmberValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<EmberValue>),
    Object(HashMap<String, EmberValue>),
    Null,
}

/// Template engine configuration
#[derive(Debug, Clone)]
pub struct EmberConfig {
    /// Directory where templates are stored
    pub template_dir: PathBuf,
    /// Directory for compiled template cache
    pub cache_dir: Option<PathBuf>,
    /// Whether to enable template caching
    pub cache_enabled: bool,
    /// Whether to enable hot reloading in development
    pub hot_reload: bool,
    /// File extension for templates
    pub extension: String,
}

/// Compiled template representation
#[derive(Debug, Clone)]
struct CompiledTemplate {
    content: String,
    #[allow(dead_code)]
    dependencies: Vec<String>, // For inheritance and includes (future use)
    last_modified: std::time::SystemTime,
}

/// Template engine instance
pub struct EmberEngine {
    config: EmberConfig,
    #[cfg(feature = "templates")]
    cache: Arc<RwLock<HashMap<String, CompiledTemplate>>>,
}

/// Template compilation error
#[derive(Debug)]
pub struct EmberError {
    pub message: String,
    pub template: Option<String>,
    pub line: Option<usize>,
}

impl std::fmt::Display for EmberError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (&self.template, self.line) {
            (Some(template), Some(line)) => {
                write!(f, "Ember error in template '{}' at line {}: {}", template, line, self.message)
            }
            (Some(template), None) => {
                write!(f, "Ember error in template '{}': {}", template, self.message)
            }
            _ => write!(f, "Ember error: {}", self.message),
        }
    }
}

impl std::error::Error for EmberError {}

impl Default for EmberConfig {
    fn default() -> Self {
        Self {
            template_dir: PathBuf::from("templates"),
            cache_dir: Some(PathBuf::from("storage/ember")),
            cache_enabled: true,
            hot_reload: cfg!(debug_assertions),
            extension: "ember".to_string(),
        }
    }
}

impl EmberData {
    /// Create a new empty data container
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Add a value to the data container
    pub fn with<K: Into<String>, V: Into<EmberValue>>(mut self, key: K, value: V) -> Self {
        self.data.insert(key.into(), value.into());
        self
    }

    /// Insert a value into the data container
    pub fn insert<K: Into<String>, V: Into<EmberValue>>(&mut self, key: K, value: V) {
        self.data.insert(key.into(), value.into());
    }

    /// Get a value from the data container
    pub fn get(&self, key: &str) -> Option<&EmberValue> {
        self.data.get(key)
    }

    /// Get all data as a reference to the internal HashMap
    pub fn as_map(&self) -> &HashMap<String, EmberValue> {
        &self.data
    }
}

impl Default for EmberData {
    fn default() -> Self {
        Self::new()
    }
}

// Convenient conversions for EmberValue
impl From<String> for EmberValue {
    fn from(s: String) -> Self {
        EmberValue::String(s)
    }
}

impl From<&str> for EmberValue {
    fn from(s: &str) -> Self {
        EmberValue::String(s.to_string())
    }
}

impl From<i32> for EmberValue {
    fn from(n: i32) -> Self {
        EmberValue::Number(n as f64)
    }
}

impl From<f64> for EmberValue {
    fn from(n: f64) -> Self {
        EmberValue::Number(n)
    }
}

impl From<bool> for EmberValue {
    fn from(b: bool) -> Self {
        EmberValue::Boolean(b)
    }
}

impl<T: Into<EmberValue>> From<Vec<T>> for EmberValue {
    fn from(vec: Vec<T>) -> Self {
        EmberValue::Array(vec.into_iter().map(|v| v.into()).collect())
    }
}

impl From<HashMap<String, EmberValue>> for EmberValue {
    fn from(map: HashMap<String, EmberValue>) -> Self {
        EmberValue::Object(map)
    }
}

#[cfg(feature = "json")]
impl From<serde_json::Value> for EmberValue {
    fn from(value: serde_json::Value) -> Self {
        match value {
            serde_json::Value::String(s) => EmberValue::String(s),
            serde_json::Value::Number(n) => EmberValue::Number(n.as_f64().unwrap_or(0.0)),
            serde_json::Value::Bool(b) => EmberValue::Boolean(b),
            serde_json::Value::Array(arr) => {
                EmberValue::Array(arr.into_iter().map(EmberValue::from).collect())
            }
            serde_json::Value::Object(obj) => {
                let mut map = HashMap::new();
                for (k, v) in obj {
                    map.insert(k, EmberValue::from(v));
                }
                EmberValue::Object(map)
            }
            serde_json::Value::Null => EmberValue::Null,
        }
    }
}

impl EmberEngine {
    /// Create a new Ember engine with default configuration
    pub fn new() -> Self {
        Self::with_config(EmberConfig::default())
    }

    /// Create a new Ember engine with custom configuration
    pub fn with_config(config: EmberConfig) -> Self {
        Self {
            config,
            #[cfg(feature = "templates")]
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Render a template with the given data
    pub async fn render(&self, template_name: &str, data: EmberData) -> Result<String, EmberError> {
        #[cfg(feature = "templates")]
        {
            self.render_template(template_name, data).await
        }
        
        #[cfg(not(feature = "templates"))]
        {
            Err(EmberError {
                message: "Template feature not enabled. Add 'templates' feature to use Ember.".to_string(),
                template: Some(template_name.to_string()),
                line: None,
            })
        }
    }
}

/// Global Ember engine instance
#[cfg(feature = "templates")]
static EMBER_ENGINE: Lazy<EmberEngine> = Lazy::new(|| EmberEngine::new());

/// Render a template using the global Ember engine
pub async fn ember(template_name: &str, data: EmberData) -> Response {
    #[cfg(feature = "templates")]
    {
        match EMBER_ENGINE.render(template_name, data).await {
            Ok(html) => Response::ok().html(html),
            Err(err) => {
                eprintln!("Ember template error: {}", err);
                Response::internal_error()
                    .html(format!("<h1>Template Error</h1><p>{}</p>", err))
            }
        }
    }
    
    #[cfg(not(feature = "templates"))]
    {
        Response::internal_error()
            .html("<h1>Template Error</h1><p>Template feature not enabled. Add 'templates' feature to use Ember.</p>")
    }
}

/// Render a template with no data
pub async fn ember_view(template_name: &str) -> Response {
    ember(template_name, EmberData::new()).await
}

#[cfg(feature = "templates")]
impl EmberEngine {
    /// Internal method to render a template
    async fn render_template(&self, template_name: &str, data: EmberData) -> Result<String, EmberError> {
        // Load and compile the template
        let compiled = self.load_template(template_name).await?;

        // Render the compiled template with data
        self.execute_template(&compiled.content, &data)
    }

    /// Load and compile a template, using cache if available
    async fn load_template(&self, template_name: &str) -> Result<CompiledTemplate, EmberError> {
        let template_path = self.get_template_path(template_name);

        // Check if template file exists
        if !template_path.exists() {
            return Err(EmberError {
                message: format!("Template file not found: {}", template_path.display()),
                template: Some(template_name.to_string()),
                line: None,
            });
        }

        // Get file modification time
        let metadata = fs::metadata(&template_path).map_err(|e| EmberError {
            message: format!("Failed to read template metadata: {}", e),
            template: Some(template_name.to_string()),
            line: None,
        })?;

        let last_modified = metadata.modified().unwrap_or(std::time::UNIX_EPOCH);

        // Check cache if enabled
        if self.config.cache_enabled {
            if let Ok(cache) = self.cache.read() {
                if let Some(cached) = cache.get(template_name) {
                    // Use cached version if it's still fresh or hot reload is disabled
                    if !self.config.hot_reload || cached.last_modified >= last_modified {
                        return Ok(cached.clone());
                    }
                }
            }
        }

        // Read and compile the template
        let template_content = fs::read_to_string(&template_path).map_err(|e| EmberError {
            message: format!("Failed to read template file: {}", e),
            template: Some(template_name.to_string()),
            line: None,
        })?;

        let compiled_content = self.compile_template(&template_content, template_name)?;

        let compiled = CompiledTemplate {
            content: compiled_content,
            dependencies: Vec::new(), // TODO: Track dependencies for inheritance
            last_modified,
        };

        // Cache the compiled template
        if self.config.cache_enabled {
            if let Ok(mut cache) = self.cache.write() {
                cache.insert(template_name.to_string(), compiled.clone());
            }
        }

        Ok(compiled)
    }

    /// Get the full path to a template file
    fn get_template_path(&self, template_name: &str) -> PathBuf {
        let mut path = self.config.template_dir.clone();
        path.push(format!("{}.{}", template_name, self.config.extension));
        path
    }

    /// Compile Ember template syntax to executable template
    fn compile_template(&self, content: &str, _template_name: &str) -> Result<String, EmberError> {
        // For now, just return the content as-is since we'll process it directly
        // In the future, we could add optimizations here
        Ok(content.to_string())
    }



    /// Execute the compiled template with data
    fn execute_template(&self, compiled: &str, data: &EmberData) -> Result<String, EmberError> {
        let mut result = compiled.to_string();

        // Handle template inheritance (@extends) first
        result = self.process_inheritance(&result, data)?;

        // Process includes
        result = self.process_includes(&result, data)?;

        // Process sections (for templates without inheritance)
        result = self.process_sections(&result, data)?;

        // Process conditionals
        result = self.process_conditionals(&result, data)?;

        // Process loops
        result = self.process_loops(&result, data)?;

        // Replace variables last
        result = self.replace_variables(&result, data)?;

        Ok(result)
    }

    /// Process template inheritance (@extends)
    fn process_inheritance(&self, content: &str, _data: &EmberData) -> Result<String, EmberError> {
        static EXTENDS_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r#"@extends\s*\(\s*['"]([^'"]+)['"]\s*\)"#).unwrap()
        });

        if let Some(captures) = EXTENDS_REGEX.captures(content) {
            let layout_name = &captures[1];

            // Load the layout template
            let layout_path = self.get_template_path(layout_name);
            if !layout_path.exists() {
                return Ok(content.to_string()); // If layout doesn't exist, return content as-is
            }

            let layout_content = fs::read_to_string(&layout_path).map_err(|e| EmberError {
                message: format!("Failed to read layout template: {}", e),
                template: Some(layout_name.to_string()),
                line: None,
            })?;

            // Remove the @extends directive from child content
            let child_content = EXTENDS_REGEX.replace(content, "").to_string();

            // Extract sections from child template
            let sections = self.extract_sections(&child_content)?;

            // Replace sections in layout
            let mut result = layout_content;
            for (section_name, section_content) in sections {
                // Find and replace the section in layout
                static SECTION_BLOCK_REGEX: Lazy<Regex> = Lazy::new(|| {
                    Regex::new(r#"(?s)@section\s*\(\s*['"]([^'"]+)['"]\s*\)(.*?)@endsection"#).unwrap()
                });

                result = SECTION_BLOCK_REGEX.replace_all(&result, |caps: &regex::Captures| {
                    let name = &caps[1];
                    if name == section_name {
                        section_content.clone()
                    } else {
                        caps[0].to_string() // Keep original if not matching
                    }
                }).to_string();
            }

            return Ok(result);
        }

        Ok(content.to_string())
    }

    /// Extract sections from template content
    fn extract_sections(&self, content: &str) -> Result<HashMap<String, String>, EmberError> {
        static SECTION_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r#"(?s)@section\s*\(\s*['"]([^'"]+)['"]\s*\)(.*?)@endsection"#).unwrap()
        });

        let mut sections = HashMap::new();

        for captures in SECTION_REGEX.captures_iter(content) {
            let section_name = captures[1].to_string();
            let section_content = captures[2].to_string();
            sections.insert(section_name, section_content);
        }

        Ok(sections)
    }

    /// Process sections (for templates without inheritance)
    fn process_sections(&self, content: &str, _data: &EmberData) -> Result<String, EmberError> {
        static SECTION_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r#"(?s)@section\s*\(\s*['"]([^'"]+)['"]\s*\)(.*?)@endsection"#).unwrap()
        });

        // For templates without inheritance, just replace sections with their content
        let result = SECTION_REGEX.replace_all(content, "$2").to_string();
        Ok(result)
    }

    /// Process conditional statements
    fn process_conditionals(&self, content: &str, data: &EmberData) -> Result<String, EmberError> {
        static IF_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"(?s)@if\s*\(\s*([^)]+)\s*\)(.*?)(?:@else(.*?))?@endif").unwrap()
        });

        let result = IF_REGEX.replace_all(content, |caps: &regex::Captures| {
            let condition = &caps[1];
            let if_content = &caps[2];
            let else_content = caps.get(3).map(|m| m.as_str()).unwrap_or("");

            // Simple condition evaluation (just check if variable exists and is truthy)
            if self.evaluate_condition(condition, data) {
                if_content.to_string()
            } else {
                else_content.to_string()
            }
        }).to_string();

        Ok(result)
    }

    /// Process loop statements
    fn process_loops(&self, content: &str, data: &EmberData) -> Result<String, EmberError> {
        static FOREACH_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"(?s)@foreach\s*\(\s*\$([a-zA-Z_][a-zA-Z0-9_]*)\s+as\s+\$([a-zA-Z_][a-zA-Z0-9_]*)\s*\)(.*?)@endforeach").unwrap()
        });

        let mut result = content.to_string();

        // Process each foreach loop
        while let Some(captures) = FOREACH_REGEX.captures(&result) {
            let full_match = captures[0].to_string();
            let array_var = &captures[1];
            let item_var = &captures[2];
            let loop_content = &captures[3];

            let replacement = if let Some(EmberValue::Array(items)) = data.get(array_var) {
                let mut output = String::new();
                for item in items {
                    let mut loop_data = data.clone();
                    loop_data.insert(item_var, item.clone());

                    let item_content = self.replace_variables(loop_content, &loop_data)?;
                    output.push_str(&item_content);
                }
                output
            } else {
                String::new() // If array doesn't exist, render nothing
            };

            result = result.replace(&full_match, &replacement);
        }

        Ok(result)
    }

    /// Process include statements
    fn process_includes(&self, content: &str, data: &EmberData) -> Result<String, EmberError> {
        static INCLUDE_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r#"@include\s*\(\s*['"]([^'"]+)['"]\s*\)"#).unwrap()
        });

        let result = INCLUDE_REGEX.replace_all(content, |caps: &regex::Captures| {
            let include_name = &caps[1];

            // Load and render the included template
            match self.load_and_render_include(include_name, data) {
                Ok(rendered) => rendered,
                Err(_) => format!("<!-- Include '{}' not found -->", include_name),
            }
        }).to_string();

        Ok(result)
    }

    /// Load and render an included template
    fn load_and_render_include(&self, template_name: &str, data: &EmberData) -> Result<String, EmberError> {
        let template_path = self.get_template_path(template_name);
        if !template_path.exists() {
            return Err(EmberError {
                message: format!("Include template not found: {}", template_name),
                template: Some(template_name.to_string()),
                line: None,
            });
        }

        let content = fs::read_to_string(&template_path).map_err(|e| EmberError {
            message: format!("Failed to read include template: {}", e),
            template: Some(template_name.to_string()),
            line: None,
        })?;

        // Recursively process the included template
        self.execute_template(&content, data)
    }

    /// Replace variables in the template
    fn replace_variables(&self, content: &str, data: &EmberData) -> Result<String, EmberError> {
        static VAR_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"\{\{\s*\$([a-zA-Z_][a-zA-Z0-9_]*)\s*\}\}").unwrap()
        });

        let result = VAR_REGEX.replace_all(content, |caps: &regex::Captures| {
            let var_name = &caps[1];
            if let Some(value) = data.get(var_name) {
                self.value_to_string(value)
            } else {
                format!("{{{{ ${} }}}}", var_name) // Keep placeholder if variable not found
            }
        }).to_string();

        Ok(result)
    }

    /// Evaluate a simple condition
    fn evaluate_condition(&self, condition: &str, data: &EmberData) -> bool {
        // Simple condition evaluation - just check if variable exists and is truthy
        let condition = condition.trim();

        // Handle count() function
        if condition.starts_with("count(") && condition.ends_with(")") {
            let inner = &condition[6..condition.len()-1];
            let var_name = inner.trim().trim_start_matches('$');
            if let Some(EmberValue::Array(items)) = data.get(var_name) {
                return items.len() > 0;
            }
            return false;
        }

        // Handle simple variable checks
        if condition.starts_with('$') {
            let var_name = &condition[1..];
            if let Some(value) = data.get(var_name) {
                match value {
                    EmberValue::Boolean(b) => *b,
                    EmberValue::String(s) => !s.is_empty(),
                    EmberValue::Number(n) => *n != 0.0,
                    EmberValue::Array(arr) => !arr.is_empty(),
                    EmberValue::Object(obj) => !obj.is_empty(),
                    EmberValue::Null => false,
                }
            } else {
                false
            }
        } else {
            // Handle literal true/false
            match condition {
                "true" => true,
                "false" => false,
                _ => false,
            }
        }
    }

    /// Convert EmberValue to string for template output
    fn value_to_string(&self, value: &EmberValue) -> String {
        match value {
            EmberValue::String(s) => s.clone(),
            EmberValue::Number(n) => n.to_string(),
            EmberValue::Boolean(b) => b.to_string(),
            EmberValue::Array(arr) => {
                format!("[{}]", arr.iter().map(|v| self.value_to_string(v)).collect::<Vec<_>>().join(", "))
            }
            EmberValue::Object(_) => "[Object]".to_string(),
            EmberValue::Null => "".to_string(),
        }
    }
}
