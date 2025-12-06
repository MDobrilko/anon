use serde::{Deserialize, Serialize};

pub type ChatId = i64;

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateMessage {
    pub update_id: i64,
    pub message: Option<Message>,
    pub callback_query: Option<CallbackQuery>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookResponse {
    pub method: String,
    #[serde(flatten)]
    pub params: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub message_id: i32,
    pub from: Option<User>,
    pub chat: Chat,
    pub text: Option<String>,
    pub date: i32,
    pub callback_query: Option<CallbackQuery>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub is_bot: bool,
    pub username: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Chat {
    pub id: ChatId,
    #[serde(rename = "type")]
    pub chat_type: ChatType,
    pub title: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatType {
    Private,
    Group,
    Supergroup,
    Channel,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CallbackQuery {
    pub id: String,
    pub from: User,
    pub message: Option<Box<Message>>, // This is MaybeInaccessibleMessage in docs but for now it's fields are almost common with regular
    pub data: Option<CallbackData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InlineKeyboardMarkup {
    pub inline_keyboard: Vec<Vec<InlineKeyboardButton>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InlineKeyboardButton {
    pub text: String,
    pub callback_data: CallbackData,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CallbackData {
    ActionSend,
}
