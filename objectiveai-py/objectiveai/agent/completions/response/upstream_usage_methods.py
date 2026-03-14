"""Methods for UpstreamUsage."""
from __future__ import annotations

from objectiveai.push_utils import push_option
from objectiveai.agent.completions.response.upstream_usage import (
    AgentCompletionsResponseUpstreamUsage,
)


def _push(self, other: AgentCompletionsResponseUpstreamUsage) -> None:
    self.completion_tokens += other.completion_tokens
    self.prompt_tokens += other.prompt_tokens
    self.total_tokens += other.total_tokens
    self.cost += other.cost
    self.total_cost += other.total_cost

    self.completion_tokens_details = push_option(
        self.completion_tokens_details, other.completion_tokens_details,
    )
    self.prompt_tokens_details = push_option(
        self.prompt_tokens_details, other.prompt_tokens_details,
    )
    self.cost_details = push_option(self.cost_details, other.cost_details)

    # cost_multiplier and is_byok are immutable — kept from self


AgentCompletionsResponseUpstreamUsage.push = _push
