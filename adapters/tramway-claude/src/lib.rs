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
        let ctx = IntelligenceContext {
            input: "Say hello".to_string(),
            system: "You are a helpful assistant.".to_string(),
            history: vec![],
            metadata: Default::default(),
        };
        let reply = claude.respond(ctx).await.unwrap();
        assert!(!reply.trim().is_empty());
    }
}

use async_trait::async_trait;
use tramway_core::{Intelligence, IntelligenceContext, TramwayError};
use std::env;
use serde::{Deserialize, Serialize};
use reqwest::Client;

pub struct ClaudeIntelligence;

impl ClaudeIntelligence {
    pub fn new() -> Self {
        ClaudeIntelligence
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
    async fn respond(&self, context: IntelligenceContext, model: &str) -> Result<String, TramwayError> {
        let api_key = env::var("ANTHROPIC_API_KEY")
            .map_err(|_| TramwayError::Intelligence("ANTHROPIC_API_KEY not set".to_string()))?;

        let client = Client::new();

        // Map shorthand model names to real Claude API model strings
        let resolved_model = match model {
            "sonnet" => "claude-sonnet-4-5",
            "haiku"  => "claude-haiku-4-5-20251001",
            other    => other,
        };

        let system = if context.system.is_empty() {
            None
        } else {
            Some(context.system.clone())
        };

        let mut messages = vec![];

        // Add history
        for entry in &context.history {
            let role = match entry.role {
                tramway_core::HistoryRole::User => "user",
                tramway_core::HistoryRole::Assistant => "assistant",
                tramway_core::HistoryRole::System => continue,
            };
            messages.push(ClaudeMessage {
                role: role.to_string(),
                content: entry.content.clone(),
            });
        }

        // Add current input
        messages.push(ClaudeMessage {
            role: "user".to_string(),
            content: context.input.clone(),
        });

        let req_body = ClaudeRequest {
            model: resolved_model.to_string(),
            max_tokens: 1024,
            system,
            messages,
        };

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
