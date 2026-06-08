//! OpenAI-compatible protocol adapter for Tramway.
//!
//! Implements the OpenAI chat completions wire format, translating to and from
//! [`IntelligenceContext`]. Any client that speaks OpenAI can talk to Tramway
//! without modification — just point it at the right base URL.
//!
//! # Model naming
//!
//! Tramway uses a namespaced model string to select the backend adapter:
//!
//! ```text
//! "ollama/phi4"        → OllamaIntelligence, model = "phi4"
//! "claude/sonnet"      → ClaudeIntelligence, model = "sonnet"
//! ```
//!
//! Standard OpenAI model names (e.g. `"gpt-4o"`) are passed through as-is
//! once an OpenAI backend adapter exists.
//!
//! # Extensions
//!
//! Tramway-specific behaviour can be requested via the optional `x_tramway`
//! field in the request body. Standard OpenAI clients that don't know about
//! this field will simply omit it and everything works normally.
//!
//! ```json
//! {
//!   "model": "ollama/phi4",
//!   "messages": [...],
//!   "x_tramway": {
//!     "extensions": ["trace"],
//!     "trace_id": "abc-123"
//!   }
//! }
//! ```

pub mod error;
pub mod model;
pub mod request;
pub mod response;

pub use error::ProtocolError;
pub use model::ModelTarget;
pub use request::{ChatRequest, Message, Role, TramwayExtensions};
pub use response::{
    ChatCompletionChunk, ChatResponse, Choice, ChunkChoice, DeltaMessage, ResponseMessage,
    StreamEncoder, Usage,
};

use tramway_core::{HistoryEntry, HistoryRole, IntelligenceContext};

/// Translate an [`IntelligenceContext`] back into a [`ChatResponse`].
///
/// `original_request` is needed to echo the model name and generate
/// a consistent response envelope, as the OpenAI spec requires.
pub fn encode(output: String, original_request: &ChatRequest) -> ChatResponse {
    ChatResponse::from_output(output, &original_request.model)
}

/// Translate a [`ChatRequest`] into an [`IntelligenceContext`].
///
/// Returns both the context and the parsed [`ModelTarget`] so the caller
/// can select the right backend adapter without re-parsing the model string.
pub fn decode(req: ChatRequest) -> Result<(IntelligenceContext, ModelTarget), ProtocolError> {
    let target = ModelTarget::parse(&req.model)?;
    let ctx = req.into_context();
    Ok((ctx, target))
}

// ---------------------------------------------------------------------------
// Conversion: ChatRequest → IntelligenceContext
// ---------------------------------------------------------------------------

impl ChatRequest {
    fn into_context(self) -> IntelligenceContext {
        let mut system = String::new();
        let mut history: Vec<HistoryEntry> = Vec::new();
        let mut input = String::new();

        // Partition messages into system prompt, history, and final user input.
        // OpenAI convention: system is first, last user message is the input,
        // everything in between is history.
        let mut non_system: Vec<Message> = Vec::new();

        for msg in self.messages {
            match msg.role {
                Role::System => {
                    if !system.is_empty() {
                        system.push('\n');
                    }
                    system.push_str(&msg.content);
                }
                _ => non_system.push(msg),
            }
        }

        if let Some(last) = non_system.last() {
            if last.role == Role::User {
                input = last.content.clone();
                let history_msgs = &non_system[..non_system.len() - 1];
                history = history_msgs.iter().map(HistoryEntry::from).collect();
            } else {
                // Last message isn't from user — treat all as history, input empty
                history = non_system.iter().map(HistoryEntry::from).collect();
            }
        }

        let mut metadata = std::collections::HashMap::new();

        // Carry optional sampling params as metadata so adapters can use them
        if let Some(temp) = self.temperature {
            metadata.insert("temperature".to_string(), temp.to_string());
        }
        if let Some(max_tokens) = self.max_tokens {
            metadata.insert("max_tokens".to_string(), max_tokens.to_string());
        }

        // Carry extension trace_id if present
        if let Some(ext) = &self.x_tramway {
            if let Some(trace_id) = &ext.trace_id {
                metadata.insert("trace_id".to_string(), trace_id.clone());
            }
        }

        IntelligenceContext {
            input,
            system,
            history,
            metadata,
        }
    }
}

impl From<&Message> for HistoryEntry {
    fn from(msg: &Message) -> Self {
        HistoryEntry {
            role: match msg.role {
                Role::User => HistoryRole::User,
                Role::Assistant => HistoryRole::Assistant,
                Role::System => HistoryRole::System,
            },
            content: msg.content.clone(),
        }
    }
}
