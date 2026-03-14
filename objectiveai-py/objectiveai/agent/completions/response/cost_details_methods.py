"""Methods for CostDetails."""
from __future__ import annotations

from objectiveai.agent.completions.response.cost_details import (
    AgentCompletionsResponseCostDetails,
)


def _push(self, other: AgentCompletionsResponseCostDetails) -> None:
    self.upstream_inference_cost += other.upstream_inference_cost
    self.upstream_upstream_inference_cost += other.upstream_upstream_inference_cost


AgentCompletionsResponseCostDetails.push = _push
