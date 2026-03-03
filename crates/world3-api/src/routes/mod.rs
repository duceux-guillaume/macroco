use std::sync::Arc;

use axum::{
    http::Method,
    routing::{get, post, put},
    Router,
};
use tower_http::{
    cors::{Any, CorsLayer},
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

use crate::state::AppState;

mod health;
mod params;
pub mod scenarios;
pub mod ws;

pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(Any);

    let api = Router::new()
        .route("/health", get(health::health))
        .route("/params/schema", get(params::schema))
        // Scenarios collection
        .route("/scenarios", get(scenarios::list_scenarios).post(scenarios::create_scenario))
        .route("/presets", get(scenarios::list_presets))
        // Individual scenario
        .route("/scenarios/:id", get(scenarios::get_scenario).delete(scenarios::delete_scenario))
        .route("/scenarios/:id/params", put(scenarios::update_params))
        .route("/scenarios/:id/run", post(scenarios::run_scenario))
        // WebSocket
        .route("/ws", get(ws::ws_handler));

    let static_dir = std::env::var("STATIC_DIR").unwrap_or_else(|_| "./static".into());

    let mut router = Router::new()
        .nest("/api/v1", api)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(Arc::new(state));

    // Serve frontend static files if the directory exists
    let static_path = std::path::Path::new(&static_dir);
    if static_path.exists() {
        let index = static_path.join("index.html");
        if !index.exists() {
            tracing::warn!(
                "index.html not found in {static_dir} — SPA fallback will return 404"
            );
        }
        let spa_fallback = ServeDir::new(&static_dir).fallback(ServeFile::new(index));
        router = router.fallback_service(spa_fallback);
        tracing::info!("Serving static files from {static_dir}");
    } else {
        tracing::info!("No static dir at {static_dir} — API-only mode");
    }

    router
}
