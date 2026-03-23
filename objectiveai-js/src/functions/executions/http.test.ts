import * as path from "path";
import { httpTestSuite } from "../../httpTestUtil";
import { functionsExecutionsResponseStreamingFunctionExecutionChunkMerged } from "./response/streaming/functionExecutionChunkMerged";
import {
  wasmFunctionsExecutionsResponseStreamingFunctionExecutionChunkToUnary,
  wasmFunctionsExecutionsResponseStreamingNormalizeFunctionExecutionForTests as normalize,
} from "./response/streaming/wasm";
import type { FunctionsExecutionsResponseStreamingFunctionExecutionChunk } from "./response/streaming/functionExecutionChunk";
import type { FunctionsExecutionsResponseUnaryFunctionExecution } from "./response/unary/functionExecution";

function executionBody(name: string): { function: object; profile: object } {
  return {
    function: { remote: "mock", name },
    profile: { remote: "mock", name },
  };
}

httpTestSuite<FunctionsExecutionsResponseStreamingFunctionExecutionChunk, FunctionsExecutionsResponseUnaryFunctionExecution>({
  name: "functions executions http",
  endpoint: "/functions/executions",
  snapshotsDir: path.resolve(__dirname, "../../../../objectiveai-api/assets/functions/executions/client_tests"),
  merge: functionsExecutionsResponseStreamingFunctionExecutionChunkMerged,
  chunkToUnary: wasmFunctionsExecutionsResponseStreamingFunctionExecutionChunkToUnary,
  normalize,
  cases: [
    {
      snapshot: "mock_1_scalar_leaf_binary_seed_42",
      body: { ...executionBody("mock-1"), input: { text: "Hello world" }, seed: 42 },
    },
    {
      snapshot: "mock_7_vector_5_criteria_seed_42",
      body: { ...executionBody("mock-7"), input: { items: ["Option A", "Option B", "Option C"] }, seed: 42 },
    },
    {
      snapshot: "mock_20_vector_super_branch_seed_42",
      body: { ...executionBody("mock-20"), input: { items: ["Alpha", "Beta", "Gamma"] }, seed: 42 },
    },
  ],
});
