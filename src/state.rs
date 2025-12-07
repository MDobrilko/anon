use std::{collections::HashMap, path::Path, sync::Arc};

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
            .context("Failed to open chats storage")?;
        let user_chats = open_user_chats(&config.user_chats_storage)
            .await
            .context("Failed to open user chats storage")?;

        Ok(Self(Arc::new(AppStateInner {
            config,
            tg_client,
            chats,
            user_chats,
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

    pub fn user_chats(&self) -> &RwLock<HashMap<i64, i64>> {
        &self.0.user_chats
    }

    pub async fn save_user_chats(&self) -> anyhow::Result<()> {
        let contents = { serde_json::to_vec_pretty(&*self.0.user_chats.read().await) }?;
        tokio::fs::write(&self.config().user_chats_storage, contents).await?;

        Ok(())
    }

    pub fn cancellation_token(&self) -> &CancellationToken {
        &self.0.cancellation_token
    }
}

struct AppStateInner {
    config: Config,
    tg_client: TelegramClient,
    chats: Chats,
    user_chats: RwLock<HashMap<i64, i64>>,
    cancellation_token: CancellationToken,
}

async fn open_user_chats(file: &Path) -> anyhow::Result<RwLock<HashMap<i64, i64>>> {
    if !file.exists() {
        return Ok(RwLock::new(HashMap::new()));
    }

    let contents = tokio::fs::read(file)
        .await
        .context("Failed to open user chats storage file")?;

    let user_chats = serde_json::from_slice(&contents)?;

    Ok(RwLock::new(user_chats))
}
