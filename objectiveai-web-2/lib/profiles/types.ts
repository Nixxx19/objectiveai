/** A profile as returned by the list endpoint */
export interface ProfileListItem {
  remote: string;
  owner: string;
  repository: string;
  commit: string;
}

/** An LLM within a profile's ensemble */
export interface ProfileLlm {
  model: string;
  outputMode: string;
  topLogprobs: number | null;
  temperature: number | null;
  reasoning: boolean | null;
  count: number;
  fallbacks: ProfileFallback[];
}

export interface ProfileFallback {
  model: string;
  outputMode: string;
  topLogprobs: number | null;
  reasoning: boolean | null;
}

/** A task-level ensemble config (for tasks-based profiles) */
export interface ProfileTaskConfig {
  llms: ProfileLlm[];
  weights: number[];
}

/** Resolved profile with detail */
export interface ProfileMeta {
  remote: string;
  owner: string;
  repository: string;
  commit: string;
  name: string;
  description: string;
  kind: "auto" | "tasks";
  /** Auto profiles: single ensemble + weights */
  llms: ProfileLlm[];
  weights: number[];
  /** Tasks profiles: per-task configs + task-level weights */
  taskConfigs: ProfileTaskConfig[];
  taskWeights: number[];
  /** Paired function (if any) */
  pairedFunction: ProfileListItem | null;
}
