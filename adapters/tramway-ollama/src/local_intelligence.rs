//! [`Intelligence`] implementation backed by a locally running Ollama daemon.
//!
//! [Ollama](https://ollama.com) serves large language models over a local HTTP
//! API, enabling on-device inference without sending data to an external
//! service. This module provides [`LocalIntelligence`], which connects to that
//! daemon via its `/api/chat` endpoint.
//!
//! The default Ollama address is `http://localhost:11434`, matching the port
//! that `ollama serve` listens on out of the box.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tramway_core::{HistoryRole, Intelligence, IntelligenceContext, TramwayError};

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

// ---------------------------------------------------------------------------
// LocalIntelligence — adapter for the Ollama local LLM API
// ---------------------------------------------------------------------------

/// Adapter that implements [`Intelligence`] by forwarding requests to a locally
/// running [Ollama](https://ollama.com/) instance via its `/api/chat` endpoint.
///
/// The struct holds no per-request mutable state, so a single instance can be
/// shared across concurrent callers.
pub struct LocalIntelligence {
    /// Base URL of the Ollama server, without a trailing slash
    /// (e.g. `"http://localhost:11434"`).
    base_url: String,
    /// Name of the model to use for completions (e.g. `"llama3"`, `"mistral"`).
    model: String,
    /// Shared HTTP client; reusing it enables connection pooling across calls.
    client: reqwest::Client,
}

impl LocalIntelligence {
    /// Create a new [`LocalIntelligence`] adapter.
    ///
    /// # Arguments
    /// * `base_url` – Base URL of the Ollama server (e.g. `"http://localhost:11434"`).
    /// * `model`    – Name of the model to use (e.g. `"llama3"`).
    pub fn new(base_url: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            model: model.into(),
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
}

#[async_trait]
impl Intelligence for LocalIntelligence {
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
        let mut messages: Vec<OllamaMessage> = Vec::with_capacity(context.history.len() + 2);

        // System prompt first (if provided).
        if !context.system.is_empty() {
            messages.push(OllamaMessage {
                role: "system",
                content: context.system,
            });
        }

        // Prior conversation turns.
        for entry in &context.history {
            messages.push(OllamaMessage {
                role: Self::role_str(&entry.role),
                content: entry.content.clone(),
            });
        }

        // Current user input.
        messages.push(OllamaMessage {
            role: "user",
            content: context.input,
        });

        let request_body = OllamaChatRequest {
            model: self.model.clone(),
            messages,
            stream: false,
        };

        let url = format!("{}/api/chat", self.base_url);

        let response = self
            .client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| TramwayError::Intelligence(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            // Attempt to include the body in the error for easier debugging;
            // fall back gracefully if the body itself cannot be read.
            let body = response
                .text()
                .await
                .unwrap_or_else(|e| format!("<failed to read body: {e}>"));
            return Err(TramwayError::Intelligence(format!(
                "Ollama returned {status}: {body}"
            )));
        }

        // Deserialise the JSON response into the expected shape.
        let chat_response: OllamaChatResponse = response
            .json()
            .await
            .map_err(|e| TramwayError::Intelligence(e.to_string()))?;

        Ok(chat_response.message.content)
    }
}
