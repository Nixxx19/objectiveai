"""HTTP integration tests for function executions."""

import math
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

SNAPSHOTS_DIR = ASSETS_DIR / "functions" / "executions" / "client_tests"


def execution_endpoint(repo: str) -> str:
    return f"/functions/mock/mock/{repo}/mock/profiles/mock/mock/{repo}/mock"


def round_float(n):
    if isinstance(n, (int, float)):
        return round(n * 1e10) / 1e10
    return n


def round_logprobs(logprobs):
    if not logprobs:
        return logprobs
    def round_entry(e):
        result = {**e, "logprob": round(e["logprob"] * 1e12) / 1e12}
        if e.get("top_logprobs"):
            result["top_logprobs"] = [
                {**t, "logprob": round(t["logprob"] * 1e12) / 1e12}
                for t in e["top_logprobs"]
            ]
        return result
    return {
        **logprobs,
        "content": [round_entry(e) for e in logprobs["content"]] if logprobs.get("content") else logprobs.get("content"),
        "refusal": [round_entry(e) for e in logprobs["refusal"]] if logprobs.get("refusal") else logprobs.get("refusal"),
    }


def normalize_completion(c: dict) -> dict:
    return {
        **c,
        "id": "",
        "created": 0,
        "messages": [
            {**m, "upstream_id": "", "created": 0, "logprobs": round_logprobs(m.get("logprobs"))}
            if m.get("role") == "assistant"
            else m
            for m in c.get("messages", [])
        ],
    }


def normalize_vc_task(task: dict) -> dict:
    completions = [normalize_completion(c) for c in task.get("completions", [])]
    completions.sort(
        key=lambda c: (
            c.get("messages", [{}])[0].get("agent") or "",
            c.get("messages", [{}])[0].get("content") or "",
        )
    )
    for i, c in enumerate(completions):
        c["index"] = i

    votes = [
        {**v, "prompt_id": "", "responses_ids": [], "vote": [round_float(x) for x in v.get("vote", [])]}
        for v in task.get("votes", [])
    ]
    votes.sort(key=lambda v: v.get("agent", ""))

    scores = [round_float(s) for s in task.get("scores", [])] if task.get("scores") else task.get("scores")
    weights = [round_float(w) for w in task.get("weights", [])] if task.get("weights") else task.get("weights")

    return {**task, "id": "", "created": 0, "completions": completions, "votes": votes, "scores": scores, "weights": weights}


def normalize_fe(fe: dict) -> dict:
    output = fe.get("output")
    if isinstance(output, list):
        output = [round_float(x) for x in output]
    elif isinstance(output, (int, float)):
        output = round_float(output)

    tasks = []
    for task in fe.get("tasks", []):
        obj = task.get("object", "")
        if obj == "vector.completion":
            tasks.append(normalize_vc_task(task))
        elif obj and obj.endswith(".function.execution"):
            tasks.append(normalize_fe(task))
        else:
            tasks.append(task)

    return {
        **fe,
        "id": "",
        "created": 0,
        "retry_token": None,
        "output": output,
        "tasks": tasks,
    }


CASES = [
    HttpTestCase(
        snapshot="mock_1_scalar_leaf_binary_seed_42",
        endpoint=execution_endpoint("mock-1"),
        body={"input": {"text": "Hello world"}, "seed": 42},
    ),
    HttpTestCase(
        snapshot="mock_7_vector_5_criteria_seed_42",
        endpoint=execution_endpoint("mock-7"),
        body={"input": {"items": ["Option A", "Option B", "Option C"]}, "seed": 42},
    ),
    HttpTestCase(
        snapshot="mock_20_vector_super_branch_seed_42",
        endpoint=execution_endpoint("mock-20"),
        body={"input": {"items": ["Alpha", "Beta", "Gamma"]}, "seed": 42},
    ),
]


@pytest.fixture
def client():
    return get_test_client()


@pytest.mark.asyncio
@pytest.mark.parametrize("case", CASES, ids=[c.snapshot for c in CASES])
async def test_unary(client, case):
    expected = normalize_fe(load_snapshot(SNAPSHOTS_DIR, case.snapshot))
    result = normalize_fe(await run_unary(client, case.endpoint, case.body))
    assert result == expected


@pytest.mark.asyncio
@pytest.mark.parametrize("case", CASES, ids=[c.snapshot for c in CASES])
async def test_streaming(client, case):
    import objectiveai_pyo3

    expected = normalize_fe(load_snapshot(SNAPSHOTS_DIR, case.snapshot))

    def merge(acc, chunk):
        return objectiveai_pyo3.function_execution_chunk_merged(acc, chunk)

    def chunk_to_unary(acc):
        return objectiveai_pyo3.function_execution_chunk_to_unary(acc)

    result = normalize_fe(
        await run_streaming(client, case.endpoint, case.body, merge, chunk_to_unary)
    )
    assert result == expected
