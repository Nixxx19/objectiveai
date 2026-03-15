"""HTTP integration tests for function inventions."""

import pytest

from tests.http_test_util import (
    ASSETS_DIR,
    HttpTestCase,
    get_test_client,
    load_snapshot,
    run_streaming,
    run_unary,
)

SNAPSHOTS_DIR = ASSETS_DIR / "functions" / "inventions" / "client_tests"
ENDPOINT = "/functions/inventions"

MOCK_INVENTION_AGENT = {"upstream": "mock", "output_mode": "instruction", "invention": True}


def normalize(fi: dict) -> dict:
    return {
        **fi,
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
            for c in fi.get("completions", [])
        ],
    }


CASES = [
    HttpTestCase(
        snapshot="scalar_leaf_s42_0",
        body={
            "state": {
                "type": "alpha.scalar.leaf.function",
                "depth": 0, "min_branch_width": 3, "max_branch_width": 5,
                "min_leaf_width": 3, "max_leaf_width": 5,
                "name": "sl-default",
                "spec": "Test function spec for mock invention.",
            },
            "agent": MOCK_INVENTION_AGENT,
            "seed": 42,
            "stream": True,
            "max_step_retries": 1,
        },
    ),
    HttpTestCase(
        snapshot="vector_branch_s2025_0",
        body={
            "state": {
                "type": "alpha.vector.branch.function",
                "depth": 3, "min_branch_width": 2, "max_branch_width": 4,
                "min_leaf_width": 2, "max_leaf_width": 4,
                "name": "vb-deep",
                "spec": "Test function spec for mock invention.",
            },
            "agent": MOCK_INVENTION_AGENT,
            "seed": 2025,
            "stream": True,
            "max_step_retries": 1,
        },
    ),
    HttpTestCase(
        snapshot="scalar_leaf_schema_kitchen_0",
        body={
            "state": {
                "type": "alpha.scalar.leaf.function",
                "depth": 0, "min_branch_width": 3, "max_branch_width": 5,
                "min_leaf_width": 3, "max_leaf_width": 5,
                "name": "sl-kitchen",
                "spec": "Test function spec for mock invention.",
                "input_schema": {
                    "type": "object",
                    "properties": {
                        "name": {"type": "string"},
                        "age": {"type": "integer"},
                        "score": {"type": "number"},
                        "active": {"type": "boolean"},
                        "avatar": {"type": "image"},
                        "voicemail": {"type": "audio"},
                        "demo": {"type": "video"},
                        "resume": {"type": "file"},
                        "aliases": {
                            "type": "array",
                            "items": {"anyOf": [{"type": "string"}, {"type": "integer"}]},
                            "minItems": 1,
                            "maxItems": 8,
                        },
                        "extra": {
                            "anyOf": [
                                {"type": "string"},
                                {
                                    "type": "array",
                                    "items": {
                                        "type": "object",
                                        "properties": {
                                            "key": {"type": "string"},
                                            "val": {"anyOf": [{"type": "number"}, {"type": "boolean"}, {"type": "image"}]},
                                        },
                                        "required": ["key", "val"],
                                    },
                                    "minItems": 1,
                                    "maxItems": 3,
                                },
                            ],
                        },
                    },
                    "required": ["name", "age", "score", "active", "avatar", "voicemail", "demo", "resume", "aliases", "extra"],
                },
            },
            "agent": MOCK_INVENTION_AGENT,
            "seed": 80004,
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
        return objectiveai_pyo3.function_invention_chunk_merged(acc, chunk)

    def chunk_to_unary(acc):
        return objectiveai_pyo3.function_invention_chunk_to_unary(acc)

    result = normalize(
        await run_streaming(client, ENDPOINT, case.body, merge, chunk_to_unary)
    )
    assert result == expected
