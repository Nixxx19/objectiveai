export interface ToolState {
  spawnFunctionAgentsHasSpawned: boolean;
  spawnFunctionAgentsRespawnRejected: boolean;
  editInputSchemaModalityRemovalRejected: boolean;
  runNetworkTestsApiBase: string | undefined;
  runNetworkTestsApiKey: string | undefined;
  readPlanIndex: number;
  writePlanIndex: number;
  submitApiBase: string | undefined;
  submitApiKey: string | undefined;
  gitUserName: string | undefined;
  gitUserEmail: string | undefined;
}

export function makeToolState(options: {
  apiBase?: string;
  apiKey?: string;
  readPlanIndex: number;
  writePlanIndex: number;
  gitUserName?: string;
  gitUserEmail?: string;
}): ToolState {
  return {
    spawnFunctionAgentsHasSpawned: false,
    spawnFunctionAgentsRespawnRejected: false,
    editInputSchemaModalityRemovalRejected: false,
    runNetworkTestsApiBase: options.apiBase,
    runNetworkTestsApiKey: options.apiKey,
    readPlanIndex: options.readPlanIndex,
    writePlanIndex: options.writePlanIndex,
    submitApiBase: options.apiBase,
    submitApiKey: options.apiKey,
    gitUserName: options.gitUserName,
    gitUserEmail: options.gitUserEmail,
  };
}
