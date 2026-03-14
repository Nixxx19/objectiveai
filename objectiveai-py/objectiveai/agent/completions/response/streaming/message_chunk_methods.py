"""Methods for MessageChunk."""
from __future__ import annotations

from objectiveai.agent.completions.response.streaming.message_chunk import (
    AgentCompletionsResponseStreamingMessageChunk,
    AgentCompletionsResponseStreamingMessageChunkVariant1,
)


def _push(self, other: AgentCompletionsResponseStreamingMessageChunk) -> None:
    self_inner = self.root
    other_inner = other.root

    # Only merge if both are assistant variants
    if (
        isinstance(self_inner, AgentCompletionsResponseStreamingMessageChunkVariant1)
        and isinstance(other_inner, AgentCompletionsResponseStreamingMessageChunkVariant1)
    ):
        self_inner.root.push(other_inner.root)


AgentCompletionsResponseStreamingMessageChunk.push = _push
