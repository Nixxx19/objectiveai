"""HTTP integration tests for recursive function inventions."""

import pytest

from tests.http_test_util import (
    ASSETS_DIR,
    HttpTestCase,
    get_test_client,
    load_snapshot,
    run_streaming,
    run_unary,
)

SNAPSHOTS_DIR = ASSETS_DIR / "functions" / "inventions" / "recursive_client_tests"
ENDPOINT = "/functions/inventions/recursive"

MOCK_INVENTION_AGENT = {"upstream": "mock", "output_mode": "instruction", "invention": True}


def normalize(fi: dict) -> dict:
    inventions = [
        {
            **inv,
            "id": "",
            "created": 0,
            "completions": [
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
                for c in inv.get("completions", [])
            ],
        }
        for inv in fi.get("inventions", [])
    ]
    inventions.sort(key=lambda inv: inv.get("state", {}).get("name", ""))
    for i, inv in enumerate(inventions):
        inv["index"] = i

    return {**fi, "id": "", "created": 0, "inventions": inventions}


CASES = [
    HttpTestCase(
        snapshot="valid_schema_valid_tasks_scalar_leaf",
        body={
            "remote": "mock",
            "name": "test/recursive",
            "state": {
                "type": "alpha.scalar.leaf.function",
                "depth": 0, "min_branch_width": 1, "max_branch_width": 1,
                "min_leaf_width": 2, "max_leaf_width": 4,
                "name": "inv-good-sl",
                "spec": "Test function spec for mock recursive invention.",
                "input_schema": {
                    "type": "object",
                    "properties": {
                        "sentiment": {"type": "string", "enum": ["positive", "negative"]},
                    },
                    "required": ["sentiment"],
                },
                "essay_tasks": "Good tasks incoming.",
                "tasks": [
                    {
                        "type": "vector.completion",
                        "messages": {"$starlark": '[{"role": "user", "content": [{"type": "text", "text": str(input)}]}]'},
                        "responses": ["yes", "no"],
                    },
                    {
                        "type": "vector.completion",
                        "messages": {"$starlark": '[{"role": "user", "content": [{"type": "text", "text": str(input)}]}]'},
                        "responses": ["yes", "no"],
                    },
                ],
                "tasks_length": 2,
                "description": "A valid scalar function.",
            },
            "agent": MOCK_INVENTION_AGENT,
            "seed": 5300,
            "stream": True,
            "max_step_retries": 1,
        },
    ),
    HttpTestCase(
        snapshot="valid_vector_schema_valid_tasks",
        body={
            "remote": "mock",
            "name": "test/recursive",
            "state": {
                "type": "alpha.vector.leaf.function",
                "depth": 0, "min_branch_width": 1, "max_branch_width": 1,
                "min_leaf_width": 2, "max_leaf_width": 4,
                "name": "inv-good-vl",
                "spec": "Test function spec for mock recursive invention.",
                "essay": "Ranking things.",
                "input_schema": {
                    "items": {"type": "string", "enum": ["apple", "banana"]},
                },
                "tasks": [
                    {
                        "type": "vector.completion",
                        "messages": {"$starlark": '[{"role": "user", "content": [{"type": "text", "text": "rank these"}]}]'},
                        "responses": {"$starlark": '[[{"type": "text", "text": str(item)}] for item in input[\'items\']]'},
                    },
                    {
                        "type": "vector.completion",
                        "messages": {"$starlark": '[{"role": "user", "content": [{"type": "text", "text": "rank these"}]}]'},
                        "responses": {"$starlark": '[[{"type": "text", "text": str(item)}] for item in input[\'items\']]'},
                    },
                ],
                "tasks_length": 2,
            },
            "agent": MOCK_INVENTION_AGENT,
            "seed": 5400,
            "stream": True,
            "max_step_retries": 1,
        },
    ),
    HttpTestCase(
        snapshot="valid_schema_no_tasks_with_essay",
        body={
            "remote": "mock",
            "name": "test/recursive",
            "state": {
                "type": "alpha.scalar.leaf.function",
                "depth": 0, "min_branch_width": 1, "max_branch_width": 1,
                "min_leaf_width": 2, "max_leaf_width": 4,
                "name": "inv-schema-only",
                "spec": "Test function spec for mock recursive invention.",
                "essay": "A great essay about things.",
                "input_schema": {
                    "type": "object",
                    "properties": {
                        "sentiment": {"type": "string", "enum": ["positive", "negative"]},
                    },
                    "required": ["sentiment"],
                },
            },
            "agent": MOCK_INVENTION_AGENT,
            "seed": 5900,
            "stream": True,
            "max_step_retries": 1,
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
        return objectiveai_pyo3.function_invention_recursive_chunk_merged(acc, chunk)

    def chunk_to_unary(acc):
        return objectiveai_pyo3.function_invention_recursive_chunk_to_unary(acc)

    result = normalize(
        await run_streaming(client, ENDPOINT, case.body, merge, chunk_to_unary)
    )
    assert result == expected
