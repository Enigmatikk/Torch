use torch_web::{App, Request, Response};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = App::new()
        .get("/", |_req: Request| async {
            Response::ok().body("Hello, World! 🔥")
        })
        .get("/hello/:name", |req: Request| async move {
            let name = req.param("name").unwrap_or("Anonymous");
            Response::ok().body(format!("Hello, {}! 🔥", name))
        })
        .get("/json", |_req: Request| async {
            #[cfg(feature = "json")]
            {
                use serde_json::json;
                Response::ok()
                    .json(&json!({
                        "message": "Hello from Torch!",
                        "framework": "torch",
                        "version": "0.1.0"
                    }))
                    .unwrap()
            }
            #[cfg(not(feature = "json"))]
            {
                Response::ok()
                    .content_type("application/json")
                    .body(r#"{"message": "Hello from Torch!", "framework": "torch"}"#)
            }
        });

    println!("🔥 Starting Torch Hello World example...");
    app.listen("127.0.0.1:3000").await
}
