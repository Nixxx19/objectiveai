"""Methods for AgentCompletionChunk."""
from __future__ import annotations

from objectiveai.push_utils import push_by_index, push_option
from objectiveai.agent.completions.response.streaming.agent_completion_chunk import (
    AgentCompletionChunk,
)


def _push(self, other: AgentCompletionChunk) -> None:
    # messages: merge by index
    push_by_index(self.messages, other.messages)

    # usage: delegate
    self.usage = push_option(self.usage, other.usage)

    # error: lazy set
    if self.error is None:
        self.error = other.error

    # id, created, object, upstream are immutable


AgentCompletionChunk.push = _push
