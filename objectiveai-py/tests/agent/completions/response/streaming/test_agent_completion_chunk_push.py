"""Fuzz test: Python push vs PyO3 (Rust) push for AgentCompletionChunk."""
import pytest

objectiveai_pyo3 = pytest.importorskip("objectiveai_pyo3")

from polyfactory.factories.pydantic_factory import ModelFactory

from objectiveai.agent.completions.response.streaming import AgentCompletionChunk
from tests.push_test_utils import run_push_fuzz_test


class AgentCompletionChunkFactory(ModelFactory):
    __model__ = AgentCompletionChunk


def test_push_fuzz():
    run_push_fuzz_test(
        factory_cls=AgentCompletionChunkFactory,
        model_cls=AgentCompletionChunk,
        normalize_fn=objectiveai_pyo3.agent_completion_chunk_normalized,
        merged_fn=objectiveai_pyo3.agent_completion_chunk_merged,
    )
