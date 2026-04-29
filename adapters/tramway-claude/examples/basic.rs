use tramway_core::{Intelligence, IntelligenceContext};
use tramway_claude::ClaudeIntelligence;
use anyhow::Result;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let prompt = match args.next() {
        Some(p) => p,
        None => {
            eprintln!("Usage: basic <prompt>");
            return Ok(());
        }
    };
    let claude = ClaudeIntelligence::new();
    let ctx = IntelligenceContext {
        input: prompt,
        system: "You are a helpful assistant.".to_string(),
        history: vec![],
        metadata: Default::default(),
    };
    match claude.respond(ctx).await {
        Ok(reply) => println!("{}", reply),
        Err(e) => eprintln!("Error: {}", e),
    }
    Ok(())
}
