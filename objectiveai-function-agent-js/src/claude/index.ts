import { AgentOptions } from "../agentOptions";
import { createFileLogger } from "../logging";
import { init } from "../init";
import { prepare } from "./prepare";
import { inventMcp } from "./invent";
import { makeToolState } from "../tools/claude/toolState";

export * from "./prepare";
export * from "./invent";

export async function invent(options: AgentOptions = {}): Promise<void> {
  const log = options.log ?? createFileLogger().log;
  const toolState = options.toolState ?? makeToolState({
    apiBase: options.apiBase,
    apiKey: options.apiKey,
    readPlanIndex: 0,
    writePlanIndex: 0,
  });
  options = { ...options, log, toolState };

  log("=== Initializing workspace ===");
  await init(options);

  log("=== Preparing ===");
  const sessionId = await prepare(options);

  log("=== Inventing ===");
  await inventMcp({ ...options, sessionId });
}
