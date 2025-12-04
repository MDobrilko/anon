use axum::{Json, Router, extract::State, http::StatusCode, routing::post};

use crate::{
    bot::api::{entities::UpdateMessage, headers::ApiSecretToken},
    log::{debug, error, info},
    state::AppState,
};

mod entities;
mod headers;

pub fn make_router(state: AppState) -> Router {
    Router::new()
        .route("/update", post(update))
        .with_state(state)
}

#[axum::debug_handler]
pub async fn update(
    State(state): State<AppState>,
    api_token: Option<ApiSecretToken>,
    Json(request): Json<serde_json::Value>,
) -> StatusCode {
    debug!("Got new request: {request:#?}");

    if state
        .config()
        .auth
        .api_token
        .as_deref()
        .is_some_and(|expected_token| Some(expected_token) != api_token.as_deref())
    {
        error!("Incorrect request token");
        return StatusCode::BAD_REQUEST;
    }

    if let Err(err) = handle_request(request).await {
        error!("Error during request handling: {err:#}");
    }

    StatusCode::OK
}

async fn handle_request(request: serde_json::Value) -> anyhow::Result<()> {
    let Ok(message) = serde_json::from_value::<UpdateMessage>(request) else {
        info!("Failed to parse message. Skipping");
        return Ok(());
    };

    Ok(())
}
