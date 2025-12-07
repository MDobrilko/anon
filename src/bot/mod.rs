use std::net::SocketAddr;

use crate::{
    config::Config,
    log::{debug, error, info},
    state::AppState,
};

use anyhow::Context;
use axum_server::tls_rustls::RustlsConfig;
use futures::future::maybe_done;
use tokio::{
    signal::unix::{SignalKind, signal},
    sync::mpsc::Receiver,
};
use tokio_util::sync::CancellationToken;

pub use api::entities;

mod api;
pub mod client;

pub fn setup(config: Config) -> anyhow::Result<()> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async move {
            let tg_client = client::Client::new(&config)?;
            tg_client.setup(&config).await?;

            Ok(())
        })
}

pub fn start(config: Config) -> anyhow::Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(run_app(config))
}

async fn run_app(config: Config) -> anyhow::Result<()> {
    let state = AppState::new(config)
        .await
        .context("Failed to create app state")?;

    let mut web_handle = std::pin::pin!(maybe_done(tokio::spawn(run_server(state.clone()))));
    let mut shutdown_rx = spawn_shutdown_signal_watcher(state.cancellation_token().clone())?;

    tokio::select! {
        biased;
        _ = &mut web_handle => {},
        _ = shutdown_rx.recv() => {},
    }

    state.cancellation_token().cancel();

    tokio::select! {
        biased;
        _ = &mut web_handle => {
            match web_handle.take_output() {
                Some(res) => res?,
                None => Ok(()),
            }
        },
        _ = shutdown_rx.recv() => { info!("Terminating"); Ok(()) },
    }
}

fn spawn_shutdown_signal_watcher(ct: CancellationToken) -> anyhow::Result<Receiver<()>> {
    let mut interrupt = signal(SignalKind::interrupt())?;
    let mut terminate = signal(SignalKind::terminate())?;

    let (sender, receiver) = tokio::sync::mpsc::channel(2);

    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = interrupt.recv() => {},
                _ = terminate.recv() => {},
                _ = ct.cancelled() => break,
            }

            info!("Shutting down");
            if let Err(err) = sender.send(()).await {
                error!("Failed to send shutdown signal: {err}");
            }
        }
    });

    Ok(receiver)
}

async fn run_server(state: AppState) -> anyhow::Result<()> {
    info!("Starting ...");
    debug!("Run server with config: {:#?}", state.config());

    let addr = SocketAddr::from(([0, 0, 0, 0], state.config().http.port));
    let tls_config =
        RustlsConfig::from_pem_file(&state.config().http.tls.cert, &state.config().http.tls.key)
            .await?;

    tokio::select! {
        res = axum_server::bind_rustls(addr, tls_config).serve(api::make_router(state.clone()).into_make_service()) => res,
        _ = state.cancellation_token().cancelled() => Ok(())
    }?;

    Ok(())
}
