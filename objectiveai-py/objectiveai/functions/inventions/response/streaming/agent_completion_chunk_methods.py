"""Methods for inventions AgentCompletionChunk (flattened AgentCompletionChunk + index)."""
from __future__ import annotations

from objectiveai.push_utils import push_by_index, push_option, push_lazy_set
from objectiveai.functions.inventions.response.streaming.agent_completion_chunk import (
    AgentCompletionChunk,
)


def _push(self, other: AgentCompletionChunk) -> None:
    # messages: merge by index
    push_by_index(self.messages, other.messages)

    # error: lazy set
    self.error = push_lazy_set(self.error, other.error)

    # usage: delegate
    self.usage = push_option(self.usage, other.usage)

    # id, created, object, upstream, index are immutable


AgentCompletionChunk.push = _push
