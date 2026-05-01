"""HTTP functions for agent completions."""

from __future__ import annotations

from typing import TYPE_CHECKING, Union

if TYPE_CHECKING:
    from objectiveai.agent.completions.request import AgentCompletionCreateParams
    from objectiveai.agent.completions.response.streaming import AgentCompletionChunk
    from objectiveai.agent.completions.response.unary import AgentCompletion
    from objectiveai.client import ObjectiveAI
    from objectiveai.stream import Stream


async def create_agent_completion(
    client: ObjectiveAI,
    params: AgentCompletionCreateParams,
) -> Union[AgentCompletion, Stream[AgentCompletionChunk]]:
    """Create an agent completion.

    If ``params.stream`` is true, returns a streaming response.
    Otherwise returns the complete response.
    """
    if getattr(params, "stream", None):
        return await client.post_streaming("agent/completions", params)
    return await client.post_unary("agent/completions", params)
