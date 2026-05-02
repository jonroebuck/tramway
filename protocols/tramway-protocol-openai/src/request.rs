use serde::{Deserialize, Serialize};

/// An OpenAI-compatible chat completions request.
///
/// Standard OpenAI clients will populate `model` and `messages`.
/// The optional `x_tramway` field carries Tramway-specific extensions
/// and is ignored by clients that don't know about it.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChatRequest {
    /// The model to use. Tramway uses a `provider/model` naming convention:
    /// `"ollama/phi4"`, `"claude/sonnet"`. Plain names are passed through.
    pub model: String,

    /// Conversation messages in OpenAI format.
    pub messages: Vec<Message>,

    /// Sampling temperature (0.0–2.0). Passed to the backend if supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Maximum tokens to generate. Passed to the backend if supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,

    /// Whether to stream the response. Reserved for future use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// Optional Tramway-specific extensions. Ignored by standard OpenAI clients.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x_tramway: Option<TramwayExtensions>,
}

/// A single message in the conversation.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

/// OpenAI message roles.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

/// Tramway-specific extensions, passed in the `x_tramway` field.
///
/// These allow callers to opt into Tramway behaviour without breaking
/// standard OpenAI clients, which simply omit this field.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct TramwayExtensions {
    /// Named extensions to activate, e.g. `["trace", "cache"]`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Vec<String>>,

    /// Optional trace ID for observability.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,

    /// Routing hint — prefer local model if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefer_local: Option<bool>,
}
