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
    pub photo: Option<Vec<PhotoSize>>,
    pub animation: Option<Animation>,
    pub caption: Option<String>,
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
pub struct PhotoSize {
    pub file_id: String,
    pub file_unique_id: String,
    pub width: i64,
    pub height: i64,
    pub file_size: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Animation {
    pub file_id: String,
    pub file_unique_id: String,
    pub width: i64,
    pub height: i64,
    pub duration: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CallbackQuery {
    pub id: String,
    pub from: User,
    pub message: Option<Box<Message>>, // This is MaybeInaccessibleMessage in docs but for now it's fields are almost common with regular
    #[serde(with = "callback_data_as_string_opt")]
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

#[derive(Debug, Serialize)]
pub struct SendPhotoPayload<'a> {
    pub chat_id: i64,
    pub photo: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<&'a str>,
}

#[derive(Debug, Serialize)]
pub struct SendAnimationPayload<'a> {
    pub chat_id: i64,
    pub animation: &'a str,
    pub duration: Option<i64>,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub caption: Option<&'a str>,
}

mod callback_data_as_string {
    use serde::{Deserializer, Serializer, de::Visitor, ser::Error as _};

    use super::CallbackData;

    pub fn serialize<S: Serializer>(
        callback_data: &CallbackData,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(
            &serde_json::to_string(callback_data).map_err(|err| {
                S::Error::custom(format!("Failed to serialize callback data: {err}"))
            })?,
        )
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
                serde_json::from_str(v)
                    .map_err(|err| E::custom(format!("Failed to deserialize callback data: {err}")))
            }
        }

        deserializer.deserialize_string(StrVisitor)
    }
}

mod callback_data_as_string_opt {
    use serde::{Deserializer, Serializer, de::Visitor};

    use super::CallbackData;

    pub fn serialize<S: Serializer>(
        callback_data: &Option<CallbackData>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match callback_data.as_ref() {
            Some(data) => super::callback_data_as_string::serialize(data, serializer),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<CallbackData>, D::Error> {
        struct OptStrVisitor;

        impl<'de> Visitor<'de> for OptStrVisitor {
            type Value = Option<CallbackData>;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "a string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Option<CallbackData>, E>
            where
                E: serde::de::Error,
            {
                serde_json::from_str(v)
                    .map_err(|err| E::custom(format!("Failed to deserialize callback data: {err}")))
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Option<CallbackData>, D::Error>
            where
                D: Deserializer<'de>,
            {
                super::callback_data_as_string::deserialize(deserializer).map(Some)
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(None)
            }
        }

        deserializer.deserialize_option(OptStrVisitor)
    }
}
