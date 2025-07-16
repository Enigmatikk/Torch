# Torch CLI Tutorial: Building a Blog Application

This tutorial demonstrates how to use the Torch CLI to build a complete blog application from scratch, showcasing all the major CLI features.

## Prerequisites

- Rust installed (1.75+)
- Torch CLI installed: `cargo install torch-web --features cli`
- PostgreSQL running (optional, for database features)

## Step 1: Create New Project

```bash
# Create a new Torch application
torch new torch-blog

# Navigate to the project
cd torch-blog

# Check the project structure
ls -la
```

This creates a complete project structure with:
- `src/` - Application source code
- `templates/` - Ember templates
- `static/` - Static assets
- `config/` - Configuration files
- `migrations/` - Database migrations
- `storage/` - Application storage

## Step 2: Start Development Server

```bash
# Start development server with hot reload
torch serve --hot
```

Visit `http://localhost:3000` to see your application running.

## Step 3: Generate Models

```bash
# Generate User model with migration, factory, seeder, and policy
torch make model User --migration --factory --seeder --policy

# Generate Post model with all features
torch make model Post --migration --factory --seeder --policy

# Generate Comment model
torch make model Comment --migration --factory --seeder
```

This creates:
- Model files in `src/models/`
- Migration files in `migrations/`
- Factory files in `src/factories/`
- Seeder files in `src/seeders/`
- Policy files in `src/policies/`

## Step 4: Create Controllers

```bash
# Generate resource controllers for API endpoints
torch make controller UserController --resource --api
torch make controller PostController --resource --api
torch make controller CommentController --resource --api

# Generate web controllers for HTML pages
torch make controller WebController
torch make controller AuthController
```

## Step 5: Generate Templates

```bash
# Create layout templates
torch make template layouts/app
torch make template layouts/auth

# Create user templates
torch make template users/index --layout app
torch make template users/show --layout app
torch make template users/profile --layout app

# Create post templates
torch make template posts/index --layout app
torch make template posts/show --layout app
torch make template posts/create --layout app
torch make template posts/edit --layout app

# Create auth templates
torch make template auth/login --layout auth
torch make template auth/register --layout auth

# Create component templates
torch make template components/navbar
torch make template components/footer
torch make template components/post-card
```

## Step 6: Create Middleware

```bash
# Generate authentication middleware
torch make middleware AuthMiddleware

# Generate CORS middleware
torch make middleware CorsMiddleware

# Generate rate limiting middleware
torch make middleware RateLimitMiddleware

# Generate logging middleware
torch make middleware LoggingMiddleware
```

## Step 7: Database Setup

```bash
# Install migration repository
torch migrate install

# Run migrations
torch migrate

# Check migration status
torch migrate status

# Seed the database
torch db seed

# Check database status
torch db status
```

## Step 8: Generate Events and Listeners

```bash
# Generate events
torch make event UserRegistered
torch make event PostPublished
torch make event CommentPosted

# Generate listeners
torch make listener SendWelcomeEmail --event UserRegistered
torch make listener NotifySubscribers --event PostPublished
torch make listener SendCommentNotification --event CommentPosted
```

## Step 9: Create Jobs

```bash
# Generate background jobs
torch make job SendEmailJob
torch make job ProcessImageJob
torch make job GenerateReportJob --sync
torch make job CleanupTempFilesJob
```

## Step 10: Generate Notifications

```bash
# Generate notification classes
torch make notification WelcomeNotification
torch make notification PostPublishedNotification
torch make notification CommentNotification
torch make notification WeeklyDigestNotification
```

## Step 11: Create Tests

```bash
# Generate unit tests
torch make test UserTest --unit
torch make test PostTest --unit
torch make test CommentTest --unit

# Generate integration tests
torch make test ApiTest
torch make test WebTest
torch make test AuthTest
```

## Step 12: Run Tests

```bash
# Run all tests
torch test

# Run specific tests
torch test --filter user

# Run only unit tests
torch test --unit

# Run only integration tests
torch test --integration
```

## Step 13: Cache Management

```bash
# Cache configuration for better performance
torch config cache

# Cache routes
torch route cache

# Cache views
torch view cache

# Check cache statistics
torch cache stats
```

## Step 14: Interactive Development

```bash
# Start interactive shell
torch tinker
```

In the tinker shell, try:
```
# Check application info
app

# List routes
routes

# Show configuration
config

# Show models
models

# Test expressions
2 + 2
$app_name
chrono::Utc::now()

# Exit
exit
```

## Step 15: Queue Management

```bash
# Start queue worker in background
torch queue work --queue default &

# Check failed jobs
torch queue failed

# Restart workers
torch queue restart
```

## Step 16: Schedule Tasks

```bash
# Generate scheduled task
torch make command DailyCleanupCommand

# List scheduled tasks
torch schedule list

# Run scheduled tasks manually
torch schedule run
```

## Step 17: Production Optimization

```bash
# Optimize application for production
torch optimize

# Build optimized release
torch build --release

# Check the deployment artifacts
ls -la target/deploy/
```

## Step 18: Maintenance Mode

```bash
# Put application in maintenance mode
torch down --secret mySecret123

# Check if it's working (visit your app)

# Bring application back online
torch up
```

## Project Structure After CLI Generation

```
torch-blog/
├── src/
│   ├── controllers/
│   │   ├── user_controller.rs
│   │   ├── post_controller.rs
│   │   ├── comment_controller.rs
│   │   ├── web_controller.rs
│   │   └── auth_controller.rs
│   ├── models/
│   │   ├── user.rs
│   │   ├── post.rs
│   │   └── comment.rs
│   ├── middleware/
│   │   ├── auth_middleware.rs
│   │   ├── cors_middleware.rs
│   │   ├── rate_limit_middleware.rs
│   │   └── logging_middleware.rs
│   ├── events/
│   │   ├── user_registered.rs
│   │   ├── post_published.rs
│   │   └── comment_posted.rs
│   ├── listeners/
│   │   ├── send_welcome_email.rs
│   │   ├── notify_subscribers.rs
│   │   └── send_comment_notification.rs
│   ├── jobs/
│   │   ├── send_email_job.rs
│   │   ├── process_image_job.rs
│   │   ├── generate_report_job.rs
│   │   └── cleanup_temp_files_job.rs
│   ├── notifications/
│   │   ├── welcome_notification.rs
│   │   ├── post_published_notification.rs
│   │   ├── comment_notification.rs
│   │   └── weekly_digest_notification.rs
│   ├── factories/
│   │   ├── user_factory.rs
│   │   ├── post_factory.rs
│   │   └── comment_factory.rs
│   ├── seeders/
│   │   ├── user_seeder.rs
│   │   ├── post_seeder.rs
│   │   └── comment_seeder.rs
│   ├── policies/
│   │   ├── user_policy.rs
│   │   └── post_policy.rs
│   └── main.rs
├── templates/
│   ├── layouts/
│   │   ├── app.ember
│   │   └── auth.ember
│   ├── users/
│   │   ├── index.ember
│   │   ├── show.ember
│   │   └── profile.ember
│   ├── posts/
│   │   ├── index.ember
│   │   ├── show.ember
│   │   ├── create.ember
│   │   └── edit.ember
│   ├── auth/
│   │   ├── login.ember
│   │   └── register.ember
│   └── components/
│       ├── navbar.ember
│       ├── footer.ember
│       └── post-card.ember
├── migrations/
│   ├── 2024_01_01_000001_create_users_table.rs
│   ├── 2024_01_01_000002_create_posts_table.rs
│   └── 2024_01_01_000003_create_comments_table.rs
├── tests/
│   ├── user_test.rs
│   ├── post_test.rs
│   ├── comment_test.rs
│   ├── api_test.rs
│   ├── web_test.rs
│   └── auth_test.rs
├── config/
│   ├── app.toml
│   └── database.toml
├── static/
│   ├── css/
│   ├── js/
│   └── images/
├── storage/
│   ├── logs/
│   └── framework/
├── target/
│   └── deploy/
│       ├── server
│       ├── static/
│       ├── templates/
│       ├── manifest.json
│       └── Dockerfile
├── Cargo.toml
├── README.md
└── .gitignore
```

## Key CLI Commands Used

1. **Project Creation**: `torch new`
2. **Code Generation**: `torch make` (controller, model, middleware, etc.)
3. **Database**: `torch migrate`, `torch db`
4. **Development**: `torch serve --hot`
5. **Testing**: `torch test`
6. **Caching**: `torch cache`, `torch config cache`, `torch route cache`
7. **Interactive**: `torch tinker`
8. **Production**: `torch optimize`, `torch build --release`
9. **Maintenance**: `torch down`, `torch up`

## Next Steps

1. **Customize the generated code** to fit your specific requirements
2. **Add business logic** to controllers and models
3. **Style your templates** with CSS and JavaScript
4. **Configure your database** connection
5. **Set up deployment** using the generated Dockerfile
6. **Add monitoring** and logging for production

## Tips for Effective CLI Usage

1. **Use resource controllers** for standard CRUD operations
2. **Generate complete features** with all related files at once
3. **Use hot reload** during development for faster iteration
4. **Cache everything** for production performance
5. **Test regularly** with the built-in test runner
6. **Use tinker** for debugging and experimentation
7. **Optimize before deployment** for best performance

This tutorial demonstrates the power of the Torch CLI in rapidly scaffolding a complete web application with all the necessary components for a production-ready system.
