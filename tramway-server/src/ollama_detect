use tracing::{info, warn};

/// Probe a list of candidate URLs to find a running Ollama instance.
///
/// Returns the first URL that responds to `GET /api/tags`, or `None` if
/// none of them are reachable. Tries each candidate with a short timeout
/// and a couple of retries to handle Docker startup ordering.
pub async fn detect_ollama() -> Option<String> {
    let candidates = vec![
        "http://ollama:11434",                  // bundled Docker sidecar
        "http://host.docker.internal:11434",    // native Ollama on Mac/Windows host
        "http://localhost:11434",               // native Ollama on Linux
    ];

    for url in candidates {
        if ping(url).await {
            return Some(url.to_string());
        }
    }

    None
}

/// Returns true if Ollama is reachable at the given base URL.
///
/// Tries up to 3 times with a 1-second gap, to handle the case where
/// the bundled Ollama container is still starting up.
async fn ping(base_url: &str) -> bool {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .unwrap();

    let url = format!("{base_url}/api/tags");

    for attempt in 1..=3 {
        match client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                info!("Ollama responded at {base_url} (attempt {attempt})");
                return true;
            }
            Ok(resp) => {
                warn!("Ollama at {base_url} returned status {} (attempt {attempt})", resp.status());
            }
            Err(e) => {
                warn!("Ollama not reachable at {base_url}: {e} (attempt {attempt})");
            }
        }

        if attempt < 3 {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }

    false
}
