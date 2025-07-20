//! Development server command

use colored::*;
use std::process::{Command, Child, Stdio};
use std::path::Path;
use std::time::Duration;
use std::thread;
use std::sync::mpsc;
use walkdir::WalkDir;

/// Start development server
pub fn start_server(host: &str, port: u16, hot: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("{} Starting Torch development server...", "🔥".yellow());
    println!("{} Server will be available at: {}", "📡".blue(), format!("http://{}:{}", host, port).cyan().underline());

    if hot {
        println!("{} Hot reload enabled - watching for file changes", "🔄".green());
        start_hot_reload_server(host, port)?;
    } else {
        println!("{} Hot reload disabled", "📝".blue());
        start_normal_server()?;
    }

    Ok(())
}

/// Start server with hot reload functionality
fn start_hot_reload_server(_host: &str, _port: u16) -> Result<(), Box<dyn std::error::Error>> {
    println!("{} Press Ctrl+C to stop", "💡".yellow());
    println!();

    let (tx, rx) = mpsc::channel();

    // Start file watcher in a separate thread
    let watcher_tx = tx.clone();
    thread::spawn(move || {
        if let Err(e) = watch_files(watcher_tx) {
            eprintln!("{} File watcher error: {}", "❌".red(), e);
        }
    });

    let mut server_process: Option<Child> = None;

    // Start initial server
    server_process = Some(start_server_process()?);
    println!("{} Server started successfully", "✅".green());

    // Listen for file changes
    loop {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(file_path) => {
                println!("{} File changed: {}", "🔄".yellow(), file_path.cyan());

                // Kill existing server
                if let Some(mut process) = server_process.take() {
                    println!("{} Stopping server...", "🛑".yellow());
                    let _ = process.kill();
                    let _ = process.wait();
                }

                // Wait a bit for file operations to complete
                thread::sleep(Duration::from_millis(500));

                // Rebuild and restart server
                println!("{} Rebuilding application...", "🔨".blue());
                if rebuild_application().is_ok() {
                    server_process = Some(start_server_process()?);
                    println!("{} Server restarted successfully", "✅".green());
                } else {
                    println!("{} Build failed - waiting for next change", "❌".red());
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // Check if server process is still running
                if let Some(ref mut process) = server_process {
                    match process.try_wait() {
                        Ok(Some(status)) => {
                            println!("{} Server process exited with status: {}", "⚠️".yellow(), status);
                            break;
                        }
                        Ok(None) => {
                            // Process is still running
                        }
                        Err(e) => {
                            println!("{} Error checking server process: {}", "❌".red(), e);
                            break;
                        }
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                println!("{} File watcher disconnected", "⚠️".yellow());
                break;
            }
        }
    }

    // Clean up server process
    if let Some(mut process) = server_process {
        let _ = process.kill();
        let _ = process.wait();
    }

    Ok(())
}

/// Start normal server without hot reload
fn start_normal_server() -> Result<(), Box<dyn std::error::Error>> {
    println!("{} Press Ctrl+C to stop", "💡".yellow());
    println!();

    let mut cmd = Command::new("cargo");

    // Check if we're in a Torch project directory
    if Path::new("Cargo.toml").exists() {
        // Try to run the server binary, fallback to default main
        cmd.arg("run").arg("--bin").arg("server");

        // If that fails, try just cargo run
        let status = cmd.status();
        if status.is_err() || !status.unwrap().success() {
            let mut fallback_cmd = Command::new("cargo");
            fallback_cmd.arg("run");
            let fallback_status = fallback_cmd.status()?;

            if !fallback_status.success() {
                return Err("Failed to start server. Make sure you're in a Torch project directory.".into());
            }
        }
    } else {
        return Err("No Cargo.toml found. Make sure you're in a Rust project directory.".into());
    }

    Ok(())
}

/// Start server process for hot reload
fn start_server_process() -> Result<Child, Box<dyn std::error::Error>> {
    let mut cmd = Command::new("cargo");

    // Check if we're in a Torch project directory
    if Path::new("Cargo.toml").exists() {
        // Try to run the server binary first
        cmd.arg("run")
           .arg("--bin")
           .arg("server")
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());

        // Try to spawn the process
        match cmd.spawn() {
            Ok(child) => Ok(child),
            Err(_) => {
                // Fallback to default cargo run
                let mut fallback_cmd = Command::new("cargo");
                fallback_cmd.arg("run")
                           .stdout(Stdio::piped())
                           .stderr(Stdio::piped());

                fallback_cmd.spawn().map_err(|e| {
                    format!("Failed to start server: {}. Make sure you're in a Torch project directory.", e).into()
                })
            }
        }
    } else {
        Err("No Cargo.toml found. Make sure you're in a Rust project directory.".into())
    }
}

/// Rebuild the application
fn rebuild_application() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new("cargo");
    cmd.arg("build");

    let output = cmd.output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("{} Build errors:", "❌".red());
        println!("{}", stderr);
        return Err("Build failed".into());
    }

    Ok(())
}

/// Watch files for changes
fn watch_files(tx: mpsc::Sender<String>) -> Result<(), Box<dyn std::error::Error>> {
    let watch_paths = vec!["src", "templates", "static", "Cargo.toml"];
    let mut last_modified = std::collections::HashMap::new();

    loop {
        for watch_path in &watch_paths {
            if !Path::new(watch_path).exists() {
                continue;
            }

            for entry in WalkDir::new(watch_path) {
                let entry = entry?;
                let path = entry.path();

                // Skip directories and hidden files
                if path.is_dir() || path.file_name().unwrap_or_default().to_string_lossy().starts_with('.') {
                    continue;
                }

                // Only watch relevant file types
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy();
                    if !matches!(ext_str.as_ref(), "rs" | "toml" | "ember" | "html" | "css" | "js") {
                        continue;
                    }
                }

                if let Ok(metadata) = path.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        let path_str = path.to_string_lossy().to_string();

                        if let Some(&last_mod) = last_modified.get(&path_str) {
                            if modified > last_mod {
                                last_modified.insert(path_str.clone(), modified);
                                if tx.send(path_str).is_err() {
                                    return Ok(()); // Channel closed
                                }
                            }
                        } else {
                            last_modified.insert(path_str, modified);
                        }
                    }
                }
            }
        }

        thread::sleep(Duration::from_millis(500));
    }
}
