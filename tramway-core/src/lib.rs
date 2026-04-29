//! Core abstractions for the tramway framework.
//!
//! This crate defines the **port** interfaces that every AI adapter must
//! implement. It sits at the centre of the tramway architecture and carries no
//! dependency on any concrete AI provider, keeping callers fully decoupled from
//! provider-specific details.
//!
//! # Usage
//!
//! ```rust
//! use tramway_core::{Intelligence, IntelligenceContext, MockIntelligence};
//!
//! # #[tokio::main]
//! # async fn main() {
//! let intel = MockIntelligence;
//! let ctx = IntelligenceContext {
//!     input: "Hello".to_string(),
//!     system: "You are a helpful assistant.".to_string(),
//!     history: vec![],
//!     metadata: Default::default(),
//! };
//! let reply = intel.respond(ctx).await.unwrap();
//! # }
//! ```

use std::collections::HashMap;

use async_trait::async_trait;
use thiserror::Error;

/// The top-level error type for all tramway operations.
///
/// Variants are intentionally broad so adapters can wrap provider-specific
/// errors without exposing implementation details to callers.
#[derive(Debug, Error)]
pub enum TramwayError {
    /// An error that occurred during an intelligence (AI provider) call.
    ///
    /// The inner string carries a human-readable description of what went wrong.
    #[error("intelligence error: {0}")]
    Intelligence(String),
}

/// The speaker role for a single entry in a conversation history.
///
/// Providers map these roles to their own API concepts; adapters are
/// responsible for translating them correctly before forwarding the request.
#[derive(Debug, Clone)]
pub enum HistoryRole {
    /// A message authored by the end user.
    User,
    /// A message authored by the AI assistant.
    Assistant,
    /// A system-level instruction that shapes model behaviour.
    System,
}

/// A single message in a multi-turn conversation.
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    /// Who produced this message.
    pub role: HistoryRole,
    /// The text of the message.
    pub content: String,
}

/// Everything a provider needs to generate a response.
///
/// Build one of these and pass it to [`Intelligence::respond`] to obtain a
/// completion from the underlying adapter.
#[derive(Debug, Clone)]
pub struct IntelligenceContext {
    /// The user's current prompt or question.
    pub input: String,
    /// A system prompt that sets the model's persona or constraints.
    /// May be empty if no system-level instruction is required.
    pub system: String,
    /// Previous turns in the conversation, in chronological order.
    /// An empty `Vec` represents a fresh, single-turn request.
    pub history: Vec<HistoryEntry>,
    /// Arbitrary key-value pairs that adapters may forward to their provider
    /// (e.g. `"temperature"`, `"max_tokens"`) or use for internal routing.
    pub metadata: HashMap<String, String>,
}

/// The core port that every AI adapter must implement.
///
/// Adapters wrap a concrete AI provider (Ollama, Claude, …) and expose it
/// through this single async method, keeping callers provider-agnostic.
#[async_trait]
pub trait Intelligence {
    /// Ask the underlying provider to respond to `context`.
    ///
    /// Returns the provider's reply as a plain string, or a [`TramwayError`]
    /// if the request fails for any reason.
    async fn respond(&self, context: IntelligenceContext) -> Result<String, TramwayError>;
}

/// A test-only [`Intelligence`] that echoes the input back without calling any
/// external service.
///
/// Use this in unit tests and examples where a live AI provider is not
/// available or not desirable.
pub struct MockIntelligence;

#[async_trait]
impl Intelligence for MockIntelligence {
    /// Returns a fixed string containing the original input so that tests can
    /// assert on recognisable content without needing a live provider.
    async fn respond(&self, context: IntelligenceContext) -> Result<String, TramwayError> {
        Ok(format!("mock response to: {}", context.input))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    fn make_context(input: &str) -> IntelligenceContext {
        IntelligenceContext {
            input: input.to_string(),
            system: "system".to_string(),
            history: vec![],
            metadata: Default::default(),
        }
    }

    #[tokio::test]
    async fn mock_responds_to_input() {
        // Given a MockIntelligence and a context with input "test input"
        // When respond is called
        // Then the response contains "test input"
        let intel = MockIntelligence;
        let ctx = make_context("test input");
        let reply = intel.respond(ctx).await.unwrap();
        assert!(reply.contains("test input"));
    }

    #[tokio::test]
    async fn context_history_can_be_built() {
        // Given an IntelligenceContext with one HistoryEntry for each HistoryRole variant
        // When the context is constructed
        // Then the history has three entries
        let roles = [HistoryRole::User, HistoryRole::Assistant, HistoryRole::System];
        let history: Vec<HistoryEntry> = roles.iter().map(|role| HistoryEntry {
            role: role.clone(),
            content: format!("msg for {:?}", role),
        }).collect();
        let ctx = IntelligenceContext {
            input: "input".to_string(),
            system: "system".to_string(),
            history: history.clone(),
            metadata: Default::default(),
        };
        assert_eq!(ctx.history.len(), 3);
    }
}
