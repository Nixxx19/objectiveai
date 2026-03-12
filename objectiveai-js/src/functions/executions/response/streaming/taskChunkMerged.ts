import type { FunctionsExecutionsResponseStreamingTaskChunk } from "./taskChunk";
import type { FunctionsExecutionsResponseStreamingFunctionExecutionTaskChunk } from "./functionExecutionTaskChunk";
import type { FunctionsExecutionsResponseStreamingVectorCompletionTaskChunk } from "./vectorCompletionTaskChunk";
import { functionsExecutionsResponseStreamingVectorCompletionTaskChunkMerged } from "./vectorCompletionTaskChunkMerged";
import { functionsExecutionsResponseStreamingFunctionExecutionChunkFieldsMerged } from "./functionExecutionChunkFieldsMerged";

function isVectorCompletionTaskChunk(
  chunk: FunctionsExecutionsResponseStreamingTaskChunk,
): chunk is FunctionsExecutionsResponseStreamingVectorCompletionTaskChunk {
  return "scores" in chunk;
}

function taskChunkIndex(chunk: FunctionsExecutionsResponseStreamingTaskChunk): number {
  return (chunk as { index: number }).index;
}

function functionsExecutionsResponseStreamingFunctionExecutionTaskChunkMerged(
  a: FunctionsExecutionsResponseStreamingFunctionExecutionTaskChunk,
  b: FunctionsExecutionsResponseStreamingFunctionExecutionTaskChunk,
): [FunctionsExecutionsResponseStreamingFunctionExecutionTaskChunk, boolean] {
  const fields = functionsExecutionsResponseStreamingFunctionExecutionChunkFieldsMerged(a, b, functionsExecutionsResponseStreamingTaskChunkMergedList);
  if (!fields.changed) return [a, false];
  return [{
    index: a.index,
    task_index: a.task_index,
    task_path: a.task_path,
    ...(a.swiss_pool_index != null ? { swiss_pool_index: a.swiss_pool_index } : {}),
    ...(a.swiss_round != null ? { swiss_round: a.swiss_round } : {}),
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

export function functionsExecutionsResponseStreamingTaskChunkMerged(
  a: FunctionsExecutionsResponseStreamingTaskChunk,
  b: FunctionsExecutionsResponseStreamingTaskChunk,
): [FunctionsExecutionsResponseStreamingTaskChunk, boolean] {
  if (isVectorCompletionTaskChunk(a) && isVectorCompletionTaskChunk(b)) {
    return functionsExecutionsResponseStreamingVectorCompletionTaskChunkMerged(a, b);
  }
  if (!isVectorCompletionTaskChunk(a) && !isVectorCompletionTaskChunk(b)) {
    return functionsExecutionsResponseStreamingFunctionExecutionTaskChunkMerged(a, b);
  }
  return [a, false];
}

export function functionsExecutionsResponseStreamingTaskChunkMergedList(
  a: FunctionsExecutionsResponseStreamingTaskChunk[],
  b: FunctionsExecutionsResponseStreamingTaskChunk[],
): [FunctionsExecutionsResponseStreamingTaskChunk[], boolean] {
  let changed = false;
  const result = [...a];
  for (const bItem of b) {
    const bIndex = taskChunkIndex(bItem);
    const existingIdx = result.findIndex((x) => taskChunkIndex(x) === bIndex);
    if (existingIdx !== -1) {
      const [merged, c] = functionsExecutionsResponseStreamingTaskChunkMerged(result[existingIdx], bItem);
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
