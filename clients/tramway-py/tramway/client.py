"""
Tramway Python client.

Provides a simple interface for sending requests to a running Tramway server.
The server speaks the OpenAI chat completions protocol, so this client
translates Python calls into the correct HTTP requests and returns the
model's response as a plain string.

Basic usage::

    from tramway import Tramway

    tramway = Tramway()
    response = tramway.complete("ollama/phi4", "tell me a joke")
    print(response)

Builder usage::

    response = (tramway.builder("claude/sonnet")
        .system("You are a helpful assistant")
        .user("What is hexagonal architecture?")
        .send())
"""

import json
import urllib.request
import urllib.error
from typing import Optional

from .builder import CompletionBuilder
from .exceptions import TramwayException

DEFAULT_BASE_URL = "http://localhost:8080"


class Tramway:
    """Client for the Tramway LLM gateway."""

    def __init__(self, base_url: str = DEFAULT_BASE_URL, timeout: int = 120):
        """
        Create a Tramway client.

        :param base_url: Base URL of the Tramway server. Defaults to http://localhost:8080.
        :param timeout: Request timeout in seconds. Defaults to 120.
        """
        self.base_url = base_url.rstrip("/")
        self.timeout = timeout

    def complete(self, model: str, message: str) -> str:
        """
        Send a single user message and return the model's response.

        :param model: The model to use, e.g. ``"ollama/phi4"`` or ``"claude/sonnet"``.
        :param message: The user message.
        :returns: The model's response as a plain string.
        :raises TramwayException: If the request fails or the server returns an error.
        """
        return self.builder(model).user(message).send()

    def builder(self, model: str) -> CompletionBuilder:
        """
        Start building a request for the given model.

        Use this when you need a system prompt, conversation history,
        or Tramway extensions.

        :param model: The model to use, e.g. ``"ollama/phi4"``.
        :returns: A :class:`CompletionBuilder` for chaining.
        """
        return CompletionBuilder(model=model, base_url=self.base_url, timeout=self.timeout)

    def _send_request(self, body: dict) -> str:
        """
        Send a pre-built request body to the server and return the response text.
        Used internally by CompletionBuilder.
        """
        url = f"{self.base_url}/v1/chat/completions"
        data = json.dumps(body).encode("utf-8")

        req = urllib.request.Request(
            url,
            data=data,
            headers={"Content-Type": "application/json"},
            method="POST",
        )

        try:
            with urllib.request.urlopen(req, timeout=self.timeout) as resp:
                raw = resp.read().decode("utf-8")
        except urllib.error.HTTPError as e:
            body_text = e.read().decode("utf-8", errors="replace")
            raise TramwayException(
                f"Tramway server returned HTTP {e.code}: {body_text}"
            ) from e
        except urllib.error.URLError as e:
            raise TramwayException(
                f"Failed to reach Tramway server at {self.base_url}: {e.reason}"
            ) from e

        return _extract_content(raw)


def _extract_content(response_json: str) -> str:
    """Extract the assistant message content from an OpenAI chat completions response."""
    try:
        data = json.loads(response_json)
        return data["choices"][0]["message"]["content"]
    except (KeyError, IndexError, json.JSONDecodeError) as e:
        raise TramwayException(
            f"Unexpected response format from Tramway server: {response_json}"
        ) from e
