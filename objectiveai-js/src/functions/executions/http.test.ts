import * as path from "path";
import { httpTestSuite } from "../../httpTestUtil";
import { functionsExecutionsResponseStreamingFunctionExecutionChunkMerged } from "./response/streaming/functionExecutionChunkMerged";
import {
  wasmFunctionsExecutionsResponseStreamingFunctionExecutionChunkToUnary,
  wasmFunctionsExecutionsResponseStreamingNormalizeFunctionExecutionForTests as normalize,
} from "./response/streaming/wasm";
import type { FunctionsExecutionsResponseStreamingFunctionExecutionChunk } from "./response/streaming/functionExecutionChunk";
import type { FunctionsExecutionsResponseUnaryFunctionExecution } from "./response/unary/functionExecution";

function executionEndpoint(repo: string): string {
  return `/functions/mock/mock/${repo}/mock/profiles/mock/mock/${repo}/mock`;
}

httpTestSuite<FunctionsExecutionsResponseStreamingFunctionExecutionChunk, FunctionsExecutionsResponseUnaryFunctionExecution>({
  name: "functions executions http",
  endpoint: "",
  snapshotsDir: path.resolve(__dirname, "../../../../objectiveai-api/assets/functions/executions/client_tests"),
  merge: functionsExecutionsResponseStreamingFunctionExecutionChunkMerged,
  chunkToUnary: wasmFunctionsExecutionsResponseStreamingFunctionExecutionChunkToUnary,
  normalize,
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
