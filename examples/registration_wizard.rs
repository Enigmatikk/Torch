use torch_web::{App, Request, Response, main, ember::*};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// Step 1: Basic Information
#[derive(Debug, Deserialize, Serialize, Clone)]
struct BasicInfo {
    first_name: String,
    last_name: String,
    email: String,
    phone: String,
}

// Step 2: Account Details
#[derive(Debug, Deserialize, Serialize, Clone)]
struct AccountDetails {
    username: String,
    password: String,
    confirm_password: String,
    terms_accepted: bool,
}

// Step 3: Profile Information
#[derive(Debug, Deserialize, Serialize, Clone)]
struct ProfileInfo {
    bio: String,
    interests: Vec<String>,
    newsletter: bool,
}

// Complete registration data
#[derive(Debug, Deserialize, Serialize, Clone)]
struct RegistrationData {
    basic_info: Option<BasicInfo>,
    account_details: Option<AccountDetails>,
    profile_info: Option<ProfileInfo>,
    current_step: u8,
}

impl Default for RegistrationData {
    fn default() -> Self {
        Self {
            basic_info: None,
            account_details: None,
            profile_info: None,
            current_step: 1,
        }
    }
}

// Simple in-memory session store (in production, use Redis/Database)
type SessionStore = std::sync::Arc<std::sync::Mutex<HashMap<String, RegistrationData>>>;

#[main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Create templates directory
    std::fs::create_dir_all("templates/layouts").ok();
    std::fs::create_dir_all("templates/registration").ok();
    std::fs::create_dir_all("templates/components").ok();

    // Create the main layout template
    create_layout_template().await?;
    
    // Create registration step templates
    create_registration_templates().await?;
    
    // Create component templates
    create_component_templates().await?;

    // Initialize session store
    let sessions: SessionStore = std::sync::Arc::new(std::sync::Mutex::new(HashMap::new()));

    // Clone sessions for each route handler
    let sessions_1 = sessions.clone();
    let sessions_2 = sessions.clone();
    let sessions_3 = sessions.clone();
    let sessions_4 = sessions.clone();
    let sessions_5 = sessions.clone();

    let app = App::new()
        // Home page
        .get::<_, ()>("/", |_req: Request| async {
            let data = EmberData::new()
                .with("title", "Welcome to Torch Registration Demo")
                .with("page_title", "Multi-Step Registration Wizard")
                .with("description", "Experience the power of Torch with Ember templating");

            ember("home", data).await
        })

        // Registration wizard - Step 1
        .get::<_, ()>("/register", move |req: Request| {
            let sessions = sessions_1.clone();
            async move {
                let session_id = get_or_create_session(&req);
                let registration_data = get_session_data(&sessions, &session_id);

                let data = EmberData::new()
                    .with("title", "Registration - Step 1")
                    .with("step", 1)
                    .with("step_title", "Basic Information")
                    .with("progress", 33)
                    .with("first_name", registration_data.basic_info.as_ref().map(|b| b.first_name.clone()).unwrap_or_default())
                    .with("last_name", registration_data.basic_info.as_ref().map(|b| b.last_name.clone()).unwrap_or_default())
                    .with("email", registration_data.basic_info.as_ref().map(|b| b.email.clone()).unwrap_or_default())
                    .with("phone", registration_data.basic_info.as_ref().map(|b| b.phone.clone()).unwrap_or_default());

                ember("registration/step1", data).await
            }
        })

        // Handle Step 1 form submission
        .post::<_, ()>("/register/step1", move |req: Request| {
            let sessions = sessions_2.clone();
            async move {
                let session_id = get_or_create_session(&req);

                // In a real app, you'd parse the form data from req.body()
                // For demo purposes, we'll simulate form data
                let basic_info = BasicInfo {
                    first_name: "John".to_string(),
                    last_name: "Doe".to_string(),
                    email: "john.doe@example.com".to_string(),
                    phone: "+1-555-0123".to_string(),
                };

                // Update session data
                update_session_data(&sessions, &session_id, |data| {
                    data.basic_info = Some(basic_info);
                    data.current_step = 2;
                });

                // Redirect to step 2
                Response::redirect_found("/register/step2")
            }
        })

        // Registration wizard - Step 2
        .get::<_, ()>("/register/step2", move |req: Request| {
            let sessions = sessions_3.clone();
            async move {
                let session_id = get_or_create_session(&req);
                let registration_data = get_session_data(&sessions, &session_id);

                let data = EmberData::new()
                    .with("title", "Registration - Step 2")
                    .with("step", 2)
                    .with("step_title", "Account Details")
                    .with("progress", 66)
                    .with("username", registration_data.account_details.as_ref().map(|a| a.username.clone()).unwrap_or_default());

                ember("registration/step2", data).await
            }
        })

        // Registration wizard - Step 3
        .get::<_, ()>("/register/step3", move |req: Request| {
            let sessions = sessions_4.clone();
            async move {
                let session_id = get_or_create_session(&req);
                let registration_data = get_session_data(&sessions, &session_id);

                let data = EmberData::new()
                    .with("title", "Registration - Step 3")
                    .with("step", 3)
                    .with("step_title", "Profile Information")
                    .with("progress", 100)
                    .with("bio", registration_data.profile_info.as_ref().map(|p| p.bio.clone()).unwrap_or_default())
                    .with("interests", vec!["Technology", "Sports", "Music", "Travel"])
                    .with("newsletter", registration_data.profile_info.as_ref().map(|p| p.newsletter).unwrap_or(false));

                ember("registration/step3", data).await
            }
        })

        // Registration complete
        .get::<_, ()>("/register/complete", move |req: Request| {
            let sessions = sessions_5.clone();
            async move {
                let session_id = get_or_create_session(&req);
                let registration_data = get_session_data(&sessions, &session_id);

                let data = EmberData::new()
                    .with("title", "Registration Complete!")
                    .with("user_name", registration_data.basic_info.as_ref().map(|b| format!("{} {}", b.first_name, b.last_name)).unwrap_or_default())
                    .with("email", registration_data.basic_info.as_ref().map(|b| b.email.clone()).unwrap_or_default());

                ember("registration/complete", data).await
            }
        });

    println!("🔥 Torch Registration Wizard Demo starting...");
    println!("🌐 Visit http://localhost:3000 to see the demo");
    println!("📋 Registration wizard: http://localhost:3000/register");
    
    app.listen("127.0.0.1:3000").await
}

// Helper functions for session management
fn get_or_create_session(req: &Request) -> String {
    // In a real app, you'd extract session ID from cookies
    // For demo, we'll use a simple approach
    req.header("x-session-id").unwrap_or("demo-session").to_string()
}

fn get_session_data(sessions: &SessionStore, session_id: &str) -> RegistrationData {
    let sessions = sessions.lock().unwrap();
    sessions.get(session_id).cloned().unwrap_or_default()
}

fn update_session_data<F>(sessions: &SessionStore, session_id: &str, updater: F) 
where 
    F: FnOnce(&mut RegistrationData)
{
    let mut sessions = sessions.lock().unwrap();
    let mut data = sessions.get(session_id).cloned().unwrap_or_default();
    updater(&mut data);
    sessions.insert(session_id.to_string(), data);
}

// Template creation functions
async fn create_layout_template() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    std::fs::write("templates/layouts/main.ember", r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{ $title }} - Torch Demo</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            color: #333;
        }
        .header {
            background: rgba(255,255,255,0.1);
            backdrop-filter: blur(10px);
            padding: 1rem 0;
            border-bottom: 1px solid rgba(255,255,255,0.2);
        }
        .nav {
            max-width: 1200px;
            margin: 0 auto;
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 0 2rem;
        }
        .logo {
            font-size: 1.5rem;
            font-weight: bold;
            color: white;
            text-decoration: none;
        }
        .nav-links {
            display: flex;
            list-style: none;
            gap: 2rem;
        }
        .nav-links a {
            color: white;
            text-decoration: none;
            transition: opacity 0.3s;
        }
        .nav-links a:hover { opacity: 0.8; }
        .container {
            max-width: 1200px;
            margin: 0 auto;
            padding: 2rem;
        }
        .footer {
            background: rgba(0,0,0,0.2);
            color: white;
            text-align: center;
            padding: 2rem;
            margin-top: auto;
        }
    </style>
</head>
<body>
    @include('components/header')
    
    <main class="container">
        @section('content')
            <p>Default content</p>
        @endsection
    </main>
    
    @include('components/footer')
</body>
</html>"#)?;
    Ok(())
}

async fn create_registration_templates() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Home page template
    std::fs::write("templates/home.ember", r#"@extends('layouts/main')

@section('content')
    <div style="background: rgba(255,255,255,0.95); border-radius: 15px; padding: 3rem; text-align: center; box-shadow: 0 10px 30px rgba(0,0,0,0.2);">
        <h1 style="font-size: 3rem; margin-bottom: 1rem; color: #333;">{{ $page_title }}</h1>
        <p style="font-size: 1.2rem; margin-bottom: 2rem; color: #666;">{{ $description }}</p>
        
        <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 2rem; margin: 3rem 0;">
            <div style="background: #f8f9fa; padding: 2rem; border-radius: 10px; border-left: 4px solid #667eea;">
                <h3 style="color: #667eea; margin-bottom: 1rem;">🔥 Template Inheritance</h3>
                <p>Consistent layouts with @extends and @section directives</p>
            </div>
            <div style="background: #f8f9fa; padding: 2rem; border-radius: 10px; border-left: 4px solid #764ba2;">
                <h3 style="color: #764ba2; margin-bottom: 1rem;">📋 Multi-Step Forms</h3>
                <p>Complex workflows with session state management</p>
            </div>
            <div style="background: #f8f9fa; padding: 2rem; border-radius: 10px; border-left: 4px solid #ff6b6b;">
                <h3 style="color: #ff6b6b; margin-bottom: 1rem;">🎨 Component System</h3>
                <p>Reusable components with @include directives</p>
            </div>
        </div>
        
        <a href="/register" style="display: inline-block; background: linear-gradient(45deg, #667eea, #764ba2); color: white; padding: 1rem 2rem; border-radius: 50px; text-decoration: none; font-weight: bold; font-size: 1.1rem; transition: transform 0.3s; box-shadow: 0 4px 15px rgba(0,0,0,0.2);">
            Start Registration Wizard 🚀
        </a>
    </div>
@endsection"#)?;

    // Step 2 template
    std::fs::write("templates/registration/step2.ember", r#"@extends('layouts/main')

@section('content')
    @include('components/progress_bar')

    <div style="background: white; border-radius: 15px; padding: 3rem; max-width: 600px; margin: 2rem auto; box-shadow: 0 10px 30px rgba(0,0,0,0.1);">
        <h2 style="text-align: center; margin-bottom: 2rem; color: #333;">{{ $step_title }}</h2>

        <form method="POST" action="/register/step2" style="display: grid; gap: 1.5rem;">
            <div>
                <label style="display: block; margin-bottom: 0.5rem; font-weight: bold; color: #555;">Username</label>
                <input type="text" name="username" value="{{ $username }}" required
                       style="width: 100%; padding: 0.75rem; border: 2px solid #e1e5e9; border-radius: 8px; font-size: 1rem;">
                <small style="color: #6c757d;">Choose a unique username (3-20 characters)</small>
            </div>

            <div>
                <label style="display: block; margin-bottom: 0.5rem; font-weight: bold; color: #555;">Password</label>
                <input type="password" name="password" required
                       style="width: 100%; padding: 0.75rem; border: 2px solid #e1e5e9; border-radius: 8px; font-size: 1rem;">
                <small style="color: #6c757d;">Minimum 8 characters with letters and numbers</small>
            </div>

            <div>
                <label style="display: block; margin-bottom: 0.5rem; font-weight: bold; color: #555;">Confirm Password</label>
                <input type="password" name="confirm_password" required
                       style="width: 100%; padding: 0.75rem; border: 2px solid #e1e5e9; border-radius: 8px; font-size: 1rem;">
            </div>

            <div style="display: flex; align-items: center; gap: 0.5rem; margin: 1rem 0;">
                <input type="checkbox" name="terms_accepted" required style="transform: scale(1.2);">
                <label style="color: #555;">I agree to the <a href="/terms" style="color: #667eea;">Terms of Service</a> and <a href="/privacy" style="color: #667eea;">Privacy Policy</a></label>
            </div>

            <div style="display: flex; justify-content: space-between; margin-top: 2rem;">
                <a href="/register" style="padding: 0.75rem 1.5rem; background: #6c757d; color: white; text-decoration: none; border-radius: 8px;">← Previous Step</a>
                <button type="submit" style="padding: 0.75rem 2rem; background: linear-gradient(45deg, #667eea, #764ba2); color: white; border: none; border-radius: 8px; font-size: 1rem; cursor: pointer;">
                    Continue to Step 3 →
                </button>
            </div>
        </form>
    </div>
@endsection"#)?;

    // Step 3 template
    std::fs::write("templates/registration/step3.ember", r#"@extends('layouts/main')

@section('content')
    @include('components/progress_bar')

    <div style="background: white; border-radius: 15px; padding: 3rem; max-width: 600px; margin: 2rem auto; box-shadow: 0 10px 30px rgba(0,0,0,0.1);">
        <h2 style="text-align: center; margin-bottom: 2rem; color: #333;">{{ $step_title }}</h2>

        <form method="POST" action="/register/step3" style="display: grid; gap: 1.5rem;">
            <div>
                <label style="display: block; margin-bottom: 0.5rem; font-weight: bold; color: #555;">Bio</label>
                <textarea name="bio" rows="4" placeholder="Tell us about yourself..."
                          style="width: 100%; padding: 0.75rem; border: 2px solid #e1e5e9; border-radius: 8px; font-size: 1rem; resize: vertical;">{{ $bio }}</textarea>
            </div>

            <div>
                <label style="display: block; margin-bottom: 1rem; font-weight: bold; color: #555;">Interests</label>
                <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 0.5rem;">
                @foreach($interests as $interest)
                    <label style="display: flex; align-items: center; gap: 0.5rem; padding: 0.5rem; background: #f8f9fa; border-radius: 6px; cursor: pointer;">
                        <input type="checkbox" name="interests[]" value="{{ $interest }}">
                        <span>{{ $interest }}</span>
                    </label>
                @endforeach
                </div>
            </div>

            <div style="display: flex; align-items: center; gap: 0.5rem; margin: 1rem 0;">
                <input type="checkbox" name="newsletter" @if($newsletter)checked@endif style="transform: scale(1.2);">
                <label style="color: #555;">Subscribe to our newsletter for updates and tips</label>
            </div>

            <div style="display: flex; justify-content: space-between; margin-top: 2rem;">
                <a href="/register/step2" style="padding: 0.75rem 1.5rem; background: #6c757d; color: white; text-decoration: none; border-radius: 8px;">← Previous Step</a>
                <button type="submit" style="padding: 0.75rem 2rem; background: linear-gradient(45deg, #28a745, #20c997); color: white; border: none; border-radius: 8px; font-size: 1rem; cursor: pointer;">
                    Complete Registration ✓
                </button>
            </div>
        </form>
    </div>
@endsection"#)?;

    // Completion template
    std::fs::write("templates/registration/complete.ember", r#"@extends('layouts/main')

@section('content')
    <div style="background: white; border-radius: 15px; padding: 3rem; max-width: 600px; margin: 2rem auto; box-shadow: 0 10px 30px rgba(0,0,0,0.1); text-align: center;">
        <div style="font-size: 4rem; margin-bottom: 1rem;">🎉</div>
        <h2 style="color: #28a745; margin-bottom: 1rem;">Registration Complete!</h2>
        <p style="font-size: 1.2rem; margin-bottom: 2rem; color: #666;">
            Welcome aboard, <strong>{{ $user_name }}</strong>!
        </p>

        <div style="background: #f8f9fa; padding: 1.5rem; border-radius: 10px; margin: 2rem 0; text-align: left;">
            <h4 style="color: #333; margin-bottom: 1rem;">What's Next?</h4>
            <ul style="color: #666; line-height: 1.6;">
                <li>Check your email ({{ $email }}) for verification</li>
                <li>Complete your profile setup</li>
                <li>Explore our features and documentation</li>
                <li>Join our community forum</li>
            </ul>
        </div>

        <div style="display: flex; gap: 1rem; justify-content: center; margin-top: 2rem;">
            <a href="/dashboard" style="padding: 0.75rem 1.5rem; background: linear-gradient(45deg, #667eea, #764ba2); color: white; text-decoration: none; border-radius: 8px;">
                Go to Dashboard
            </a>
            <a href="/" style="padding: 0.75rem 1.5rem; background: #6c757d; color: white; text-decoration: none; border-radius: 8px;">
                Back to Home
            </a>
        </div>
    </div>
@endsection"#)?;

    // Step 1 template
    std::fs::write("templates/registration/step1.ember", r#"@extends('layouts/main')

@section('content')
    @include('components/progress_bar')
    
    <div style="background: white; border-radius: 15px; padding: 3rem; max-width: 600px; margin: 2rem auto; box-shadow: 0 10px 30px rgba(0,0,0,0.1);">
        <h2 style="text-align: center; margin-bottom: 2rem; color: #333;">{{ $step_title }}</h2>
        
        <form method="POST" action="/register/step1" style="display: grid; gap: 1.5rem;">
            <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 1rem;">
                <div>
                    <label style="display: block; margin-bottom: 0.5rem; font-weight: bold; color: #555;">First Name</label>
                    <input type="text" name="first_name" value="{{ $first_name }}" required 
                           style="width: 100%; padding: 0.75rem; border: 2px solid #e1e5e9; border-radius: 8px; font-size: 1rem;">
                </div>
                <div>
                    <label style="display: block; margin-bottom: 0.5rem; font-weight: bold; color: #555;">Last Name</label>
                    <input type="text" name="last_name" value="{{ $last_name }}" required
                           style="width: 100%; padding: 0.75rem; border: 2px solid #e1e5e9; border-radius: 8px; font-size: 1rem;">
                </div>
            </div>
            
            <div>
                <label style="display: block; margin-bottom: 0.5rem; font-weight: bold; color: #555;">Email Address</label>
                <input type="email" name="email" value="{{ $email }}" required
                       style="width: 100%; padding: 0.75rem; border: 2px solid #e1e5e9; border-radius: 8px; font-size: 1rem;">
            </div>
            
            <div>
                <label style="display: block; margin-bottom: 0.5rem; font-weight: bold; color: #555;">Phone Number</label>
                <input type="tel" name="phone" value="{{ $phone }}" required
                       style="width: 100%; padding: 0.75rem; border: 2px solid #e1e5e9; border-radius: 8px; font-size: 1rem;">
            </div>
            
            <div style="display: flex; justify-content: space-between; margin-top: 2rem;">
                <a href="/" style="padding: 0.75rem 1.5rem; background: #6c757d; color: white; text-decoration: none; border-radius: 8px;">← Back to Home</a>
                <button type="submit" style="padding: 0.75rem 2rem; background: linear-gradient(45deg, #667eea, #764ba2); color: white; border: none; border-radius: 8px; font-size: 1rem; cursor: pointer;">
                    Continue to Step 2 →
                </button>
            </div>
        </form>
    </div>
@endsection"#)?;

    Ok(())
}

async fn create_component_templates() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Header component
    std::fs::write("templates/components/header.ember", r#"<header class="header">
    <nav class="nav">
        <a href="/" class="logo">🔥 Torch Demo</a>
        <ul class="nav-links">
            <li><a href="/">Home</a></li>
            <li><a href="/register">Register</a></li>
            <li><a href="/about">About</a></li>
        </ul>
    </nav>
</header>"#)?;

    // Footer component
    std::fs::write("templates/components/footer.ember", r#"<footer class="footer">
    <p>&copy; 2024 Torch Web Framework. Built with 🔥 and ❤️</p>
    <p>Powered by Ember Template Engine</p>
</footer>"#)?;

    // Progress bar component
    std::fs::write("templates/components/progress_bar.ember", r#"<div style="background: white; border-radius: 10px; padding: 1.5rem; margin-bottom: 2rem; box-shadow: 0 4px 15px rgba(0,0,0,0.1);">
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem;">
        <span style="font-weight: bold; color: #333;">Step {{ $step }} of 3</span>
        <span style="color: #667eea; font-weight: bold;">{{ $progress }}% Complete</span>
    </div>
    <div style="background: #e9ecef; height: 8px; border-radius: 4px; overflow: hidden;">
        <div style="background: linear-gradient(45deg, #667eea, #764ba2); height: 100%; width: {{ $progress }}%; transition: width 0.3s ease;"></div>
    </div>
</div>"#)?;

    Ok(())
}
