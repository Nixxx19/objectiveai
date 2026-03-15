"""HTTP functions for agent endpoints."""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from objectiveai.agent import GetAgent, ListAgent, UsageAgent
    from objectiveai.client import ObjectiveAI


async def list_agents(client: ObjectiveAI) -> ListAgent:
    """List all agents that have been used."""
    return await client.get_unary("agents")


async def get_agent(client: ObjectiveAI, agent_id: str) -> GetAgent:
    """Retrieve a specific agent by its content-addressed ID."""
    return await client.get_unary(f"agents/{agent_id}")


async def get_agent_usage(client: ObjectiveAI, agent_id: str) -> UsageAgent:
    """Retrieve usage statistics for a specific agent."""
    return await client.get_unary(f"agents/{agent_id}/usage")
