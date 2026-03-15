"""Fuzz test: Python push vs PyO3 (Rust) push for FunctionProfileComputationChunk."""
import pytest

objectiveai_pyo3 = pytest.importorskip("objectiveai_pyo3")

from polyfactory.factories.pydantic_factory import ModelFactory

from objectiveai.functions.profiles.computations.response.streaming import (
    FunctionProfileComputationChunk,
)
from tests.push_test_utils import run_push_fuzz_test


class FunctionProfileComputationChunkFactory(ModelFactory):
    __model__ = FunctionProfileComputationChunk


def test_push_fuzz():
    run_push_fuzz_test(
        factory_cls=FunctionProfileComputationChunkFactory,
        model_cls=FunctionProfileComputationChunk,
        normalize_fn=objectiveai_pyo3.function_profile_computation_chunk_normalized,
        merged_fn=objectiveai_pyo3.function_profile_computation_chunk_merged,
    )
