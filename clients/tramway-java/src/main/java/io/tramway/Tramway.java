package io.tramway;

import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.time.Duration;

public class Tramway {

    private final String baseUrl;
    private final HttpClient httpClient;

    public Tramway() {
        this("http://localhost:8080");
    }

    public Tramway(String baseUrl) {
        this.baseUrl = baseUrl.replaceAll("/+$", "");
        this.httpClient = HttpClient.newBuilder()
                .connectTimeout(Duration.ofSeconds(10))
                .build();
    }

    public String complete(String model, String message) throws TramwayException {
        String escapedModel = jsonEscape(model);
        String escapedMessage = jsonEscape(message);
        String jsonBody = "{\"model\":\"" + escapedModel + "\","
                + "\"messages\":[{\"role\":\"user\",\"content\":\"" + escapedMessage + "\"}]}";
        return sendRequest(jsonBody);
    }

    public CompletionBuilder builder(String model) {
        return new CompletionBuilder(model, baseUrl, httpClient);
    }

    String sendRequest(String jsonBody) throws TramwayException {
        HttpRequest request = HttpRequest.newBuilder()
                .uri(URI.create(baseUrl + "/v1/chat/completions"))
                .header("Content-Type", "application/json")
                .POST(HttpRequest.BodyPublishers.ofString(jsonBody))
                .timeout(Duration.ofSeconds(120))
                .build();

        HttpResponse<String> response;
        try {
            response = httpClient.send(request, HttpResponse.BodyHandlers.ofString());
        } catch (java.io.IOException e) {
            throw new TramwayException("Failed to send request to Tramway gateway", e);
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            throw new TramwayException("Request to Tramway gateway was interrupted", e);
        }

        if (response.statusCode() != 200) {
            throw new TramwayException("Tramway gateway returned non-200 status: " + response.statusCode());
        }

        return ResponseParser.extractContent(response.body());
    }

    private static String jsonEscape(String value) {
        StringBuilder sb = new StringBuilder(value.length() + 16);
        for (int i = 0; i < value.length(); i++) {
            char c = value.charAt(i);
            switch (c) {
                case '\\': sb.append("\\\\"); break;
                case '"':  sb.append("\\\""); break;
                case '\n': sb.append("\\n");  break;
                case '\r': sb.append("\\r");  break;
                case '\t': sb.append("\\t");  break;
                case '\b': sb.append("\\b");  break;
                case '\f': sb.append("\\f");  break;
                default:
                    if (c < 0x20) {
                        sb.append(String.format("\\u%04x", (int) c));
                    } else {
                        sb.append(c);
                    }
                    break;
            }
        }
        return sb.toString();
    }
}
