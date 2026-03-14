"""Methods for MessageChunk."""
from __future__ import annotations

from objectiveai.agent.completions.response.streaming.message_chunk import (
    MessageChunk,
    MessageChunkVariant1,
)


def _push(self, other: MessageChunk) -> None:
    self_inner = self.root
    other_inner = other.root

    # Only merge if both are assistant variants
    if (
        isinstance(self_inner, MessageChunkVariant1)
        and isinstance(other_inner, MessageChunkVariant1)
    ):
        self_inner.root.push(other_inner.root)


MessageChunk.push = _push
