import type { FunctionsExecutionsResponseStreamingFunctionExecutionChunk } from "./functionExecutionChunk";
import { functionsExecutionsResponseStreamingFunctionExecutionChunkFieldsMerged } from "./functionExecutionChunkFieldsMerged";
import { functionsExecutionsResponseStreamingTaskChunkMergedList } from "./taskChunkMerged";

export function functionsExecutionsResponseStreamingFunctionExecutionChunkMerged(
  a: FunctionsExecutionsResponseStreamingFunctionExecutionChunk,
  b: FunctionsExecutionsResponseStreamingFunctionExecutionChunk,
): [FunctionsExecutionsResponseStreamingFunctionExecutionChunk, boolean] {
  const fields = functionsExecutionsResponseStreamingFunctionExecutionChunkFieldsMerged(a, b, functionsExecutionsResponseStreamingTaskChunkMergedList);
  if (!fields.changed) return [a, false];
  return [{
    id: a.id,
    tasks: fields.tasks,
    ...(fields.tasks_errors != null ? { tasks_errors: fields.tasks_errors } : {}),
    ...(fields.reasoning != null ? { reasoning: fields.reasoning } : {}),
    ...(fields.output != null ? { output: fields.output } : {}),
    ...(fields.error != null ? { error: fields.error } : {}),
    ...(fields.retry_token != null ? { retry_token: fields.retry_token } : {}),
    created: a.created,
    ...(a.function != null ? { function: a.function } : {}),
    ...(a.profile != null ? { profile: a.profile } : {}),
    object: a.object,
    ...(fields.usage != null ? { usage: fields.usage } : {}),
  }, true];
}
