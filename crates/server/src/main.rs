#![deny(unsafe_code)]

mod config;
mod middleware;
mod routes;
mod state;
mod templates;

use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use tracing_subscriber::EnvFilter;

use agentbin_core::FileStorage;

use config::{LogFormat, ServerConfig};
use routes::create_router;
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = ServerConfig::from_env();

    // Initialise tracing subscriber based on configured log format.
    match config.log_format {
        LogFormat::Json => {
            tracing_subscriber::fmt()
                .json()
                .with_env_filter(EnvFilter::from_default_env())
                .init();
        }
        LogFormat::Pretty => {
            tracing_subscriber::fmt()
                .with_env_filter(EnvFilter::from_default_env())
                .init();
        }
    }

    tracing::info!(
        storage_path = %config.storage_path,
        listen_addr  = %config.listen_addr,
        base_url     = %config.base_url,
        "agentbin-server starting",
    );

    let storage =
        FileStorage::new(&config.storage_path).context("Failed to initialise file storage")?;

    let state = AppState {
        storage: Arc::new(storage),
        base_url: config.base_url.clone(),
    };

    let router = create_router(state);

    let listener = tokio::net::TcpListener::bind(&config.listen_addr)
        .await
        .with_context(|| format!("Failed to bind to {}", config.listen_addr))?;

    tracing::info!(addr = %listener.local_addr()?, "listening");

    // Use an mpsc channel to decouple signal detection from the graceful-shutdown
    // trigger so that we can apply a 30-second drain timeout after the signal.
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel::<()>(1);

    // Spawn the server in a task so main can await the drain with a timeout.
    let server_task = tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(async move {
                shutdown_rx.recv().await;
                tracing::info!("graceful shutdown triggered, draining connections");
            })
            .await
    });

    // Block until a termination signal is received.
    shutdown_signal().await;
    tracing::info!("shutdown signal received");

    // Signal the server to stop accepting new connections.
    let _ = shutdown_tx.send(()).await;

    // Wait up to 30 s for in-flight connections to drain.
    match tokio::time::timeout(Duration::from_secs(30), server_task).await {
        Ok(Ok(Ok(()))) => tracing::info!("server shut down gracefully"),
        Ok(Ok(Err(e))) => {
            tracing::error!(error = %e, "server error during shutdown");
            return Err(anyhow::anyhow!("server error: {e}"));
        }
        Ok(Err(e)) => {
            tracing::error!(error = %e, "server task panicked");
            return Err(anyhow::anyhow!("server task panicked: {e}"));
        }
        Err(_) => tracing::warn!("graceful shutdown timed out after 30 s, forcing exit"),
    }

    tracing::info!("agentbin-server stopped");
    Ok(())
}

/// Wait for SIGINT (Ctrl-C) or SIGTERM.
async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        if let Err(e) = signal::ctrl_c().await {
            tracing::error!(error = %e, "failed to install Ctrl-C handler");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match signal::unix::signal(signal::unix::SignalKind::terminate()) {
            Ok(mut sig) => {
                sig.recv().await;
            }
            Err(e) => {
                tracing::error!(error = %e, "failed to install SIGTERM handler");
                std::future::pending::<()>().await;
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {}
        () = terminate => {}
    }
}
