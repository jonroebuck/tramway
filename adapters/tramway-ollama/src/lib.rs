//! Ollama adapter for the tramway framework.
//!
//! This crate is an **adapter** in the tramway architecture: it bridges the
//! [`Intelligence`] port defined in `tramway-core` and the Ollama HTTP API.
//! Use it to route prompts to a model served by an Ollama instance.
//!
//! Two implementations are provided:
//!
//! - [`OllamaIntelligence`] — a general-purpose entry point re-exported from
//!   [`local_intelligence`].
//! - [`local_intelligence::LocalOllamaIntelligence`] — targets a locally
//!   running Ollama daemon (default `http://localhost:11434`).
//!
//! # Usage
//!
//! ```rust,no_run
//! use tramway_ollama::local_intelligence::LocalOllamaIntelligence;
//! use tramway_core::{Intelligence, IntelligenceContext};
//!
//! # #[tokio::main]
//! # async fn main() {
//! let intel = LocalOllamaIntelligence::new("llama3".to_string());
//! let ctx = IntelligenceContext {
//!     input: "What is 2 + 2?".to_string(),
//!     system: String::new(),
//!     history: vec![],
//!     metadata: Default::default(),
//! };
//! let response = intel.respond(ctx).await.unwrap();
//! # }
//! ```

pub mod local_intelligence;

use async_trait::async_trait;
use tramway_core::{Intelligence, IntelligenceContext, TramwayError};

/// [`Intelligence`] adapter that forwards requests to an Ollama endpoint.
///
/// This is a thin facade over [`local_intelligence::LocalOllamaIntelligence`]
/// for callers that do not need to configure the target host or model directly.
/// For full control, construct a [`local_intelligence::LocalOllamaIntelligence`]
/// directly.
pub struct OllamaIntelligence;

#[async_trait]
impl Intelligence for OllamaIntelligence {
    /// Send `context` to the configured Ollama endpoint and return the model's
    /// reply as a plain string.
    ///
    /// # Errors
    ///
    /// Returns [`TramwayError::Intelligence`] if the HTTP request fails or the
    /// response cannot be parsed.
    async fn respond(&self, _context: IntelligenceContext) -> Result<String, TramwayError> {
        todo!()
    }
}
