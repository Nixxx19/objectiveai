import { describe, it, expect } from "vitest";
import { zocker } from "zocker";
import { AgentCompletionsResponseStreamingAgentCompletionChunkSchema } from "./agentCompletionChunk";
import { agentCompletionsResponseStreamingAgentCompletionChunkMerged } from "./agentCompletionChunkMerged";
import { agentCompletionChunkMerged as wasmMerged, agentCompletionChunkNormalized as wasmNormalized } from "../../../../../dist/wasm/loader.js";
import { sanitizeZocker } from "../../../../sanitizeZocker";

const gen = zocker(AgentCompletionsResponseStreamingAgentCompletionChunkSchema);

describe("agentCompletionChunkMerged fuzz", () => {
  for (let i = 0; i < 20; i++) {
    it(`stream ${i}`, () => {
      const initial = sanitizeZocker(gen.generate());
      let tsAcc = JSON.parse(wasmNormalized(initial));
      let wasmAcc = structuredClone(tsAcc);
      for (let j = 0; j < 20; j++) {
        const chunk = sanitizeZocker(gen.generate());
        [tsAcc] = agentCompletionsResponseStreamingAgentCompletionChunkMerged(tsAcc as any, chunk as any);
        wasmAcc = JSON.parse(wasmMerged(wasmAcc, chunk));
        expect(tsAcc, `chunk ${j}`).toEqual(wasmAcc);
      }
    });
  }
});
