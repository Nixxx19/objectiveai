"""HTTP functions for swarm endpoints."""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from objectiveai.client import ObjectiveAI
    from objectiveai.swarm import GetSwarm, ListSwarm, UsageSwarm


async def list_swarms(client: ObjectiveAI) -> ListSwarm:
    """List all swarms that have been used."""
    return await client.get_unary("swarms")


async def get_swarm(client: ObjectiveAI, swarm_id: str) -> GetSwarm:
    """Retrieve a specific swarm by its content-addressed ID."""
    return await client.get_unary(f"swarms/{swarm_id}")


async def get_swarm_usage(
    client: ObjectiveAI, swarm_id: str,
) -> UsageSwarm:
    """Retrieve usage statistics for a specific swarm."""
    return await client.get_unary(f"swarms/{swarm_id}/usage")
