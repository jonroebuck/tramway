use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tramway_core::{HistoryRole, Intelligence, IntelligenceContext, TramwayError};

// ---------------------------------------------------------------------------
// Ollama API request / response types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
struct OllamaMessage {
    role: &'static str,
    content: String,
}

#[derive(Debug, Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    message: OllamaChatMessage,
}

#[derive(Debug, Deserialize)]
struct OllamaChatMessage {
    content: String,
}

// ---------------------------------------------------------------------------
// LocalIntelligence — adapter for the Ollama local LLM API
// ---------------------------------------------------------------------------

/// Adapter that implements [`Intelligence`] by forwarding requests to a locally
/// running [Ollama](https://ollama.com/) instance via its `/api/chat` endpoint.
pub struct LocalIntelligence {
    base_url: String,
    model: String,
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
            let body = response
                .text()
                .await
                .unwrap_or_else(|e| format!("<failed to read body: {e}>"));
            return Err(TramwayError::Intelligence(format!(
                "Ollama returned {status}: {body}"
            )));
        }

        let chat_response: OllamaChatResponse = response
            .json()
            .await
            .map_err(|e| TramwayError::Intelligence(e.to_string()))?;

        Ok(chat_response.message.content)
    }
}
