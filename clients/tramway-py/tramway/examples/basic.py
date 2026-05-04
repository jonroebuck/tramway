"""
Basic Tramway example — demonstrates the simple and builder APIs.

Make sure tramway-server is running before executing:

    cargo run -p tramway-server

Then run this example with:

    python -m tramway.examples.basic

Or from the clients/tramway-py directory:

    python -m tramway.examples.basic
"""

from tramway import Tramway, TramwayExtensions, TramwayException


def main():
    tramway = Tramway()

    # ── Simple API ────────────────────────────────────────────────────────
    print("=== Simple completion ===")
    try:
        response = tramway.complete("ollama/phi4", "tell me a short joke")
        print(response)
    except TramwayException as e:
        print(f"Failed: {e}")

    # ── Builder API with system prompt ────────────────────────────────────
    print("\n=== With system prompt ===")
    try:
        response = (tramway.builder("ollama/phi4")
            .system("You are a concise assistant. Answer in one sentence.")
            .user("What is hexagonal architecture?")
            .send())
        print(response)
    except TramwayException as e:
        print(f"Failed: {e}")

    # ── Builder API with conversation history ─────────────────────────────
    print("\n=== Multi-turn conversation ===")
    try:
        response = (tramway.builder("ollama/phi4")
            .system("You are a helpful assistant.")
            .user("My name is Alice.")
            .assistant("Nice to meet you, Alice!")
            .user("What is my name?")
            .send())
        print(response)
    except TramwayException as e:
        print(f"Failed: {e}")

    # ── Builder API with Tramway extensions ───────────────────────────────
    print("\n=== With Tramway extensions ===")
    try:
        ext = TramwayExtensions(trace_id="example-trace-001", prefer_local=True)
        response = (tramway.builder("ollama/phi4")
            .user("What is the capital of France?")
            .extensions(ext)
            .send())
        print(response)
    except TramwayException as e:
        print(f"Failed: {e}")

    # ── Custom server URL ─────────────────────────────────────────────────
    # Uncomment to test against a non-default server address:
    #
    # from tramway import Tramway
    # remote = Tramway("http://my-server:8080")
    # print(remote.complete("claude/sonnet", "hello"))


if __name__ == "__main__":
    main()
