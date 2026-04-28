use async_trait::async_trait;
use tramway_core::{Intelligence, IntelligenceContext, TramwayError};

pub struct ClaudeIntelligence;

#[async_trait]
impl Intelligence for ClaudeIntelligence {
    async fn respond(&self, _context: IntelligenceContext) -> Result<String, TramwayError> {
        todo!()
    }
}
