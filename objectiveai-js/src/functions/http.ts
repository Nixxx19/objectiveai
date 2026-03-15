import { ObjectiveAI, type RequestOptions } from "../client";
import type { FunctionsRemote } from "./remote";
import type { FunctionsListFunctionsSource } from "./listFunctionsSource";
import type { FunctionsListFunction } from "./listFunction";
import type { FunctionsGetFunction } from "./getFunction";
import type { FunctionsUsageFunction } from "./usageFunction";
import type { FunctionsListFunctionProfilePairsSource } from "./listFunctionProfilePairsSource";
import type { FunctionsListFunctionProfilePair } from "./listFunctionProfilePair";
import type { FunctionsUsageFunctionProfilePair } from "./usageFunctionProfilePair";

export function functionsListFunctions(
  client: ObjectiveAI,
  source?: FunctionsListFunctionsSource | null,
  options?: RequestOptions,
): Promise<FunctionsListFunction> {
  const path = source != null ? `/functions?source=${source}` : "/functions";
  return client.get_unary<FunctionsListFunction>(path, undefined, options);
}

export function functionsGetFunction(
  client: ObjectiveAI,
  remote: FunctionsRemote,
  owner: string,
  repository: string,
  commit?: string | null,
  options?: RequestOptions,
): Promise<FunctionsGetFunction> {
  const path =
    commit != null
      ? `/functions/${remote}/${owner}/${repository}/${commit}`
      : `/functions/${remote}/${owner}/${repository}`;
  return client.get_unary<FunctionsGetFunction>(path, undefined, options);
}

export function functionsGetFunctionUsage(
  client: ObjectiveAI,
  fremote: FunctionsRemote,
  fowner: string,
  frepository: string,
  fcommit?: string | null,
  options?: RequestOptions,
): Promise<FunctionsUsageFunction> {
  const path =
    fcommit != null
      ? `/functions/${fremote}/${fowner}/${frepository}/${fcommit}/usage`
      : `/functions/${fremote}/${fowner}/${frepository}/usage`;
  return client.get_unary<FunctionsUsageFunction>(path, undefined, options);
}

export function functionsListFunctionProfilePairs(
  client: ObjectiveAI,
  source?: FunctionsListFunctionProfilePairsSource | null,
  options?: RequestOptions,
): Promise<FunctionsListFunctionProfilePair> {
  const path =
    source != null
      ? `/functions/profiles/pairs?source=${source}`
      : "/functions/profiles/pairs";
  return client.get_unary<FunctionsListFunctionProfilePair>(path, undefined, options);
}

export function functionsGetFunctionProfilePairUsage(
  client: ObjectiveAI,
  fremote: FunctionsRemote,
  fowner: string,
  frepository: string,
  fcommit: string | null | undefined,
  premote: FunctionsRemote,
  powner: string,
  prepository: string,
  pcommit?: string | null,
  options?: RequestOptions,
): Promise<FunctionsUsageFunctionProfilePair> {
  let path = `/functions/${fremote}/${fowner}/${frepository}`;
  if (fcommit != null) path += `/${fcommit}`;
  path += `/profiles/${premote}/${powner}/${prepository}`;
  if (pcommit != null) path += `/${pcommit}`;
  path += "/usage";
  return client.get_unary<FunctionsUsageFunctionProfilePair>(path, undefined, options);
}
