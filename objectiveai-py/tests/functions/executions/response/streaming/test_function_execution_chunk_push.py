"""Fuzz test: Python push vs PyO3 (Rust) push for FunctionExecutionChunk."""
import pytest

objectiveai_pyo3 = pytest.importorskip("objectiveai_pyo3")

from polyfactory.factories.pydantic_factory import ModelFactory

from objectiveai.functions.executions.response.streaming import FunctionExecutionChunk
from tests.push_test_utils import run_push_fuzz_test


class FunctionExecutionChunkFactory(ModelFactory):
    __model__ = FunctionExecutionChunk


def test_push_fuzz():
    run_push_fuzz_test(
        factory_cls=FunctionExecutionChunkFactory,
        model_cls=FunctionExecutionChunk,
        normalize_fn=objectiveai_pyo3.function_execution_chunk_normalized,
        merged_fn=objectiveai_pyo3.function_execution_chunk_merged,
    )
