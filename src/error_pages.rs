use crate::{Request, Response};
use std::collections::HashMap;

/// Error page configuration and rendering
#[derive(Clone)]
pub struct ErrorPages {
    custom_pages: HashMap<u16, String>,
    use_default_styling: bool,
}

impl ErrorPages {
    /// Create a new error pages handler with default styling
    pub fn new() -> Self {
        Self {
            custom_pages: HashMap::new(),
            use_default_styling: true,
        }
    }

    /// Disable default styling (use plain HTML)
    pub fn without_default_styling(mut self) -> Self {
        self.use_default_styling = false;
        self
    }

    /// Set a custom error page for a specific status code
    pub fn custom_page(mut self, status_code: u16, html: String) -> Self {
        self.custom_pages.insert(status_code, html);
        self
    }

    /// Set a custom 404 page
    pub fn custom_404(self, html: String) -> Self {
        self.custom_page(404, html)
    }

    /// Set a custom 500 page
    pub fn custom_500(self, html: String) -> Self {
        self.custom_page(500, html)
    }

    /// Get a random fun 404 message (Sinatra-inspired with Torch flair)
    pub fn random_404_message() -> &'static str {
        let messages = [
            "🔥 Torch doesn't know this route, but the flame burns eternal!",
            "🔥 This path hasn't been lit by the Torch yet.",
            "🔥 Torch searched high and low, but this page got extinguished.",
            "🔥 Even the brightest flame can't illuminate this missing page.",
            "🔥 Torch doesn't know this ditty, but it's got plenty of other hot tracks!",
            "🔥 This route went up in smoke before Torch could catch it.",
            "🔥 Torch's flame doesn't reach this corner of the web.",
            "🔥 Page not found - looks like this one slipped through the fire.",
            "🔥 Torch is blazing bright, but this path remains in the dark.",
            "🔥 This URL got burned in the great page fire of... well, never.",
        ];

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let index = (now / 5) as usize % messages.len(); // Change every 5 seconds for more variety
        messages[index]
    }

    /// Generate an error response for the given status code
    pub fn render_error(&self, status_code: u16, message: Option<&str>, _req: &Request) -> Response {
        let status = http::StatusCode::from_u16(status_code).unwrap_or(http::StatusCode::INTERNAL_SERVER_ERROR);
        
        // Check for custom page first
        if let Some(custom_html) = self.custom_pages.get(&status_code) {
            return Response::with_status(status)
                .header("Content-Type", "text/html; charset=utf-8")
                .body(custom_html.clone());
        }

        // Generate default error page
        let html = if self.use_default_styling {
            self.generate_styled_error_page(status_code, message)
        } else {
            self.generate_plain_error_page(status_code, message)
        };

        Response::with_status(status)
            .header("Content-Type", "text/html; charset=utf-8")
            .body(html)
    }

    /// Generate a beautifully styled error page with the Torch logo
    fn generate_styled_error_page(&self, status_code: u16, message: Option<&str>) -> String {
        let (title, description) = self.get_error_info(status_code);
        let message = message.unwrap_or(description);

        format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{} - Torch</title>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
            background: linear-gradient(135deg, #1a1a1a 0%, #2d2d2d 100%);
            color: #ffffff;
            min-height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
            padding: 20px;
        }}
        
        .error-container {{
            text-align: center;
            max-width: 600px;
            width: 100%;
        }}
        
        .logo {{
            width: 120px;
            height: 120px;
            margin: 0 auto 30px;
            background: url('data:image/svg+xml;base64,{}') no-repeat center center;
            background-size: contain;
            opacity: 0.9;
        }}
        
        .error-code {{
            font-size: 8rem;
            font-weight: 300;
            background: linear-gradient(135deg, #ff6b35, #f7931e);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            background-clip: text;
            margin-bottom: 20px;
            line-height: 1;
        }}
        
        .error-title {{
            font-size: 2.5rem;
            font-weight: 600;
            margin-bottom: 20px;
            color: #ffffff;
        }}
        
        .error-message {{
            font-size: 1.2rem;
            color: #cccccc;
            margin-bottom: 40px;
            line-height: 1.6;
        }}
        
        .actions {{
            display: flex;
            gap: 20px;
            justify-content: center;
            flex-wrap: wrap;
        }}
        
        .btn {{
            padding: 12px 24px;
            border: none;
            border-radius: 8px;
            font-size: 1rem;
            font-weight: 500;
            text-decoration: none;
            cursor: pointer;
            transition: all 0.3s ease;
            display: inline-block;
        }}
        
        .btn-primary {{
            background: linear-gradient(135deg, #ff6b35, #f7931e);
            color: white;
        }}
        
        .btn-primary:hover {{
            transform: translateY(-2px);
            box-shadow: 0 8px 25px rgba(255, 107, 53, 0.3);
        }}
        
        .btn-secondary {{
            background: transparent;
            color: #cccccc;
            border: 2px solid #555;
        }}
        
        .btn-secondary:hover {{
            background: #555;
            color: white;
        }}
        
        .footer {{
            margin-top: 60px;
            color: #888;
            font-size: 0.9rem;
        }}
        
        @media (max-width: 768px) {{
            .error-code {{
                font-size: 6rem;
            }}
            
            .error-title {{
                font-size: 2rem;
            }}
            
            .error-message {{
                font-size: 1rem;
            }}
            
            .actions {{
                flex-direction: column;
                align-items: center;
            }}
            
            .btn {{
                width: 200px;
            }}
        }}
    </style>
</head>
<body>
    <div class="error-container">
        <div class="logo"></div>
        <div class="error-code">{}</div>
        <h1 class="error-title">{}</h1>
        <p class="error-message">{}</p>
        
        <div class="actions">
            <a href="/" class="btn btn-primary">🏠 Go Home</a>
            <a href="javascript:history.back()" class="btn btn-secondary">← Go Back</a>
        </div>
        
        <div class="footer">
            <p>Powered by <strong>Torch</strong> 🔥</p>
        </div>
    </div>
</body>
</html>"#, 
            title, 
            self.get_torch_logo_base64(),
            status_code, 
            title, 
            message
        )
    }

    /// Generate a plain error page without styling
    fn generate_plain_error_page(&self, status_code: u16, message: Option<&str>) -> String {
        let (title, description) = self.get_error_info(status_code);
        let message = message.unwrap_or(description);

        format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{} - Error</title>
</head>
<body>
    <h1>{} {}</h1>
    <p>{}</p>
    <hr>
    <p><a href="/">Go Home</a> | <a href="javascript:history.back()">Go Back</a></p>
</body>
</html>"#, title, status_code, title, message)
    }

    /// Get error information for common status codes
    fn get_error_info(&self, status_code: u16) -> (&'static str, &'static str) {
        match status_code {
            400 => ("Bad Request", "🔥 Torch couldn't parse that request - try fanning the flames differently!"),
            401 => ("Unauthorized", "🔥 You need to light your credentials before entering this flame zone."),
            403 => ("Forbidden", "🔥 This area is protected by firewall - Torch can't let you pass."),
            404 => ("Page Not Found", self.get_404_message()),
            405 => ("Method Not Allowed", "🔥 That HTTP method got extinguished - try a different approach."),
            408 => ("Request Timeout", "🔥 Your request took too long and the flame went out."),
            418 => ("I'm a teapot", "🫖 Torch is a flame, not a teapot - but we appreciate the confusion!"),
            429 => ("Too Many Requests", "🔥 Whoa there! You're making Torch burn too bright - slow down a bit."),
            500 => ("Internal Server Error", "🔥 Torch had a flare-up! Our engineers are working to contain the blaze."),
            502 => ("Bad Gateway", "🔥 The upstream server sent smoke signals we couldn't decode."),
            503 => ("Service Unavailable", "🔥 Torch is temporarily dimmed for maintenance - we'll be back blazing soon!"),
            504 => ("Gateway Timeout", "🔥 The upstream server's flame went out before we could connect."),
            _ => ("Error", "🔥 Something unexpected happened in the flame chamber."),
        }
    }

    /// Get a fun, Sinatra-inspired 404 message with Torch flair
    fn get_404_message(&self) -> &'static str {
        // Rotate through different fun messages inspired by Sinatra's "Sinatra doesn't know this ditty"
        let messages = [
            "🔥 Torch doesn't know this route, but the flame burns eternal!",
            "🔥 This path hasn't been lit by the Torch yet.",
            "🔥 Torch searched high and low, but this page got extinguished.",
            "🔥 Even the brightest flame can't illuminate this missing page.",
            "🔥 Torch doesn't know this ditty, but it's got plenty of other hot tracks!",
            "🔥 This route went up in smoke before Torch could catch it.",
            "🔥 Torch's flame doesn't reach this corner of the web.",
            "🔥 Page not found - looks like this one slipped through the fire.",
            "🔥 Torch is blazing bright, but this path remains in the dark.",
            "🔥 This URL got burned in the great page fire of... well, never.",
        ];

        // Use a simple hash of the current time to pseudo-randomly select a message
        // This gives variety while being deterministic within the same second
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let index = (now / 10) as usize % messages.len(); // Change every 10 seconds
        messages[index]
    }

    /// Get the Torch logo as base64 SVG
    fn get_torch_logo_base64(&self) -> &'static str {
        // Base64 encoded SVG version of the Torch logo - beautiful flame design
        "PHN2ZyB3aWR0aD0iMTIwIiBoZWlnaHQ9IjEyMCIgdmlld0JveD0iMCAwIDEyMCAxMjAiIGZpbGw9Im5vbmUiIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyI+CjxkZWZzPgo8bGluZWFyR3JhZGllbnQgaWQ9ImZsYW1lR3JhZGllbnQiIHgxPSIwJSIgeTE9IjAlIiB4Mj0iMTAwJSIgeTI9IjEwMCUiPgo8c3RvcCBvZmZzZXQ9IjAlIiBzdHlsZT0ic3RvcC1jb2xvcjojZmZkNzAwO3N0b3Atb3BhY2l0eToxIiAvPgo8c3RvcCBvZmZzZXQ9IjUwJSIgc3R5bGU9InN0b3AtY29sb3I6I2ZmNmIzNTtzdG9wLW9wYWNpdHk6MSIgLz4KPHN0b3Agb2Zmc2V0PSIxMDAlIiBzdHlsZT0ic3RvcC1jb2xvcjojZjc5MzFlO3N0b3Atb3BhY2l0eToxIiAvPgo8L2xpbmVhckdyYWRpZW50Pgo8bGluZWFyR3JhZGllbnQgaWQ9InRvcmNoR3JhZGllbnQiIHgxPSIwJSIgeTE9IjAlIiB4Mj0iMTAwJSIgeTI9IjEwMCUiPgo8c3RvcCBvZmZzZXQ9IjAlIiBzdHlsZT0ic3RvcC1jb2xvcjojZGRhNTIwO3N0b3Atb3BhY2l0eToxIiAvPgo8c3RvcCBvZmZzZXQ9IjEwMCUiIHN0eWxlPSJzdG9wLWNvbG9yOiNiODg2MWE7c3RvcC1vcGFjaXR5OjEiIC8+CjwvbGluZWFyR3JhZGllbnQ+CjwvZGVmcz4KCjwhLS0gTWFpbiBGbGFtZSAtLT4KPHA+CjxwYXRoIGQ9Ik02MCAyMEM2MCAyMCA0MiAxOCA0MiAzOEM0MiA1OCA2MCA2NSA2MCA2NUM2MCA2NSA3OCA1OCA3OCAzOEM3OCAxOCA2MCAyMCA2MCAyMFoiIGZpbGw9InVybCgjZmxhbWVHcmFkaWVudCkiLz4KPC9wPgoKPCEtLSBJbm5lciBGbGFtZSAtLT4KPHA+CjxwYXRoIGQ9Ik02MCAzMEM2MCAzMCA0OCAyOCA0OCA0NEM0OCA1NiA2MCA2MCA2MCA2MEM2MCA2MCA3MiA1NiA3MiA0NEM3MiAyOCA2MCAzMCA2MCAzMFoiIGZpbGw9InVybCgjZmxhbWVHcmFkaWVudCkiIG9wYWNpdHk9IjAuOCIvPgo8L3A+Cgo8IS0tIFNtYWxsIEZsYW1lIC0tPgo8cD4KPHA+CjxwYXRoIGQ9Ik02MCA0MEM2MCA0MCA1MiAzOCA1MiA0OEM1MiA1NCA2MCA1NiA2MCA1NkM2MCA1NiA2OCA1NCA2OCA0OEM2OCAzOCA2MCA0MCA2MCA0MFoiIGZpbGw9InVybCgjZmxhbWVHcmFkaWVudCkiIG9wYWNpdHk9IjAuNiIvPgo8L3A+Cgo8IS0tIFRvcmNoIEhhbmRsZSAtLT4KPHJlY3QgeD0iNTQiIHk9IjY1IiB3aWR0aD0iMTIiIGhlaWdodD0iMzUiIGZpbGw9InVybCgjdG9yY2hHcmFkaWVudCkiIHJ4PSIyIi8+Cgo8IS0tIEhhbmRsZSBEZXRhaWxzIC0tPgo8cmVjdCB4PSI1NiIgeT0iNzAiIHdpZHRoPSI4IiBoZWlnaHQ9IjIiIGZpbGw9IiNhYTc0MTYiLz4KPHJlY3QgeD0iNTYiIHk9Ijc1IiB3aWR0aD0iOCIgaGVpZ2h0PSIyIiBmaWxsPSIjYWE3NDE2Ii8+CjxyZWN0IHg9IjU2IiB5PSI4MCIgd2lkdGg9IjgiIGhlaWdodD0iMiIgZmlsbD0iI2FhNzQxNiIvPgoKPCEtLSBUb3JjaCBCYXNlIC0tPgo8ZWxsaXBzZSBjeD0iNjAiIGN5PSIxMDAiIHJ4PSIxOCIgcnk9IjgiIGZpbGw9InVybCgjdG9yY2hHcmFkaWVudCkiLz4KPGVsbGlwc2UgY3g9IjYwIiBjeT0iOTgiIHJ4PSIxNCIgcnk9IjYiIGZpbGw9IiNkZGE1MjAiLz4KCjx0ZXh0IHg9IjYwIiB5PSIxMTUiIGZvbnQtZmFtaWx5PSJBcmlhbCwgc2Fucy1zZXJpZiIgZm9udC1zaXplPSIxMCIgZm9udC13ZWlnaHQ9ImJvbGQiIGZpbGw9IiNkZGE1MjAiIHRleHQtYW5jaG9yPSJtaWRkbGUiPlRPUkNIPC90ZXh0Pgo8L3N2Zz4="
    }
}

impl Default for ErrorPages {
    fn default() -> Self {
        Self::new()
    }
}
