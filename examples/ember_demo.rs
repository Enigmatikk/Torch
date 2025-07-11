use torch_web::{App, Request, main, ember::*};

#[main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Create templates directory if it doesn't exist
    std::fs::create_dir_all("templates").ok();
    
    // Create a sample layout template
    std::fs::write("templates/layout.ember", r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{ $title }} - Torch with Ember</title>
    <style>
        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            margin: 0;
            padding: 0;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            color: white;
        }
        .container {
            max-width: 1200px;
            margin: 0 auto;
            padding: 2rem;
        }
        .header {
            text-align: center;
            margin-bottom: 3rem;
        }
        .header h1 {
            font-size: 3rem;
            margin: 0;
            text-shadow: 2px 2px 4px rgba(0,0,0,0.3);
        }
        .flame {
            font-size: 4rem;
            margin-bottom: 1rem;
        }
        .content {
            background: rgba(255,255,255,0.1);
            backdrop-filter: blur(10px);
            border-radius: 15px;
            padding: 2rem;
            box-shadow: 0 8px 32px rgba(0,0,0,0.1);
        }
        .user-list {
            list-style: none;
            padding: 0;
        }
        .user-list li {
            background: rgba(255,255,255,0.1);
            margin: 0.5rem 0;
            padding: 1rem;
            border-radius: 8px;
            border-left: 4px solid #ff6b6b;
        }
        .stats {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 1rem;
            margin: 2rem 0;
        }
        .stat-card {
            background: rgba(255,255,255,0.1);
            padding: 1.5rem;
            border-radius: 10px;
            text-align: center;
        }
        .stat-number {
            font-size: 2rem;
            font-weight: bold;
            color: #ff6b6b;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div class="flame">🔥</div>
            <h1>Torch + Ember</h1>
            <p>Blazing fast templates for blazing fast web apps</p>
        </div>
        
        <div class="content">
            @section('content')
                <p>Default content</p>
            @endsection
        </div>
    </div>
</body>
</html>"#).ok();

    std::fs::write("templates/home.ember", r#"@extends('layout')

@section('content')
    <h2>{{ $title }}</h2>
    <p>{{ $description }}</p>
    
    <div class="stats">
        <div class="stat-card">
            <div class="stat-number">{{ $user_count }}</div>
            <div>Active Users</div>
        </div>
        <div class="stat-card">
            <div class="stat-number">{{ $request_count }}</div>
            <div>Requests Served</div>
        </div>
        <div class="stat-card">
            <div class="stat-number">{{ $uptime }}</div>
            <div>Uptime (hours)</div>
        </div>
    </div>
    
    @if(count($users) > 0)
        <h3>Our Amazing Users</h3>
        <ul class="user-list">
        @foreach($users as $user)
            <li>🔥 {{ $user }}</li>
        @endforeach
        </ul>
    @else
        <p>No users yet. Be the first to join!</p>
    @endif
    
    <h3>Features</h3>
    <ul>
        @foreach($features as $feature)
            <li>✨ {{ $feature }}</li>
        @endforeach
    </ul>
@endsection"#).ok();

    // Create a simple about template
    std::fs::write("templates/about.ember", r#"@extends('layout')

@section('content')
    <h2>About Ember Templates</h2>
    
    <p>Ember is Torch's powerful templating engine, inspired by Laravel's Blade but built for Rust performance.</p>
    
    <h3>Key Features:</h3>
    <ul>
        <li>🔥 <strong>Blazing Fast:</strong> Compiled templates with intelligent caching</li>
        <li>🎨 <strong>Familiar Syntax:</strong> Laravel Blade-inspired directives</li>
        <li>🔒 <strong>Secure:</strong> Automatic XSS protection and input escaping</li>
        <li>🔄 <strong>Hot Reload:</strong> Templates update automatically in development</li>
        <li>📦 <strong>Component System:</strong> Reusable template components</li>
        <li>🏗️ <strong>Template Inheritance:</strong> Build complex layouts easily</li>
    </ul>
    
    <h3>Template Directives:</h3>
    <div style="background: rgba(0,0,0,0.2); padding: 1rem; border-radius: 8px; font-family: monospace;">
        <p><strong>Variables:</strong> {{ $variable }}</p>
        <p><strong>Conditionals:</strong> @if($condition) ... @else ... @endif</p>
        <p><strong>Loops:</strong> @foreach($items as $item) ... @endforeach</p>
        <p><strong>Includes:</strong> @include('partial')</p>
        <p><strong>Sections:</strong> @section('name') ... @endsection</p>
        <p><strong>Extends:</strong> @extends('layout')</p>
    </div>
    
    <p><strong>Version:</strong> {{ $version }}</p>
    <p><strong>Engine:</strong> {{ $engine }}</p>
@endsection"#).ok();

    let app = App::new()
        .get::<_, ()>("/", |_req: Request| async {
            let data = EmberData::new()
                .with("title", "Welcome to Torch")
                .with("description", "Experience the power of Ember templating with Torch web framework")
                .with("user_count", 1337)
                .with("request_count", 42069)
                .with("uptime", 24)
                .with("users", vec!["Alice", "Bob", "Charlie", "Diana", "Eve"])
                .with("features", vec![
                    "Lightning-fast routing",
                    "Type-safe extractors", 
                    "Powerful middleware system",
                    "Beautiful error pages",
                    "Production-ready security",
                    "Ember templating engine"
                ]);
            
            ember("home", data).await
        })
        .get::<_, ()>("/about", |_req: Request| async {
            let data = EmberData::new()
                .with("title", "About Ember")
                .with("version", "0.2.2")
                .with("engine", "Ember Template Engine");
            
            ember("about", data).await
        })
        .get::<_, ()>("/simple", |_req: Request| async {
            // Example of a simple template without layout
            std::fs::write("templates/simple.ember", r#"<!DOCTYPE html>
<html>
<head>
    <title>{{ $title }}</title>
    <style>
        body { font-family: Arial, sans-serif; padding: 2rem; background: #f0f0f0; }
        .card { background: white; padding: 2rem; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
    </style>
</head>
<body>
    <div class="card">
        <h1>{{ $title }}</h1>
        <p>{{ $message }}</p>
        
        @if($show_list)
            <h3>Items:</h3>
            <ul>
            @foreach($items as $item)
                <li>{{ $item }}</li>
            @endforeach
            </ul>
        @endif
        
        <p><a href="/">← Back to Home</a></p>
    </div>
</body>
</html>"#).ok();

            let data = EmberData::new()
                .with("title", "Simple Ember Template")
                .with("message", "This is a standalone template without layout inheritance.")
                .with("show_list", true)
                .with("items", vec!["First item", "Second item", "Third item"]);
            
            ember("simple", data).await
        });

    println!("🔥 Torch server with Ember templates starting...");
    println!("📁 Templates directory: ./templates/");
    println!("🌐 Visit http://localhost:3000 to see Ember in action!");
    println!("🔗 Routes:");
    println!("   • http://localhost:3000/ - Home page with layout");
    println!("   • http://localhost:3000/about - About Ember");
    println!("   • http://localhost:3000/simple - Simple template example");
    
    app.listen("127.0.0.1:3000").await
}
