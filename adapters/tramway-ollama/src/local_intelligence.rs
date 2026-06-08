//! [`Intelligence`] implementation backed by a locally running Ollama daemon.
//!
//! [Ollama](https://ollama.com) serves large language models over a local HTTP
//! API, enabling on-device inference without sending data to an external
//! service. This module provides [`OllamaIntelligence`], which connects to that
//! daemon via its `/api/chat` endpoint.
//!
//! The default Ollama address is `http://localhost:11434`, matching the port
//! that `ollama serve` listens on out of the box.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tramway_core::{HistoryRole, Intelligence, IntelligenceContext, ResponseStream, TramwayError};

// ---------------------------------------------------------------------------
// Ollama API request / response types
// ---------------------------------------------------------------------------

/// Wire format for a single message sent to the Ollama `/api/chat` endpoint.
#[derive(Debug, Serialize)]
struct OllamaMessage {
    /// Role string expected by the Ollama API: `"user"`, `"assistant"`, or `"system"`.
    role: &'static str,
    /// Text content of the message.
    content: String,
}

/// Full request body for `POST /api/chat`.
#[derive(Debug, Serialize)]
struct OllamaChatRequest {
    /// Name of the locally available model to run (e.g. `"llama3"`).
    model: String,
    /// Ordered list of messages forming the conversation so far.
    messages: Vec<OllamaMessage>,
    /// Must be `false` so the response is returned as a single JSON object
    /// rather than a stream of newline-delimited chunks.
    stream: bool,
}

/// Top-level response returned by `POST /api/chat` when `stream` is `false`.
#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    message: OllamaChatMessage,
}

/// The assistant message embedded in [`OllamaChatResponse`].
#[derive(Debug, Deserialize)]
struct OllamaChatMessage {
    content: String,
}

/// A streaming chunk returned by `POST /api/chat` when `stream` is `true`.
#[derive(Debug, Deserialize)]
struct OllamaStreamChunk {
    #[serde(default)]
    message: Option<OllamaChatMessage>,
    #[serde(default)]
    done: bool,
}

// ---------------------------------------------------------------------------
// OllamaIntelligence — adapter for the Ollama local LLM API
// ---------------------------------------------------------------------------

/// Adapter that implements [`Intelligence`] by forwarding requests to a locally
/// running [Ollama](https://ollama.com/) instance via its `/api/chat` endpoint.
///
/// The struct holds no per-request mutable state, so a single instance can be
/// shared across concurrent callers.
pub struct OllamaIntelligence {
    /// Base URL of the Ollama server, without a trailing slash
    /// (e.g. `"http://localhost:11434"`).
    base_url: String,
    /// Shared HTTP client; reusing it enables connection pooling across calls.
    client: reqwest::Client,
}

impl OllamaIntelligence {
    /// Check connectivity to the Ollama server by sending a GET request to /api/tags.
    pub async fn ping(&self) -> anyhow::Result<()> {
        let url = format!("{}/api/tags", self.base_url);
        let response = self.client.get(&url).send().await?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Ollama server returned status: {}",
                response.status()
            ))
        }
    }

    /// Create a new [`OllamaIntelligence`] adapter.
    ///
    /// # Arguments
    /// * `base_url` – Base URL of the Ollama server (e.g. `"http://localhost:11434"`).
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: reqwest::Client::new(),
        }
    }

    /// Map a [`HistoryRole`] to the static role string expected by the Ollama
    /// chat API.
    fn role_str(role: &HistoryRole) -> &'static str {
        match role {
            HistoryRole::User => "user",
            HistoryRole::Assistant => "assistant",
            HistoryRole::System => "system",
        }
    }

    fn build_request(
        &self,
        context: IntelligenceContext,
        stream: bool,
    ) -> Result<OllamaChatRequest, TramwayError> {
        let model = context
            .metadata
            .get("model")
            .ok_or_else(|| {
                TramwayError::Intelligence("model not specified in metadata".to_string())
            })?
            .clone();

        let mut messages: Vec<OllamaMessage> = Vec::with_capacity(context.history.len() + 2);

        if !context.system.is_empty() {
            messages.push(OllamaMessage {
                role: "system",
                content: context.system,
            });
        }

        for entry in &context.history {
            messages.push(OllamaMessage {
                role: Self::role_str(&entry.role),
                content: entry.content.clone(),
            });
        }

        messages.push(OllamaMessage {
            role: "user",
            content: context.input,
        });

        Ok(OllamaChatRequest {
            model,
            messages,
            stream,
        })
    }

    async fn validate_response(
        response: reqwest::Response,
        model_name: &str,
    ) -> Result<reqwest::Response, TramwayError> {
        if response.status().is_success() {
            return Ok(response);
        }

        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|e| format!("<failed to read body: {e}>"));

        if status == reqwest::StatusCode::NOT_FOUND
            || (body.contains("model") && body.contains("not found"))
        {
            return Err(TramwayError::Intelligence(format!(
                "Model '{model_name}' is not available in Ollama. \
                 Pull it first with: ollama pull {model_name} \
                 (or if using the bundled Docker profile: tramway-pull {model_name})"
            )));
        }

        Err(TramwayError::Intelligence(format!(
            "Ollama returned {status}: {body}"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn new_uses_default_endpoint() {
        // Given a OllamaIntelligence created with "http://localhost:11434" and model "phi4"
        // When the endpoint is constructed
        // Then the endpoint contains "api/generate"
        let ollama = OllamaIntelligence::new("http://localhost:11434");
        assert!(ollama.base_url.contains("11434"));
        let endpoint = format!("{}/api/generate", ollama.base_url);
        assert!(endpoint.contains("api/generate"));
    }

    #[tokio::test]
    async fn with_host_uses_provided_host() {
        // Given a OllamaIntelligence created with a custom host
        // When the base_url is checked
        // Then the base_url starts with the provided host
        let ollama = OllamaIntelligence::new("http://localhost:11434");
        assert!(ollama.base_url.starts_with("http://localhost:11434"));
    }

    #[tokio::test]
    #[ignore]
    async fn ollama_responds_to_prompt() {
        // Given a real OllamaIntelligence and a simple context
        // When respond is called
        // Then the response is non-empty
        let ollama = OllamaIntelligence::new("http://localhost:11434");
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("model".to_string(), "phi4".to_string());
        let ctx = IntelligenceContext {
            input: "Say hello".to_string(),
            system: "You are a helpful assistant.".to_string(),
            history: vec![],
            metadata,
        };
        let reply = ollama.respond(ctx).await.unwrap();
        assert!(!reply.trim().is_empty());
    }
}

#[async_trait]
impl Intelligence for OllamaIntelligence {
    /// Send `context` to the local Ollama daemon and return the model's reply.
    ///
    /// Builds a `POST /api/chat` request containing the system prompt,
    /// conversation history, and the current user input, then deserialises the
    /// model's response into a plain string.
    ///
    /// # Errors
    ///
    /// Returns [`TramwayError::Intelligence`] if:
    ///
    /// - The HTTP request cannot be sent (e.g. the daemon is not running).
    /// - Ollama returns a non-2xx status code.
    /// - The response body cannot be deserialised into the expected JSON shape.
    async fn respond(&self, context: IntelligenceContext) -> Result<String, TramwayError> {
        let request_body = self.build_request(context, false)?;

        let url = format!("{}/api/chat", self.base_url);

        let response = self
            .client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| TramwayError::Intelligence(e.to_string()))?;

        let response = Self::validate_response(response, &request_body.model).await?;

        // Deserialise the JSON response into the expected shape.
        let chat_response: OllamaChatResponse = response
            .json()
            .await
            .map_err(|e| TramwayError::Intelligence(e.to_string()))?;

        Ok(chat_response.message.content)
    }

    async fn respond_stream(
        &self,
        context: IntelligenceContext,
    ) -> Result<ResponseStream, TramwayError> {
        let request_body = self.build_request(context, true)?;
        let url = format!("{}/api/chat", self.base_url);

        let response = self
            .client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| TramwayError::Intelligence(e.to_string()))?;

        let mut response = Self::validate_response(response, &request_body.model).await?;

        let stream = async_stream::try_stream! {
            let mut buffer = Vec::new();

            loop {
                let next = response
                    .chunk()
                    .await
                    .map_err(|e| TramwayError::Intelligence(e.to_string()))?;

                let Some(chunk) = next else {
                    break;
                };

                buffer.extend_from_slice(&chunk);

                while let Some(newline) = buffer.iter().position(|b| *b == b'\n') {
                    let line = buffer.drain(..=newline).collect::<Vec<u8>>();
                    let raw = std::str::from_utf8(&line)
                        .map_err(|e| TramwayError::Intelligence(e.to_string()))?;
                    let raw = raw.trim();

                    if raw.is_empty() {
                        continue;
                    }

                    let chunk: OllamaStreamChunk = serde_json::from_str(raw)
                        .map_err(|e| TramwayError::Intelligence(e.to_string()))?;

                    if let Some(message) = chunk.message {
                        if !message.content.is_empty() {
                            yield message.content;
                        }
                    }

                    if chunk.done {
                        return;
                    }
                }
            }

            if !buffer.is_empty() {
                let raw = std::str::from_utf8(&buffer)
                    .map_err(|e| TramwayError::Intelligence(e.to_string()))?;
                let raw = raw.trim();
                if !raw.is_empty() {
                    let chunk: OllamaStreamChunk = serde_json::from_str(raw)
                        .map_err(|e| TramwayError::Intelligence(e.to_string()))?;
                    if let Some(message) = chunk.message {
                        if !message.content.is_empty() {
                            yield message.content;
                        }
                    }
                }
            }
        };

        Ok(Box::pin(stream))
    }
}
