use std::{collections::HashMap, sync::Arc};

use anyhow::Context;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

use crate::{bot::client::Client as TelegramClient, chats::Chats, config::Config};

#[derive(Clone)]
pub struct AppState(Arc<AppStateInner>);

impl AppState {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        let tg_client = TelegramClient::new(&config)?;
        let chats = Chats::open(&config.chats_storage)
            .await
            .context("Failed to create chats")?;

        Ok(Self(Arc::new(AppStateInner {
            config,
            tg_client,
            chats,
            user_letters: RwLock::new(HashMap::new()),
            cancellation_token: CancellationToken::new(),
        })))
    }

    pub fn config(&self) -> &Config {
        &self.0.config
    }

    pub fn tg_client(&self) -> &TelegramClient {
        &self.0.tg_client
    }

    pub fn chats(&self) -> &Chats {
        &self.0.chats
    }

    pub async fn save_chats(&self) -> anyhow::Result<()> {
        self.0.chats.save(&self.0.config.chats_storage).await
    }

    pub fn user_letters(&self) -> &RwLock<HashMap<i64, i64>> {
        &self.0.user_letters
    }

    pub fn cancellation_token(&self) -> &CancellationToken {
        &self.0.cancellation_token
    }
}

struct AppStateInner {
    config: Config,
    tg_client: TelegramClient,
    chats: Chats,
    user_letters: RwLock<HashMap<i64, i64>>,
    cancellation_token: CancellationToken,
}
