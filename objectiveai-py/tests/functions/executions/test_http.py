"""HTTP integration tests for function executions."""

import objectiveai_pyo3

from objectiveai.functions.executions.response.streaming import FunctionExecutionChunk
from tests.http_test_util import HttpTestCase, http_test_suite, ASSETS_DIR


def execution_endpoint(repo: str) -> str:
    return f"/functions/mock/mock/{repo}/mock/profiles/mock/mock/{repo}/mock"


globals().update(http_test_suite(
    name="function executions http",
    endpoint="",
    snapshots_dir=ASSETS_DIR / "functions" / "executions" / "client_tests",
    chunk_cls=FunctionExecutionChunk,
    chunk_to_unary=objectiveai_pyo3.function_execution_chunk_to_unary,
    normalize=objectiveai_pyo3.normalize_function_execution_for_tests,
    cases=[
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
    ],
))
