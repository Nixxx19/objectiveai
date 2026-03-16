"""Shared utilities for HTTP integration tests.

Requires a running ObjectiveAI API server. Set OBJECTIVEAI_TEST_PORT
environment variable to the server's port.
"""

from __future__ import annotations

import json
import os
from pathlib import Path
from typing import Any, Callable, Type

import pytest

from tests.push_test_utils import rounded

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
    chunk_cls: Type,
    chunk_to_unary: Callable[[dict], dict],
) -> dict:
    """POST to endpoint with stream=true, push chunks via Pydantic, convert to unary."""
    stream = await client.post_streaming(endpoint, {**body, "stream": True})
    acc = None
    async for raw_chunk in stream:
        chunk = chunk_cls.model_validate(raw_chunk)
        if acc is None:
            acc = chunk
        else:
            acc.push(chunk)
    assert acc is not None, "Stream yielded no chunks"
    return chunk_to_unary(acc.model_dump(mode="python", by_alias=True, exclude_unset=True))


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


def http_test_suite(
    *,
    name: str,
    endpoint: str,
    snapshots_dir: Path,
    chunk_cls: Type,
    chunk_to_unary: Callable[[dict], dict],
    normalize: Callable[[dict], dict],
    cases: list[HttpTestCase],
):
    """Generate a parametrized HTTP test suite (unary + streaming).

    Mirrors httpTestSuite() from objectiveai-js/src/httpTestUtil.ts.
    Streaming tests use the Pydantic chunk_cls.push() method (the native
    Python SDK implementation), matching how JS tests use the TS merge.
    Returns a dict of test functions that pytest will collect from the
    caller's module globals.
    """

    @pytest.fixture
    def client():
        return get_test_client()

    @pytest.mark.asyncio
    @pytest.mark.parametrize("case", cases, ids=[c.snapshot for c in cases])
    async def test_unary(client, case):
        expected = rounded(load_snapshot(snapshots_dir, case.snapshot))
        result = rounded(normalize(await run_unary(client, case.endpoint or endpoint, case.body)))
        assert result == expected

    @pytest.mark.asyncio
    @pytest.mark.parametrize("case", cases, ids=[c.snapshot for c in cases])
    async def test_streaming(client, case):
        expected = rounded(load_snapshot(snapshots_dir, case.snapshot))
        result = rounded(normalize(
            await run_streaming(client, case.endpoint or endpoint, case.body, chunk_cls, chunk_to_unary)
        ))
        assert result == expected

    return {"client": client, "test_unary": test_unary, "test_streaming": test_streaming}
