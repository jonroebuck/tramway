use async_trait::async_trait;
use tramway_core::{HistoryRole, Intelligence, IntelligenceContext, TramwayError};
use std::env;
use serde::{Deserialize, Serialize};
use reqwest::Client;

pub struct GeminiIntelligence;

impl GeminiIntelligence {
    pub fn new() -> Self {
        GeminiIntelligence
    }
}

#[derive(Serialize)]
struct GeminiRequest {
    model: String,
    messages: Vec<GeminiMessage>,
}

#[derive(Serialize)]
struct GeminiMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    choices: Vec<GeminiChoice>,
}

#[derive(Deserialize)]
struct GeminiChoice {
    message: GeminiChoiceMessage,
}

#[derive(Deserialize)]
struct GeminiChoiceMessage {
    content: String,
}

#[async_trait]
impl Intelligence for GeminiIntelligence {
    async fn respond(&self, context: IntelligenceContext) -> Result<String, TramwayError> {
        let api_key = env::var("GEMINI_API_KEY")
            .map_err(|_| TramwayError::Intelligence("GEMINI_API_KEY not set".to_string()))?;

        let model = context
            .metadata
            .get("model")
            .cloned()
            .unwrap_or_else(|| "gemini-2.0-flash".to_string());

        let mut messages: Vec<GeminiMessage> = vec![];

        if !context.system.is_empty() {
            messages.push(GeminiMessage {
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
            messages.push(GeminiMessage {
                role: role.to_string(),
                content: entry.content.clone(),
            });
        }

        messages.push(GeminiMessage {
            role: "user".to_string(),
            content: context.input.clone(),
        });

        let req_body = GeminiRequest { model, messages };

        let client = Client::new();
        let resp = client
            .post("https://generativelanguage.googleapis.com/v1beta/openai/chat/completions")
            .bearer_auth(api_key)
            .json(&req_body)
            .send()
            .await
            .map_err(|e| TramwayError::Intelligence(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_else(|e| format!("<failed to read body: {e}>"));
            return Err(TramwayError::Intelligence(format!("Gemini API returned {status}: {body}")));
        }

        let json: GeminiResponse = resp
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
        // Given GEMINI_API_KEY is set in the environment
        // When GeminiIntelligence::new() is called
        // Then it does not panic
        env::set_var("GEMINI_API_KEY", "dummy_key");
        let _ = GeminiIntelligence::new();
    }

    #[tokio::test]
    #[ignore]
    async fn gemini_responds_to_prompt() {
        // Given a real GeminiIntelligence and a simple context
        // When respond is called
        // Then the response is non-empty
        let gemini = GeminiIntelligence::new();
        let ctx = IntelligenceContext {
            input: "Say hello".to_string(),
            system: "You are a helpful assistant.".to_string(),
            history: vec![],
            metadata: Default::default(),
        };
        let reply = gemini.respond(ctx).await.unwrap();
        assert!(!reply.trim().is_empty());
    }
}
