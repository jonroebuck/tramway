use std::collections::HashMap;
use std::env;
use tramway_core::{Intelligence, IntelligenceContext};
use tramway_ollama::OllamaIntelligence;
use anyhow::{Context, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let prompt = match args.next() {
        Some(p) => p,
        None => {
            eprintln!("Usage: basic <prompt> [model]");
            return Ok(());
        }
    };
    let model = args.next().unwrap_or_else(|| "phi4".to_string());
    let ollama = OllamaIntelligence::new("http://localhost:11434", model);
    ollama.ping().await.context("Could not connect to Ollama daemon. Is it running on http://localhost:11434?")?;
    let ctx = IntelligenceContext {
        input: prompt,
        system: "You are a helpful assistant.".to_string(),
        history: vec![],
        metadata: HashMap::new(),
    };
    match ollama.respond(ctx).await {
        Ok(reply) => println!("{}", reply),
        Err(e) => eprintln!("Error: {}", e),
    }
    Ok(())
}
