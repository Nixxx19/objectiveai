import * as path from "path";
import { httpTestSuite } from "../../httpTestUtil";
import { functionsExecutionsResponseStreamingFunctionExecutionChunkMerged } from "./response/streaming/functionExecutionChunkMerged";
import { wasmFunctionsExecutionsResponseStreamingFunctionExecutionChunkToUnary } from "./response/streaming/wasm";
import type { FunctionsExecutionsResponseStreamingFunctionExecutionChunk } from "./response/streaming/functionExecutionChunk";
import type { FunctionsExecutionsResponseUnaryFunctionExecution } from "./response/unary/functionExecution";

function executionEndpoint(repo: string): string {
  return `/functions/mock/mock/${repo}/mock/profiles/mock/mock/${repo}/mock`;
}

function roundLogprobs(logprobs: any): any {
  if (!logprobs) return logprobs;
  const roundEntry = (e: any) => ({
    ...e,
    logprob: Math.round(e.logprob * 1e12) / 1e12,
    top_logprobs: e.top_logprobs?.map((t: any) => ({
      ...t,
      logprob: Math.round(t.logprob * 1e12) / 1e12,
    })),
  });
  return {
    ...logprobs,
    content: logprobs.content?.map(roundEntry),
    refusal: logprobs.refusal?.map(roundEntry),
  };
}

function normalizeCompletion(c: any): any {
  return {
    ...c,
    id: "",
    created: 0,
    messages: c.messages.map((m: any) =>
      m.role === "assistant"
        ? { ...m, upstream_id: "", created: 0, logprobs: roundLogprobs(m.logprobs) }
        : m,
    ),
  };
}

function roundFloat(n: number): number {
  return Math.round(n * 1e10) / 1e10;
}

function normalizeVcTask(task: any): any {
  const completions = task.completions.map(normalizeCompletion);
  completions.sort((a: any, b: any) => {
    const agentA = a.messages[0]?.agent ?? "";
    const agentB = b.messages[0]?.agent ?? "";
    if (agentA !== agentB) return agentA < agentB ? -1 : 1;
    const contentA = a.messages[0]?.content ?? "";
    const contentB = b.messages[0]?.content ?? "";
    return contentA < contentB ? -1 : contentA > contentB ? 1 : 0;
  });
  completions.forEach((c: any, i: number) => { c.index = i; });
  const votes = task.votes.map((v: any) => ({
    ...v,
    prompt_id: "",
    responses_ids: [],
    vote: v.vote.map(roundFloat),
  }));
  votes.sort((a: any, b: any) => {
    const agentA = a.agent ?? "";
    const agentB = b.agent ?? "";
    return agentA < agentB ? -1 : agentA > agentB ? 1 : 0;
  });
  return {
    ...task,
    id: "",
    created: 0,
    completions,
    votes,
    scores: task.scores?.map(roundFloat),
    weights: task.weights?.map(roundFloat),
  };
}

function normalizeFe(fe: any): any {
  return {
    ...fe,
    id: "",
    created: 0,
    retry_token: null,
    output: Array.isArray(fe.output) ? fe.output.map(roundFloat) : typeof fe.output === "number" ? roundFloat(fe.output) : fe.output,
    tasks: fe.tasks.map((task: any) => {
      if (task.object === "vector.completion") {
        return normalizeVcTask(task);
      }
      if (task.object?.endsWith(".function.execution")) {
        return normalizeFe(task);
      }
      return task;
    }),
  };
}

httpTestSuite<FunctionsExecutionsResponseStreamingFunctionExecutionChunk, FunctionsExecutionsResponseUnaryFunctionExecution>({
  name: "functions executions http",
  endpoint: "",
  snapshotsDir: path.resolve(__dirname, "../../../../objectiveai-api/assets/functions/executions/client_tests"),
  merge: functionsExecutionsResponseStreamingFunctionExecutionChunkMerged,
  chunkToUnary: wasmFunctionsExecutionsResponseStreamingFunctionExecutionChunkToUnary,
  normalize: normalizeFe,
  cases: [
    {
      snapshot: "mock_1_scalar_leaf_binary_seed_42",
      endpoint: executionEndpoint("mock-1"),
      body: { input: { text: "Hello world" }, seed: 42 },
    },
    {
      snapshot: "mock_7_vector_5_criteria_seed_42",
      endpoint: executionEndpoint("mock-7"),
      body: { input: { items: ["Option A", "Option B", "Option C"] }, seed: 42 },
    },
    {
      snapshot: "mock_20_vector_super_branch_seed_42",
      endpoint: executionEndpoint("mock-20"),
      body: { input: { items: ["Alpha", "Beta", "Gamma"] }, seed: 42 },
    },
  ],
});
