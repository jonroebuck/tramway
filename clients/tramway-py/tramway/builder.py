"""
Fluent builder for Tramway chat completion requests.
"""

from __future__ import annotations
from typing import Optional, List, TYPE_CHECKING

if TYPE_CHECKING:
    from .client import Tramway


class TramwayExtensions:
    """
    Optional Tramway-specific extensions for a completion request.

    These map to the ``x_tramway`` field in the OpenAI-compatible request body.
    Standard OpenAI clients that don't know about this field simply omit it.

    Usage::

        ext = TramwayExtensions(trace_id="my-trace-123", prefer_local=True)

        response = (tramway.builder("ollama/phi4")
            .user("tell me a joke")
            .extensions(ext)
            .send())
    """

    def __init__(
        self,
        trace_id: Optional[str] = None,
        prefer_local: Optional[bool] = None,
        extensions: Optional[List[str]] = None,
    ):
        self.trace_id = trace_id
        self.prefer_local = prefer_local
        self.extensions = extensions

    def to_dict(self) -> dict:
        result = {}
        if self.trace_id is not None:
            result["trace_id"] = self.trace_id
        if self.prefer_local is not None:
            result["prefer_local"] = self.prefer_local
        if self.extensions is not None:
            result["extensions"] = self.extensions
        return result


class CompletionBuilder:
    """
    Fluent builder for a Tramway chat completion request.

    Usage::

        response = (tramway.builder("claude/sonnet")
            .system("You are a helpful assistant")
            .user("What is hexagonal architecture?")
            .send())
    """

    def __init__(self, model: str, base_url: str, timeout: int):
        self._model = model
        self._base_url = base_url
        self._timeout = timeout
        self._system: Optional[str] = None
        self._messages: list = []
        self._extensions: Optional[TramwayExtensions] = None

    def system(self, prompt: str) -> CompletionBuilder:
        """Set the system prompt."""
        self._system = prompt
        return self

    def user(self, content: str) -> CompletionBuilder:
        """Add a user message."""
        self._messages.append({"role": "user", "content": content})
        return self

    def assistant(self, content: str) -> CompletionBuilder:
        """Add an assistant message (for multi-turn history)."""
        self._messages.append({"role": "assistant", "content": content})
        return self

    def extensions(self, ext: TramwayExtensions) -> CompletionBuilder:
        """Attach Tramway-specific extensions."""
        self._extensions = ext
        return self

    def send(self) -> str:
        """
        Send the request and return the model's response.

        :raises TramwayException: If no user message has been added, or if the
            request fails or the server returns an error.
        """
        from .exceptions import TramwayException
        from .client import Tramway

        if not self._messages:
            raise TramwayException("At least one user message is required")

        body = self._build_body()
        client = Tramway(base_url=self._base_url, timeout=self._timeout)
        return client._send_request(body)

    def _build_body(self) -> dict:
        messages = []

        if self._system:
            messages.append({"role": "system", "content": self._system})

        messages.extend(self._messages)

        body: dict = {
            "model": self._model,
            "messages": messages,
        }

        if self._extensions:
            ext_dict = self._extensions.to_dict()
            if ext_dict:
                body["x_tramway"] = ext_dict

        return body
