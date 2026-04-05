import { apiFetch } from "../client";
import type {
  ProfileListItem,
  ProfileMeta,
  ProfileLlm,
  ProfileFallback,
  ProfileTaskConfig,
} from "./types";

interface PairsResponse {
  data: Array<{
    function: ProfileListItem;
    profile: ProfileListItem;
  }>;
}

interface RawLlm {
  model: string;
  output_mode: string;
  top_logprobs?: number | null;
  temperature?: number | null;
  reasoning?: { enabled: boolean } | null;
  count?: number | null;
  fallbacks?: RawLlm[] | null;
}

interface RawAutoProfile {
  remote: string;
  owner: string;
  repository: string;
  commit: string;
  description?: string;
  ensemble: { llms: RawLlm[] };
  profile: number[];
}

interface RawTasksProfile {
  remote: string;
  owner: string;
  repository: string;
  commit: string;
  description?: string;
  tasks: Array<{ ensemble: { llms: RawLlm[] }; profile: number[] }>;
  profile: number[];
}

type RawProfile = RawAutoProfile | RawTasksProfile;

let profileCache: { data: ProfileMeta[]; ts: number } | null = null;
const CACHE_TTL = 60_000;

export async function fetchAllProfiles(): Promise<ProfileMeta[]> {
  if (profileCache && Date.now() - profileCache.ts < CACHE_TTL) {
    return profileCache.data;
  }

  const [list, pairs] = await Promise.all([
    apiFetch<{ data: ProfileListItem[] }>("/functions/profiles"),
    apiFetch<PairsResponse>("/functions/profiles/pairs"),
  ]);

  // Build a map from profile repo → paired function
  const pairMap = new Map<string, ProfileListItem>();
  for (const p of pairs.data) {
    const key = `${p.profile.owner}/${p.profile.repository}`;
    // Only store if the function is different from the profile itself
    if (p.function.repository !== p.profile.repository || p.function.owner !== p.profile.owner) {
      pairMap.set(key, p.function);
    }
  }

  const results = await Promise.allSettled(
    list.data.map((item) => resolveProfile(item, pairMap))
  );

  const profiles = results
    .filter((r): r is PromiseFulfilledResult<ProfileMeta> => r.status === "fulfilled")
    .map((r) => r.value);

  profileCache = { data: profiles, ts: Date.now() };
  return profiles;
}

function parseLlm(raw: RawLlm): ProfileLlm {
  return {
    model: raw.model,
    outputMode: raw.output_mode,
    topLogprobs: raw.top_logprobs ?? null,
    temperature: raw.temperature ?? null,
    reasoning: raw.reasoning?.enabled ?? null,
    count: raw.count ?? 1,
    fallbacks: (raw.fallbacks ?? []).map(parseFallback),
  };
}

function parseFallback(raw: RawLlm): ProfileFallback {
  return {
    model: raw.model,
    outputMode: raw.output_mode,
    topLogprobs: raw.top_logprobs ?? null,
    reasoning: raw.reasoning?.enabled ?? null,
  };
}

async function resolveProfile(
  item: ProfileListItem,
  pairMap: Map<string, ProfileListItem>
): Promise<ProfileMeta> {
  const detail = await apiFetch<RawProfile>(
    `/functions/profiles/${item.remote}/${item.owner}/${item.repository}`
  );

  const isAuto = "ensemble" in detail;
  const key = `${item.owner}/${item.repository}`;

  if (isAuto) {
    const auto = detail as RawAutoProfile;
    return {
      ...item,
      name: item.repository,
      description: auto.description ?? "",
      kind: "auto",
      llms: auto.ensemble.llms.map(parseLlm),
      weights: auto.profile,
      taskConfigs: [],
      taskWeights: [],
      pairedFunction: pairMap.get(key) ?? null,
    };
  }

  const tasks = detail as RawTasksProfile;
  return {
    ...item,
    name: item.repository,
    description: tasks.description ?? "",
    kind: "tasks",
    llms: [],
    weights: [],
    taskConfigs: tasks.tasks.map((t) => ({
      llms: t.ensemble.llms.map(parseLlm),
      weights: t.profile,
    })),
    taskWeights: tasks.profile,
    pairedFunction: pairMap.get(key) ?? null,
  };
}
