mod ollama_detect;
mod registry;
mod routes;
mod state;

use axum::{Router, routing::{get, post}};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{error, info, warn};

use ollama_detect::detect_ollama;
use registry::AdapterRegistry;
use state::AppState;

#[tokio::main]
async fn main() {
    // Initialise tracing — RUST_LOG controls verbosity, default to info
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "tramway_server=info".into()),
        )
        .init();

    info!("Tramway server starting up");

    // ── Ollama detection ──────────────────────────────────────────────────
    // If OLLAMA_BASE_URL is set explicitly, use it. Otherwise probe candidates.
    let ollama_url = match std::env::var("OLLAMA_BASE_URL") {
        Ok(url) => {
            info!("Using explicit OLLAMA_BASE_URL: {url}");
            Some(url)
        }
        Err(_) => {
            info!("OLLAMA_BASE_URL not set — probing for Ollama...");
            detect_ollama().await
        }
    };

    // ── API key detection ─────────────────────────────────────────────────
    let anthropic_api_key = std::env::var("ANTHROPIC_API_KEY").ok();

    // ── Provider availability summary ─────────────────────────────────────
    match (&ollama_url, &anthropic_api_key) {
        (None, None) => {
            error!("No intelligence providers are configured.");
            error!("Set ANTHROPIC_API_KEY to enable Claude, or start Ollama to enable local models.");
            error!("Tramway cannot start without at least one provider.");
            std::process::exit(1);
        }
        (Some(url), None) => {
            info!("Ollama detected at {url}");
            warn!("ANTHROPIC_API_KEY not set — Claude will not be available");
        }
        (None, Some(_)) => {
            info!("Claude configured");
            warn!("Ollama not detected — ollama/* models will not be available");
        }
        (Some(url), Some(_)) => {
            info!("Ollama detected at {url}");
            info!("Claude configured");
        }
    }

    // ── Adapter registry ─────────────────────────────────────────────────
    let registry = AdapterRegistry::new(ollama_url, anthropic_api_key);
    let state = AppState::new(registry);

    // ── Router ───────────────────────────────────────────────────────────
    let app = Router::new()
        // OpenAI-compatible endpoints
        .route("/v1/chat/completions", post(routes::openai::chat_completions))
        .route("/v1/models", get(routes::openai::list_models))
        // Health
        .route("/health", get(routes::health::health))
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    // ── Bind ─────────────────────────────────────────────────────────────
    let port: u16 = std::env::var("TRAMWAY_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
