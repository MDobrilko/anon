use axum::{
    Json, Router,
    body::Body,
    extract::State,
    http::{Response, StatusCode},
    response::IntoResponse,
    routing::post,
};

use crate::{
    bot::api::{
        entities::{
            CallbackData, InlineKeyboardButton, InlineKeyboardMarkup, UpdateMessage,
            WebhookResponse,
        },
        headers::ApiSecretToken,
    },
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
) -> Response<Body> {
    debug!("Got new request: {request:#?}");

    if state
        .config()
        .auth
        .api_token
        .as_deref()
        .is_some_and(|expected_token| Some(expected_token) != api_token.as_deref())
    {
        error!("Incorrect request token");
        return StatusCode::BAD_REQUEST.into_response();
    }

    match handle_request(request).await {
        Ok(Some(body)) => (StatusCode::OK, Json(body)).into_response(),
        Ok(None) => StatusCode::OK.into_response(),
        Err(err) => {
            error!("Error during request handling: {err:#}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn handle_request(request: serde_json::Value) -> anyhow::Result<Option<WebhookResponse>> {
    let Ok(UpdateMessage {
        update_id: _,
        message,
    }) = serde_json::from_value::<UpdateMessage>(request)
    else {
        info!("Failed to parse message. Skipping");
        return Ok(None);
    };
    let Some(message) = message else {
        info!("Missing message object in request. Skipping");
        return Ok(None);
    };

    Ok(Some(WebhookResponse {
        method: "sendMessage".to_string(),
        params: serde_json::json!({
            "chat_id": message.chat.id,
            "text": "Кто выпустил псину? Кто? Кто? Кто?",
            "reply_markup": InlineKeyboardMarkup {
                inline_keyboard: vec![InlineKeyboardButton {
                    text: "Отправить сообщение".to_string(),
                    callback_data: CallbackData::ActionSend,
                }]
            }
        }),
    }))
}
