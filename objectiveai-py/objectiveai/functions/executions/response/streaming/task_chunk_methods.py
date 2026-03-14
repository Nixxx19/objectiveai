"""Methods for TaskChunk (Union dispatch)."""
from __future__ import annotations

from objectiveai.functions.executions.response.streaming.task_chunk import (
    TaskChunk,
)
from objectiveai.functions.executions.response.streaming.function_execution_task_chunk import (
    FunctionExecutionTaskChunk,
)
from objectiveai.functions.executions.response.streaming.vector_completion_task_chunk import (
    VectorCompletionTaskChunk,
)


def _push(self, other: TaskChunk) -> None:
    a = self.root
    b = other.root
    if type(a) is type(b):
        a.push(b)


TaskChunk.push = _push
