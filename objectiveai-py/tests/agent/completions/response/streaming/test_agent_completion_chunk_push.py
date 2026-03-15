"""Fuzz test: Python push vs PyO3 (Rust) push for AgentCompletionChunk."""
import copy

import pytest

objectiveai_pyo3 = pytest.importorskip("objectiveai_pyo3")

from objectiveai.agent.completions.response.streaming import AgentCompletionChunk
from tests.push_test_utils import pydantic_push, rounded


@pytest.mark.parametrize("stream", range(20))
def test_push_fuzz(stream):
    seed = stream * 1000
    py_acc = objectiveai_pyo3.generate_agent_completion_chunk(seed)
    pyo3_acc = copy.deepcopy(py_acc)
    seed += 1

    for j in range(20):
        chunk = objectiveai_pyo3.generate_agent_completion_chunk(seed)
        seed += 1

        py_acc = pydantic_push(py_acc, chunk, AgentCompletionChunk)
        pyo3_acc = objectiveai_pyo3.agent_completion_chunk_merged(pyo3_acc, chunk)

        assert rounded(py_acc) == rounded(pyo3_acc), f"chunk {j}"
