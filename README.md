```markdown
# tramway

A lightweight, model-agnostic LLM gateway written in Rust.

Tramway sits in front of your AI providers and gives you a single, stable endpoint to talk to — regardless of whether you're running a local Ollama instance, hitting the Claude API, or using OpenAI or Gemini. Swap providers or add new ones without changing your client code.

## How it works

Tramway exposes an OpenAI-compatible REST API, so any client that already speaks OpenAI can point at Tramway with no changes — just swap the base URL. Internally it routes requests to the appropriate backend adapter based on the model name prefix.

```
POST /v1/chat/completions
{ "model": "ollama/phi4", ... }             → Ollama
{ "model": "claude/sonnet", ... }           → Anthropic Claude
{ "model": "openai/gpt-4o", ... }          → OpenAI
{ "model": "gemini/gemini-2.0-flash", ... } → Google Gemini
```

## Providers

| Provider | Model prefix | Configured via                |
| -------- | ------------ | ----------------------------- |
| Ollama   | `ollama/`    | **Required.** Auto-detected, or set `OLLAMA_URL` explicitly |
| Claude   | `claude/`    | `ANTHROPIC_API_KEY` (optional) |
| OpenAI   | `openai/`    | `OPENAI_API_KEY` (optional)   |
| Gemini   | `gemini/`    | `GEMINI_API_KEY` (optional)   |

Ollama is required — Tramway will not start unless a reachable Ollama instance is found. The other three providers are optional; Tramway starts fine with none, some, or all of their keys set, and simply won't route requests for whichever prefixes lack a key.

### Ollama detection

On startup, Tramway looks for Ollama in this order:

1. **`OLLAMA_URL` environment variable**, if set. This is treated as a deliberate configuration, not a hint — Tramway tries only this address (3 attempts, 1s apart) and **fails to start** if it doesn't respond. Use this when Ollama is running on a different host (e.g. reachable over Tailscale).
2. **Autodetection**, if `OLLAMA_URL` is not set. Tramway probes, in order:
   - `http://ollama:11434` (bundled Docker sidecar)
   - `http://host.docker.internal:11434` (native Ollama on Mac/Windows host)
   - `http://localhost:11434` (native Ollama on Linux)

If none of these respond, Tramway logs the failure and **exits non-zero** rather than starting in a degraded state — a missing Ollama connection is treated as a configuration error, not something to silently work around.

## Running

**With Docker:**

```
docker run -e ANTHROPIC_API_KEY=sk-... -e OLLAMA_URL=http://ollama-host:11434 -p 8080:8080 ghcr.io/jonroebuck/tramway:latest
```

**With Ollama bundled (Linux + NVIDIA GPU):**

```
docker compose --profile bundled up
```

**Natively:**

```
cargo run -p tramway-server
```

If Ollama is already running locally, Tramway detects it automatically on startup — no configuration needed. If Ollama runs elsewhere, set `OLLAMA_URL` to point at it.

## Streaming responses

Tramway supports OpenAI-compatible streaming on `/v1/chat/completions` when `"stream": true` is set:

```
curl -N http://localhost:8080/v1/chat/completions \
  -H "content-type: application/json" \
  -d '{
    "model": "ollama/phi4",
    "stream": true,
    "messages": [
      { "role": "user", "content": "Write a haiku about Rust" }
    ]
  }'
```

The response is emitted as SSE `data:` events in `chat.completion.chunk` format, followed by a final `data: [DONE]` event.

## Client libraries

Tramway includes client libraries for Python and Java so you don't have to construct HTTP requests by hand. Both support a simple one-liner API and a builder API for multi-turn conversations, system prompts, and extensions.

**Python** (`clients/tramway-py`):

```python
from tramway import Tramway

tramway = Tramway()  # defaults to http://localhost:8080

# Simple completion
response = tramway.complete("ollama/phi4", "tell me a short joke")

# Builder API — system prompt, history, extensions
response = (tramway.builder("claude/sonnet")
    .system("You are a concise assistant.")
    .user("What is hexagonal architecture?")
    .send())
```

See [`clients/tramway-py/tramway/examples/basic.py`](https://github.com/jonroebuck/tramway/blob/main/clients/tramway-py/tramway/examples/basic.py) for a full working example.

**Java** (`clients/tramway-java`):

```java
Tramway tramway = new Tramway(); // defaults to http://localhost:8080

// Simple completion
String response = tramway.complete("ollama/phi4", "tell me a short joke");

// Builder API — system prompt, history, extensions
String response = tramway.builder("claude/sonnet")
    .system("You are a concise assistant.")
    .user("What is hexagonal architecture?")
    .send();
```

See [`clients/tramway-java/src/main/java/io/tramway/examples/BasicExample.java`](https://github.com/jonroebuck/tramway/blob/main/clients/tramway-java/src/main/java/io/tramway/examples/BasicExample.java) for a full working example.

Both clients also support Tramway extensions for passing trace IDs and routing hints, and accept a custom server URL for connecting to a non-default Tramway instance.

If you're already using an OpenAI-compatible client, you can skip the language libraries entirely and just point your existing client at Tramway's base URL.

## Adding a private adapter

Tramway supports registering adapters for models that aren't publicly available. Implement the `Intelligence` trait from `tramway-core` in your own crate, then register it at startup:

```rust
let mut registry = AdapterRegistry::new(ollama_url, anthropic_key, openai_key, gemini_key);
registry.register_external("internal/my-model", MyPrivateAdapter::new());
```

The adapter is compiled in but never needs to be published.

## Crates

| Crate                     | Description                                                                |
| ------------------------- | -------------------------------------------------------------------------- |
| `tramway-core`            | `Intelligence` trait and `IntelligenceContext` — the stable port interface |
| `tramway-server`          | Axum HTTP server with OpenAI-compatible endpoints                          |
| `tramway-ollama`          | Ollama backend adapter                                                     |
| `tramway-claude`          | Anthropic Claude backend adapter                                           |
| `tramway-openai`          | OpenAI backend adapter                                                     |
| `tramway-gemini`          | Google Gemini backend adapter                                              |
| `tramway-protocol-openai` | OpenAI wire format — decodes incoming requests and encodes responses       |

## Pulling models with the bundled profile

If you're running the bundled Docker profile, use the included tramway-pull script to download models into the containerized Ollama instance:

```
./tramway-pull phi4
./tramway-pull llama3
```

This is equivalent to `ollama pull` but targets the Ollama container rather than a local installation. Models are persisted in a Docker volume and survive container restarts.

If you have Ollama installed natively, just use `ollama pull` as normal — Tramway will detect it automatically on startup.
```