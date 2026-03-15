"""HTTP integration tests for agent completions."""

from pathlib import Path

import pytest

from tests.http_test_util import (
    ASSETS_DIR,
    HttpTestCase,
    get_test_client,
    load_snapshot,
    run_streaming,
    run_unary,
)

SNAPSHOTS_DIR = ASSETS_DIR / "agent" / "completions" / "client_tests"
ENDPOINT = "/agent/completions"


def normalize(c: dict) -> dict:
    return {
        **c,
        "id": "",
        "created": 0,
        "messages": [
            {**m, "upstream_id": "", "created": 0}
            for m in c.get("messages", [])
        ],
    }


CASES = [
    HttpTestCase(
        snapshot="test_basic_mock_agent_seed_42",
        body={
            "messages": [],
            "agent": {"upstream": "mock", "output_mode": "instruction"},
            "seed": 42,
        },
    ),
    HttpTestCase(
        snapshot="test_with_developer_and_user_messages",
        body={
            "messages": [
                {"role": "developer", "content": "You are a helpful assistant."},
                {"role": "user", "content": "What is 2+2?"},
            ],
            "agent": {"upstream": "mock", "output_mode": "instruction"},
            "seed": 99,
        },
    ),
    HttpTestCase(
        snapshot="test_json_object_response_format",
        body={
            "messages": [],
            "agent": {"upstream": "mock", "output_mode": "instruction"},
            "response_format": {"type": "json_object"},
            "seed": 42,
        },
    ),
]


@pytest.fixture
def client():
    return get_test_client()


@pytest.mark.asyncio
@pytest.mark.parametrize("case", CASES, ids=[c.snapshot for c in CASES])
async def test_unary(client, case):
    expected = normalize(load_snapshot(SNAPSHOTS_DIR, case.snapshot))
    result = normalize(await run_unary(client, ENDPOINT, case.body))
    assert result == expected


@pytest.mark.asyncio
@pytest.mark.parametrize("case", CASES, ids=[c.snapshot for c in CASES])
async def test_streaming(client, case):
    import objectiveai_pyo3

    expected = normalize(load_snapshot(SNAPSHOTS_DIR, case.snapshot))

    def merge(acc, chunk):
        return objectiveai_pyo3.agent_completion_chunk_merged(acc, chunk)

    def chunk_to_unary(acc):
        return objectiveai_pyo3.agent_completion_chunk_to_unary(acc)

    result = normalize(
        await run_streaming(client, ENDPOINT, case.body, merge, chunk_to_unary)
    )
    assert result == expected
