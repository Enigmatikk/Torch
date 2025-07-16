use std::env;

fn main() {
    // Only show instructions when building with CLI feature
    if env::var("CARGO_FEATURE_CLI").is_ok() {
        show_installation_instructions();
    }
}

fn show_installation_instructions() {
    println!("cargo:warning=");
    println!("cargo:warning=🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥");
    println!("cargo:warning=🔥                                                          🔥");
    println!("cargo:warning=🔥                    TORCH CLI INSTALLED                   🔥");
    println!("cargo:warning=🔥                                                          🔥");
    println!("cargo:warning=🔥        Fast & Lightweight Web Framework for Rust        🔥");
    println!("cargo:warning=🔥                                                          🔥");
    println!("cargo:warning=🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥");
    println!("cargo:warning=");
    
    // Detect OS and show appropriate instructions
    let os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_else(|_| "unknown".to_string());
    
    match os.as_str() {
        "windows" => show_windows_instructions(),
        "macos" => show_macos_instructions(),
        "linux" => show_linux_instructions(),
        _ => show_generic_instructions(),
    }
    
    show_quick_start();
}

fn show_windows_instructions() {
    println!("cargo:warning=📦 Windows Installation Complete!");
    println!("cargo:warning=");
    println!("cargo:warning=The 'torch' command should now be available in your terminal.");
    println!("cargo:warning=If not found, the binary is located at:");
    println!("cargo:warning=  %USERPROFILE%\\.cargo\\bin\\torch.exe");
    println!("cargo:warning=");
    println!("cargo:warning=To add to PATH permanently:");
    println!("cargo:warning=  1. Open System Properties > Environment Variables");
    println!("cargo:warning=  2. Add %USERPROFILE%\\.cargo\\bin to your PATH");
    println!("cargo:warning=  3. Restart your terminal");
    println!("cargo:warning=");
    println!("cargo:warning=Or run this PowerShell command as Administrator:");
    println!("cargo:warning=  [Environment]::SetEnvironmentVariable('PATH', $env:PATH + ';' + $env:USERPROFILE + '\\.cargo\\bin', 'Machine')");
    println!("cargo:warning=");
}

fn show_macos_instructions() {
    println!("cargo:warning=📦 macOS Installation Complete!");
    println!("cargo:warning=");
    println!("cargo:warning=The 'torch' command should now be available in your terminal.");
    println!("cargo:warning=If not found, add Cargo's bin directory to your PATH:");
    println!("cargo:warning=");
    println!("cargo:warning=For Bash (~/.bash_profile or ~/.bashrc):");
    println!("cargo:warning=  echo 'export PATH=\"$HOME/.cargo/bin:$PATH\"' >> ~/.bash_profile");
    println!("cargo:warning=  source ~/.bash_profile");
    println!("cargo:warning=");
    println!("cargo:warning=For Zsh (~/.zshrc):");
    println!("cargo:warning=  echo 'export PATH=\"$HOME/.cargo/bin:$PATH\"' >> ~/.zshrc");
    println!("cargo:warning=  source ~/.zshrc");
    println!("cargo:warning=");
    println!("cargo:warning=For Fish (~/.config/fish/config.fish):");
    println!("cargo:warning=  echo 'set -gx PATH $HOME/.cargo/bin $PATH' >> ~/.config/fish/config.fish");
    println!("cargo:warning=");
}

fn show_linux_instructions() {
    println!("cargo:warning=📦 Linux Installation Complete!");
    println!("cargo:warning=");
    println!("cargo:warning=The 'torch' command should now be available in your terminal.");
    println!("cargo:warning=If not found, add Cargo's bin directory to your PATH:");
    println!("cargo:warning=");
    println!("cargo:warning=For Bash (~/.bashrc):");
    println!("cargo:warning=  echo 'export PATH=\"$HOME/.cargo/bin:$PATH\"' >> ~/.bashrc");
    println!("cargo:warning=  source ~/.bashrc");
    println!("cargo:warning=");
    println!("cargo:warning=For Zsh (~/.zshrc):");
    println!("cargo:warning=  echo 'export PATH=\"$HOME/.cargo/bin:$PATH\"' >> ~/.zshrc");
    println!("cargo:warning=  source ~/.zshrc");
    println!("cargo:warning=");
    println!("cargo:warning=For Fish (~/.config/fish/config.fish):");
    println!("cargo:warning=  echo 'set -gx PATH $HOME/.cargo/bin $PATH' >> ~/.config/fish/config.fish");
    println!("cargo:warning=");
    println!("cargo:warning=System-wide installation (requires sudo):");
    println!("cargo:warning=  sudo ln -sf ~/.cargo/bin/torch /usr/local/bin/torch");
    println!("cargo:warning=");
}

fn show_generic_instructions() {
    println!("cargo:warning=📦 Installation Complete!");
    println!("cargo:warning=");
    println!("cargo:warning=The 'torch' command should now be available in your terminal.");
    println!("cargo:warning=If not found, add Cargo's bin directory to your PATH:");
    println!("cargo:warning=  export PATH=\"$HOME/.cargo/bin:$PATH\"");
    println!("cargo:warning=");
}

fn show_quick_start() {
    println!("cargo:warning=🚀 Quick Start:");
    println!("cargo:warning=  torch new my-app        # Create a new Torch application");
    println!("cargo:warning=  cd my-app");
    println!("cargo:warning=  torch serve --hot       # Start development server with hot reload");
    println!("cargo:warning=");
    println!("cargo:warning=📚 More Commands:");
    println!("cargo:warning=  torch make controller   # Generate controllers, models, etc.");
    println!("cargo:warning=  torch migrate           # Run database migrations");
    println!("cargo:warning=  torch tinker            # Interactive REPL shell");
    println!("cargo:warning=  torch --help            # Show all available commands");
    println!("cargo:warning=");
    println!("cargo:warning=📖 Documentation: https://docs.rs/torch-web");
    println!("cargo:warning=🐙 GitHub: https://github.com/Enigmatikk/Torch");
    println!("cargo:warning=");
    println!("cargo:warning=🔥 Happy coding with Torch! 🔥");
    println!("cargo:warning=");
}
