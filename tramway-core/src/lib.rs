use std::collections::HashMap;

use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TramwayError {
    #[error("intelligence error: {0}")]
    Intelligence(String),
}

#[derive(Debug, Clone)]
pub enum HistoryRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub role: HistoryRole,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct IntelligenceContext {
    pub input: String,
    pub system: String,
    pub history: Vec<HistoryEntry>,
    pub metadata: HashMap<String, String>,
}

#[async_trait]
pub trait Intelligence {
    async fn respond(&self, context: IntelligenceContext) -> Result<String, TramwayError>;
}

pub struct MockIntelligence;

#[async_trait]
impl Intelligence for MockIntelligence {
    async fn respond(&self, context: IntelligenceContext) -> Result<String, TramwayError> {
        Ok(format!("mock response to: {}", context.input))
    }
}
