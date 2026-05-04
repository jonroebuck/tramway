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
        } catch (Exception e) {
            throw new TramwayException("Failed to send request to Tramway gateway", e);
        }

        if (response.statusCode() != 200) {
            throw new TramwayException("Tramway gateway returned non-200 status: " + response.statusCode());
        }

        return ResponseParser.extractContent(response.body());
    }

    private static String jsonEscape(String value) {
        return value
                .replace("\\", "\\\\")
                .replace("\"", "\\\"")
                .replace("\n", "\\n")
                .replace("\r", "\\r")
                .replace("\t", "\\t");
    }
}
