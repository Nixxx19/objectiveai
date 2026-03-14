"""Methods for inventions AgentCompletionChunk (flattened AgentCompletionChunk + index)."""
from __future__ import annotations

from objectiveai.push_utils import push_by_index, push_option, push_replace
from objectiveai.functions.inventions.response.streaming.agent_completion_chunk import (
    AgentCompletionChunk,
)


def _push(self, other: AgentCompletionChunk) -> None:
    # messages: merge by index
    push_by_index(self.messages, other.messages)

    # error: replace
    self.error = push_replace(self.error, other.error)

    # usage: delegate
    self.usage = push_option(self.usage, other.usage)

    # id, created, object, upstream, index are immutable


AgentCompletionChunk.push = _push
