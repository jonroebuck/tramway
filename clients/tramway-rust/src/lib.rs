use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

// --- Request types ---

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

// --- Response types ---

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: Option<String>,
}

// --- Client ---

pub struct Tramway {
    client: Client,
    base_url: String,
}

impl Tramway {
    /// Create a client pointing at the default local Tramway server
    pub fn new() -> Self {
        Self::with_url("http://localhost:8080")
    }

    /// Create a client pointing at a specific Tramway server
    pub fn with_url(url: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: url.trim_end_matches('/').to_string(),
        }
    }

    /// Simple one-liner completion — no system prompt
    pub async fn complete(&self, model: &str, prompt: &str) -> Result<String> {
        self.respond(model, "", prompt).await
    }

    /// Full completion with system prompt
    pub async fn respond(&self, model: &str, system: &str, input: &str) -> Result<String> {
        self.respond_with_history(model, system, input, vec![]).await
    }

    /// Full completion with system prompt and prior conversation history.
    /// Each entry in `history` is a `(user, assistant)` pair of prior turns.
    pub async fn respond_with_history(
        &self,
        model: &str,
        system: &str,
        input: &str,
        history: Vec<(String, String)>,
    ) -> Result<String> {
        let mut messages = vec![];

        if !system.is_empty() {
            messages.push(Message {
                role: "system".to_string(),
                content: system.to_string(),
            });
        }

        for (user_turn, assistant_turn) in history {
            messages.push(Message {
                role: "user".to_string(),
                content: user_turn,
            });
            messages.push(Message {
                role: "assistant".to_string(),
                content: assistant_turn,
            });
        }

        messages.push(Message {
            role: "user".to_string(),
            content: input.to_string(),
        });

        let request = ChatRequest {
            model: model.to_string(),
            messages,
        };

        let response = self.client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .json(&request)
            .send()
            .await?
            .json::<ChatResponse>()
            .await?;

        let content = response.choices
            .into_iter()
            .next()
            .and_then(|c| c.message.content)
            .ok_or_else(|| anyhow!("no content in response"))?;

        Ok(content)
    }
}

impl Default for Tramway {
    fn default() -> Self {
        Self::new()
    }
}
