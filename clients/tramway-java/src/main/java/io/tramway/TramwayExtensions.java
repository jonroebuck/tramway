package io.tramway;

import java.util.List;

/**
 * Optional Tramway-specific extensions for a completion request.
 *
 * <p>These map to the {@code x_tramway} field in the OpenAI-compatible
 * request body. Standard OpenAI clients that don't know about this field
 * simply omit it — tramway users can opt in when they need the extra behaviour.
 *
 * <pre>{@code
 * TramwayExtensions ext = new TramwayExtensions.Builder()
 *     .traceId("my-trace-123")
 *     .preferLocal(true)
 *     .build();
 *
 * String response = tramway.builder("ollama/phi4")
 *     .user("tell me a joke")
 *     .extensions(ext)
 *     .send();
 * }</pre>
 */
public class TramwayExtensions {

    private final String traceId;
    private final Boolean preferLocal;
    private final List<String> extensions;

    private TramwayExtensions(Builder builder) {
        this.traceId = builder.traceId;
        this.preferLocal = builder.preferLocal;
        this.extensions = builder.extensions;
    }

    public String getTraceId() { return traceId; }
    public Boolean getPreferLocal() { return preferLocal; }
    public List<String> getExtensions() { return extensions; }

    public static class Builder {
        private String traceId;
        private Boolean preferLocal;
        private List<String> extensions;

        /** An optional trace ID for observability. */
        public Builder traceId(String traceId) {
            this.traceId = traceId;
            return this;
        }

        /** Hint to tramway to prefer a local model if one is available. */
        public Builder preferLocal(boolean preferLocal) {
            this.preferLocal = preferLocal;
            return this;
        }

        /** Named extensions to activate, e.g. {@code List.of("trace", "cache")}. */
        public Builder extensions(List<String> extensions) {
            this.extensions = extensions;
            return this;
        }

        public TramwayExtensions build() {
            return new TramwayExtensions(this);
        }
    }
}
