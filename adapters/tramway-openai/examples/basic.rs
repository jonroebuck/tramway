use tramway_core::{Intelligence, IntelligenceContext};
use tramway_openai::OpenAiIntelligence;
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

    let openai = OpenAiIntelligence::new();
    let ctx = IntelligenceContext {
        input: prompt,
        system: "You are a helpful assistant.".to_string(),
        history: vec![],
        metadata: Default::default(),
    };

    match openai.respond(ctx).await {
        Ok(reply) => println!("{}", reply),
        Err(e) => eprintln!("Error: {}", e),
    }

    Ok(())
}
