import type { FunctionsExecutionsResponseStreamingTaskChunk } from "./taskChunk";
import type { FunctionsExecutionsResponseStreamingReasoningSummaryChunk } from "./reasoningSummaryChunk";
import { functionsExecutionsResponseStreamingReasoningSummaryChunkMerged } from "./reasoningSummaryChunkMerged";
import { agentCompletionsResponseUsageMerged } from "../../../../agent/completions/response/usageMerged";
import type { AgentCompletionsResponseUsage } from "../../../../agent/completions/response/usage";
import type { ResponseError } from "../../../../responseError";

type FunctionExecutionFields = {
  tasks: FunctionsExecutionsResponseStreamingTaskChunk[];
  tasks_errors?: boolean | null;
  reasoning?: FunctionsExecutionsResponseStreamingReasoningSummaryChunk | null;
  output?: unknown;
  error?: ResponseError | null;
  retry_token?: string | null;
  usage?: AgentCompletionsResponseUsage | null;
};

export function functionsExecutionsResponseStreamingFunctionExecutionChunkFieldsMerged<T extends FunctionExecutionFields>(
  a: T,
  b: FunctionExecutionFields,
  taskChunkMergedList: (
    a: FunctionsExecutionsResponseStreamingTaskChunk[],
    b: FunctionsExecutionsResponseStreamingTaskChunk[],
  ) => [FunctionsExecutionsResponseStreamingTaskChunk[], boolean],
): {
  changed: boolean;
  tasks: FunctionsExecutionsResponseStreamingTaskChunk[];
  tasks_errors: boolean | null | undefined;
  reasoning: FunctionsExecutionsResponseStreamingReasoningSummaryChunk | null | undefined;
  output: unknown;
  error: ResponseError | null | undefined;
  retry_token: string | null | undefined;
  usage: AgentCompletionsResponseUsage | null | undefined;
} {
  let changed = false;

  const [tasks, c1] = taskChunkMergedList(a.tasks, b.tasks);
  if (c1) changed = true;

  let tasks_errors = a.tasks_errors;
  if (b.tasks_errors === true) {
    if (a.tasks_errors !== true) changed = true;
    tasks_errors = true;
  }

  let reasoning = a.reasoning;
  if (a.reasoning != null && b.reasoning != null) {
    const [merged, c] = functionsExecutionsResponseStreamingReasoningSummaryChunkMerged(a.reasoning, b.reasoning);
    reasoning = merged;
    if (c) changed = true;
  } else if (b.reasoning != null) {
    reasoning = b.reasoning;
    changed = true;
  }

  let output = a.output;
  if (b.output != null) {
    output = b.output;
    changed = true;
  }

  let retry_token = a.retry_token;
  if (b.retry_token != null) {
    retry_token = b.retry_token;
    changed = true;
  }

  let error = a.error;
  if (b.error != null) {
    error = b.error;
    changed = true;
  }

  let usage = a.usage;
  if (a.usage != null && b.usage != null) {
    const [merged, c] = agentCompletionsResponseUsageMerged(a.usage, b.usage);
    usage = merged;
    if (c) changed = true;
  } else if (b.usage != null) {
    usage = b.usage;
    changed = true;
  }

  return { changed, tasks, tasks_errors, reasoning, output, error, retry_token, usage };
}
