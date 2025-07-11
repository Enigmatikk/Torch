use torch_web::{App, Request, main, ember::*};

#[main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Create templates directory if it doesn't exist
    std::fs::create_dir_all("templates").ok();
    
    // Create a very simple test template
    std::fs::write("templates/test.ember", r#"<!DOCTYPE html>
<html>
<head>
    <title>{{ $title }}</title>
</head>
<body>
    <h1>{{ $title }}</h1>
    <p>{{ $message }}</p>
    
    @if($show_list)
        <h3>Items:</h3>
        <ul>
        @foreach($items as $item)
            <li>{{ $item }}</li>
        @endforeach
        </ul>
    @else
        <p>No items to show.</p>
    @endif
</body>
</html>"#).ok();

    let app = App::new()
        .get::<_, ()>("/", |_req: Request| async {
            let data = EmberData::new()
                .with("title", "Simple Test")
                .with("message", "This is a test message")
                .with("show_list", true)
                .with("items", vec!["First item", "Second item", "Third item"]);
            
            ember("test", data).await
        });

    println!("🔥 Torch server with Ember test starting...");
    println!("🌐 Visit http://localhost:3000 to test Ember");
    
    app.listen("127.0.0.1:3000").await
}
