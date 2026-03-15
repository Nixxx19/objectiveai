"""Fuzz test: Python push vs PyO3 (Rust) push for VectorCompletionChunk."""
import pytest

objectiveai_pyo3 = pytest.importorskip("objectiveai_pyo3")

from polyfactory.factories.pydantic_factory import ModelFactory

from objectiveai.vector.completions.response.streaming import VectorCompletionChunk
from tests.push_test_utils import run_push_fuzz_test


class VectorCompletionChunkFactory(ModelFactory):
    __model__ = VectorCompletionChunk


def test_push_fuzz():
    run_push_fuzz_test(
        factory_cls=VectorCompletionChunkFactory,
        model_cls=VectorCompletionChunk,
        normalize_fn=objectiveai_pyo3.vector_completion_chunk_normalized,
        merged_fn=objectiveai_pyo3.vector_completion_chunk_merged,
    )
