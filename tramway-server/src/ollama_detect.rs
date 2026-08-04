use thiserror::Error;
use tracing::{info, warn};

#[derive(Debug, Error)]
pub enum OllamaDetectError {
    #[error("OLLAMA_URL is set to '{0}' but Ollama did not respond there after 3 attempts")]
    ConfiguredUrlUnreachable(String),

    #[error("No Ollama instance found — tried {0} candidate address(es), none responded")]
    NotFound(usize),
}

/// Detect a running Ollama instance.
///
/// If `OLLAMA_URL` is set in the environment, only that address is tried —
/// it's treated as a deliberate configuration, not a hint, so failure to
/// reach it is an error rather than a fallback trigger.
///
/// If `OLLAMA_URL` is not set, falls back to probing common local/sidecar
/// addresses. Either way, if nothing responds, returns an error rather
/// than silently starting without Ollama support.
pub async fn detect_ollama() -> Result<String, OllamaDetectError> {
    if let Ok(configured) = std::env::var("OLLAMA_URL") {
        return if ping(&configured).await {
            info!("Ollama available at {configured} (from OLLAMA_URL)");
            Ok(configured)
        } else {
            Err(OllamaDetectError::ConfiguredUrlUnreachable(configured))
        };
    }

    let candidates = vec![
        "http://ollama:11434",       // bundled Docker sidecar
        "http://host.docker.internal:11434", // native Ollama on Mac/Windows host
        "http://localhost:11434",    // native Ollama on Linux
    ];

    for url in &candidates {
        if ping(url).await {
            info!("Ollama available at {url} (autodetected)");
            return Ok(url.to_string());
        }
    }

    Err(OllamaDetectError::NotFound(candidates.len()))
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
