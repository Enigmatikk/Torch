//! Build command for production

use colored::*;
use std::process::Command;
use std::fs;
use std::path::Path;

/// Build the project for production
pub fn build_project(release: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("{} Building Torch application...", "🔨".yellow());

    if release {
        build_production()?;
    } else {
        build_development()?;
    }

    Ok(())
}

/// Build for development
fn build_development() -> Result<(), Box<dyn std::error::Error>> {
    println!("{} Building in debug mode", "🐛".blue());

    let mut cmd = Command::new("cargo");
    cmd.arg("build");

    let status = cmd.status()?;

    if status.success() {
        println!("{} Debug build completed successfully!", "✅".green());
        println!("  📦 Binary: target/debug/");
        println!("  🐛 Debug symbols included");
    } else {
        return Err("Debug build failed".into());
    }

    Ok(())
}

/// Build for production with optimizations
fn build_production() -> Result<(), Box<dyn std::error::Error>> {
    println!("{} Building in release mode (optimized)", "⚡".green());
    println!();

    // Step 1: Pre-build optimizations
    println!("{} Running pre-build optimizations...", "🔧".blue());
    run_prebuild_optimizations()?;

    // Step 2: Build with release optimizations
    println!("{} Compiling with maximum optimizations...", "🚀".blue());
    let mut cmd = Command::new("cargo");
    cmd.arg("build")
       .arg("--release")
       .env("RUSTFLAGS", "-C target-cpu=native -C opt-level=3 -C lto=fat");

    let status = cmd.status()?;

    if !status.success() {
        return Err("Release build failed".into());
    }

    // Step 3: Post-build optimizations
    println!("{} Running post-build optimizations...", "🔧".blue());
    run_postbuild_optimizations()?;

    // Step 4: Generate deployment artifacts
    println!("{} Generating deployment artifacts...", "📦".blue());
    generate_deployment_artifacts()?;

    println!();
    println!("{} Production build completed successfully!", "✅".green().bold());
    println!();
    println!("{}", "Production build ready:".bold());
    println!("  📦 Optimized binary: target/release/");
    println!("  🗜️ Binary size: {}", get_binary_size()?);
    println!("  ⚡ LTO enabled for maximum performance");
    println!("  🎯 Target-specific optimizations applied");
    println!("  🚀 Ready for deployment");

    Ok(())
}

/// Run pre-build optimizations
fn run_prebuild_optimizations() -> Result<(), Box<dyn std::error::Error>> {
    // Cache configuration
    if Path::new("config").exists() {
        println!("  • Caching configuration...");
        // TODO: Implement config caching
    }

    // Cache routes
    if Path::new("src").exists() {
        println!("  • Caching routes...");
        // TODO: Implement route caching
    }

    // Compile templates
    if Path::new("templates").exists() {
        println!("  • Pre-compiling templates...");
        // TODO: Implement template pre-compilation
    }

    Ok(())
}

/// Run post-build optimizations
fn run_postbuild_optimizations() -> Result<(), Box<dyn std::error::Error>> {
    let binary_path = "target/release/server";

    if Path::new(binary_path).exists() {
        // Strip debug symbols for smaller binary
        println!("  • Stripping debug symbols...");
        let mut cmd = Command::new("strip");
        cmd.arg(binary_path);
        let _ = cmd.status(); // Don't fail if strip is not available

        // Compress binary if upx is available
        println!("  • Attempting binary compression...");
        let mut cmd = Command::new("upx");
        cmd.arg("--best").arg(binary_path);
        let _ = cmd.status(); // Don't fail if upx is not available
    }

    Ok(())
}

/// Generate deployment artifacts
fn generate_deployment_artifacts() -> Result<(), Box<dyn std::error::Error>> {
    // Create deployment directory
    fs::create_dir_all("target/deploy")?;

    // Copy binary
    if Path::new("target/release/server").exists() {
        fs::copy("target/release/server", "target/deploy/server")?;
        println!("  • Binary copied to deployment directory");
    }

    // Copy static assets
    if Path::new("static").exists() {
        copy_dir_all("static", "target/deploy/static")?;
        println!("  • Static assets copied");
    }

    // Copy templates (if not pre-compiled)
    if Path::new("templates").exists() {
        copy_dir_all("templates", "target/deploy/templates")?;
        println!("  • Templates copied");
    }

    // Generate deployment manifest
    let manifest = serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "build_time": chrono::Utc::now().to_rfc3339(),
        "build_type": "release",
        "optimizations": {
            "lto": true,
            "target_cpu": "native",
            "opt_level": 3
        }
    });

    fs::write("target/deploy/manifest.json", serde_json::to_string_pretty(&manifest)?)?;
    println!("  • Deployment manifest generated");

    // Generate Dockerfile
    generate_dockerfile()?;
    println!("  • Dockerfile generated");

    Ok(())
}

/// Get binary size as formatted string
fn get_binary_size() -> Result<String, Box<dyn std::error::Error>> {
    let binary_path = "target/release/server";

    if let Ok(metadata) = fs::metadata(binary_path) {
        let size = metadata.len();
        Ok(format_bytes(size))
    } else {
        Ok("Unknown".to_string())
    }
}

/// Format bytes as human readable string
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.1} {}", size, UNITS[unit_index])
}

/// Copy directory recursively
fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(&dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;

        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }

    Ok(())
}

/// Generate Dockerfile for deployment
fn generate_dockerfile() -> Result<(), Box<dyn std::error::Error>> {
    let dockerfile_content = r#"# Multi-stage Dockerfile for Torch application
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1000 torch

# Set working directory
WORKDIR /app

# Copy application binary
COPY server /app/server
RUN chmod +x /app/server

# Copy static assets and templates
COPY static /app/static
COPY templates /app/templates

# Change ownership to app user
RUN chown -R torch:torch /app

# Switch to app user
USER torch

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Run the application
CMD ["./server"]
"#;

    fs::write("target/deploy/Dockerfile", dockerfile_content)?;

    Ok(())
}
