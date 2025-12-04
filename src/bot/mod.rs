use crate::{config::Config, log::error, state::AppState};

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

fn spawn_shutdown_signal_watcher(ct: CancellationToken) {
    tokio::spawn(async move {
        if let Err(err) = tokio::signal::ctrl_c().await {
            error!("Failed to receive ctrl+c signal: {err:#}");
        }
        ct.cancel();
    });
}

async fn run_server(state: AppState) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", state.config().http.port)).await?;

    tokio::select! {
        res = axum::serve(listener, api::make_router(state.clone())) => res,
        _ = state.cancellation_token().cancelled() => Ok(())
    }?;

    Ok(())
}
