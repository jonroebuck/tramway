use std::sync::Arc;
use tramway_core::{Intelligence, IntelligenceContext, TramwayError};
use tramway_ollama::OllamaIntelligence;
use tramway_claude::ClaudeIntelligence;

/// Holds configured adapter instances and routes by provider name.
///
/// This is built once at startup and shared across all requests via `AppState`.
/// Adding a new backend adapter means adding a field here and a match arm in
/// `get()`.
pub struct AdapterRegistry {
    ollama: Option<Arc<OllamaIntelligence>>,
    claude: Option<Arc<ClaudeIntelligence>>,
}

impl AdapterRegistry {
    pub fn new(ollama_url: Option<String>, anthropic_api_key: Option<String>) -> Self {
        let ollama = ollama_url.map(|url| Arc::new(OllamaIntelligence::new(&url)));

        let claude = anthropic_api_key.map(|_key| Arc::new(ClaudeIntelligence::new()));

        AdapterRegistry { ollama, claude }
    }

    /// Look up the adapter for a given provider string and run a completion.
    ///
    /// The `model` argument is the bare model name (prefix already stripped by
    /// the protocol layer), e.g. `"phi4"` or `"sonnet"`.
    pub async fn complete(
        &self,
        provider: &str,
        model: &str,
        mut ctx: IntelligenceContext,
    ) -> Result<String, AdapterError> {
        // Inject the model name into metadata so the adapter knows which
        // model to request — avoids changing the Intelligence trait signature.
        ctx.metadata.insert("model".to_string(), model.to_string());

        match provider {
            "ollama" => {
                let adapter = self.ollama.as_ref().ok_or(AdapterError::NotConfigured("ollama"))?;
                adapter.respond(ctx).await.map_err(AdapterError::Intelligence)
            }
            "claude" => {
                let adapter = self.claude.as_ref().ok_or(AdapterError::NotConfigured("claude"))?;
                adapter.respond(ctx).await.map_err(AdapterError::Intelligence)
            }
            other => Err(AdapterError::UnknownProvider(other.to_string())),
        }
    }

    /// List the providers that are currently configured and available.
    pub fn available_providers(&self) -> Vec<&'static str> {
        let mut providers = vec![];
        if self.ollama.is_some() { providers.push("ollama"); }
        if self.claude.is_some() { providers.push("claude"); }
        providers
    }
}

#[derive(Debug)]
pub enum AdapterError {
    NotConfigured(&'static str),
    UnknownProvider(String),
    Intelligence(TramwayError),
}

impl std::fmt::Display for AdapterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdapterError::NotConfigured(p) => write!(f, "provider '{p}' is not configured"),
            AdapterError::UnknownProvider(p) => write!(f, "unknown provider '{p}'"),
            AdapterError::Intelligence(e) => write!(f, "adapter error: {e}"),
        }
    }
}
