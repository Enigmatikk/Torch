//! Torch CLI - Command-line interface for the Torch web framework
//!
//! This binary provides Laravel Artisan-like functionality for Torch applications.

#[cfg(feature = "cli")]
fn main() {
    torch_web::cli::run();
}

#[cfg(not(feature = "cli"))]
fn main() {
    eprintln!("❌ CLI feature not enabled. Install with: cargo install torch-web --features cli");
    std::process::exit(1);
}
