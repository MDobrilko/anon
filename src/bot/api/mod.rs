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
    bot::{
        api::{
            entities::{
                CallbackData, CallbackQuery, ChatType, InlineKeyboardButton, InlineKeyboardMarkup,
                Message, UpdateMessage, WebhookResponse,
            },
            headers::ApiSecretToken,
        },
        entities::{SendAnimationPayload, SendPhotoPayload},
    },
    log::{FutureExt, debug, error, info, logger, o},
    state::AppState,
};

pub mod entities;
mod headers;

pub fn make_router(state: AppState) -> Router {
    Router::new()
        .route("/update", post(update))
        .with_state(state)
}

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

    let parsed_request = match serde_json::from_value::<UpdateMessage>(request) {
        Ok(req) => req,
        Err(err) => {
            info!("Failed to parse message: {err}. Skipping");
            return Ok(None);
        }
    };

    if let Some(message) = parsed_request.message.as_ref() {
        handle_message(state, message).await?;
    };
    if let Some(callback_query) = parsed_request.callback_query.as_ref() {
        handle_button_click(state, callback_query).await?;
    }

    Ok(None)
}

async fn handle_message(state: &AppState, message: &Message) -> anyhow::Result<()> {
    let is_send_cmd = message
        .text
        .as_deref()
        .is_some_and(|text| text.starts_with("/send"));

    if is_send_cmd {
        handle_send_command(state, message).await
    } else {
        handle_text_message(state, message).await
    }
}

async fn handle_send_command(state: &AppState, message: &Message) -> anyhow::Result<()> {
    let user = match message.from.as_ref() {
        Some(user) if !user.is_bot => user,
        _ => return Ok(()),
    };

    match message.chat.chat_type {
        ChatType::Private => {
            let payload = make_bot_main_message(message.chat.id);

            state.tg_client().send_message(&payload).await;
        }
        _ => {
            let added = state.chats().add_user_chat(user.id, &message.chat).await;
            if added {
                state.save_chats().await?;
            }

            let tagged_username = user
                .username
                .as_deref()
                .map(|username| format!("@{username}"));
            let message_username = tagged_username.as_deref().unwrap_or("anonimous");

            let response_message_text = match added {
                true => format!(
                    "{message_username} теперь может отправлять анонимные сообщения в этот чат"
                ),
                false => format!(
                    "{message_username} уже может отправлять анонимные сообщения в этот чат"
                ),
            };

            let payload = make_bot_text_message(message.chat.id, &response_message_text);
            state.tg_client().send_message(&payload).await;
        }
    }

    Ok(())
}

async fn handle_text_message(state: &AppState, message: &Message) -> anyhow::Result<()> {
    if !matches!(message.chat.chat_type, ChatType::Private) {
        return Ok(());
    }
    let Some(user) = message.from.as_ref() else {
        return Ok(());
    };

    let target_chat_id = { state.user_letters().read().await.get(&user.id).copied() };

    match target_chat_id {
        Some(chat_id) => resend_message_anonimously(state, message, chat_id).await,
        None => {
            let payload = make_bot_main_message(message.chat.id);

            state.tg_client().send_message(&payload).await;

            Ok(())
        }
    }
}

async fn resend_message_anonimously(
    state: &AppState,
    message: &Message,
    target_chat_id: i64,
) -> anyhow::Result<()> {
    if let Some(text) = message.text.as_deref() {
        let payload = make_bot_text_message(target_chat_id, text);
        state.tg_client().send_message(&payload).await;
    }

    if let Some(photo_sizes) = message.photo.as_ref().filter(|v| !v.is_empty()) {
        let file_id = &*photo_sizes[0].file_id;

        state
            .tg_client()
            .send_photo(SendPhotoPayload {
                chat_id: target_chat_id,
                photo: file_id,
                caption: message.caption.as_deref(),
            })
            .await;
    }

    if let Some(animation) = message.animation.as_ref() {
        state
            .tg_client()
            .send_animation(SendAnimationPayload {
                chat_id: target_chat_id,
                animation: animation.file_id.as_ref(),
                duration: Some(animation.duration),
                width: Some(animation.width),
                height: Some(animation.height),
                caption: message.caption.as_deref(),
            })
            .await;
    }

    Ok(())
}

async fn handle_button_click(state: &AppState, query: &CallbackQuery) -> anyhow::Result<()> {
    if query.from.is_bot {
        return Ok(());
    }

    match query.data.as_ref() {
        Some(CallbackData::ActionSend) => {
            handle_chat_select_button_clicked(state, query).await?;
        }
        Some(CallbackData::SendTo(target_chat_id)) => {
            handle_chat_button_clicked(state, query, *target_chat_id).await?;
        }
        None => {}
    }

    Ok(())
}

async fn handle_chat_select_button_clicked(
    state: &AppState,
    query: &CallbackQuery,
) -> anyhow::Result<()> {
    let chats = state.chats().get_chats(query.from.id).await;
    if chats.is_empty() {
        return Ok(());
    }

    let Some(orig_message) = query.message.as_deref() else {
        return Ok(());
    };

    let buttons = chats
        .iter()
        .filter_map(|chat| match chat.title.as_deref() {
            Some(title) => Some(vec![InlineKeyboardButton {
                text: title.to_string(),
                callback_data: CallbackData::SendTo(chat.id),
            }]),
            None => None,
        })
        .collect::<Vec<_>>();

    let payload = serde_json::json!({
        "chat_id": orig_message.chat.id,
        "text": "Выбери чат",
        "reply_markup": InlineKeyboardMarkup {
            inline_keyboard: buttons,
        }
    });

    state.tg_client().send_message(&payload).await;
    state
        .tg_client()
        .answer_callback_query(&query.id, None)
        .await;

    Ok(())
}

async fn handle_chat_button_clicked(
    state: &AppState,
    query: &CallbackQuery,
    target_chat: i64,
) -> anyhow::Result<()> {
    {
        state
            .user_letters()
            .write()
            .await
            .entry(query.from.id)
            .and_modify(|chat| *chat = target_chat)
            .or_insert(target_chat);
    }

    state
        .tg_client()
        .answer_callback_query(&query.id, None)
        .await;

    let Some(orig_message) = query.message.as_deref() else {
        return Ok(());
    };

    let payload = make_bot_text_message(orig_message.chat.id, "Напиши текст сообщения");
    state.tg_client().send_message(&payload).await;

    Ok(())
}

fn make_bot_text_message(chat_id: i64, text: &str) -> serde_json::Value {
    serde_json::json!({
        "chat_id": chat_id,
        "text": text,
    })
}

fn make_bot_main_message(chat_id: i64) -> serde_json::Value {
    serde_json::json!({
        "chat_id": chat_id,
        "text": "Кто выпустил псину? Кто? Кто? Кто?",
        "reply_markup": InlineKeyboardMarkup {
            inline_keyboard: vec![vec![InlineKeyboardButton {
                text: "Отправить сообщение".to_string(),
                callback_data: CallbackData::ActionSend,
            }]]
        }
    })
}
