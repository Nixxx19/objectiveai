"""Methods for AssistantResponseChunk."""
from __future__ import annotations

from objectiveai.push_utils import push_by_index, push_option, push_option_string
from objectiveai.agent.completions.response.streaming.assistant_response_chunk import (
    AgentCompletionsResponseStreamingAssistantResponseChunk,
)


def _push(self, other: AgentCompletionsResponseStreamingAssistantResponseChunk) -> None:
    # reasoning: string concat
    self.reasoning = push_option_string(self.reasoning, other.reasoning)

    # tool_calls: merge by index
    if self.tool_calls is not None and other.tool_calls is not None:
        push_by_index(self.tool_calls, other.tool_calls)
    elif other.tool_calls is not None:
        self.tool_calls = list(other.tool_calls)

    # content: delegate
    self.content = push_option(self.content, other.content)

    # refusal: string concat
    self.refusal = push_option_string(self.refusal, other.refusal)

    # finish_reason: lazy set
    if self.finish_reason is None:
        self.finish_reason = other.finish_reason

    # logprobs: delegate
    self.logprobs = push_option(self.logprobs, other.logprobs)

    # upstream_id: replace if empty → non-empty
    if not self.upstream_id and other.upstream_id:
        self.upstream_id = other.upstream_id

    # service_tier: lazy set
    if self.service_tier is None:
        self.service_tier = other.service_tier

    # system_fingerprint: lazy set
    if self.system_fingerprint is None:
        self.system_fingerprint = other.system_fingerprint

    # provider: lazy set
    if self.provider is None:
        self.provider = other.provider

    # usage: delegate
    self.usage = push_option(self.usage, other.usage)

    # role, index, created, agent, model are immutable


AgentCompletionsResponseStreamingAssistantResponseChunk.push = _push
