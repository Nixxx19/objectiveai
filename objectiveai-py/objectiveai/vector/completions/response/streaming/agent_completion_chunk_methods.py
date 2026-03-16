"""Methods for vector.completions AgentCompletionChunk."""
from __future__ import annotations

from objectiveai.push_utils import push_by_index, push_option, push_replace
from objectiveai.vector.completions.response.streaming.agent_completion_chunk import (
    AgentCompletionChunk,
)


def _push(self, other: AgentCompletionChunk) -> None:
    push_by_index(self.messages, other.messages)
    push_option(self, "usage", other.usage)
    push_replace(self, "error", other.error)


AgentCompletionChunk.push = _push
