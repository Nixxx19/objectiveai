"""Methods for ReasoningSummaryChunk (flattened AgentCompletionChunk + error)."""
from __future__ import annotations

from objectiveai.push_utils import push_by_index, push_option, push_lazy_set
from objectiveai.functions.executions.response.streaming.reasoning_summary_chunk import (
    ReasoningSummaryChunk,
)


def _push(self, other: ReasoningSummaryChunk) -> None:
    # messages: merge by index
    push_by_index(self.messages, other.messages)

    # error: lazy set
    self.error = push_lazy_set(self.error, other.error)

    # usage: delegate
    self.usage = push_option(self.usage, other.usage)

    # id, created, object, upstream are immutable


ReasoningSummaryChunk.push = _push
