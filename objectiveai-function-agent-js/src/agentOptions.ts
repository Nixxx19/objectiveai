import { ToolState } from "./tools/claude/toolState";

export type LogFn = (...args: unknown[]) => void;

export interface AgentOptions {
  name?: string;
  spec?: string;
  apiBase?: string;
  apiKey?: string;
  sessionId?: string;
  log?: LogFn;
  depth?: number;
  instructions?: string;
  toolState?: ToolState;
  gitUserName?: string;
  gitUserEmail?: string;
}
