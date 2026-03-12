import { ObjectiveAI, type RequestOptions } from "../../../client";
import { Stream } from "../../../stream";
import type { FunctionsProfilesComputationsRequestRequest } from "./request/request";
import type { FunctionsProfilesComputationsResponseUnaryFunctionProfileComputation } from "./response/unary/functionProfileComputation";
import type { FunctionsProfilesComputationsResponseStreamingFunctionProfileComputationChunk } from "./response/streaming/functionProfileComputationChunk";

function buildComputationPath(request: FunctionsProfilesComputationsRequestRequest): string {
  if (!("path" in request)) {
    return "/functions/profiles/compute";
  }
  const { path } = request;
  let url = `/functions/${path.fremote}/${path.fowner}/${path.frepository}`;
  if (path.fcommit != null) url += `/${path.fcommit}`;
  url += "/profiles/compute";
  return url;
}

export function functionsProfilesComputationsComputeProfile(
  client: ObjectiveAI,
  request: FunctionsProfilesComputationsRequestRequest & { body: { stream: true } },
  options?: RequestOptions,
): Promise<Stream<FunctionsProfilesComputationsResponseStreamingFunctionProfileComputationChunk>>;
export function functionsProfilesComputationsComputeProfile(
  client: ObjectiveAI,
  request: FunctionsProfilesComputationsRequestRequest & { body: { stream?: false | null } },
  options?: RequestOptions,
): Promise<FunctionsProfilesComputationsResponseUnaryFunctionProfileComputation>;
export function functionsProfilesComputationsComputeProfile(
  client: ObjectiveAI,
  request: FunctionsProfilesComputationsRequestRequest,
  options?: RequestOptions,
): Promise<
  | Stream<FunctionsProfilesComputationsResponseStreamingFunctionProfileComputationChunk>
  | FunctionsProfilesComputationsResponseUnaryFunctionProfileComputation
> {
  const path = buildComputationPath(request);
  if (request.body.stream) {
    return client.post_streaming<FunctionsProfilesComputationsResponseStreamingFunctionProfileComputationChunk>(
      path,
      request.body,
      options,
    );
  }
  return client.post_unary<FunctionsProfilesComputationsResponseUnaryFunctionProfileComputation>(
    path,
    request.body,
    options,
  );
}
