import { ObjectiveAI, type RequestOptions } from "../../client";
import type { FunctionsRemote } from "../remote";
import type { FunctionsProfilesListProfilesSource } from "./listProfilesSource";
import type { FunctionsProfilesListProfile } from "./listProfile";
import type { FunctionsProfilesGetProfile } from "./getProfile";
import type { FunctionsProfilesUsageProfile } from "./usageProfile";

export function functionsProfilesListProfiles(
  client: ObjectiveAI,
  source?: FunctionsProfilesListProfilesSource | null,
  options?: RequestOptions,
): Promise<FunctionsProfilesListProfile> {
  const path = source != null ? `/functions/profiles?source=${source}` : "/functions/profiles";
  return client.get_unary<FunctionsProfilesListProfile>(path, undefined, options);
}

export function functionsProfilesGetProfile(
  client: ObjectiveAI,
  remote: FunctionsRemote,
  owner: string,
  repository: string,
  commit?: string | null,
  options?: RequestOptions,
): Promise<FunctionsProfilesGetProfile> {
  const path =
    commit != null
      ? `/functions/profiles/${remote}/${owner}/${repository}/${commit}`
      : `/functions/profiles/${remote}/${owner}/${repository}`;
  return client.get_unary<FunctionsProfilesGetProfile>(path, undefined, options);
}

export function functionsProfilesGetProfileUsage(
  client: ObjectiveAI,
  premote: FunctionsRemote,
  powner: string,
  prepository: string,
  pcommit?: string | null,
  options?: RequestOptions,
): Promise<FunctionsProfilesUsageProfile> {
  const path =
    pcommit != null
      ? `/functions/profiles/${premote}/${powner}/${prepository}/${pcommit}/usage`
      : `/functions/profiles/${premote}/${powner}/${prepository}/usage`;
  return client.get_unary<FunctionsProfilesUsageProfile>(path, undefined, options);
}
