use std::sync::Arc;

use tokio_util::sync::CancellationToken;

use crate::config::Config;

#[derive(Clone)]
pub struct AppState(Arc<AppStateInner>);

impl AppState {
    pub fn new(config: Config) -> Self {
        Self(Arc::new(AppStateInner {
            config,
            cancellation_token: CancellationToken::new(),
        }))
    }

    pub fn cancellation_token(&self) -> &CancellationToken {
        &self.0.cancellation_token
    }

    pub fn config(&self) -> &Config {
        &self.0.config
    }
}

struct AppStateInner {
    config: Config,
    cancellation_token: CancellationToken,
}
