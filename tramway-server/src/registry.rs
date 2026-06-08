use std::collections::HashMap;
use std::sync::Arc;
use tramway_claude::ClaudeIntelligence;
use tramway_core::{Intelligence, IntelligenceContext, ResponseStream, TramwayError};
use tramway_gemini::GeminiIntelligence;
use tramway_ollama::OllamaIntelligence;
use tramway_openai::OpenAiIntelligence;

pub struct AdapterRegistry {
    ollama: Option<Arc<OllamaIntelligence>>,
    claude: Option<Arc<ClaudeIntelligence>>,
    openai: Option<Arc<OpenAiIntelligence>>,
    gemini: Option<Arc<GeminiIntelligence>>,
    external: HashMap<String, Arc<dyn Intelligence + Send + Sync>>,
}

impl AdapterRegistry {
    pub fn new(
        ollama_url: Option<String>,
        anthropic_api_key: Option<String>,
        openai_api_key: Option<String>,
        gemini_api_key: Option<String>,
    ) -> Self {
        let ollama = ollama_url.map(|url| Arc::new(OllamaIntelligence::new(&url)));
        let claude = anthropic_api_key.map(|_| Arc::new(ClaudeIntelligence::new()));
        let openai = openai_api_key.map(|_| Arc::new(OpenAiIntelligence::new()));
        let gemini = gemini_api_key.map(|_| Arc::new(GeminiIntelligence::new()));
        AdapterRegistry {
            ollama,
            claude,
            openai,
            gemini,
            external: HashMap::new(),
        }
    }

    pub fn register_external(
        &mut self,
        name: impl Into<String>,
        adapter: impl Intelligence + Send + Sync + 'static,
    ) -> &mut Self {
        self.external.insert(name.into(), Arc::new(adapter));
        self
    }

    pub async fn complete(
        &self,
        provider: &str,
        model: &str,
        mut ctx: IntelligenceContext,
    ) -> Result<String, AdapterError> {
        ctx.metadata.insert("model".to_string(), model.to_string());

        match provider {
            "ollama" => {
                let adapter = self
                    .ollama
                    .as_ref()
                    .ok_or(AdapterError::NotConfigured("ollama"))?;
                adapter
                    .respond(ctx)
                    .await
                    .map_err(AdapterError::Intelligence)
            }
            "claude" => {
                let adapter = self
                    .claude
                    .as_ref()
                    .ok_or(AdapterError::NotConfigured("claude"))?;
                adapter
                    .respond(ctx)
                    .await
                    .map_err(AdapterError::Intelligence)
            }
            "openai" => {
                let adapter = self
                    .openai
                    .as_ref()
                    .ok_or(AdapterError::NotConfigured("openai"))?;
                adapter
                    .respond(ctx)
                    .await
                    .map_err(AdapterError::Intelligence)
            }
            "gemini" => {
                let adapter = self
                    .gemini
                    .as_ref()
                    .ok_or(AdapterError::NotConfigured("gemini"))?;
                adapter
                    .respond(ctx)
                    .await
                    .map_err(AdapterError::Intelligence)
            }
            other => {
                let adapter = self
                    .external
                    .get(other)
                    .ok_or_else(|| AdapterError::UnknownProvider(other.to_string()))?;
                adapter
                    .respond(ctx)
                    .await
                    .map_err(AdapterError::Intelligence)
            }
        }
    }

    pub async fn complete_stream(
        &self,
        provider: &str,
        model: &str,
        mut ctx: IntelligenceContext,
    ) -> Result<ResponseStream, AdapterError> {
        ctx.metadata.insert("model".to_string(), model.to_string());

        match provider {
            "ollama" => {
                let adapter = self
                    .ollama
                    .as_ref()
                    .ok_or(AdapterError::NotConfigured("ollama"))?;
                adapter
                    .respond_stream(ctx)
                    .await
                    .map_err(AdapterError::Intelligence)
            }
            "claude" => {
                let adapter = self
                    .claude
                    .as_ref()
                    .ok_or(AdapterError::NotConfigured("claude"))?;
                adapter
                    .respond_stream(ctx)
                    .await
                    .map_err(AdapterError::Intelligence)
            }
            "openai" => {
                let adapter = self
                    .openai
                    .as_ref()
                    .ok_or(AdapterError::NotConfigured("openai"))?;
                adapter
                    .respond_stream(ctx)
                    .await
                    .map_err(AdapterError::Intelligence)
            }
            "gemini" => {
                let adapter = self
                    .gemini
                    .as_ref()
                    .ok_or(AdapterError::NotConfigured("gemini"))?;
                adapter
                    .respond_stream(ctx)
                    .await
                    .map_err(AdapterError::Intelligence)
            }
            other => {
                let adapter = self
                    .external
                    .get(other)
                    .ok_or_else(|| AdapterError::UnknownProvider(other.to_string()))?;
                adapter
                    .respond_stream(ctx)
                    .await
                    .map_err(AdapterError::Intelligence)
            }
        }
    }

    pub fn available_providers(&self) -> Vec<String> {
        let mut providers = vec![];
        if self.ollama.is_some() {
            providers.push("ollama".to_string());
        }
        if self.claude.is_some() {
            providers.push("claude".to_string());
        }
        if self.openai.is_some() {
            providers.push("openai".to_string());
        }
        if self.gemini.is_some() {
            providers.push("gemini".to_string());
        }
        providers.extend(self.external.keys().cloned());
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
