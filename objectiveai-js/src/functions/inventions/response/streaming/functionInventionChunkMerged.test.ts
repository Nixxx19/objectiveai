import { describe, it, expect } from "vitest";
import { zocker } from "zocker";
import { FunctionsInventionsResponseStreamingFunctionInventionChunkSchema } from "./functionInventionChunk";
import { functionsInventionsResponseStreamingFunctionInventionChunkMerged } from "./functionInventionChunkMerged";
import { functionInventionChunkMerged as wasmMerged, functionInventionChunkNormalized as wasmNormalized } from "../../../../wasm/loader.js";
import { zockerParse } from "../../../../zockerParse";

const gen = zocker(FunctionsInventionsResponseStreamingFunctionInventionChunkSchema).array({ max: 3 });
const parse = () => zockerParse(gen, wasmNormalized);

describe("functionInventionChunkMerged fuzz", () => {
  for (let i = 0; i < 20; i++) {
    it(`stream ${i}`, () => {
      let tsAcc = parse();
      let wasmAcc = structuredClone(tsAcc);
      for (let j = 0; j < 20; j++) {
        const chunk = parse();
        [tsAcc] = functionsInventionsResponseStreamingFunctionInventionChunkMerged(tsAcc as any, chunk as any);
        wasmAcc = JSON.parse(wasmMerged(wasmAcc, chunk));
        expect(tsAcc, `chunk ${j}`).toEqual(wasmAcc);
      }
    });
  }
});
