import { describe, it, expect } from "vitest";
import { functionsInventionsResponseStreamingFunctionInventionChunkMerged } from "./functionInventionChunkMerged";
import {
  wasmFunctionsInventionsResponseStreamingFunctionInventionChunkMerged as wasmMerged,
  wasmFunctionsInventionsResponseStreamingGenerateFunctionInventionChunk as generate,
} from "./wasm";
import { rounded } from "../../../../mergeTestUtil";

describe("functionInventionChunkMerged fuzz", () => {
  for (let i = 0; i < 20; i++) {
    it(`stream ${i}`, () => {
      let seed = i * 1000;
      let tsAcc = generate(seed++);
      let wasmAcc = structuredClone(tsAcc);
      for (let j = 0; j < 20; j++) {
        const chunk = generate(seed++);
        [tsAcc] = functionsInventionsResponseStreamingFunctionInventionChunkMerged(tsAcc, chunk);
        wasmAcc = wasmMerged(wasmAcc, chunk);
        expect(rounded(tsAcc), `chunk ${j}`).toEqual(rounded(wasmAcc));
      }
    });
  }
});
