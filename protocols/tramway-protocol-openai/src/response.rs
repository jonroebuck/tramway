use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// An OpenAI-compatible chat completions response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    pub index: u32,
    pub message: ResponseMessage,
    pub finish_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMessage {
    pub role: String,
    pub content: String,
}

/// Token usage — Tramway doesn't track tokens yet, so these are zeroed.
/// Future adapters can populate them if the backend provides the data.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionChunk {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChunkChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkChoice {
    pub index: u32,
    pub delta: DeltaMessage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeltaMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StreamEncoder {
    id: String,
    created: u64,
    model: String,
}

impl ChatResponse {
    pub fn from_output(content: String, model: &str) -> Self {
        let id = format!("chatcmpl-{}", Uuid::new_v4().simple());
        let created = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        ChatResponse {
            id,
            object: "chat.completion".to_string(),
            created,
            model: model.to_string(),
            choices: vec![Choice {
                index: 0,
                message: ResponseMessage {
                    role: "assistant".to_string(),
                    content,
                },
                finish_reason: "stop".to_string(),
            }],
            usage: Usage::default(),
        }
    }
}

impl StreamEncoder {
    pub fn new(model: &str) -> Self {
        Self {
            id: format!("chatcmpl-{}", Uuid::new_v4().simple()),
            created: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            model: model.to_string(),
        }
    }

    pub fn role_chunk(&self) -> ChatCompletionChunk {
        self.chunk(
            DeltaMessage {
                role: Some("assistant".to_string()),
                content: None,
            },
            None,
        )
    }

    pub fn content_chunk(&self, content: String) -> ChatCompletionChunk {
        self.chunk(
            DeltaMessage {
                role: None,
                content: Some(content),
            },
            None,
        )
    }

    pub fn end_chunk(&self) -> ChatCompletionChunk {
        self.chunk(DeltaMessage::default(), Some("stop".to_string()))
    }

    fn chunk(&self, delta: DeltaMessage, finish_reason: Option<String>) -> ChatCompletionChunk {
        ChatCompletionChunk {
            id: self.id.clone(),
            object: "chat.completion.chunk".to_string(),
            created: self.created,
            model: self.model.clone(),
            choices: vec![ChunkChoice {
                index: 0,
                delta,
                finish_reason,
            }],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StreamEncoder;

    #[test]
    fn stream_encoder_builds_openai_chunk_shape() {
        let enc = StreamEncoder::new("ollama/phi4");

        let role = enc.role_chunk();
        assert_eq!(role.object, "chat.completion.chunk");
        assert_eq!(role.choices[0].delta.role.as_deref(), Some("assistant"));
        assert!(role.choices[0].finish_reason.is_none());

        let delta = enc.content_chunk("hello".to_string());
        assert_eq!(delta.choices[0].delta.content.as_deref(), Some("hello"));
        assert!(delta.choices[0].delta.role.is_none());

        let end = enc.end_chunk();
        assert_eq!(end.choices[0].finish_reason.as_deref(), Some("stop"));
    }
}
