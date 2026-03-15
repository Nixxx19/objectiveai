"""Shared utilities for push/merge fuzz tests.

Mirrors the JS merged.test.ts pattern:
1. Generate deterministic chunks via PyO3 (Rust arbitrary with seed)
2. Push via Python (Pydantic) + via PyO3 (Rust merged)
3. Compare with rounded floats for precision tolerance
"""
from __future__ import annotations

import copy
import math
from typing import Any

from pydantic import BaseModel

DIGITS = 8


def rounded(value: Any) -> Any:
    """Round all floats to DIGITS significant figures for comparison.

    Mirrors mergeTestUtil.ts rounded().
    """
    if isinstance(value, bool):
        return value
    if isinstance(value, float):
        if value == 0 or not math.isfinite(value):
            return value
        return float(f"{value:.{DIGITS}g}")
    if isinstance(value, list):
        return [rounded(v) for v in value]
    if isinstance(value, dict):
        return {k: rounded(v) for k, v in value.items()}
    return value


def pydantic_push(acc: dict, chunk: dict, model_cls: type[BaseModel]) -> dict:
    """Push a chunk into an accumulator using the Python Pydantic implementation.

    Deserializes both dicts into Pydantic models, calls push(), and
    re-serializes to a dict for comparison.
    """
    acc_model = model_cls.model_validate(acc)
    chunk_model = model_cls.model_validate(chunk)
    acc_model.push(chunk_model)
    return acc_model.model_dump(mode="python", by_alias=True)
