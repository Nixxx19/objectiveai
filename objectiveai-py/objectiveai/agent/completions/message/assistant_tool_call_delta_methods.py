"""Methods for AssistantToolCallDelta."""
from __future__ import annotations

from objectiveai.push_utils import push_option
from objectiveai.agent.completions.message.assistant_tool_call_delta import (
    AssistantToolCallDelta,
)


def _push(self, other: AssistantToolCallDelta) -> None:
    # type: lazy set
    if self.type_ is None:
        self.type_ = other.type_
    # id: lazy set
    if self.id is None:
        self.id = other.id
    # function: delegate
    self.function = push_option(self.function, other.function)


AssistantToolCallDelta.push = _push
