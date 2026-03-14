"""Fuzz test: Python push vs PyO3 (Rust) push for FunctionInventionChunk."""
import pytest

objectiveai_pyo3 = pytest.importorskip("objectiveai_pyo3")

from polyfactory.factories.pydantic_factory import ModelFactory

from objectiveai.functions.inventions.response.streaming import FunctionInventionChunk
from tests.push_test_utils import run_push_fuzz_test


class FunctionInventionChunkFactory(ModelFactory):
    __model__ = FunctionInventionChunk


def test_push_fuzz():
    run_push_fuzz_test(
        factory_cls=FunctionInventionChunkFactory,
        model_cls=FunctionInventionChunk,
        normalize_fn=objectiveai_pyo3.function_invention_chunk_normalized,
        merged_fn=objectiveai_pyo3.function_invention_chunk_merged,
    )
