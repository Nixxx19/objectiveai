"""HTTP functions for function management endpoints."""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from objectiveai.client import ObjectiveAI
    from objectiveai.functions import (
        GetFunction,
        ListFunction,
        ListFunctionProfilePair,
        UsageFunction,
        UsageFunctionProfilePair,
    )


async def list_functions(
    client: ObjectiveAI, *, source: str | None = None,
) -> ListFunction:
    """List all functions accessible to the authenticated user."""
    path = f"functions?source={source}" if source else "functions"
    return await client.get_unary(path)


async def get_function(
    client: ObjectiveAI,
    remote: str,
    owner: str,
    repository: str,
    commit: str | None = None,
) -> GetFunction:
    """Retrieve a function definition from a remote source."""
    if commit:
        path = f"functions/{remote}/{owner}/{repository}/{commit}"
    else:
        path = f"functions/{remote}/{owner}/{repository}"
    return await client.get_unary(path)


async def get_function_usage(
    client: ObjectiveAI,
    remote: str,
    owner: str,
    repository: str,
    commit: str | None = None,
) -> UsageFunction:
    """Retrieve usage statistics for a specific function."""
    if commit:
        path = f"functions/{remote}/{owner}/{repository}/{commit}/usage"
    else:
        path = f"functions/{remote}/{owner}/{repository}/usage"
    return await client.get_unary(path)


async def list_function_profile_pairs(
    client: ObjectiveAI, *, source: str | None = None,
) -> ListFunctionProfilePair:
    """List all function-profile pairs accessible to the authenticated user."""
    path = (
        f"functions/profiles/pairs?source={source}"
        if source
        else "functions/profiles/pairs"
    )
    return await client.get_unary(path)


async def get_function_profile_pair_usage(
    client: ObjectiveAI,
    fremote: str,
    fowner: str,
    frepository: str,
    premote: str,
    powner: str,
    prepository: str,
    fcommit: str | None = None,
    pcommit: str | None = None,
) -> UsageFunctionProfilePair:
    """Retrieve usage statistics for a specific function-profile pair."""
    parts = [f"functions/{fremote}/{fowner}/{frepository}"]
    if fcommit:
        parts.append(fcommit)
    parts.append(f"profiles/{premote}/{powner}/{prepository}")
    if pcommit:
        parts.append(pcommit)
    parts.append("usage")
    return await client.get_unary("/".join(parts))
