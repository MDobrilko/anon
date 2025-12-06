use std::sync::Arc;

use tokio_util::sync::CancellationToken;

use crate::{bot::client::Client as TelegramClient, config::Config};

#[derive(Clone)]
pub struct AppState(Arc<AppStateInner>);

impl AppState {
    pub fn new(config: Config) -> anyhow::Result<Self> {
        let tg_client = TelegramClient::new(&config)?;
        Ok(Self(Arc::new(AppStateInner {
            config,
            tg_client,
            cancellation_token: CancellationToken::new(),
        })))
    }

    pub fn cancellation_token(&self) -> &CancellationToken {
        &self.0.cancellation_token
    }

    pub fn config(&self) -> &Config {
        &self.0.config
    }

    pub fn tg_client(&self) -> &TelegramClient {
        &self.0.tg_client
    }
}

struct AppStateInner {
    config: Config,
    tg_client: TelegramClient,
    cancellation_token: CancellationToken,
}
