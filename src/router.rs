use crate::automation::*;
use axum::{routing::post, Router};

pub fn router() -> Router {
    Router::<()>::new().route("/webhook", post(handle_webhook))
}
