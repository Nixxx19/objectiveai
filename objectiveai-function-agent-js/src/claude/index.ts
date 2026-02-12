import { AgentOptions, makeAgentOptions } from "../agentOptions";
import { init } from "../init";
import { prepare } from "./prepare";
import { inventMcp } from "./invent";
import { makeToolState } from "../tools/claude/toolState";
import { getNextPlanIndex } from "./planIndex";
import { Dashboard } from "../dashboard";
import { createRootLogger, createChildLogger, createFileLogger } from "../logging";
import { serializeEvent, AgentEvent } from "../events";
import { readName } from "../tools/name";

export * from "./prepare";
export * from "./invent";

export async function invent(partialOptions: Partial<AgentOptions> = {}): Promise<void> {
  const isChild = !!process.env.OBJECTIVEAI_PARENT_PID;

  let dashboard: Dashboard | undefined;
  let onChildEvent: ((evt: AgentEvent) => void) | undefined;
  let logOverride: { log: AgentOptions["log"]; logPath: string } | undefined;

  if (isChild) {
    logOverride = createChildLogger();
    onChildEvent = (evt) => {
      process.stdout.write(serializeEvent(evt) + "\n");
    };
  } else if (process.stdout.isTTY) {
    dashboard = new Dashboard(5);
    logOverride = createRootLogger(dashboard);
    onChildEvent = (evt) => dashboard!.handleEvent(evt);
  }

  const options = makeAgentOptions({
    ...partialOptions,
    ...(logOverride && { log: logOverride.log }),
    onChildEvent,
  });

  const nextPlanIndex = getNextPlanIndex();
  const toolState = makeToolState({
    apiBase: options.apiBase,
    apiKey: options.apiKey,
    readPlanIndex: nextPlanIndex,
    writePlanIndex: nextPlanIndex,
    gitUserName: options.gitUserName,
    gitUserEmail: options.gitUserEmail,
    ghToken: options.ghToken,
    minWidth: options.minWidth,
    maxWidth: options.maxWidth,
    onChildEvent,
  });

  options.log("=== Initializing workspace ===");
  await init(options);

  options.log("=== Preparing ===");
  const sessionId = await prepare(toolState, options);

  // Update dashboard/parent with the function name after prepare
  const nameResult = readName();
  if (nameResult.ok && nameResult.value) {
    const name = nameResult.value.trim();
    if (dashboard) {
      dashboard.setRootName(name);
    }
    if (isChild) {
      process.stdout.write(serializeEvent({ event: "name", path: "", name }) + "\n");
    }
  }

  options.log("=== Inventing ===");
  await inventMcp(toolState, { ...options, sessionId });

  // Signal done if child
  if (isChild) {
    process.stdout.write(serializeEvent({ event: "done", path: "" }) + "\n");
  }

  if (dashboard) {
    dashboard.dispose();
  }
}
