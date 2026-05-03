use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::Serialize;
use tracing::{info, warn};

use tramway_protocol_openai::{decode, encode, ChatRequest, ChatResponse};

use crate::registry::AdapterError;
use crate::state::AppState;

// ── POST /v1/chat/completions ─────────────────────────────────────────────

pub async fn chat_completions(
    State(state): State<AppState>,
    Json(req): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("POST /v1/chat/completions model={}", req.model);

    // Decode: OpenAI request → IntelligenceContext + ModelTarget
    let (ctx, target) = decode(req.clone()).map_err(|e| {
        warn!("Protocol decode error: {e}");
        error_response(StatusCode::BAD_REQUEST, &e.to_string())
    })?;

    // Dispatch to the correct backend adapter
    let output = state
        .registry
        .complete(&target.provider, &target.model, ctx)
        .await
        .map_err(|e| {
            warn!("Adapter error: {e}");
            match e {
                AdapterError::NotConfigured(_) | AdapterError::UnknownProvider(_) => {
                    error_response(StatusCode::BAD_REQUEST, &e.to_string())
                }
                AdapterError::Intelligence(_) => {
                    error_response(StatusCode::BAD_GATEWAY, &e.to_string())
                }
            }
        })?;

    // Encode: output string → OpenAI response shape
    let response = encode(output, &req);

    Ok(Json(response))
}

// ── GET /v1/models ────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct ModelsResponse {
    object: &'static str,
    data: Vec<ModelEntry>,
}

#[derive(Serialize)]
pub struct ModelEntry {
    id: String,
    object: &'static str,
    owned_by: String,
}

pub async fn list_models(State(state): State<AppState>) -> Json<ModelsResponse> {
    // Return a model entry per configured provider.
    // Clients can use these IDs directly in chat completions requests.
    let data = state
        .registry
        .available_providers()
        .into_iter()
        .map(|provider| ModelEntry {
            id: format!("{provider}/*"),
            object: "model",
            owned_by: provider.to_string(),
        })
        .collect();

    Json(ModelsResponse {
        object: "list",
        data,
    })
}

// ── Error helpers ─────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct ErrorResponse {
    error: ErrorDetail,
}

#[derive(Serialize)]
pub struct ErrorDetail {
    message: String,
    #[serde(rename = "type")]
    kind: &'static str,
}

fn error_response(
    status: StatusCode,
    message: &str,
) -> (StatusCode, Json<ErrorResponse>) {
    (
        status,
        Json(ErrorResponse {
            error: ErrorDetail {
                message: message.to_string(),
                kind: "tramway_error",
            },
        }),
    )
}
