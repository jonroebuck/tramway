tramway
A lightweight, model-agnostic LLM gateway written in Rust.
Tramway sits in front of your AI providers and gives you a single, stable endpoint to talk to — regardless of whether you're running a local Ollama instance, hitting the Claude API, or using OpenAI or Gemini. Swap providers or add new ones without changing your client code.
How it works
Tramway exposes an OpenAI-compatible REST API, so any client that already speaks OpenAI can point at Tramway with no changes — just swap the base URL. Internally it routes requests to the appropriate backend adapter based on the model name prefix.
POST /v1/chat/completions
{ "model": "ollama/phi4", ... }             → Ollama
{ "model": "claude/sonnet", ... }           → Anthropic Claude
{ "model": "openai/gpt-4o", ... }          → OpenAI
{ "model": "gemini/gemini-2.0-flash", ... } → Google Gemini
Providers
ProviderModel prefixConfigured viaOllamaollama/Auto-detected (no key needed)Claudeclaude/ANTHROPIC_API_KEYOpenAIopenai/OPENAI_API_KEYGeminigemini/GEMINI_API_KEY
Tramway starts with whatever providers are available. You don't need all four — if only ANTHROPIC_API_KEY is set, Tramway starts fine and just won't route ollama/* requests.
Running
With Docker:
bashdocker run -e ANTHROPIC_API_KEY=sk-... -p 8080:8080 ghcr.io/jonroebuck/tramway:latest
With Ollama bundled (Linux + NVIDIA GPU):
bashdocker compose --profile bundled up
Natively:
bashcargo run -p tramway-server
Tramway auto-detects a local or sidecar Ollama instance on startup. No configuration needed if Ollama is already running.
Client libraries
Tramway includes client libraries for Python and Java so you don't have to construct HTTP requests by hand. Both support a simple one-liner API and a builder API for multi-turn conversations, system prompts, and extensions.
Python (clients/tramway-py):
pythonfrom tramway import Tramway

tramway = Tramway()  # defaults to http://localhost:8080

# Simple completion
response = tramway.complete("ollama/phi4", "tell me a short joke")

# Builder API — system prompt, history, extensions
response = (tramway.builder("claude/sonnet")
    .system("You are a concise assistant.")
    .user("What is hexagonal architecture?")
    .send())
See clients/tramway-py/tramway/examples/basic.py for a full working example.
Java (clients/tramway-java):
javaTramway tramway = new Tramway(); // defaults to http://localhost:8080

// Simple completion
String response = tramway.complete("ollama/phi4", "tell me a short joke");

// Builder API — system prompt, history, extensions
String response = tramway.builder("claude/sonnet")
    .system("You are a concise assistant.")
    .user("What is hexagonal architecture?")
    .send();
See clients/tramway-java/src/main/java/io/tramway/examples/BasicExample.java for a full working example.
Both clients also support Tramway extensions for passing trace IDs and routing hints, and accept a custom server URL for connecting to a non-default Tramway instance.
If you're already using an OpenAI-compatible client, you can skip the language libraries entirely and just point your existing client at Tramway's base URL.
Adding a private adapter
Tramway supports registering adapters for models that aren't publicly available. Implement the Intelligence trait from tramway-core in your own crate, then register it at startup:
rustlet mut registry = AdapterRegistry::new(ollama_url, anthropic_key, openai_key, gemini_key);
registry.register_external("internal/my-model", MyPrivateAdapter::new());
The adapter is compiled in but never needs to be published.
Crates
CrateDescriptiontramway-coreIntelligence trait and IntelligenceContext — the stable port interfacetramway-serverAxum HTTP server with OpenAI-compatible endpointstramway-ollamaOllama backend adaptertramway-claudeAnthropic Claude backend adaptertramway-openaiOpenAI backend adaptertramway-geminiGoogle Gemini backend adaptertramway-protocol-openaiOpenAI wire format — decodes incoming requests and encodes responses
