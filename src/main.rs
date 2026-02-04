use axum::serve;
use dotenv::dotenv;
use notion_ai_webhook::{
    router::{router, AppState},
    service::{GeminiService, NotionService},
};
use reqwest::Client;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let client = Client::new();
    let state = AppState {
        notion_service: NotionService::new(client.clone())?,
        gemini_service: GeminiService::new(client.clone())?,
    };
    let app = router(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    serve(listener, app).await.unwrap();

    Ok(())
}
