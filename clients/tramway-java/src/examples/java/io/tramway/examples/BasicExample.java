package io.tramway.examples;

import io.tramway.Tramway;
import io.tramway.TramwayException;
import io.tramway.TramwayExtensions;

/**
 * Basic Tramway example — demonstrates the simple and builder APIs.
 *
 * Make sure tramway-server is running before executing:
 *
 *   cargo run -p tramway-server
 *
 * Then run this example with:
 *
 *   ./gradlew runExample
 */
public class BasicExample {

    public static void main(String[] args) {
        Tramway tramway = new Tramway();

        // ── Simple API ────────────────────────────────────────────────────
        System.out.println("=== Simple completion ===");
        try {
            String response = tramway.complete("ollama/phi4", "tell me a short joke");
            System.out.println(response);
        } catch (TramwayException e) {
            System.err.println("Failed: " + e.getMessage());
        }

        // ── Builder API with system prompt ────────────────────────────────
        System.out.println("\n=== With system prompt ===");
        try {
            String response = tramway.builder("ollama/phi4")
                    .system("You are a concise assistant. Answer in one sentence.")
                    .user("What is hexagonal architecture?")
                    .send();
            System.out.println(response);
        } catch (TramwayException e) {
            System.err.println("Failed: " + e.getMessage());
        }

        // ── Builder API with conversation history ─────────────────────────
        System.out.println("\n=== Multi-turn conversation ===");
        try {
            String response = tramway.builder("ollama/phi4")
                    .system("You are a helpful assistant.")
                    .user("My name is Alice.")
                    .assistant("Nice to meet you, Alice!")
                    .user("What is my name?")
                    .send();
            System.out.println(response);
        } catch (TramwayException e) {
            System.err.println("Failed: " + e.getMessage());
        }

        // ── Builder API with Tramway extensions ───────────────────────────
        System.out.println("\n=== With Tramway extensions ===");
        try {
            TramwayExtensions ext = new TramwayExtensions.Builder()
                    .traceId("example-trace-001")
                    .preferLocal(true)
                    .build();

            String response = tramway.builder("ollama/phi4")
                    .user("What is the capital of France?")
                    .extensions(ext)
                    .send();
            System.out.println(response);
        } catch (TramwayException e) {
            System.err.println("Failed: " + e.getMessage());
        }

        // ── Custom server URL ─────────────────────────────────────────────
        // Uncomment to test against a non-default server address:
        //
        // Tramway remote = new Tramway("http://my-server:8080");
        // String response = remote.complete("claude/sonnet", "hello");
    }
}
