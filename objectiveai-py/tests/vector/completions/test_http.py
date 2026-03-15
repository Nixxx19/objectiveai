"""HTTP integration tests for vector completions."""

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

SNAPSHOTS_DIR = ASSETS_DIR / "vector" / "completions" / "client_tests"
ENDPOINT = "/vector/completions"

MOCK_AGENT = {"upstream": "mock", "output_mode": "instruction"}


def normalize(vc: dict) -> dict:
    completions = [
        {
            **c,
            "id": "",
            "created": 0,
            "messages": [
                {**m, "upstream_id": "", "created": 0}
                if m.get("role") == "assistant"
                else m
                for m in c.get("messages", [])
            ],
        }
        for c in vc.get("completions", [])
    ]
    completions.sort(
        key=lambda c: (
            c.get("messages", [{}])[0].get("agent", ""),
            c.get("messages", [{}])[0].get("content", ""),
        )
    )
    for i, c in enumerate(completions):
        c["index"] = i

    votes = [
        {**v, "prompt_id": "", "responses_ids": []}
        for v in vc.get("votes", [])
    ]
    votes.sort(key=lambda v: v.get("agent", ""))

    return {**vc, "id": "", "created": 0, "completions": completions, "votes": votes}


CASES = [
    HttpTestCase(
        snapshot="single_agent_2_responses_instruction_seed_42",
        body={
            "messages": [{"role": "user", "content": "Which is better?"}],
            "ensemble": {"agents": [MOCK_AGENT]},
            "profile": ["1"],
            "responses": ["Response A", "Response B"],
            "seed": 42,
        },
    ),
    HttpTestCase(
        snapshot="many_responses_deep_prefix_tree_seed_42",
        body={
            "messages": [{"role": "user", "content": "Pick the best"}],
            "ensemble": {"agents": [MOCK_AGENT]},
            "profile": ["1"],
            "responses": [f"Response {i}" for i in range(25)],
            "seed": 42,
        },
    ),
    HttpTestCase(
        snapshot="mixed_output_modes_seed_88",
        body={
            "messages": [
                {"role": "user", "content": "Compare these vacation destinations"},
            ],
            "ensemble": {
                "agents": [
                    {"upstream": "mock", "output_mode": "instruction"},
                    {"upstream": "mock", "output_mode": "json_schema"},
                    {"upstream": "mock", "output_mode": "tool_call"},
                ],
            },
            "profile": ["0.4", "0.3", "0.3"],
            "responses": [
                "Kyoto, Japan",
                "Reykjavik, Iceland",
                "Patagonia, Argentina",
            ],
            "seed": 88,
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
        return objectiveai_pyo3.vector_completion_chunk_merged(acc, chunk)

    def chunk_to_unary(acc):
        return objectiveai_pyo3.vector_completion_chunk_to_unary(acc)

    result = normalize(
        await run_streaming(client, ENDPOINT, case.body, merge, chunk_to_unary)
    )
    assert result == expected
