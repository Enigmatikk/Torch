//! About command - Show Torch information

use colored::*;

/// Show information about Torch
pub fn show_info() {
    println!("{}", "🔥 Torch Web Framework".yellow().bold());
    println!("{} {}", "Version:".bold(), env!("CARGO_PKG_VERSION"));
    println!("{} Fast & Secure Web Framework for Rust", "Description:".bold());
    println!();
    
    println!("{}", "Features:".bold());
    println!("  🚀 Compile-time route registration");
    println!("  🎨 Ember templating engine");
    println!("  ⚡ Type-safe extractors");
    println!("  🛡️ Security-first design");
    println!("  📦 Production-ready");
    println!("  🛠️ Great developer experience");
    println!();
    
    println!("{}", "Links:".bold());
    println!("  📚 Documentation: https://docs.rs/torch-web");
    println!("  🐙 GitHub: https://github.com/Enigmatikk/torch");
    println!("  🛠️ VS Code Extension: https://github.com/Enigmatikk/torch-vscode");
    println!();
    
    println!("{}", "Built with ❤️ for the Rust community".italic());
}
