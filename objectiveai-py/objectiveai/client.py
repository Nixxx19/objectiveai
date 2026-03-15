"""ObjectiveAI API client."""

from __future__ import annotations

import os
from typing import Any, TypeVar

import httpx

from objectiveai.error import ObjectiveAIFetchError
from objectiveai.stream import Stream

T = TypeVar("T")

DEFAULT_API_BASE = "https://api.objective-ai.io"


class ObjectiveAI:
    """Client for the ObjectiveAI API.

    Args:
        api_key: API key for authentication.
            Falls back to ``OBJECTIVEAI_API_KEY`` env var.
        api_base: Base URL for the API.
            Falls back to ``OBJECTIVEAI_API_BASE`` env var,
            then ``https://api.objective-ai.io``.
        user_agent: ``User-Agent`` header.
            Falls back to ``USER_AGENT`` env var.
        x_title: ``X-Title`` header.
            Falls back to ``X_TITLE`` env var.
        http_referer: ``HTTP-Referer`` header.
            Falls back to ``HTTP_REFERER`` env var.
        x_github_authorization: ``X-GITHUB-AUTHORIZATION`` header
            for GitHub-hosted function/profile access.
        x_openrouter_authorization: ``X-OPENROUTER-AUTHORIZATION`` header
            for BYOK (Bring Your Own Key) support.
        x_mcp_authorization: Map from MCP server URL to authorization
            header value, sent as ``X-MCP-AUTHORIZATION``.
        timeout: Request timeout in seconds (default 60).

    Usage::

        from objectiveai import ObjectiveAI

        client = ObjectiveAI(api_key="apk_...")
    """

    def __init__(
        self,
        *,
        api_key: str | None = None,
        api_base: str | None = None,
        user_agent: str | None = None,
        x_title: str | None = None,
        http_referer: str | None = None,
        x_github_authorization: str | None = None,
        x_openrouter_authorization: str | None = None,
        x_mcp_authorization: dict[str, str] | None = None,
        timeout: float = 60.0,
    ) -> None:
        self.api_key = api_key or os.environ.get("OBJECTIVEAI_API_KEY")
        self.api_base = (
            api_base
            or os.environ.get("OBJECTIVEAI_API_BASE")
            or DEFAULT_API_BASE
        )
        self.user_agent = user_agent or os.environ.get("USER_AGENT")
        self.x_title = x_title or os.environ.get("X_TITLE")
        self.http_referer = http_referer or os.environ.get("HTTP_REFERER")
        self.x_github_authorization = x_github_authorization
        self.x_openrouter_authorization = x_openrouter_authorization
        self.x_mcp_authorization = x_mcp_authorization
        self.timeout = timeout

    def _build_headers(
        self,
        extra_headers: dict[str, str] | None = None,
    ) -> dict[str, str]:
        """Build headers for a request."""
        headers: dict[str, str] = {"Content-Type": "application/json"}

        if self.api_key:
            headers["Authorization"] = f"Bearer {self.api_key}"
        if self.user_agent:
            headers["User-Agent"] = self.user_agent
        if self.x_title:
            headers["X-Title"] = self.x_title
        if self.http_referer:
            headers["HTTP-Referer"] = self.http_referer
        if self.x_github_authorization:
            headers["X-GITHUB-AUTHORIZATION"] = self.x_github_authorization
        if self.x_openrouter_authorization:
            headers["X-OPENROUTER-AUTHORIZATION"] = self.x_openrouter_authorization
        if self.x_mcp_authorization:
            import json
            headers["X-MCP-AUTHORIZATION"] = json.dumps(self.x_mcp_authorization)

        if extra_headers:
            headers.update(extra_headers)

        return headers

    def _build_url(self, path: str) -> str:
        """Build the full URL for a path."""
        base = self.api_base.rstrip("/")
        if not path.startswith("/"):
            path = f"/{path}"
        return f"{base}{path}"

    @staticmethod
    async def _handle_error_response(response: httpx.Response) -> ObjectiveAIFetchError:
        """Create an error from a failed response."""
        try:
            raw_body = response.text
        except Exception:
            raw_body = None
        return ObjectiveAIFetchError(response.status_code, raw_body)

    # ------------------------------------------------------------------
    # Unary requests
    # ------------------------------------------------------------------

    async def get_unary(
        self,
        path: str,
        body: Any = None,
        *,
        headers: dict[str, str] | None = None,
    ) -> Any:
        """Perform a GET request and return the parsed JSON response."""
        async with httpx.AsyncClient(timeout=self.timeout) as http:
            response = await http.request(
                "GET",
                self._build_url(path),
                headers=self._build_headers(headers),
                content=_json_body(body),
            )
        if not response.is_success:
            raise await self._handle_error_response(response)
        return response.json()

    async def post_unary(
        self,
        path: str,
        body: Any = None,
        *,
        headers: dict[str, str] | None = None,
    ) -> Any:
        """Perform a POST request and return the parsed JSON response."""
        async with httpx.AsyncClient(timeout=self.timeout) as http:
            response = await http.request(
                "POST",
                self._build_url(path),
                headers=self._build_headers(headers),
                content=_json_body(body),
            )
        if not response.is_success:
            raise await self._handle_error_response(response)
        return response.json()

    async def delete_unary(
        self,
        path: str,
        body: Any = None,
        *,
        headers: dict[str, str] | None = None,
    ) -> Any:
        """Perform a DELETE request and return the parsed JSON response."""
        async with httpx.AsyncClient(timeout=self.timeout) as http:
            response = await http.request(
                "DELETE",
                self._build_url(path),
                headers=self._build_headers(headers),
                content=_json_body(body),
            )
        if not response.is_success:
            raise await self._handle_error_response(response)
        return response.json()

    # ------------------------------------------------------------------
    # Streaming requests
    # ------------------------------------------------------------------

    async def get_streaming(
        self,
        path: str,
        body: Any = None,
        *,
        headers: dict[str, str] | None = None,
    ) -> Stream[Any]:
        """Perform a GET request and return an SSE stream."""
        h = self._build_headers(headers)
        h["Accept"] = "text/event-stream"

        http = httpx.AsyncClient(timeout=self.timeout)
        response = await http.send(
            http.build_request(
                "GET",
                self._build_url(path),
                headers=h,
                content=_json_body(body),
            ),
            stream=True,
        )

        if not response.is_success:
            await response.aread()
            await http.aclose()
            raise await self._handle_error_response(response)

        return Stream(response)

    async def post_streaming(
        self,
        path: str,
        body: Any = None,
        *,
        headers: dict[str, str] | None = None,
    ) -> Stream[Any]:
        """Perform a POST request and return an SSE stream."""
        h = self._build_headers(headers)
        h["Accept"] = "text/event-stream"

        http = httpx.AsyncClient(timeout=self.timeout)
        response = await http.send(
            http.build_request(
                "POST",
                self._build_url(path),
                headers=h,
                content=_json_body(body),
            ),
            stream=True,
        )

        if not response.is_success:
            await response.aread()
            await http.aclose()
            raise await self._handle_error_response(response)

        return Stream(response)

    async def delete_streaming(
        self,
        path: str,
        body: Any = None,
        *,
        headers: dict[str, str] | None = None,
    ) -> Stream[Any]:
        """Perform a DELETE request and return an SSE stream."""
        h = self._build_headers(headers)
        h["Accept"] = "text/event-stream"

        http = httpx.AsyncClient(timeout=self.timeout)
        response = await http.send(
            http.build_request(
                "DELETE",
                self._build_url(path),
                headers=h,
                content=_json_body(body),
            ),
            stream=True,
        )

        if not response.is_success:
            await response.aread()
            await http.aclose()
            raise await self._handle_error_response(response)

        return Stream(response)


def _json_body(body: Any) -> bytes | None:
    """Serialize body to JSON bytes, or None if absent."""
    if body is None:
        return None
    import json
    return json.dumps(body).encode("utf-8")
