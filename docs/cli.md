# Torch CLI Documentation

The Torch CLI provides Laravel Artisan-like functionality for Torch applications, offering powerful code generation, project management, and development tools.

## Installation

The CLI is available as a feature flag in the main torch-web crate:

```bash
cargo install torch-web --features cli
```

Or add it to your `Cargo.toml`:

```toml
[dependencies]
torch-web = { version = "0.2.6", features = ["cli"] }
```

## Getting Started

Once installed, you can use the `torch` command:

```bash
torch --help
```

## Commands Overview

### Project Management

#### `torch new <name>`
Create a new Torch application.

```bash
# Create a new application
torch new my-app

# Create with minimal template
torch new my-app --minimal
```

**Options:**
- `--minimal` - Create a minimal project without examples and additional features

#### `torch serve`
Start the development server with optional hot reload.

```bash
# Start development server
torch serve

# Start with hot reload
torch serve --hot

# Specify port and host
torch serve --port 8080 --host 0.0.0.0
```

**Options:**
- `--port, -p` - Port to serve on (default: 3000)
- `--host` - Host to bind to (default: 127.0.0.1)
- `--hot` - Enable hot reload for development

#### `torch build`
Build the application for production.

```bash
# Debug build
torch build

# Release build with optimizations
torch build --release
```

**Options:**
- `--release` - Build in release mode with full optimizations

### Code Generation

#### `torch make controller <name>`
Generate a new controller.

```bash
# Basic controller
torch make controller UserController

# Resource controller with CRUD methods
torch make controller UserController --resource

# API controller
torch make controller UserController --resource --api
```

**Options:**
- `--resource` - Generate with CRUD methods
- `--api` - Generate API controller

#### `torch make model <name>`
Generate a new model.

```bash
# Basic model
torch make model User

# Model with migration
torch make model User --migration

# Model with factory and seeder
torch make model User --migration --factory --seeder --policy
```

**Options:**
- `--migration, -m` - Generate with migration
- `--factory, -f` - Generate with factory
- `--seeder, -s` - Generate with seeder
- `--policy, -p` - Generate with policy

#### `torch make middleware <name>`
Generate a new middleware.

```bash
torch make middleware AuthMiddleware
```

#### `torch make template <name>`
Generate a new Ember template.

```bash
# Basic template
torch make template users/index

# Template with custom layout
torch make template users/show --layout admin
```

**Options:**
- `--layout` - Specify layout to extend

#### `torch make migration <name>`
Generate a new migration.

```bash
# Basic migration
torch make migration add_email_to_users

# Create table migration
torch make migration create_users_table --create users

# Modify table migration
torch make migration add_index_to_users --table users
```

**Options:**
- `--create` - Create table migration
- `--table` - Modify table migration

#### `torch make seeder <name>`
Generate a new seeder.

```bash
torch make seeder UserSeeder
```

#### `torch make factory <name>`
Generate a new factory.

```bash
# Basic factory
torch make factory UserFactory

# Factory for specific model
torch make factory UserFactory --model User
```

**Options:**
- `--model` - Specify model for factory

#### `torch make policy <name>`
Generate a new policy.

```bash
# Basic policy
torch make policy UserPolicy

# Policy for specific model
torch make policy UserPolicy --model User
```

**Options:**
- `--model` - Specify model for policy

#### `torch make event <name>`
Generate a new event.

```bash
torch make event UserRegistered
```

#### `torch make listener <name>`
Generate a new listener.

```bash
# Basic listener
torch make listener SendWelcomeEmail

# Listener for specific event
torch make listener SendWelcomeEmail --event UserRegistered
```

**Options:**
- `--event` - Specify event to listen for

#### `torch make job <name>`
Generate a new job.

```bash
# Asynchronous job
torch make job ProcessPayment

# Synchronous job
torch make job ProcessPayment --sync
```

**Options:**
- `--sync` - Make job synchronous

#### `torch make notification <name>`
Generate a new notification.

```bash
torch make notification WelcomeNotification
```

#### `torch make test <name>`
Generate a new test.

```bash
# Integration test
torch make test UserTest

# Unit test
torch make test UserTest --unit
```

**Options:**
- `--unit` - Generate unit test

#### `torch make command <name>`
Generate a new CLI command.

```bash
torch make command SendEmails
```

### Database Operations

#### `torch migrate`
Run database migrations.

```bash
# Run all pending migrations
torch migrate

# Rollback last migration batch
torch migrate rollback

# Rollback specific number of batches
torch migrate rollback --step 2

# Reset all migrations
torch migrate reset --force

# Fresh migration (drop all tables and re-run)
torch migrate fresh

# Fresh migration with seeding
torch migrate fresh --seed

# Show migration status
torch migrate status

# Install migration repository
torch migrate install
```

#### `torch db`
Database operations.

```bash
# Seed database
torch db seed

# Seed specific seeder
torch db seed --class UserSeeder

# Wipe database
torch db wipe --force

# Show database status
torch db status
```

### Cache Management

#### `torch cache`
Cache operations.

```bash
# Clear all caches
torch cache clear

# Clear specific cache
torch cache config
torch cache route
torch cache view

# Show cache statistics
torch cache stats
```

### Configuration

#### `torch config`
Configuration operations.

```bash
# Cache configuration
torch config cache

# Clear configuration cache
torch config clear

# Show configuration
torch config show

# Show specific config key
torch config show app.name
```

### Route Management

#### `torch route`
Route operations.

```bash
# List all routes
torch route list

# Filter routes by method
torch route list --method GET

# Filter routes by name
torch route list --name users

# Cache routes
torch route cache

# Clear route cache
torch route clear
```

### View Management

#### `torch view`
View operations.

```bash
# Compile and cache views
torch view cache

# Clear view cache
torch view clear
```

### Queue Management

#### `torch queue`
Queue operations.

```bash
# Start queue worker
torch queue work

# Work specific queue
torch queue work --queue emails

# Restart queue workers
torch queue restart

# Clear failed jobs
torch queue clear

# List failed jobs
torch queue failed

# Retry failed jobs
torch queue retry

# Retry specific job
torch queue retry 123
```

### Testing

#### `torch test`
Run tests.

```bash
# Run all tests
torch test

# Run with filter
torch test --filter user

# Run unit tests only
torch test --unit

# Run integration tests only
torch test --integration
```

### Maintenance Mode

#### `torch down`
Put application in maintenance mode.

```bash
# Enable maintenance mode
torch down

# With bypass secret
torch down --secret mySecret123

# With custom view
torch down --render maintenance
```

#### `torch up`
Bring application out of maintenance mode.

```bash
torch up
```

### Optimization

#### `torch optimize`
Optimize application for production.

```bash
# Run all optimizations
torch optimize

# Clear optimization caches
torch optimize --clear
```

### Interactive Shell

#### `torch tinker`
Start interactive REPL.

```bash
torch tinker
```

**Available commands in tinker:**
- `help` - Show help
- `app` - Show application info
- `routes` - Show routes
- `config` - Show configuration
- `models` - Show models
- `db` - Show database info
- `cache` - Show cache info
- `vars` - Show variables
- `history` - Show command history
- `clear` - Clear screen
- `exit` - Exit tinker

### Schedule Management

#### `torch schedule`
Schedule operations.

```bash
# Run scheduled tasks
torch schedule run

# List scheduled tasks
torch schedule list

# Clear schedule cache
torch schedule clear-cache
```

### Information

#### `torch about`
Show application information.

```bash
torch about
```

## Examples

### Creating a Complete Feature

```bash
# Create model with migration, factory, seeder, and policy
torch make model Post --migration --factory --seeder --policy

# Create resource controller
torch make controller PostController --resource

# Create templates
torch make template posts/index
torch make template posts/show
torch make template posts/create
torch make template posts/edit

# Run migrations
torch migrate

# Seed database
torch db seed
```

### Setting Up Development Environment

```bash
# Create new project
torch new blog-app

# Navigate to project
cd blog-app

# Start development server with hot reload
torch serve --hot
```

### Production Deployment

```bash
# Optimize for production
torch optimize

# Build release version
torch build --release

# The optimized application is ready in target/deploy/
```

## Configuration

The CLI reads configuration from `config/` directory files. Key configuration files:

- `config/app.toml` - Application settings
- `config/database.toml` - Database configuration
- `config/cache.toml` - Cache settings

## Tips and Best Practices

1. **Use hot reload during development**: `torch serve --hot`
2. **Generate complete features**: Use flags like `--migration --factory --seeder`
3. **Optimize before deployment**: Run `torch optimize` for production builds
4. **Use tinker for debugging**: Interactive shell for testing code
5. **Keep migrations organized**: Use descriptive names and proper ordering
6. **Test your application**: Use `torch test` regularly during development

## Troubleshooting

### Common Issues

1. **CLI not found**: Ensure you installed with `--features cli`
2. **Hot reload not working**: Check file permissions and paths
3. **Database connection failed**: Verify database configuration
4. **Build failures**: Check Rust version and dependencies

### Getting Help

- Use `torch --help` for general help
- Use `torch <command> --help` for command-specific help
- Check the documentation at [docs.rs/torch-web](https://docs.rs/torch-web)
- Visit the GitHub repository for issues and discussions
