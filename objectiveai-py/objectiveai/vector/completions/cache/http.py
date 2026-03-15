"""HTTP functions for vector completion cache endpoints."""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from objectiveai.client import ObjectiveAI
    from objectiveai.vector.completions.cache import CacheVoteRequest, CompletionVotes
    from objectiveai.vector.completions.response import Vote


async def get_completion_votes(
    client: ObjectiveAI, completion_id: str,
) -> CompletionVotes:
    """Retrieve votes for a specific vector completion."""
    return await client.get_unary(f"vector/completions/{completion_id}")


async def get_cache_vote(
    client: ObjectiveAI, body: CacheVoteRequest,
) -> Vote:
    """Retrieve a cached vote."""
    return await client.get_unary("vector/completions/cache", body)
