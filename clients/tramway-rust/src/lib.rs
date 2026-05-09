use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<Message<'a>>,
}

#[derive(Serialize)]
struct Message<'a> {
    role: &'a str,
    content: &'a str,
}

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

    /// Simple one-liner completion
    pub async fn complete(&self, model: &str, prompt: &str) -> Result<String> {
        self.respond(model, "", prompt).await
    }

    /// Full completion with system prompt
    pub async fn respond(&self, model: &str, system: &str, input: &str) -> Result<String> {
        let mut messages = vec![];

        if !system.is_empty() {
            messages.push(Message { role: "system", content: system });
        }

        messages.push(Message { role: "user", content: input });

        let request = ChatRequest { model, messages };

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
