#[cfg(test)]
mod tests {
    use super::*;
    use tokio;
    use std::env;

    #[tokio::test]
    fn new_reads_api_key_from_env() {
        env::set_var("ANTHROPIC_API_KEY", "dummy_key");
        let _ = ClaudeIntelligence::new();
    }

    #[tokio::test]
    #[ignore]
    async fn claude_responds_to_prompt() {
        let claude = ClaudeIntelligence::new();
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("model".to_string(), "haiku".to_string());
        let ctx = IntelligenceContext {
            input: "Say hello".to_string(),
            system: "You are a helpful assistant.".to_string(),
            history: vec![],
            metadata,
        };
        let reply = claude.respond(ctx).await.unwrap();
        assert!(!reply.trim().is_empty());
    }
}

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use tramway_core::{HistoryRole, Intelligence, IntelligenceContext, TramwayError};

pub struct ClaudeIntelligence;

impl ClaudeIntelligence {
    pub fn new() -> Self {
        ClaudeIntelligence
    }

    fn resolve_model(model: &str) -> &str {
        match model {
            "sonnet" => "claude-sonnet-4-5",
            "haiku" => "claude-haiku-4-5-20251001",
            other => other,
        }
    }
}

#[derive(Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    messages: Vec<ClaudeMessage>,
}

#[derive(Serialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContentBlock>,
}

#[derive(Deserialize)]
struct ClaudeContentBlock {
    #[serde(rename = "type")]
    kind: String,
    text: Option<String>,
}

#[async_trait]
impl Intelligence for ClaudeIntelligence {
    async fn respond(&self, context: IntelligenceContext) -> Result<String, TramwayError> {
        let api_key = env::var("ANTHROPIC_API_KEY")
            .map_err(|_| TramwayError::Intelligence("ANTHROPIC_API_KEY not set".to_string()))?;

        let model = context
            .metadata
            .get("model")
            .ok_or_else(|| TramwayError::Intelligence("model not specified in metadata".to_string()))?
            .clone();

        let req_body = ClaudeRequest {
            model: Self::resolve_model(&model).to_string(),
            max_tokens: 1024,
            system: if context.system.is_empty() {
                None
            } else {
                Some(context.system.clone())
            },
            messages: context
                .history
                .iter()
                .filter_map(|entry| {
                    let role = match entry.role {
                        HistoryRole::User => "user",
                        HistoryRole::Assistant => "assistant",
                        HistoryRole::System => return None,
                    };

                    Some(ClaudeMessage {
                        role: role.to_string(),
                        content: entry.content.clone(),
                    })
                })
                .chain(std::iter::once(ClaudeMessage {
                    role: "user".to_string(),
                    content: context.input.clone(),
                }))
                .collect(),
        };

        let client = Client::new();
        let resp = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&req_body)
            .send()
            .await
            .map_err(|e| TramwayError::Intelligence(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_else(|e| format!("<failed to read body: {e}>"));
            return Err(TramwayError::Intelligence(format!("Claude API returned {status}: {body}")));
        }

        let json: ClaudeResponse = resp.json().await
            .map_err(|e| TramwayError::Intelligence(e.to_string()))?;

        let content = json.content
            .into_iter()
            .find(|b| b.kind == "text")
            .and_then(|b| b.text)
            .unwrap_or_default();

        Ok(content)
    }
}
