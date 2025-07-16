//! Cache operations commands

use crate::cli::CacheOperation;
use colored::*;
use std::fs;
use std::path::Path;

/// Handle cache operations
pub fn handle_operation(operation: CacheOperation) -> Result<(), Box<dyn std::error::Error>> {
    match operation {
        CacheOperation::Clear => {
            clear_all_caches()?;
        }
        CacheOperation::Config => {
            clear_config_cache()?;
        }
        CacheOperation::Route => {
            clear_route_cache()?;
        }
        CacheOperation::View => {
            clear_view_cache()?;
        }
        CacheOperation::Stats => {
            show_cache_stats()?;
        }
    }
    Ok(())
}

/// Clear all caches
fn clear_all_caches() -> Result<(), Box<dyn std::error::Error>> {
    println!("{} Clearing all caches...", "🗑️".yellow());
    
    let mut cleared = Vec::new();
    
    // Clear config cache
    if clear_config_cache_internal()? {
        cleared.push("Configuration");
    }
    
    // Clear route cache
    if clear_route_cache_internal()? {
        cleared.push("Routes");
    }
    
    // Clear view cache
    if clear_view_cache_internal()? {
        cleared.push("Views");
    }
    
    // Clear application cache
    if clear_app_cache_internal()? {
        cleared.push("Application");
    }
    
    if cleared.is_empty() {
        println!("{} No caches found to clear", "ℹ️".blue());
    } else {
        println!("{} Cleared caches: {}", "✅".green(), cleared.join(", "));
    }
    
    Ok(())
}

/// Clear configuration cache
fn clear_config_cache() -> Result<(), Box<dyn std::error::Error>> {
    println!("{} Clearing configuration cache...", "🗑️".yellow());
    
    if clear_config_cache_internal()? {
        println!("{} Configuration cache cleared", "✅".green());
    } else {
        println!("{} No configuration cache found", "ℹ️".blue());
    }
    
    Ok(())
}

/// Clear route cache
fn clear_route_cache() -> Result<(), Box<dyn std::error::Error>> {
    println!("{} Clearing route cache...", "🗑️".yellow());
    
    if clear_route_cache_internal()? {
        println!("{} Route cache cleared", "✅".green());
    } else {
        println!("{} No route cache found", "ℹ️".blue());
    }
    
    Ok(())
}

/// Clear view cache
fn clear_view_cache() -> Result<(), Box<dyn std::error::Error>> {
    println!("{} Clearing view cache...", "🗑️".yellow());
    
    if clear_view_cache_internal()? {
        println!("{} View cache cleared", "✅".green());
    } else {
        println!("{} No view cache found", "ℹ️".blue());
    }
    
    Ok(())
}

/// Show cache statistics
fn show_cache_stats() -> Result<(), Box<dyn std::error::Error>> {
    println!("{} Cache Statistics", "📊".yellow().bold());
    println!();
    
    // Check various cache directories and files
    let cache_items = vec![
        ("Configuration", "src/cache/config.rs"),
        ("Routes", "src/cache/routes.rs"),
        ("Views", "target/cache/views/"),
        ("Application", "target/cache/app/"),
    ];
    
    println!("{:<15} {:<10} {}", "Cache Type".bold(), "Status".bold(), "Location".bold());
    println!("{}", "-".repeat(60));
    
    for (cache_type, path) in cache_items {
        let status = if Path::new(path).exists() {
            "✅ Cached".green()
        } else {
            "❌ Empty".red()
        };
        
        println!("{:<15} {:<10} {}", cache_type, status, path.cyan());
    }
    
    Ok(())
}

// Internal helper functions

fn clear_config_cache_internal() -> Result<bool, Box<dyn std::error::Error>> {
    let path = "src/cache/config.rs";
    if Path::new(path).exists() {
        fs::remove_file(path)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

fn clear_route_cache_internal() -> Result<bool, Box<dyn std::error::Error>> {
    let path = "src/cache/routes.rs";
    if Path::new(path).exists() {
        fs::remove_file(path)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

fn clear_view_cache_internal() -> Result<bool, Box<dyn std::error::Error>> {
    let path = "target/cache/views";
    if Path::new(path).exists() {
        fs::remove_dir_all(path)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

fn clear_app_cache_internal() -> Result<bool, Box<dyn std::error::Error>> {
    let path = "target/cache/app";
    if Path::new(path).exists() {
        fs::remove_dir_all(path)?;
        Ok(true)
    } else {
        Ok(false)
    }
}
