//! Maintenance mode commands

use colored::*;
use std::fs;
use std::path::Path;

/// Put application in maintenance mode
pub fn down(secret: Option<String>, render: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    println!("{} Putting application in maintenance mode...", "🚧".yellow());
    
    let maintenance_file = "storage/framework/down";
    
    // Create storage directory if it doesn't exist
    fs::create_dir_all("storage/framework")?;
    
    let mut maintenance_data = serde_json::json!({
        "time": chrono::Utc::now().timestamp(),
        "message": "Application is temporarily unavailable for maintenance.",
        "retry": 60
    });
    
    if let Some(ref secret_key) = secret {
        maintenance_data["secret"] = serde_json::Value::String(secret_key.clone());
        println!("{} Bypass secret: {}", "🔑".blue(), secret_key.cyan());
    }

    if let Some(ref view) = render {
        maintenance_data["template"] = serde_json::Value::String(view.clone());
        println!("{} Custom view: {}", "🎨".blue(), view.cyan());
    }
    
    fs::write(maintenance_file, serde_json::to_string_pretty(&maintenance_data)?)?;
    
    // Create maintenance template if it doesn't exist
    create_maintenance_template(render.as_deref())?;
    
    println!("{} Application is now in maintenance mode", "✅".green());
    
    if let Some(ref secret_key) = secret {
        println!();
        println!("{} To bypass maintenance mode, add this to your URL:", "💡".blue());
        println!("  ?secret={}", secret_key.cyan());
    }
    
    Ok(())
}

/// Bring application out of maintenance mode
pub fn up() -> Result<(), Box<dyn std::error::Error>> {
    println!("{} Bringing application out of maintenance mode...", "🔧".yellow());
    
    let maintenance_file = "storage/framework/down";
    
    if Path::new(maintenance_file).exists() {
        fs::remove_file(maintenance_file)?;
        println!("{} Application is now live", "✅".green());
    } else {
        println!("{} Application is not in maintenance mode", "ℹ️".blue());
    }
    
    Ok(())
}

/// Create maintenance template
fn create_maintenance_template(custom_view: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let template_path = if let Some(view) = custom_view {
        format!("templates/{}.ember", view)
    } else {
        "templates/maintenance.ember".to_string()
    };
    
    // Only create if it doesn't exist
    if Path::new(&template_path).exists() {
        return Ok(());
    }
    
    // Create templates directory if it doesn't exist
    if let Some(parent) = Path::new(&template_path).parent() {
        fs::create_dir_all(parent)?;
    }
    
    let maintenance_template = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Maintenance Mode - Torch App</title>
    <style>
        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            margin: 0;
            padding: 0;
            min-height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
            color: white;
        }
        .container {
            text-align: center;
            max-width: 600px;
            padding: 40px;
            background: rgba(255, 255, 255, 0.1);
            border-radius: 20px;
            backdrop-filter: blur(10px);
            box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
        }
        .flame {
            font-size: 4rem;
            margin-bottom: 20px;
            animation: flicker 2s infinite alternate;
        }
        @keyframes flicker {
            0% { opacity: 1; }
            100% { opacity: 0.8; }
        }
        h1 {
            font-size: 2.5rem;
            margin-bottom: 20px;
            font-weight: 300;
        }
        p {
            font-size: 1.2rem;
            line-height: 1.6;
            margin-bottom: 30px;
            opacity: 0.9;
        }
        .status {
            background: rgba(255, 255, 255, 0.2);
            padding: 15px 30px;
            border-radius: 50px;
            display: inline-block;
            font-weight: 500;
        }
        .footer {
            margin-top: 40px;
            font-size: 0.9rem;
            opacity: 0.7;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="flame">🔥</div>
        <h1>Maintenance Mode</h1>
        <p>
            We're currently performing some maintenance on our application.
            We'll be back online shortly!
        </p>
        <div class="status">
            🔧 Maintenance in Progress
        </div>
        <div class="footer">
            <p>Thank you for your patience.</p>
            <p><strong>Powered by Torch</strong></p>
        </div>
    </div>
</body>
</html>
"#;
    
    fs::write(&template_path, maintenance_template)?;
    
    println!("{} Created maintenance template: {}", "📝".blue(), template_path.cyan());
    
    Ok(())
}

/// Check if application is in maintenance mode
pub fn is_maintenance_mode() -> bool {
    Path::new("storage/framework/down").exists()
}

/// Get maintenance mode data
pub fn get_maintenance_data() -> Result<Option<serde_json::Value>, Box<dyn std::error::Error>> {
    let maintenance_file = "storage/framework/down";
    
    if !Path::new(maintenance_file).exists() {
        return Ok(None);
    }
    
    let content = fs::read_to_string(maintenance_file)?;
    let data: serde_json::Value = serde_json::from_str(&content)?;
    
    Ok(Some(data))
}
