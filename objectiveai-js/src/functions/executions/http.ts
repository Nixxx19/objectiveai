import { ObjectiveAI, type RequestOptions } from "../../client";
import { Stream } from "../../stream";
import type { FunctionsExecutionsRequestRequest } from "./request/request";
import type { FunctionsExecutionsResponseUnaryFunctionExecution } from "./response/unary/functionExecution";
import type { FunctionsExecutionsResponseStreamingFunctionExecutionChunk } from "./response/streaming/functionExecutionChunk";

function buildExecutionPath(request: FunctionsExecutionsRequestRequest): string {
  if (!("path" in request)) {
    return "/functions";
  }
  const { path } = request;
  if ("fremote" in path && "premote" in path) {
    let url = `/functions/${path.fremote}/${path.fowner}/${path.frepository}`;
    if (path.fcommit != null) url += `/${path.fcommit}`;
    url += `/profiles/${path.premote}/${path.powner}/${path.prepository}`;
    if (path.pcommit != null) url += `/${path.pcommit}`;
    return url;
  }
  if ("fremote" in path) {
    let url = `/functions/${path.fremote}/${path.fowner}/${path.frepository}`;
    if (path.fcommit != null) url += `/${path.fcommit}`;
    return url;
  }
  let url = `/functions/profiles/${path.premote}/${path.powner}/${path.prepository}`;
  if (path.pcommit != null) url += `/${path.pcommit}`;
  return url;
}

export function functionsExecutionsCreateFunctionExecution(
  client: ObjectiveAI,
  request: FunctionsExecutionsRequestRequest & { body: { stream: true } },
  options?: RequestOptions,
): Promise<Stream<FunctionsExecutionsResponseStreamingFunctionExecutionChunk>>;
export function functionsExecutionsCreateFunctionExecution(
  client: ObjectiveAI,
  request: FunctionsExecutionsRequestRequest & { body: { stream?: false | null } },
  options?: RequestOptions,
): Promise<FunctionsExecutionsResponseUnaryFunctionExecution>;
export function functionsExecutionsCreateFunctionExecution(
  client: ObjectiveAI,
  request: FunctionsExecutionsRequestRequest,
  options?: RequestOptions,
): Promise<
  | Stream<FunctionsExecutionsResponseStreamingFunctionExecutionChunk>
  | FunctionsExecutionsResponseUnaryFunctionExecution
> {
  const path = buildExecutionPath(request);
  if (request.body.stream) {
    return client.post_streaming<FunctionsExecutionsResponseStreamingFunctionExecutionChunk>(
      path,
      request.body,
      options,
    );
  }
  return client.post_unary<FunctionsExecutionsResponseUnaryFunctionExecution>(
    path,
    request.body,
    options,
  );
}
