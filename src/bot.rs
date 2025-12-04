use crate::{config::Config, state::AppState};

use tokio_util::sync::CancellationToken;

mod api;

pub fn start(config: Config) -> anyhow::Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(start_app(config))
}

async fn start_app(config: Config) -> anyhow::Result<()> {
    let state = AppState::new(config);

    spawn_shutdown_signal_watcher(state.cancellation_token().clone());

    run_server(state).await
}

async fn spawn_shutdown_signal_watcher(ct: CancellationToken) {
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await;
        ct.cancel();
    });
}

async fn run_server(state: AppState) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", state.config().http.port)).await?;

    axum::serve(listener, api::make_router(state)).await?;

    Ok(())
}
