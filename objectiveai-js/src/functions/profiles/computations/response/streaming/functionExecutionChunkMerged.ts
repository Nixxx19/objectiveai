import type { FunctionsProfilesComputationsResponseStreamingFunctionExecutionChunk } from "./functionExecutionChunk";
import { functionsExecutionsResponseStreamingFunctionExecutionChunkFieldsMerged } from "../../../../executions/response/streaming/functionExecutionChunkFieldsMerged";
import { functionsExecutionsResponseStreamingTaskChunkMergedList } from "../../../../executions/response/streaming/taskChunkMerged";

export function functionsProfilesComputationsResponseStreamingFunctionExecutionChunkMerged(
  a: FunctionsProfilesComputationsResponseStreamingFunctionExecutionChunk,
  b: FunctionsProfilesComputationsResponseStreamingFunctionExecutionChunk,
): [FunctionsProfilesComputationsResponseStreamingFunctionExecutionChunk, boolean] {
  const fields = functionsExecutionsResponseStreamingFunctionExecutionChunkFieldsMerged(a, b, functionsExecutionsResponseStreamingTaskChunkMergedList);
  if (!fields.changed) return [a, false];
  return [{
    index: a.index,
    dataset: a.dataset,
    n: a.n,
    retry: a.retry,
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

export function functionsProfilesComputationsResponseStreamingFunctionExecutionChunkMergedList(
  a: FunctionsProfilesComputationsResponseStreamingFunctionExecutionChunk[],
  b: FunctionsProfilesComputationsResponseStreamingFunctionExecutionChunk[],
): [FunctionsProfilesComputationsResponseStreamingFunctionExecutionChunk[], boolean] {
  let changed = false;
  const result = [...a];
  for (const bItem of b) {
    const existingIdx = result.findIndex((x) => x.index === bItem.index);
    if (existingIdx !== -1) {
      const [merged, c] = functionsProfilesComputationsResponseStreamingFunctionExecutionChunkMerged(result[existingIdx], bItem);
      if (c) {
        result[existingIdx] = merged;
        changed = true;
      }
    } else {
      result.push(bItem);
      changed = true;
    }
  }
  if (!changed) return [a, false];
  return [result, true];
}
