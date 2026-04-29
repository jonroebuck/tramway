#[cfg(test)]
mod tests {
    use super::*;
    use tokio;
    use std::env;

    #[tokio::test]
    fn new_reads_api_key_from_env() {
        // Given ANTHROPIC_API_KEY is set in the environment
        // When ClaudeIntelligence::new() is called
        // Then it does not panic
        env::set_var("ANTHROPIC_API_KEY", "dummy_key");
        let _ = ClaudeIntelligence::new();
        // Should not panic
    }

    #[tokio::test]
    #[ignore]
    async fn claude_responds_to_prompt() {
        // Given a real ClaudeIntelligence and a simple context
        // When respond is called
        // Then the response is non-empty
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
struct ClaudeRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    messages: Vec<ClaudeMessage<'a>>,
}

#[derive(Serialize)]
struct ClaudeMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct ClaudeResponse {
    content: String,
}

#[async_trait]
impl Intelligence for ClaudeIntelligence {
    async fn respond(&self, context: IntelligenceContext) -> Result<String, TramwayError> {
        let api_key = env::var("ANTHROPIC_API_KEY")
            .map_err(|_| TramwayError::Intelligence("ANTHROPIC_API_KEY not set".to_string()))?;
        let client = Client::new();
        let url = "https://api.anthropic.com/v1/messages";
        let req_body = ClaudeRequest {
            model: "claude-haiku-4-5-20251001", // updated to a lightweight, widely available Claude model
            max_tokens: 256,
            messages: vec![
                ClaudeMessage { role: "user", content: &context.input },
            ],
        };
        let resp = client
            .post(url)
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
        let json: serde_json::Value = resp.json().await.map_err(|e| TramwayError::Intelligence(e.to_string()))?;
        // Claude API returns an array of content blocks; get the first text block
        let content = json["content"][0]["text"].as_str().unwrap_or("").to_string();
        Ok(content)
    }
}
