import * as path from "path";
import { httpTestSuite } from "../../httpTestUtil";
import { functionsInventionsResponseStreamingFunctionInventionChunkMerged } from "./response/streaming/functionInventionChunkMerged";
import { wasmFunctionsInventionsResponseStreamingFunctionInventionChunkToUnary } from "./response/streaming/wasm";
import type { FunctionsInventionsResponseStreamingFunctionInventionChunk } from "./response/streaming/functionInventionChunk";
import type { FunctionsInventionsResponseUnaryFunctionInvention } from "./response/unary/functionInvention";

const mockInventionAgent = { upstream: "mock", output_mode: "instruction", invention: true };

httpTestSuite<FunctionsInventionsResponseStreamingFunctionInventionChunk, FunctionsInventionsResponseUnaryFunctionInvention>({
  name: "functions inventions http",
  endpoint: "/functions/inventions",
  snapshotsDir: path.resolve(__dirname, "../../../../objectiveai-api/assets/functions/inventions/client_tests"),
  merge: functionsInventionsResponseStreamingFunctionInventionChunkMerged,
  chunkToUnary: wasmFunctionsInventionsResponseStreamingFunctionInventionChunkToUnary,
  normalize: (fi) => ({
    ...fi,
    id: "",
    created: 0,
    completions: fi.completions.map((c: any) => ({
      ...c,
      id: "",
      created: 0,
      messages: c.messages.map((m: any) =>
        m.role === "assistant" ? { ...m, upstream_id: "", created: 0 } : m,
      ),
    })),
  }),
  cases: [
    {
      snapshot: "scalar_leaf_s42_0",
      body: {
        state: {
          type: "alpha.scalar.leaf.function",
          depth: 0, min_branch_width: 3, max_branch_width: 5, min_leaf_width: 3, max_leaf_width: 5,
          name: "sl-default", spec: "Test function spec for mock invention.",
        },
        agent: mockInventionAgent,
        seed: 42,
        stream: true,
        max_step_retries: 1,
      },
    },
    {
      snapshot: "vector_branch_s2025_0",
      body: {
        state: {
          type: "alpha.vector.branch.function",
          depth: 3, min_branch_width: 2, max_branch_width: 4, min_leaf_width: 2, max_leaf_width: 4,
          name: "vb-deep", spec: "Test function spec for mock invention.",
        },
        agent: mockInventionAgent,
        seed: 2025,
        stream: true,
        max_step_retries: 1,
      },
    },
    {
      snapshot: "scalar_leaf_schema_kitchen_0",
      body: {
        state: {
          type: "alpha.scalar.leaf.function",
          depth: 0, min_branch_width: 3, max_branch_width: 5, min_leaf_width: 3, max_leaf_width: 5,
          name: "sl-kitchen", spec: "Test function spec for mock invention.",
          input_schema: {
            type: "object",
            properties: {
              name: { type: "string" },
              age: { type: "integer" },
              score: { type: "number" },
              active: { type: "boolean" },
              avatar: { type: "image" },
              voicemail: { type: "audio" },
              demo: { type: "video" },
              resume: { type: "file" },
              aliases: {
                type: "array",
                items: { anyOf: [{ type: "string" }, { type: "integer" }] },
                minItems: 1,
                maxItems: 8,
              },
              extra: {
                anyOf: [
                  { type: "string" },
                  {
                    type: "array",
                    items: {
                      type: "object",
                      properties: {
                        key: { type: "string" },
                        val: { anyOf: [{ type: "number" }, { type: "boolean" }, { type: "image" }] },
                      },
                      required: ["key", "val"],
                    },
                    minItems: 1,
                    maxItems: 3,
                  },
                ],
              },
            },
            required: ["name", "age", "score", "active", "avatar", "voicemail", "demo", "resume", "aliases", "extra"],
          },
        },
        agent: mockInventionAgent,
        seed: 80004,
        stream: true,
        max_step_retries: 1,
      },
    },
  ],
});
