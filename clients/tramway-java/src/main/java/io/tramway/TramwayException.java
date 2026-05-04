package io.tramway;

/**
 * Thrown when a Tramway request fails — either a network error, a server
 * error response, or an unexpected response format.
 */
public class TramwayException extends Exception {

    public TramwayException(String message) {
        super(message);
    }

    public TramwayException(String message, Throwable cause) {
        super(message, cause);
    }
}
