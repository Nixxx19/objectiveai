"""HTTP functions for ensemble endpoints."""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from objectiveai.client import ObjectiveAI
    from objectiveai.ensemble import GetEnsemble, ListEnsemble, UsageEnsemble


async def list_ensembles(client: ObjectiveAI) -> ListEnsemble:
    """List all ensembles that have been used."""
    return await client.get_unary("ensembles")


async def get_ensemble(client: ObjectiveAI, ensemble_id: str) -> GetEnsemble:
    """Retrieve a specific ensemble by its content-addressed ID."""
    return await client.get_unary(f"ensembles/{ensemble_id}")


async def get_ensemble_usage(
    client: ObjectiveAI, ensemble_id: str,
) -> UsageEnsemble:
    """Retrieve usage statistics for a specific ensemble."""
    return await client.get_unary(f"ensembles/{ensemble_id}/usage")
