use crate::{
    automation::*,
    service::{GeminiService, NotionService},
};
use axum::{routing::post, Router};

#[derive(Clone)]
pub struct AppState {
    pub notion_service: NotionService,
    pub gemini_service: GeminiService,
}

pub fn router(state: AppState) -> Router {
    let webhook_routes = Router::new()
        .route("/diary", post(handle_diary_automation))
        .route("/diary-weekly-report", post(handle_diary_automation))
        .with_state(state);

    Router::<()>::new().nest("/webhook", webhook_routes)
}
