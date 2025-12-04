use serde::{Deserialize, Serialize};

pub type ChatId = i64;

#[derive(Serialize, Deserialize)]
pub struct UpdateMessage {
    pub update_id: i32,
    pub message: Option<Message>,
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub message_id: i32,
    pub from: Option<User>,
    pub chat: Chat,
    pub text: Option<String>,
    pub date: i32,
    pub callback_query: Option<CallbackQuery>,
}

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub is_bot: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Chat {
    pub id: ChatId,
    #[serde(rename = "type")]
    pub chat_type: ChatType,
    pub title: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatType {
    Private,
    Group,
    Supergroup,
    Channel,
}

#[derive(Serialize, Deserialize)]
pub struct CallbackQuery {
    pub id: String,
    pub from: User,
    pub message: Option<Box<Message>>, // This is MaybeInaccessibleMessage in docs but for now it's fields are almost common with regular
    pub data: Option<CallbackData>,
}

#[derive(Serialize, Deserialize)]
pub struct InlineKeyboardMarkup {
    inline_keyboard: Vec<InlineKeyboardButton>,
}

#[derive(Serialize, Deserialize)]
pub struct InlineKeyboardButton {
    text: String,
    callback_data: CallbackData,
}

#[derive(Serialize, Deserialize)]
pub enum CallbackData {
    ActionSend,
}
