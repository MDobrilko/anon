use axum::{Router, routing::post};

use crate::state::AppState;

pub fn make_router(state: AppState) -> Router {
    Router::new()
        .route("/update", post(update))
        .with_state(state)
}

pub fn update() {}
