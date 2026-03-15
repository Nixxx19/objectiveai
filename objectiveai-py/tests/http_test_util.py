"""Shared utilities for HTTP integration tests.

Requires a running ObjectiveAI API server. Set OBJECTIVEAI_TEST_PORT
environment variable to the server's port.
"""

from __future__ import annotations

import json
import os
from pathlib import Path
from typing import Any, Callable

import pytest

ASSETS_DIR = Path(__file__).resolve().parent.parent.parent / "objectiveai-api" / "assets"

_port = os.environ.get("OBJECTIVEAI_TEST_PORT")


def get_test_client():
    """Create a test client connected to the local test server."""
    if not _port:
        pytest.skip("OBJECTIVEAI_TEST_PORT not set")
    from objectiveai.client import ObjectiveAI
    return ObjectiveAI(api_base=f"http://127.0.0.1:{_port}", api_key="test")


def load_snapshot(snapshots_dir: Path, name: str) -> dict:
    """Load a snapshot JSON file."""
    return json.loads((snapshots_dir / f"{name}.json").read_text(encoding="utf-8"))


async def run_unary(client, endpoint: str, body: dict) -> dict:
    """POST to endpoint with stream=false and return parsed response."""
    return await client.post_unary(endpoint, {**body, "stream": False})


async def run_streaming(
    client,
    endpoint: str,
    body: dict,
    merge: Callable[[dict, dict], dict],
    chunk_to_unary: Callable[[dict], dict],
) -> dict:
    """POST to endpoint with stream=true, merge chunks, convert to unary."""
    stream = await client.post_streaming(endpoint, {**body, "stream": True})
    acc = None
    async for chunk in stream:
        if acc is None:
            acc = chunk
        else:
            acc = merge(acc, chunk)
    assert acc is not None, "Stream yielded no chunks"
    return chunk_to_unary(acc)


class HttpTestCase:
    """A single HTTP test case."""

    def __init__(
        self,
        snapshot: str,
        body: dict,
        endpoint: str | None = None,
    ):
        self.snapshot = snapshot
        self.body = body
        self.endpoint = endpoint
