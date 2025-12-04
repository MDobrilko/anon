use std::ops::Deref;

use axum::{
    extract::OptionalFromRequestParts,
    http::{StatusCode, request::Parts},
};

use crate::log::error;

const API_SECRET_TOKEN_HEADER_NAME: &str = "X-Telegram-Bot-Api-Secret-Token";

pub struct ApiSecretToken(String);

impl Deref for ApiSecretToken {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}

impl<S: Send + Sync> OptionalFromRequestParts<S> for ApiSecretToken {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Option<Self>, Self::Rejection> {
        let Some(header) = parts.headers.get(API_SECRET_TOKEN_HEADER_NAME) else {
            return Ok(None);
        };

        header
            .to_str()
            .map(|token| Some(Self(token.to_string())))
            .map_err(|err| {
                error!("Failed to convert header \"{API_SECRET_TOKEN_HEADER_NAME}\" to str: {err}");
                StatusCode::BAD_REQUEST
            })
    }
}
