"""HTTP functions for function executions."""

from __future__ import annotations

from typing import TYPE_CHECKING, Union

if TYPE_CHECKING:
    from objectiveai.client import ObjectiveAI
    from objectiveai.functions.executions.request import FunctionExecutionCreateParams
    from objectiveai.functions.executions.response.streaming import FunctionExecutionChunk
    from objectiveai.functions.executions.response.unary import FunctionExecution
    from objectiveai.stream import Stream


async def create_function_execution(
    client: ObjectiveAI,
    params: FunctionExecutionCreateParams,
    *,
    fremote: str | None = None,
    fowner: str | None = None,
    frepository: str | None = None,
    fcommit: str | None = None,
    premote: str | None = None,
    powner: str | None = None,
    prepository: str | None = None,
    pcommit: str | None = None,
) -> Union[FunctionExecution, Stream[FunctionExecutionChunk]]:
    """Execute a function.

    Path is built from the remote source arguments. If none are provided,
    uses the inline execution path. If ``params.stream`` is true, returns
    a streaming response.
    """
    if fremote and fowner and frepository:
        parts = [f"functions/{fremote}/{fowner}/{frepository}"]
        if fcommit:
            parts.append(fcommit)
        if premote and powner and prepository:
            parts.append(f"profiles/{premote}/{powner}/{prepository}")
            if pcommit:
                parts.append(pcommit)
        path = "/".join(parts)
    else:
        path = "functions"

    if getattr(params, "stream", None):
        return await client.post_streaming(path, params)
    return await client.post_unary(path, params)
