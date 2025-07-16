# 🔥 Torch CLI Installation Guide

## Quick Installation

```bash
cargo install torch-web --features cli
```

## Platform-Specific Setup

After installation, you may need to add the Torch CLI to your system PATH.

### 🪟 Windows

The `torch` command should be automatically available. If not found:

**Option 1: PowerShell (Recommended)**
```powershell
# Add to current user PATH
$env:PATH += ";$env:USERPROFILE\.cargo\bin"

# Make permanent (run as Administrator)
[Environment]::SetEnvironmentVariable('PATH', $env:PATH + ';' + $env:USERPROFILE + '\.cargo\bin', 'Machine')
```

**Option 2: Manual Setup**
1. Open System Properties → Environment Variables
2. Add `%USERPROFILE%\.cargo\bin` to your PATH
3. Restart your terminal

**Option 3: Command Prompt**
```cmd
set PATH=%PATH%;%USERPROFILE%\.cargo\bin
```

### 🍎 macOS

**Bash** (`~/.bash_profile` or `~/.bashrc`):
```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bash_profile
source ~/.bash_profile
```

**Zsh** (`~/.zshrc`) - Default on macOS Catalina+:
```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

**Fish** (`~/.config/fish/config.fish`):
```bash
echo 'set -gx PATH $HOME/.cargo/bin $PATH' >> ~/.config/fish/config.fish
```

### 🐧 Linux

**Bash** (`~/.bashrc`):
```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

**Zsh** (`~/.zshrc`):
```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

**Fish** (`~/.config/fish/config.fish`):
```bash
echo 'set -gx PATH $HOME/.cargo/bin $PATH' >> ~/.config/fish/config.fish
```

**System-wide installation** (requires sudo):
```bash
sudo ln -sf ~/.cargo/bin/torch /usr/local/bin/torch
```

## Verify Installation

Test that the CLI is working:

```bash
# Check if torch is available
torch --version

# Show help
torch --help

# Create a test project
torch new test-app
cd test-app
torch serve
```

## Troubleshooting

### Command Not Found

If you get `torch: command not found`:

1. **Check if binary exists:**
   ```bash
   ls ~/.cargo/bin/torch*
   ```

2. **Manually add to current session:**
   ```bash
   export PATH="$HOME/.cargo/bin:$PATH"
   ```

3. **Check your shell:**
   ```bash
   echo $SHELL
   ```
   Then use the appropriate config file for your shell.

### Permission Issues (Linux/macOS)

If you get permission errors:

```bash
chmod +x ~/.cargo/bin/torch
```

### Windows PATH Issues

If PATH updates don't work:

1. Close all terminal windows
2. Restart your terminal application
3. Or log out and log back in

### Alternative: Use Full Path

You can always use the full path to the binary:

**Windows:**
```cmd
%USERPROFILE%\.cargo\bin\torch.exe --help
```

**Linux/macOS:**
```bash
~/.cargo/bin/torch --help
```

## Development Installation

If you're developing Torch itself:

```bash
# Clone the repository
git clone https://github.com/Enigmatikk/Torch.git
cd Torch

# Install from source
cargo install --path . --features cli --force

# Or build and run directly
cargo build --features cli --release
./target/release/torch --help
```

## Uninstallation

To remove the Torch CLI:

```bash
cargo uninstall torch-web
```

Then remove the PATH entries you added to your shell configuration files.

## Next Steps

Once installed, check out:

- [CLI Documentation](cli.md) - Complete command reference
- [CLI Tutorial](cli-tutorial.md) - Step-by-step guide
- [Quick Start](#quick-start) - Get up and running fast

## Quick Start

```bash
# Create a new Torch application
torch new my-blog
cd my-blog

# Generate a model with migration
torch make model Post --migration

# Generate a controller
torch make controller PostController --resource

# Start development server with hot reload
torch serve --hot

# Open another terminal and explore
torch tinker
```

## Getting Help

- **CLI Help:** `torch --help` or `torch <command> --help`
- **Documentation:** [docs.rs/torch-web](https://docs.rs/torch-web)
- **GitHub Issues:** [github.com/Enigmatikk/Torch/issues](https://github.com/Enigmatikk/Torch/issues)
- **Discussions:** [github.com/Enigmatikk/Torch/discussions](https://github.com/Enigmatikk/Torch/discussions)
