use axum::serve;
use notion_ai_webhook::router::router;
use tokio;

#[tokio::main]
async fn main() {
    let app = router();

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    serve(listener, app).await.unwrap();
}
