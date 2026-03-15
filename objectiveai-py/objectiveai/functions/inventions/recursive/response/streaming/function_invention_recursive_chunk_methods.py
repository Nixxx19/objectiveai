"""Methods for FunctionInventionRecursiveChunk."""
from __future__ import annotations

from objectiveai.push_utils import push_by_index, push_option, push_lazy_set_true
from objectiveai.functions.inventions.recursive.response.streaming.function_invention_recursive_chunk import (
    FunctionInventionRecursiveChunk,
)


def _push(self, other: FunctionInventionRecursiveChunk) -> None:
    # inventions: merge by index
    push_by_index(self.inventions, other.inventions)

    # inventions_errors: lazy set true
    self.inventions_errors = push_lazy_set_true(self.inventions_errors, other.inventions_errors)

    # usage: delegate
    self.usage = push_option(self.usage, other.usage)

    # id, created, object are immutable


FunctionInventionRecursiveChunk.push = _push
