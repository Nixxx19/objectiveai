"""Methods for AssistantToolCallFunctionDelta."""
from __future__ import annotations

from objectiveai.push_utils import push_option_string
from objectiveai.agent.completions.message.assistant_tool_call_function_delta import (
    AssistantToolCallFunctionDelta,
)


def _push(self, other: AssistantToolCallFunctionDelta) -> None:
    # name: lazy set (first wins)
    if self.name is None:
        self.name = other.name
    # arguments: string concat
    self.arguments = push_option_string(self.arguments, other.arguments)


AssistantToolCallFunctionDelta.push = _push
