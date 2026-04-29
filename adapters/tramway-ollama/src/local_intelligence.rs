//! [`Intelligence`] implementation for a locally running Ollama daemon.
//!
//! Ollama (<https://ollama.com>) serves large language models over a local HTTP
//! API, making it possible to run inference entirely on-device without sending
//! data to an external service. This module provides [`LocalOllamaIntelligence`],
//! which connects to that local daemon.
//!
//! The default base URL is `http://localhost:11434`, matching the address that
//! `ollama serve` listens on out of the box. Override it by constructing
//! [`LocalOllamaIntelligence`] with [`LocalOllamaIntelligence::with_base_url`].

use async_trait::async_trait;
use tramway_core::{Intelligence, IntelligenceContext, TramwayError};

/// Default address used by `ollama serve` when no explicit host is configured.
const DEFAULT_BASE_URL: &str = "http://localhost:11434";

/// [`Intelligence`] adapter that sends requests to a locally running Ollama
/// daemon.
///
/// Each instance is bound to a single model name and base URL. The struct holds
/// no per-request mutable state, so a single instance can be shared across
/// concurrent callers.
pub struct LocalOllamaIntelligence {
    /// The Ollama model to use for completions (e.g. `"llama3"`, `"mistral"`).
    model: String,
    /// Base URL of the Ollama HTTP API, without a trailing slash.
    base_url: String,
}

impl LocalOllamaIntelligence {
    /// Create a new adapter that targets the default local Ollama address
    /// (`http://localhost:11434`) with the given model name.
    pub fn new(model: String) -> Self {
        Self {
            model,
            base_url: DEFAULT_BASE_URL.to_string(),
        }
    }

    /// Create a new adapter with an explicit base URL, useful when Ollama is
    /// running on a non-default port or inside a container.
    ///
    /// `base_url` should not include a trailing slash
    /// (e.g. `"http://localhost:8080"`).
    pub fn with_base_url(model: String, base_url: String) -> Self {
        Self { model, base_url }
    }
}

#[async_trait]
impl Intelligence for LocalOllamaIntelligence {
    /// Send `context` to the local Ollama daemon and return the model's reply.
    ///
    /// Builds a POST request to `<base_url>/api/generate`, serialises the
    /// prompt and conversation history into the Ollama request format, and
    /// returns the model output as a plain string.
    ///
    /// # Errors
    ///
    /// Returns [`TramwayError::Intelligence`] if:
    ///
    /// - The HTTP request cannot be sent (e.g. the daemon is not running).
    /// - The response status indicates an error.
    /// - The response body cannot be deserialised.
    async fn respond(&self, _context: IntelligenceContext) -> Result<String, TramwayError> {
        // Construct the generate endpoint from the configured base URL.
        let _url = format!("{}/api/generate", self.base_url);
        // Include the model name in the request body so Ollama knows which
        // locally available model to run.
        let _model = &self.model;
        todo!()
    }
}
