"""Methods for FunctionInventionChunk."""
from __future__ import annotations

from objectiveai.push_utils import push_by_index, push_option, push_replace
from objectiveai.functions.inventions.response.streaming.function_invention_chunk import (
    FunctionInventionChunk,
)


def _push(self, other: FunctionInventionChunk) -> None:
    # completions: merge by index
    push_by_index(self.completions, other.completions)

    # state: replace
    self.state = push_replace(self.state, other.state)

    # path: replace
    self.path = push_replace(self.path, other.path)

    # function: replace
    self.function = push_replace(self.function, other.function)

    # error: replace
    self.error = push_replace(self.error, other.error)

    # usage: delegate
    self.usage = push_option(self.usage, other.usage)

    # id, created, object are immutable


FunctionInventionChunk.push = _push
