"""Methods for FunctionExecutionChunk."""
from __future__ import annotations

from objectiveai.push_utils import push_by_index, push_option, push_replace, push_lazy_set, push_lazy_set_true
from objectiveai.functions.executions.response.streaming.function_execution_chunk import (
    FunctionExecutionChunk,
)


def _push(self, other: FunctionExecutionChunk) -> None:
    # tasks: merge by index
    push_by_index(self.tasks, other.tasks)

    # tasks_errors: lazy set true
    self.tasks_errors = push_lazy_set_true(self.tasks_errors, other.tasks_errors)

    # reasoning: delegate
    self.reasoning = push_option(self.reasoning, other.reasoning)

    # output: replace
    self.output = push_replace(self.output, other.output)

    # retry_token: lazy set
    self.retry_token = push_lazy_set(self.retry_token, other.retry_token)

    # error: lazy set
    self.error = push_lazy_set(self.error, other.error)

    # usage: delegate
    self.usage = push_option(self.usage, other.usage)

    # id, created, object, function, profile are immutable


FunctionExecutionChunk.push = _push
