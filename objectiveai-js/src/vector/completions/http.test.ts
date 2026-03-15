import * as path from "path";
import { httpTestSuite } from "../../httpTestUtil";
import { vectorCompletionsResponseStreamingVectorCompletionChunkMerged } from "./response/streaming/vectorCompletionChunkMerged";
import { wasmVectorCompletionsResponseStreamingVectorCompletionChunkToUnary } from "./response/streaming/wasm";
import type { VectorCompletionsResponseStreamingVectorCompletionChunk } from "./response/streaming/vectorCompletionChunk";
import type { VectorCompletionsResponseUnaryVectorCompletion } from "./response/unary/vectorCompletion";

const mockAgent = { upstream: "mock", output_mode: "instruction" };

httpTestSuite<VectorCompletionsResponseStreamingVectorCompletionChunk, VectorCompletionsResponseUnaryVectorCompletion>({
  name: "vector completions http",
  endpoint: "/vector/completions",
  snapshotsDir: path.resolve(__dirname, "../../../../objectiveai-api/assets/vector/completions/client_tests"),
  merge: vectorCompletionsResponseStreamingVectorCompletionChunkMerged,
  chunkToUnary: wasmVectorCompletionsResponseStreamingVectorCompletionChunkToUnary,
  normalize: (vc) => {
    const completions = vc.completions.map((c: any) => ({
      ...c,
      id: "",
      created: 0,
      messages: c.messages.map((m: any) =>
        m.role === "assistant" ? { ...m, upstream_id: "", created: 0 } : m,
      ),
    }));
    completions.sort((a: any, b: any) => {
      const agentA = a.messages[0]?.agent ?? "";
      const agentB = b.messages[0]?.agent ?? "";
      if (agentA !== agentB) return agentA < agentB ? -1 : 1;
      const contentA = a.messages[0]?.content ?? "";
      const contentB = b.messages[0]?.content ?? "";
      return contentA < contentB ? -1 : contentA > contentB ? 1 : 0;
    });
    completions.forEach((c: any, i: number) => { c.index = i; });
    const votes = vc.votes.map((v: any) => ({
      ...v,
      prompt_id: "",
      responses_ids: [],
    }));
    votes.sort((a: any, b: any) => {
      const agentA = a.agent ?? "";
      const agentB = b.agent ?? "";
      return agentA < agentB ? -1 : agentA > agentB ? 1 : 0;
    });
    return { ...vc, id: "", created: 0, completions, votes };
  },
  cases: [
    {
      snapshot: "single_agent_2_responses_instruction_seed_42",
      body: {
        messages: [{ role: "user", content: "Which is better?" }],
        ensemble: { agents: [{ ...mockAgent }] },
        profile: ["1"],
        responses: ["Response A", "Response B"],
        seed: 42,
      },
    },
    {
      snapshot: "many_responses_deep_prefix_tree_seed_42",
      body: {
        messages: [{ role: "user", content: "Pick the best" }],
        ensemble: { agents: [{ ...mockAgent }] },
        profile: ["1"],
        responses: Array.from({ length: 25 }, (_, i) => `Response ${i}`),
        seed: 42,
      },
    },
    {
      snapshot: "mixed_output_modes_seed_88",
      body: {
        messages: [{ role: "user", content: "Compare these vacation destinations" }],
        ensemble: {
          agents: [
            { upstream: "mock", output_mode: "instruction" },
            { upstream: "mock", output_mode: "json_schema" },
            { upstream: "mock", output_mode: "tool_call" },
          ],
        },
        profile: ["0.4", "0.3", "0.3"],
        responses: ["Kyoto, Japan", "Reykjavik, Iceland", "Patagonia, Argentina"],
        seed: 88,
      },
    },
  ],
});
