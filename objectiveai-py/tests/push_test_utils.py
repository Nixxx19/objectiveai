"""Shared utilities for push/merge fuzz tests.

Mirrors the pattern from objectiveai-js's zockerParse.ts + merged.test.ts:
1. Generate random chunks via polyfactory
2. Normalize through PyO3 (Rust serde round-trip)
3. Push via Python implementation + via PyO3 merged function
4. Assert results are identical after each push
"""
from __future__ import annotations

import copy
import math
import random
from typing import Any

from pydantic import BaseModel


_NEGATIVE_NUMBER_MIN = -1000
_POSITIVE_NUMBER_MAX = 1000


def sanitize_for_serde(value: Any) -> Any:
    """Sanitize generated data for Rust serde compatibility.

    Mirrors fixForSerde from zockerParse.ts:
    - Non-finite floats → 0
    - Numbers below -1000 → random in [-1000, 0)
    - Numbers above 1000 → random in [0, 1000)
    - Recursively processes nested structures
    """
    # bare object() — polyfactory generates these for `object` type hints
    if type(value) is object:
        return None
    if isinstance(value, bool):
        return value
    if isinstance(value, int):
        if value < _NEGATIVE_NUMBER_MIN:
            return random.randint(_NEGATIVE_NUMBER_MIN, -1)
        if value > _POSITIVE_NUMBER_MAX:
            return random.randint(0, _POSITIVE_NUMBER_MAX)
        return value
    if isinstance(value, float):
        if not math.isfinite(value):
            return 0.0
        if value < _NEGATIVE_NUMBER_MIN:
            return float(random.randint(_NEGATIVE_NUMBER_MIN, -1))
        if value > _POSITIVE_NUMBER_MAX:
            return float(random.randint(0, _POSITIVE_NUMBER_MAX))
        return value
    if isinstance(value, dict):
        return {k: sanitize_for_serde(v) for k, v in value.items()}
    if isinstance(value, list):
        return [sanitize_for_serde(v) for v in value]
    return value


def generate_normalized(factory_cls: type, normalize_fn) -> dict:
    """Generate a random chunk, sanitize it, and normalize through Rust serde.

    Like zockerParse in JS: generate → fixForSerde → wasmNormalized → parse.
    """
    instance = factory_cls.build()
    raw = instance.model_dump(mode="python", by_alias=True)
    sanitized = sanitize_for_serde(raw)
    normalized = normalize_fn(sanitized)
    return normalized


def pydantic_push(acc: dict, chunk: dict, model_cls: type[BaseModel]) -> dict:
    """Push a chunk into an accumulator using the Python Pydantic implementation.

    Deserializes both dicts into Pydantic models, calls push(), and
    re-serializes to a dict for comparison.
    """
    acc_model = model_cls.model_validate(acc)
    chunk_model = model_cls.model_validate(chunk)
    acc_model.push(chunk_model)
    return acc_model.model_dump(mode="python", by_alias=True)


def run_push_fuzz_test(
    factory_cls: type,
    model_cls: type[BaseModel],
    normalize_fn,
    merged_fn,
    *,
    num_streams: int = 20,
    chunks_per_stream: int = 20,
) -> None:
    """Run the full fuzz test: generate random chunks, push via Python and
    PyO3, assert identical results after each push.

    Mirrors the JS test pattern:
    - num_streams independent test streams
    - chunks_per_stream chunks merged per stream
    - After each push, Python result must match PyO3 result
    """
    for _stream in range(num_streams):
        # Initial chunk (same for both implementations)
        py_acc = generate_normalized(factory_cls, normalize_fn)
        pyo3_acc = copy.deepcopy(py_acc)

        for _chunk_idx in range(chunks_per_stream):
            chunk = generate_normalized(factory_cls, normalize_fn)

            # Push via Python (Pydantic)
            py_acc = pydantic_push(py_acc, chunk, model_cls)
            # Normalize Python result through Rust serde for fair comparison
            py_acc = normalize_fn(py_acc)

            # Push via PyO3 (Rust)
            pyo3_acc = merged_fn(pyo3_acc, chunk)

            assert py_acc == pyo3_acc, (
                f"Mismatch at stream {_stream}, chunk {_chunk_idx}:\n"
                f"Python: {py_acc}\n"
                f"PyO3:   {pyo3_acc}"
            )
