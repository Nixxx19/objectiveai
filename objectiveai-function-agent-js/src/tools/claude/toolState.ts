export interface ToolState {
  spawnFunctionAgentsHasSpawned: boolean;
  spawnFunctionAgentsRespawnRejected: boolean;
  editInputSchemaModalityRemovalRejected: boolean;
  runNetworkTestsApiBase: string;
  runNetworkTestsApiKey: string;
  readPlanIndex: number;
  writePlanIndex: number;
  submitApiBase: string;
  submitApiKey: string;
  gitUserName: string;
  gitUserEmail: string;
}

export function makeToolState(options: {
  apiBase: string;
  apiKey: string;
  readPlanIndex: number;
  writePlanIndex: number;
  gitUserName: string;
  gitUserEmail: string;
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
