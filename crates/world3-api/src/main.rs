mod error;
mod models;
mod routes;
mod state;

use state::init_app_state;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Init tracing from RUST_LOG
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // 2. Build AppState
    let state = init_app_state();

    // 3. Build router
    let app = routes::build_router(state);

    // 4. Bind and serve
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".into());
    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Listening on {addr}");
    let server = axum::serve(listener, app).with_graceful_shutdown(shutdown_signal());

    // Drain timeout: Fly.io sends SIGTERM and expects exit within kill_timeout (10s).
    // If long-running connections (e.g. WebSockets) don't close in time, exit cleanly
    // rather than waiting for SIGKILL.
    match tokio::time::timeout(std::time::Duration::from_secs(15), server).await {
        Ok(result) => result?,
        Err(_) => tracing::warn!("Drain timeout reached, forcing shutdown"),
    }

    tracing::info!("Server shut down gracefully");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => tracing::info!("Received Ctrl+C, shutting down"),
        _ = terminate => tracing::info!("Received SIGTERM, shutting down"),
    }
}
