use axum::{
    extract::State,
    http::StatusCode,
    response::{
        sse::{Event, Sse},
        IntoResponse, Response,
    },
    Json,
};
use serde::Serialize;
use std::convert::Infallible;
use tracing::{info, warn};

use futures_util::StreamExt;
use tramway_protocol_openai::{decode, encode, ChatRequest, StreamEncoder};

use crate::registry::AdapterError;
use crate::state::AppState;

// ── POST /v1/chat/completions ─────────────────────────────────────────────

pub async fn chat_completions(
    State(state): State<AppState>,
    Json(req): Json<ChatRequest>,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    info!("POST /v1/chat/completions model={}", req.model);

    // Decode: OpenAI request → IntelligenceContext + ModelTarget
    let (ctx, target) = decode(req.clone()).map_err(|e| {
        warn!("Protocol decode error: {e}");
        error_response(StatusCode::BAD_REQUEST, &e.to_string())
    })?;

    let stream_requested = req.stream.unwrap_or(false);

    if stream_requested {
        let mut adapter_stream = state
            .registry
            .complete_stream(&target.provider, &target.model, ctx)
            .await
            .map_err(map_adapter_error)?;

        let encoder = StreamEncoder::new(&req.model);
        let event_stream = async_stream::stream! {
            if let Ok(json) = serde_json::to_string(&encoder.role_chunk()) {
                yield Ok::<Event, Infallible>(Event::default().data(json));
            }

            while let Some(next) = adapter_stream.next().await {
                match next {
                    Ok(content) => {
                        match serde_json::to_string(&encoder.content_chunk(content)) {
                            Ok(json) => yield Ok(Event::default().data(json)),
                            Err(e) => {
                                warn!("Failed to encode stream chunk: {e}");
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Streaming adapter error: {e}");
                        break;
                    }
                }
            }

            if let Ok(json) = serde_json::to_string(&encoder.end_chunk()) {
                yield Ok(Event::default().data(json));
            }

            yield Ok(Event::default().data("[DONE]"));
        };

        return Ok(Sse::new(event_stream).into_response());
    }

    // Dispatch to the correct backend adapter
    let output = state
        .registry
        .complete(&target.provider, &target.model, ctx)
        .await
        .map_err(map_adapter_error)?;

    // Encode: output string → OpenAI response shape
    let response = encode(output, &req);

    Ok(Json(response).into_response())
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

fn error_response(status: StatusCode, message: &str) -> (StatusCode, Json<ErrorResponse>) {
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

fn map_adapter_error(e: AdapterError) -> (StatusCode, Json<ErrorResponse>) {
    warn!("Adapter error: {e}");
    match e {
        AdapterError::NotConfigured(_) | AdapterError::UnknownProvider(_) => {
            error_response(StatusCode::BAD_REQUEST, &e.to_string())
        }
        AdapterError::Intelligence(_) => error_response(StatusCode::BAD_GATEWAY, &e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use axum::{body, routing::post, Router};
    use futures_util::stream;
    use tower::ServiceExt;
    use tramway_core::{Intelligence, IntelligenceContext, ResponseStream, TramwayError};

    use crate::{registry::AdapterRegistry, state::AppState};

    struct TestAdapter;

    #[async_trait]
    impl Intelligence for TestAdapter {
        async fn respond(&self, _context: IntelligenceContext) -> Result<String, TramwayError> {
            Ok("final response".to_string())
        }

        async fn respond_stream(
            &self,
            _context: IntelligenceContext,
        ) -> Result<ResponseStream, TramwayError> {
            Ok(Box::pin(stream::iter(vec![
                Ok("hel".to_string()),
                Ok("lo".to_string()),
            ])))
        }
    }

    fn app() -> Router {
        let mut registry = AdapterRegistry::new(None, None, None, None);
        registry.register_external("test", TestAdapter);
        let state = AppState::new(registry);
        Router::new()
            .route("/v1/chat/completions", post(chat_completions))
            .with_state(state)
    }

    #[tokio::test]
    async fn non_stream_request_returns_json() {
        let body = serde_json::json!({
            "model": "test/model-a",
            "messages": [{"role": "user", "content": "hello"}]
        });
        let req = axum::http::Request::builder()
            .method("POST")
            .uri("/v1/chat/completions")
            .header("content-type", "application/json")
            .body(body::Body::from(body.to_string()))
            .unwrap();

        let response = app().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let content_type = response
            .headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(content_type.starts_with("application/json"));

        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["choices"][0]["message"]["content"], "final response");
    }

    #[tokio::test]
    async fn stream_request_returns_sse_with_done() {
        let body = serde_json::json!({
            "model": "test/model-a",
            "stream": true,
            "messages": [{"role": "user", "content": "hello"}]
        });
        let req = axum::http::Request::builder()
            .method("POST")
            .uri("/v1/chat/completions")
            .header("content-type", "application/json")
            .body(body::Body::from(body.to_string()))
            .unwrap();

        let response = app().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let content_type = response
            .headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(content_type.starts_with("text/event-stream"));

        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.contains("chat.completion.chunk"));
        assert!(text.contains("data: [DONE]"));
    }
}
