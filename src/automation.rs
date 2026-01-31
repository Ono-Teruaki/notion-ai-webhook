use crate::{AutomationContentType, NotionWebhookContent, NotionWebhookPayload};
use axum::http::header::HeaderMap;

pub async fn handle_webhook(headers: HeaderMap, body: String) -> String {
    let content_type_str = headers
        .get("content_type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    let content_type = match content_type_str {
        "diary" => AutomationContentType::Diary,
        _ => AutomationContentType::Unknown,
    };

    let payload: NotionWebhookPayload = match serde_json::from_str(&body) {
        Ok(p) => p,
        Err(e) => {
            println!("Failed to JSON perse: {}", e);
            return "Invalid data".to_string();
        }
    };
    let webhook_content = NotionWebhookContent {
        content_type: content_type,
        payload: payload,
    };

    println!("persed data: {webhook_content:#?}");
    body
}
