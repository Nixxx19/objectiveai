"""HTTP functions for function profile endpoints."""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from objectiveai.client import ObjectiveAI
    from objectiveai.functions.profiles import GetProfile, ListProfile, UsageProfile


async def list_profiles(
    client: ObjectiveAI, *, source: str | None = None,
) -> ListProfile:
    """List all profiles accessible to the authenticated user."""
    path = (
        f"functions/profiles?source={source}" if source else "functions/profiles"
    )
    return await client.get_unary(path)


async def get_profile(
    client: ObjectiveAI,
    remote: str,
    owner: str,
    repository: str,
    commit: str | None = None,
) -> GetProfile:
    """Retrieve a profile definition from a remote source."""
    if commit:
        path = f"functions/profiles/{remote}/{owner}/{repository}/{commit}"
    else:
        path = f"functions/profiles/{remote}/{owner}/{repository}"
    return await client.get_unary(path)


async def get_profile_usage(
    client: ObjectiveAI,
    remote: str,
    owner: str,
    repository: str,
    commit: str | None = None,
) -> UsageProfile:
    """Retrieve usage statistics for a specific profile."""
    if commit:
        path = f"functions/profiles/{remote}/{owner}/{repository}/{commit}/usage"
    else:
        path = f"functions/profiles/{remote}/{owner}/{repository}/usage"
    return await client.get_unary(path)
