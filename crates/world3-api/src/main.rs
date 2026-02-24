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
    axum::serve(listener, app).await?;

    Ok(())
}
