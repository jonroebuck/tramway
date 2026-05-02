use crate::error::ProtocolError;

/// The parsed result of a `provider/model` model string.
///
/// Tramway uses namespaced model identifiers to route requests to the
/// correct backend adapter:
///
/// ```text
/// "ollama/phi4"    → ModelTarget { provider: "ollama", model: "phi4" }
/// "claude/sonnet"  → ModelTarget { provider: "claude", model: "sonnet" }
/// "gpt-4o"         → ModelTarget { provider: "openai", model: "gpt-4o" }
/// ```
///
/// Plain model names without a prefix are assumed to be OpenAI models,
/// preserving compatibility with standard OpenAI clients.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelTarget {
    /// The backend provider: `"ollama"`, `"claude"`, `"openai"`, etc.
    pub provider: String,
    /// The model name passed to the backend adapter.
    pub model: String,
}

impl ModelTarget {
    /// Parse a model string into a [`ModelTarget`].
    ///
    /// Accepts `"provider/model"` or a plain model name (defaults to `"openai"`).
    pub fn parse(model_str: &str) -> Result<Self, ProtocolError> {
        if model_str.is_empty() {
            return Err(ProtocolError::EmptyModel);
        }

        match model_str.split_once('/') {
            Some((provider, model)) => {
                if provider.is_empty() || model.is_empty() {
                    return Err(ProtocolError::InvalidModel(model_str.to_string()));
                }
                Ok(ModelTarget {
                    provider: provider.to_lowercase(),
                    model: model.to_string(),
                })
            }
            // No prefix — treat as a plain OpenAI model name
            None => Ok(ModelTarget {
                provider: "openai".to_string(),
                model: model_str.to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_namespaced_ollama() {
        let t = ModelTarget::parse("ollama/phi4").unwrap();
        assert_eq!(t.provider, "ollama");
        assert_eq!(t.model, "phi4");
    }

    #[test]
    fn parses_namespaced_claude() {
        let t = ModelTarget::parse("claude/sonnet").unwrap();
        assert_eq!(t.provider, "claude");
        assert_eq!(t.model, "sonnet");
    }

    #[test]
    fn plain_name_defaults_to_openai() {
        let t = ModelTarget::parse("gpt-4o").unwrap();
        assert_eq!(t.provider, "openai");
        assert_eq!(t.model, "gpt-4o");
    }

    #[test]
    fn rejects_empty_model() {
        assert!(ModelTarget::parse("").is_err());
    }

    #[test]
    fn rejects_slash_only() {
        assert!(ModelTarget::parse("/").is_err());
    }

    #[test]
    fn provider_is_lowercased() {
        let t = ModelTarget::parse("Ollama/Phi4").unwrap();
        assert_eq!(t.provider, "ollama");
        assert_eq!(t.model, "Phi4"); // model name preserved as-is
    }
}
