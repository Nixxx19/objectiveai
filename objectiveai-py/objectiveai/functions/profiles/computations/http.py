"""HTTP functions for profile computation endpoints."""

from __future__ import annotations

from typing import TYPE_CHECKING, Union

if TYPE_CHECKING:
    from objectiveai.client import ObjectiveAI
    from objectiveai.functions.profiles.computations.request import (
        FunctionProfileComputationCreateParams,
    )
    from objectiveai.functions.profiles.computations.response.streaming import (
        FunctionProfileComputationChunk,
    )
    from objectiveai.functions.profiles.computations.response.unary import (
        FunctionProfileComputation,
    )
    from objectiveai.stream import Stream


async def compute_profile(
    client: ObjectiveAI,
    params: FunctionProfileComputationCreateParams,
    *,
    fremote: str | None = None,
    fowner: str | None = None,
    frepository: str | None = None,
    fcommit: str | None = None,
) -> Union[FunctionProfileComputation, Stream[FunctionProfileComputationChunk]]:
    """Compute a profile for a function.

    Path is built from the remote source arguments. If none are provided,
    uses the inline computation path. If ``params.stream`` is true, returns
    a streaming response.
    """
    if fremote and fowner and frepository:
        parts = [f"functions/{fremote}/{fowner}/{frepository}"]
        if fcommit:
            parts.append(fcommit)
        parts.append("profiles/compute")
        path = "/".join(parts)
    else:
        path = "functions/profiles/compute"

    if getattr(params, "stream", None):
        return await client.post_streaming(path, params)
    return await client.post_unary(path, params)
