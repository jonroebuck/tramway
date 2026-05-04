package io.tramway;

/**
 * Extracts the assistant message content from an OpenAI chat completions
 * response body.
 *
 * <p>Uses no external JSON library. Parses only the fields tramway needs:
 * {@code choices[0].message.content}.
 */
class ResponseParser {

    /**
     * Extract the response text from a chat completions JSON response.
     *
     * @param json the raw response body from tramway-server
     * @return the assistant's message content
     * @throws TramwayException if the content cannot be found in the response
     */
    static String extractContent(String json) throws TramwayException {
        // Find "content": "..." inside the choices array.
        // The response shape is always:
        // {"choices":[{"message":{"role":"assistant","content":"..."}}]}
        int contentKey = json.indexOf("\"content\":");
        if (contentKey == -1) {
            throw new TramwayException("Unexpected response format — no content field found: " + json);
        }

        int valueStart = json.indexOf("\"", contentKey + 10);
        if (valueStart == -1) {
            throw new TramwayException("Unexpected response format — content value not found: " + json);
        }

        // Walk forward handling escape sequences
        StringBuilder result = new StringBuilder();
        int i = valueStart + 1;
        while (i < json.length()) {
            char c = json.charAt(i);
            if (c == '\\' && i + 1 < json.length()) {
                char next = json.charAt(i + 1);
                switch (next) {
                    case '"':  result.append('"');  i += 2; continue;
                    case '\\': result.append('\\'); i += 2; continue;
                    case 'n':  result.append('\n'); i += 2; continue;
                    case 'r':  result.append('\r'); i += 2; continue;
                    case 't':  result.append('\t'); i += 2; continue;
                    default:   result.append(next); i += 2; continue;
                }
            }
            if (c == '"') break; // end of string value
            result.append(c);
            i++;
        }

        return result.toString();
    }
}
