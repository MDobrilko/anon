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
    pub id: i64,
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
    #[serde(with = "callback_data_as_string")]
    pub callback_data: CallbackData,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CallbackData {
    ActionSend,
    SendTo(i64),
}

mod callback_data_as_string {
    use serde::{Deserializer, Serializer, de::Visitor};

    use super::CallbackData;

    pub fn serialize<S: Serializer>(
        callback_data: &CallbackData,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let serialized = match callback_data {
            CallbackData::ActionSend => "ActionSend".to_string(),
            CallbackData::SendTo(target) => target.to_string(),
        };

        serializer.serialize_str(&serialized)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<CallbackData, D::Error> {
        struct StrVisitor;

        impl<'de> Visitor<'de> for StrVisitor {
            type Value = CallbackData;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "a string")
            }

            fn visit_str<E>(self, v: &str) -> Result<CallbackData, E>
            where
                E: serde::de::Error,
            {
                if v == "ActionSend" {
                    return Ok(CallbackData::ActionSend);
                }

                v.parse::<i64>().map(CallbackData::SendTo).map_err(|err| {
                    E::custom(format!("Failed to parse callback data as send to: {err}"))
                })
            }
        }

        deserializer.deserialize_string(StrVisitor)
    }
}
