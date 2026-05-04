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
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "tramway_server=info".into()),
        )
        .init();

    info!("Tramway server starting up");

    // ── Ollama detection ──────────────────────────────────────────────────
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
    let openai_api_key = std::env::var("OPENAI_API_KEY").ok();
    let gemini_api_key = std::env::var("GEMINI_API_KEY").ok();

    // ── Provider availability summary ─────────────────────────────────────
    let any_configured = ollama_url.is_some()
        || anthropic_api_key.is_some()
        || openai_api_key.is_some()
        || gemini_api_key.is_some();

    if !any_configured {
        error!("No intelligence providers are configured.");
        error!("Set ANTHROPIC_API_KEY, OPENAI_API_KEY, or GEMINI_API_KEY, or start Ollama.");
        error!("Tramway cannot start without at least one provider.");
        std::process::exit(1);
    }

    if let Some(ref url) = ollama_url {
        info!("Ollama detected at {url}");
    } else {
        warn!("Ollama not detected — ollama/* models will not be available");
    }
    if anthropic_api_key.is_some() {
        info!("Claude configured");
    } else {
        warn!("ANTHROPIC_API_KEY not set — Claude will not be available");
    }
    if openai_api_key.is_some() {
        info!("OpenAI configured");
    } else {
        warn!("OPENAI_API_KEY not set — OpenAI models will not be available");
    }
    if gemini_api_key.is_some() {
        info!("Gemini configured");
    } else {
        warn!("GEMINI_API_KEY not set — Gemini models will not be available");
    }

    // ── Adapter registry ─────────────────────────────────────────────────
    let registry = AdapterRegistry::new(ollama_url, anthropic_api_key, openai_api_key, gemini_api_key);
    let state = AppState::new(registry);

    // ── Router ───────────────────────────────────────────────────────────
    let app = Router::new()
        .route("/v1/chat/completions", post(routes::openai::chat_completions))
        .route("/v1/models", get(routes::openai::list_models))
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
