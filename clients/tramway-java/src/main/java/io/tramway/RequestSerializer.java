package io.tramway;

import java.util.List;

/**
 * Builds the JSON request body for a chat completions call.
 *
 * <p>Uses no external JSON library — just string building — so tramway-java
 * has zero runtime dependencies beyond the JDK.
 */
class RequestSerializer {

    static String serialize(
            String model,
            String systemPrompt,
            List<CompletionBuilder.Message> messages,
            TramwayExtensions extensions
    ) {
        StringBuilder sb = new StringBuilder();
        sb.append("{");
        sb.append("\"model\":").append(quoted(model)).append(",");
        sb.append("\"messages\":[");

        boolean first = true;

        // System message first if present
        if (systemPrompt != null && !systemPrompt.isEmpty()) {
            sb.append(messageObject("system", systemPrompt));
            first = false;
        }

        // Conversation messages
        for (CompletionBuilder.Message msg : messages) {
            if (!first) sb.append(",");
            sb.append(messageObject(msg.role, msg.content));
            first = false;
        }

        sb.append("]");

        // Optional x_tramway extensions block
        if (extensions != null) {
            sb.append(",\"x_tramway\":{");
            boolean extFirst = true;

            if (extensions.getTraceId() != null) {
                sb.append("\"trace_id\":").append(quoted(extensions.getTraceId()));
                extFirst = false;
            }
            if (extensions.getPreferLocal() != null) {
                if (!extFirst) sb.append(",");
                sb.append("\"prefer_local\":").append(extensions.getPreferLocal());
                extFirst = false;
            }
            if (extensions.getExtensions() != null && !extensions.getExtensions().isEmpty()) {
                if (!extFirst) sb.append(",");
                sb.append("\"extensions\":[");
                for (int i = 0; i < extensions.getExtensions().size(); i++) {
                    if (i > 0) sb.append(",");
                    sb.append(quoted(extensions.getExtensions().get(i)));
                }
                sb.append("]");
            }

            sb.append("}");
        }

        sb.append("}");
        return sb.toString();
    }

    private static String messageObject(String role, String content) {
        return "{\"role\":" + quoted(role) + ",\"content\":" + quoted(content) + "}";
    }

    /** Escape a string for safe inclusion in JSON. */
    private static String quoted(String value) {
        return "\"" + value
                .replace("\\", "\\\\")
                .replace("\"", "\\\"")
                .replace("\n", "\\n")
                .replace("\r", "\\r")
                .replace("\t", "\\t")
                + "\"";
    }
}
