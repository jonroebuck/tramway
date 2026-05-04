use async_trait::async_trait;
use tramway_core::{HistoryRole, Intelligence, IntelligenceContext, TramwayError};
use std::env;
use serde::{Deserialize, Serialize};
use reqwest::Client;

pub struct OpenAiIntelligence;

impl OpenAiIntelligence {
    pub fn new() -> Self {
        OpenAiIntelligence
    }
}

#[derive(Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
}

#[derive(Serialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
}

#[derive(Deserialize)]
struct OpenAiChoice {
    message: OpenAiChoiceMessage,
}

#[derive(Deserialize)]
struct OpenAiChoiceMessage {
    content: String,
}

#[async_trait]
impl Intelligence for OpenAiIntelligence {
    async fn respond(&self, context: IntelligenceContext) -> Result<String, TramwayError> {
        let api_key = env::var("OPENAI_API_KEY")
            .map_err(|_| TramwayError::Intelligence("OPENAI_API_KEY not set".to_string()))?;

        let model = context
            .metadata
            .get("model")
            .cloned()
            .unwrap_or_else(|| "gpt-4o-mini".to_string());

        // Build messages: system prompt first, then history, then current input
        let mut messages: Vec<OpenAiMessage> = vec![];

        if !context.system.is_empty() {
            messages.push(OpenAiMessage {
                role: "system".to_string(),
                content: context.system.clone(),
            });
        }

        for entry in &context.history {
            let role = match entry.role {
                HistoryRole::User => "user",
                HistoryRole::Assistant => "assistant",
                HistoryRole::System => "system",
            };
            messages.push(OpenAiMessage {
                role: role.to_string(),
                content: entry.content.clone(),
            });
        }

        messages.push(OpenAiMessage {
            role: "user".to_string(),
            content: context.input.clone(),
        });

        let req_body = OpenAiRequest { model, messages };

        let client = Client::new();
        let resp = client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(api_key)
            .json(&req_body)
            .send()
            .await
            .map_err(|e| TramwayError::Intelligence(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_else(|e| format!("<failed to read body: {e}>"));
            return Err(TramwayError::Intelligence(format!("OpenAI API returned {status}: {body}")));
        }

        let json: OpenAiResponse = resp
            .json()
            .await
            .map_err(|e| TramwayError::Intelligence(e.to_string()))?;

        let content = json.choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .unwrap_or_default();

        Ok(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[tokio::test]
    fn new_reads_api_key_from_env() {
        // Given OPENAI_API_KEY is set in the environment
        // When OpenAiIntelligence::new() is called
        // Then it does not panic
        env::set_var("OPENAI_API_KEY", "dummy_key");
        let _ = OpenAiIntelligence::new();
    }

    #[tokio::test]
    #[ignore]
    async fn openai_responds_to_prompt() {
        // Given a real OpenAiIntelligence and a simple context
        // When respond is called
        // Then the response is non-empty
        let openai = OpenAiIntelligence::new();
        let ctx = IntelligenceContext {
            input: "Say hello".to_string(),
            system: "You are a helpful assistant.".to_string(),
            history: vec![],
            metadata: Default::default(),
        };
        let reply = openai.respond(ctx).await.unwrap();
        assert!(!reply.trim().is_empty());
    }
}
