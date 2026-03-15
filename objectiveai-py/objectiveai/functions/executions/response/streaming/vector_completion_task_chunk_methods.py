"""Methods for VectorCompletionTaskChunk (flattened VectorCompletionChunk + index fields)."""
from __future__ import annotations

from objectiveai.push_utils import push_by_index, push_option, push_replace
from objectiveai.functions.executions.response.streaming.vector_completion_task_chunk import (
    VectorCompletionTaskChunk,
)


def _push(self, other: VectorCompletionTaskChunk) -> None:
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

    # error: replace
    self.error = push_replace(self.error, other.error)

    # usage: delegate
    self.usage = push_option(self.usage, other.usage)

    # id, created, object, ensemble, index, task_index, task_path are immutable


VectorCompletionTaskChunk.push = _push
