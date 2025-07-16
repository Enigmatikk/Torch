//! # Torch CLI
//!
//! Laravel Artisan-inspired command-line interface for Torch applications.
//! Provides scaffolding, code generation, and development tools.

#[cfg(feature = "cli")]
pub mod commands;

#[cfg(feature = "cli")]
pub mod generators;

#[cfg(feature = "cli")]
pub mod templates;

#[cfg(feature = "cli")]
use clap::{Parser, Subcommand};
#[cfg(feature = "cli")]
use colored::*;

#[cfg(feature = "cli")]
#[derive(Parser)]
#[command(name = "torch")]
#[command(about = "🔥 Torch - Fast & Secure Web Framework for Rust")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[cfg(feature = "cli")]
#[derive(Subcommand)]
pub enum Commands {
    /// Create a new Torch application
    New {
        /// Name of the application
        name: String,
        /// Use minimal template (no examples)
        #[arg(long)]
        minimal: bool,
    },
    /// Generate code (controllers, models, etc.)
    Make {
        #[command(subcommand)]
        generator: Generator,
    },
    /// Start development server with hot reload
    Serve {
        /// Port to serve on
        #[arg(short, long, default_value = "3000")]
        port: u16,
        /// Host to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        /// Enable hot reload
        #[arg(long)]
        hot: bool,
    },
    /// Build the application for production
    Build {
        /// Build in release mode
        #[arg(long)]
        release: bool,
    },
    /// Show application information
    About,
    /// Database operations
    Db {
        #[command(subcommand)]
        operation: DbOperation,
    },
    /// Migration operations
    Migrate {
        #[command(subcommand)]
        operation: Option<MigrateOperation>,
    },
    /// Route operations
    Route {
        #[command(subcommand)]
        operation: RouteOperation,
    },
    /// Cache operations
    Cache {
        #[command(subcommand)]
        operation: CacheOperation,
    },
    /// Configuration operations
    Config {
        #[command(subcommand)]
        operation: ConfigOperation,
    },
    /// View operations
    View {
        #[command(subcommand)]
        operation: ViewOperation,
    },
    /// Queue operations
    Queue {
        #[command(subcommand)]
        operation: QueueOperation,
    },
    /// Testing operations
    Test {
        /// Run specific test
        #[arg(long)]
        filter: Option<String>,
        /// Run unit tests only
        #[arg(long)]
        unit: bool,
        /// Run integration tests only
        #[arg(long)]
        integration: bool,
    },
    /// Put application in maintenance mode
    Down {
        /// Secret for bypassing maintenance mode
        #[arg(long)]
        secret: Option<String>,
        /// Custom maintenance view
        #[arg(long)]
        render: Option<String>,
    },
    /// Bring application out of maintenance mode
    Up,
    /// Optimize application for production
    Optimize {
        /// Clear optimization caches
        #[arg(long)]
        clear: bool,
    },
    /// Interactive REPL for Torch
    Tinker,
    /// Schedule operations
    Schedule {
        #[command(subcommand)]
        operation: ScheduleOperation,
    },
}

#[cfg(feature = "cli")]
#[derive(Subcommand)]
pub enum Generator {
    /// Generate a new controller
    Controller {
        /// Controller name (e.g., UserController)
        name: String,
        /// Generate with CRUD methods
        #[arg(long)]
        resource: bool,
        /// Generate API controller
        #[arg(long)]
        api: bool,
    },
    /// Generate a new model
    Model {
        /// Model name (e.g., User)
        name: String,
        /// Generate with migration
        #[arg(short, long)]
        migration: bool,
        /// Generate with factory
        #[arg(short, long)]
        factory: bool,
        /// Generate with seeder
        #[arg(short, long)]
        seeder: bool,
        /// Generate with policy
        #[arg(short, long)]
        policy: bool,
    },
    /// Generate a new middleware
    Middleware {
        /// Middleware name (e.g., AuthMiddleware)
        name: String,
    },
    /// Generate a new Ember template
    Template {
        /// Template name (e.g., users/index)
        name: String,
        /// Extend a layout
        #[arg(long)]
        layout: Option<String>,
    },
    /// Generate a new migration
    Migration {
        /// Migration name (e.g., create_users_table)
        name: String,
        /// Create table migration
        #[arg(long)]
        create: Option<String>,
        /// Modify table migration
        #[arg(long)]
        table: Option<String>,
    },
    /// Generate a new seeder
    Seeder {
        /// Seeder name (e.g., UserSeeder)
        name: String,
    },
    /// Generate a new factory
    Factory {
        /// Factory name (e.g., UserFactory)
        name: String,
        /// Model to create factory for
        #[arg(long)]
        model: Option<String>,
    },
    /// Generate a new policy
    Policy {
        /// Policy name (e.g., UserPolicy)
        name: String,
        /// Model to create policy for
        #[arg(long)]
        model: Option<String>,
    },
    /// Generate a new event
    Event {
        /// Event name (e.g., UserRegistered)
        name: String,
    },
    /// Generate a new listener
    Listener {
        /// Listener name (e.g., SendWelcomeEmail)
        name: String,
        /// Event to listen for
        #[arg(long)]
        event: Option<String>,
    },
    /// Generate a new job
    Job {
        /// Job name (e.g., ProcessPayment)
        name: String,
        /// Make job synchronous
        #[arg(long)]
        sync: bool,
    },
    /// Generate a new notification
    Notification {
        /// Notification name (e.g., WelcomeNotification)
        name: String,
    },
    /// Generate a new test
    Test {
        /// Test name (e.g., UserTest)
        name: String,
        /// Generate unit test
        #[arg(long)]
        unit: bool,
    },
    /// Generate a new command
    Command {
        /// Command name (e.g., SendEmails)
        name: String,
    },
}

#[cfg(feature = "cli")]
#[derive(Subcommand)]
pub enum DbOperation {
    /// Seed the database with records
    Seed {
        /// Specific seeder class to run
        #[arg(long)]
        class: Option<String>,
    },
    /// Drop all tables, views, and types
    Wipe {
        /// Force the operation without confirmation
        #[arg(long)]
        force: bool,
    },
    /// Show database connection status
    Status,
}

#[cfg(feature = "cli")]
#[derive(Subcommand)]
pub enum MigrateOperation {
    /// Roll back the last migration batch
    Rollback {
        /// Number of batches to rollback
        #[arg(long)]
        step: Option<u32>,
    },
    /// Roll back all migrations
    Reset {
        /// Force the operation without confirmation
        #[arg(long)]
        force: bool,
    },
    /// Drop all tables and re-run all migrations
    Fresh {
        /// Also run seeders
        #[arg(long)]
        seed: bool,
    },
    /// Show the status of each migration
    Status,
    /// Install migration repository
    Install,
}

#[cfg(feature = "cli")]
#[derive(Subcommand)]
pub enum RouteOperation {
    /// List all registered routes
    List {
        /// Filter by method
        #[arg(long)]
        method: Option<String>,
        /// Filter by name
        #[arg(long)]
        name: Option<String>,
    },
    /// Cache routes for faster registration
    Cache,
    /// Clear the route cache
    Clear,
}

#[cfg(feature = "cli")]
#[derive(Subcommand)]
pub enum CacheOperation {
    /// Clear all caches
    Clear,
    /// Clear configuration cache
    Config,
    /// Clear route cache
    Route,
    /// Clear view cache
    View,
    /// Show cache statistics
    Stats,
}

#[cfg(feature = "cli")]
#[derive(Subcommand)]
pub enum ConfigOperation {
    /// Cache configuration for better performance
    Cache,
    /// Clear the configuration cache
    Clear,
    /// Show current configuration
    Show {
        /// Specific config key to show
        key: Option<String>,
    },
}

#[cfg(feature = "cli")]
#[derive(Subcommand)]
pub enum ViewOperation {
    /// Compile and cache all views
    Cache,
    /// Clear all compiled view files
    Clear,
}

#[cfg(feature = "cli")]
#[derive(Subcommand)]
pub enum QueueOperation {
    /// Start processing jobs
    Work {
        /// Queue to process
        #[arg(long)]
        queue: Option<String>,
        /// Number of seconds to sleep when no job is available
        #[arg(long)]
        sleep: Option<u64>,
        /// Number of jobs to process before stopping
        #[arg(long)]
        max_jobs: Option<u32>,
    },
    /// Restart queue workers after current jobs
    Restart,
    /// Clear all failed jobs
    Clear,
    /// List failed jobs
    Failed,
    /// Retry failed jobs
    Retry {
        /// Specific job ID to retry
        id: Option<String>,
    },
}

#[cfg(feature = "cli")]
#[derive(Subcommand)]
pub enum ScheduleOperation {
    /// Run scheduled tasks
    Run,
    /// Clear schedule cache
    ClearCache,
    /// List scheduled tasks
    List,
}

/// Main CLI entry point
#[cfg(feature = "cli")]
pub fn run() {
    let cli = Cli::parse();

    // Show installation success message on first run or help
    show_installation_success_if_needed(&cli);

    if let Err(e) = run_command(cli.command) {
        eprintln!("{} {}", "❌ Error:".red().bold(), e);
        std::process::exit(1);
    }
}

#[cfg(feature = "cli")]
fn run_command(command: Commands) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Commands::New { name, minimal } => {
            commands::new::create_project(&name, minimal)?;
        }
        Commands::Make { generator } => {
            commands::make::generate(generator)?;
        }
        Commands::Serve { port, host, hot } => {
            commands::serve::start_server(&host, port, hot)?;
        }
        Commands::Build { release } => {
            commands::build::build_project(release)?;
        }
        Commands::About => {
            commands::about::show_info();
        }
        Commands::Db { operation } => {
            commands::db::handle_operation(operation)?;
        }
        Commands::Migrate { operation } => {
            commands::migrate::handle_operation(operation)?;
        }
        Commands::Route { operation } => {
            commands::route::handle_operation(operation)?;
        }
        Commands::Cache { operation } => {
            commands::cache::handle_operation(operation)?;
        }
        Commands::Config { operation } => {
            commands::config::handle_operation(operation)?;
        }
        Commands::View { operation } => {
            commands::view::handle_operation(operation)?;
        }
        Commands::Queue { operation } => {
            commands::queue::handle_operation(operation)?;
        }
        Commands::Test { filter, unit, integration } => {
            commands::test::run_tests(filter, unit, integration)?;
        }
        Commands::Down { secret, render } => {
            commands::maintenance::down(secret, render)?;
        }
        Commands::Up => {
            commands::maintenance::up()?;
        }
        Commands::Optimize { clear } => {
            commands::optimize::handle(clear)?;
        }
        Commands::Tinker => {
            commands::tinker::start_repl()?;
        }
        Commands::Schedule { operation } => {
            commands::schedule::handle_operation(operation)?;
        }
    }
    Ok(())
}

#[cfg(not(feature = "cli"))]
pub fn run() {
    eprintln!("❌ CLI feature not enabled. Install with: cargo install torch-web --features cli");
    std::process::exit(1);
}

/// Show installation success message if this appears to be first run or help
#[cfg(feature = "cli")]
fn show_installation_success_if_needed(cli: &Cli) {
    // Show on help, version, or about commands
    match &cli.command {
        Commands::About => {
            show_installation_success();
        }
        _ => {
            // Check if this might be first run by looking for common first-time commands
            let args: Vec<String> = std::env::args().collect();
            if args.len() == 1 || args.contains(&"--help".to_string()) || args.contains(&"--version".to_string()) {
                show_installation_success();
            }
        }
    }
}

/// Show installation success and PATH instructions
#[cfg(feature = "cli")]
fn show_installation_success() {
    println!("{}", "🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥".red());
    println!("{}", "🔥                                                          🔥".red());
    println!("{}", "🔥                    TORCH CLI READY                      🔥".red());
    println!("{}", "🔥                                                          🔥".red());
    println!("{}", "🔥        Fast & Lightweight Web Framework for Rust        🔥".red());
    println!("{}", "🔥                                                          🔥".red());
    println!("{}", "🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥".red());
    println!();

    // Show PATH instructions if needed
    if !is_in_path() {
        show_path_instructions();
    }

    println!("{}", "🚀 Quick Start:".blue().bold());
    println!("  {} torch new my-app        # Create a new Torch application", "🔥".red());
    println!("  {} cd my-app", "📁".blue());
    println!("  {} torch serve --hot       # Start development server with hot reload", "⚡".yellow());
    println!();
    println!("{}", "📚 More Commands:".blue().bold());
    println!("  {} torch make controller   # Generate controllers, models, etc.", "🏗️".green());
    println!("  {} torch migrate           # Run database migrations", "🗄️".cyan());
    println!("  {} torch tinker            # Interactive REPL shell", "🔧".magenta());
    println!("  {} torch --help            # Show all available commands", "❓".yellow());
    println!();
    println!("{} Documentation: {}", "📖".blue(), "https://docs.rs/torch-web".cyan());
    println!("{} GitHub: {}", "🐙".blue(), "https://github.com/Enigmatikk/Torch".cyan());
    println!();
}

/// Check if torch is in PATH
#[cfg(feature = "cli")]
fn is_in_path() -> bool {
    std::env::var("PATH")
        .unwrap_or_default()
        .split(if cfg!(windows) { ';' } else { ':' })
        .any(|path| {
            let torch_path = if cfg!(windows) {
                format!("{}\\torch.exe", path)
            } else {
                format!("{}/torch", path)
            };
            std::path::Path::new(&torch_path).exists()
        })
}

/// Show OS-specific PATH instructions
#[cfg(feature = "cli")]
fn show_path_instructions() {
    println!("{}", "⚠️  Torch CLI may not be in your PATH".yellow().bold());
    println!();

    #[cfg(windows)]
    show_windows_path_instructions();

    #[cfg(target_os = "macos")]
    show_macos_path_instructions();

    #[cfg(not(any(windows, target_os = "macos")))]
    show_linux_path_instructions();

    println!();
}

#[cfg(all(feature = "cli", windows))]
fn show_windows_path_instructions() {
    println!("{}", "🪟 Windows PATH Setup:".blue().bold());
    println!("  {} Add to current session:", "1️⃣".blue());
    println!("    {}", "$env:PATH += \";$env:USERPROFILE\\.cargo\\bin\"".cyan());
    println!();
    println!("  {} Permanent setup (PowerShell as Admin):", "2️⃣".blue());
    println!("    {}", "[Environment]::SetEnvironmentVariable('PATH', $env:PATH + ';' + $env:USERPROFILE + '\\.cargo\\bin', 'Machine')".cyan());
    println!();
    println!("  {} Manual setup:", "3️⃣".blue());
    println!("    {} Open System Properties → Environment Variables", "•".yellow());
    println!("    {} Add %USERPROFILE%\\.cargo\\bin to PATH", "•".yellow());
    println!("    {} Restart terminal", "•".yellow());
}

#[cfg(all(feature = "cli", target_os = "macos"))]
fn show_macos_path_instructions() {
    println!("{}", "🍎 macOS PATH Setup:".blue().bold());
    println!("  {} For Zsh (default):", "1️⃣".blue());
    println!("    {}", "echo 'export PATH=\"$HOME/.cargo/bin:$PATH\"' >> ~/.zshrc".cyan());
    println!("    {}", "source ~/.zshrc".cyan());
    println!();
    println!("  {} For Bash:", "2️⃣".blue());
    println!("    {}", "echo 'export PATH=\"$HOME/.cargo/bin:$PATH\"' >> ~/.bash_profile".cyan());
    println!("    {}", "source ~/.bash_profile".cyan());
    println!();
    println!("  {} For Fish:", "3️⃣".blue());
    println!("    {}", "echo 'set -gx PATH $HOME/.cargo/bin $PATH' >> ~/.config/fish/config.fish".cyan());
}

#[cfg(all(feature = "cli", not(any(windows, target_os = "macos"))))]
fn show_linux_path_instructions() {
    println!("{}", "🐧 Linux PATH Setup:".blue().bold());
    println!("  {} For Bash:", "1️⃣".blue());
    println!("    {}", "echo 'export PATH=\"$HOME/.cargo/bin:$PATH\"' >> ~/.bashrc".cyan());
    println!("    {}", "source ~/.bashrc".cyan());
    println!();
    println!("  {} For Zsh:", "2️⃣".blue());
    println!("    {}", "echo 'export PATH=\"$HOME/.cargo/bin:$PATH\"' >> ~/.zshrc".cyan());
    println!("    {}", "source ~/.zshrc".cyan());
    println!();
    println!("  {} System-wide (requires sudo):", "3️⃣".blue());
    println!("    {}", "sudo ln -sf ~/.cargo/bin/torch /usr/local/bin/torch".cyan());
}
