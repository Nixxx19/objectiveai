// ---------------------------------------------------------------------------
// ChatBar types
// ---------------------------------------------------------------------------

export type ChatRole = "user" | "assistant" | "system";

export type ChatState = "idle" | "thinking" | "executing" | "complete" | "error";

export interface ToolCallInfo {
  /** Structured input the LLM constructed for the function. */
  input: Record<string, unknown>;
  /** Profile used for execution. */
  profile: string;
}

export interface ExecutionResult {
  output: number | number[] | null;
  error?: string;
}

export interface ChatMessage {
  id: string;
  role: ChatRole;
  content: string | null;
  /** If the assistant made a tool call to execute the function. */
  toolCall?: ToolCallInfo;
  /** Result of the function execution (set after tool call completes). */
  executionResult?: ExecutionResult;
  /** Timestamp. */
  ts: number;
}
