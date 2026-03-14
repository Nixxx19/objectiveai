"""Methods for VectorCompletionChunk."""
from __future__ import annotations

from objectiveai.push_utils import push_by_index, push_option
from objectiveai.vector.completions.response.streaming.vector_completion_chunk import (
    VectorCompletionChunk,
)


def _push(self, other: VectorCompletionChunk) -> None:
    # completions: merge by index
    push_by_index(self.completions, other.completions)

    # votes: extend
    self.votes.extend(other.votes)

    # scores: replace
    if other.scores:
        self.scores = list(other.scores)

    # weights: replace
    if other.weights:
        self.weights = list(other.weights)

    # usage: delegate
    self.usage = push_option(self.usage, other.usage)

    # id, created, ensemble, object are immutable


VectorCompletionChunk.push = _push
