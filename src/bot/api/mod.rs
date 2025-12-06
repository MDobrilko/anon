use axum::{
    Json, Router,
    body::Body,
    extract::State,
    http::{Response, StatusCode},
    response::IntoResponse,
    routing::post,
};
use uuid::Uuid;

use crate::{
    bot::api::{
        entities::{
            CallbackData, CallbackQuery, InlineKeyboardButton, InlineKeyboardMarkup, Message,
            UpdateMessage, WebhookResponse,
        },
        headers::ApiSecretToken,
    },
    log::{FutureExt, debug, error, info, logger, o},
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

    match handle_request(&state, request)
        .with_logger(logger().new(o!("uuid" => Uuid::new_v4().to_string())))
        .await
    {
        Ok(Some(body)) => (StatusCode::OK, Json(body)).into_response(),
        Ok(None) => StatusCode::OK.into_response(),
        Err(err) => {
            error!("Error during request handling: {err:#}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn handle_request(
    state: &AppState,
    request: serde_json::Value,
) -> anyhow::Result<Option<WebhookResponse>> {
    debug!("Got new request: {request:#?}");

    let Ok(parsed_request) = serde_json::from_value::<UpdateMessage>(request) else {
        info!("Failed to parse message. Skipping");
        return Ok(None);
    };

    if let Some(message) = parsed_request.message.as_ref() {
        handle_message(state, message).await?;
    };
    if let Some(callback_query) = parsed_request.callback_query.as_ref() {
        handle_send_message_button_click(state, callback_query).await;
    }

    Ok(None)
}

async fn handle_message(state: &AppState, message: &Message) -> anyhow::Result<()> {
    if message
        .text
        .as_deref()
        .is_none_or(|t| t.trim() != "bot ping")
    {
        return Ok(());
    }

    let payload = serde_json::json!({
        "chat_id": message.chat.id,
        "text": "Кто выпустил псину? Кто? Кто? Кто?",
        "reply_markup": InlineKeyboardMarkup {
            inline_keyboard: vec![vec![InlineKeyboardButton {
                text: "Отправить сообщение".to_string(),
                callback_data: CallbackData::ActionSend,
            }]]
        }
    });

    // let response = WebhookResponse {
    //     method: "sendMessage".to_string(),
    //     params: payload,
    // };
    if let Err(err) = state.tg_client().send_message(&payload).await {
        error!("Error sending message to client: {err}");
    }

    Ok(())
}

async fn handle_send_message_button_click(
    state: &AppState,
    query: &CallbackQuery,
) -> anyhow::Result<()> {
    let Some(username) = query.from.username.as_deref() else {
        return Ok(());
    };
    let Some(message) = query.message.as_deref() else {
        return Ok(());
    };

    let payload = serde_json::json!({
        "chat_id": message.chat.id,
        "text": format!("@{username} куда хватаешь?"),
    });
    if let Err(err) = state.tg_client().send_message(&payload).await {
        error!("Error sending message to client: {err}");
    }

    Ok(())
}
