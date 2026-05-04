package io.tramway;

import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

class TramwayTest {

    @Test
    void clientCreatesWithDefaultUrl() {
        Tramway tramway = new Tramway();
        assertNotNull(tramway);
    }

    @Test
    void clientCreatesWithCustomUrl() {
        Tramway tramway = new Tramway("http://localhost:9000");
        assertNotNull(tramway);
    }

    @Test
    void builderRequiresAtLeastOneMessage() {
        Tramway tramway = new Tramway();
        TramwayException ex = assertThrows(TramwayException.class, () ->
            tramway.builder("ollama/phi4").send()
        );
        assertTrue(ex.getMessage().contains("user message"));
    }

    @Test
    void requestSerializerProducesValidJson() {
        var messages = java.util.List.of(
            new CompletionBuilder.Message("user", "hello")
        );
        String json = RequestSerializer.serialize("ollama/phi4", "You are helpful", messages, null);
        assertTrue(json.contains("\"model\":\"ollama/phi4\""));
        assertTrue(json.contains("\"role\":\"system\""));
        assertTrue(json.contains("\"role\":\"user\""));
        assertTrue(json.contains("\"content\":\"hello\""));
    }

    @Test
    void requestSerializerEscapesSpecialCharacters() {
        var messages = java.util.List.of(
            new CompletionBuilder.Message("user", "say \"hello\"\nnewline")
        );
        String json = RequestSerializer.serialize("ollama/phi4", null, messages, null);
        assertTrue(json.contains("\\\"hello\\\""));
        assertTrue(json.contains("\\n"));
    }

    @Test
    void responseParserExtractsContent() throws TramwayException {
        String json = "{\"id\":\"chatcmpl-abc\",\"choices\":[{\"message\":{\"role\":\"assistant\",\"content\":\"The answer is 42.\"}}]}";
        String content = ResponseParser.extractContent(json);
        assertEquals("The answer is 42.", content);
    }

    @Test
    void responseParserHandlesEscapedContent() throws TramwayException {
        String json = "{\"choices\":[{\"message\":{\"role\":\"assistant\",\"content\":\"line1\\nline2\"}}]}";
        String content = ResponseParser.extractContent(json);
        assertEquals("line1\nline2", content);
    }

    @Test
    void extensionsBuilderSetsFields() {
        TramwayExtensions ext = new TramwayExtensions.Builder()
            .traceId("abc-123")
            .preferLocal(true)
            .build();
        assertEquals("abc-123", ext.getTraceId());
        assertTrue(ext.getPreferLocal());
    }
}
