use async_trait::async_trait;
use tramway_core::{Intelligence, IntelligenceContext, TramwayError};

pub struct OllamaIntelligence;

#[async_trait]
impl Intelligence for OllamaIntelligence {
    async fn respond(&self, _context: IntelligenceContext) -> Result<String, TramwayError> {
        todo!()
    }
}
