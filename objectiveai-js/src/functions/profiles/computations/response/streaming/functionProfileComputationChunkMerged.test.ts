import { describe, it, expect } from "vitest";
import { zocker } from "zocker";
import { FunctionsProfilesComputationsResponseStreamingFunctionProfileComputationChunkSchema } from "./functionProfileComputationChunk";
import { functionsProfilesComputationsResponseStreamingFunctionProfileComputationChunkMerged } from "./functionProfileComputationChunkMerged";
import { functionProfileComputationChunkMerged as wasmMerged, functionProfileComputationChunkNormalized as wasmNormalized } from "../../../../../../dist/wasm/loader.js";
import { zockerParse } from "../../../../../zockerParse";

const gen = zocker(FunctionsProfilesComputationsResponseStreamingFunctionProfileComputationChunkSchema).array({ max: 3 });
const parse = () => zockerParse(gen, wasmNormalized);

describe("functionProfileComputationChunkMerged fuzz", () => {
  for (let i = 0; i < 20; i++) {
    it(`stream ${i}`, () => {
      let tsAcc = parse();
      let wasmAcc = structuredClone(tsAcc);
      for (let j = 0; j < 20; j++) {
        const chunk = parse();
        [tsAcc] = functionsProfilesComputationsResponseStreamingFunctionProfileComputationChunkMerged(tsAcc as any, chunk as any);
        wasmAcc = JSON.parse(wasmMerged(wasmAcc, chunk));
        expect(tsAcc, `chunk ${j}`).toEqual(wasmAcc);
      }
    });
  }
});
