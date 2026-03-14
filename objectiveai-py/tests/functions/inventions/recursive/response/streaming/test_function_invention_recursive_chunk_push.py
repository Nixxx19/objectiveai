"""Fuzz test: Python push vs PyO3 (Rust) push for FunctionInventionRecursiveChunk."""
import pytest

objectiveai_pyo3 = pytest.importorskip("objectiveai_pyo3")

from polyfactory.factories.pydantic_factory import ModelFactory

from objectiveai.functions.inventions.recursive.response.streaming import (
    FunctionInventionRecursiveChunk,
)
from tests.push_test_utils import run_push_fuzz_test


class FunctionInventionRecursiveChunkFactory(ModelFactory):
    __model__ = FunctionInventionRecursiveChunk


def test_push_fuzz():
    run_push_fuzz_test(
        factory_cls=FunctionInventionRecursiveChunkFactory,
        model_cls=FunctionInventionRecursiveChunk,
        normalize_fn=objectiveai_pyo3.function_invention_recursive_chunk_normalized,
        merged_fn=objectiveai_pyo3.function_invention_recursive_chunk_merged,
    )
