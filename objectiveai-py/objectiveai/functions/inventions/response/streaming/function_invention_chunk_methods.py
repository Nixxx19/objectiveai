"""Methods for FunctionInventionChunk."""
from __future__ import annotations

from objectiveai.push_utils import push_by_index, push_option, push_replace, push_lazy_set
from objectiveai.functions.inventions.response.streaming.function_invention_chunk import (
    FunctionInventionChunk,
)


def _push(self, other: FunctionInventionChunk) -> None:
    # completions: merge by index
    push_by_index(self.completions, other.completions)

    # state: lazy set
    self.state = push_lazy_set(self.state, other.state)

    # path: lazy set
    self.path = push_lazy_set(self.path, other.path)

    # function: lazy set
    self.function = push_lazy_set(self.function, other.function)

    # error: replace
    self.error = push_replace(self.error, other.error)

    # usage: delegate
    self.usage = push_option(self.usage, other.usage)

    # id, created, object are immutable


FunctionInventionChunk.push = _push
