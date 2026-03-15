import { describe, it, expect } from "vitest";
import { ObjectiveAI } from "./client";
import * as fs from "fs";
import * as path from "path";

const port = process.env.OBJECTIVEAI_TEST_PORT;

export const httpTestClient = port
  ? new ObjectiveAI({ apiBase: `http://127.0.0.1:${port}`, apiKey: "test" })
  : null;

export function loadSnapshot<U>(snapshotsDir: string, name: string): U {
  return JSON.parse(fs.readFileSync(path.join(snapshotsDir, `${name}.json`), "utf-8"));
}

export interface HttpTestCase {
  snapshot: string;
  body: Record<string, unknown>;
  /** Per-case endpoint override (used when endpoint varies per test, e.g. function executions) */
  endpoint?: string;
}

export interface HttpTestSuiteOptions<Chunk, Unary> {
  /** describe() label */
  name: string;
  /** Default API path, e.g. "/agent/completions" */
  endpoint: string;
  /** Absolute path to the directory containing snapshot JSON files */
  snapshotsDir: string;
  /** Merge an accumulated chunk with a new chunk */
  merge: (acc: Chunk, chunk: Chunk) => [Chunk, boolean];
  /** Convert a fully-accumulated chunk into a unary response */
  chunkToUnary: (acc: Chunk) => Unary;
  /** Normalize non-deterministic fields before comparison */
  normalize: (unary: Unary) => Unary;
  /** The individual test cases */
  cases: HttpTestCase[];
}

export function httpTestSuite<Chunk, Unary>(opts: HttpTestSuiteOptions<Chunk, Unary>) {
  async function postUnary(endpoint: string, body: Record<string, unknown>): Promise<Unary> {
    return httpTestClient!.post_unary<Unary>(endpoint, { ...body, stream: false });
  }

  async function postStreaming(endpoint: string, body: Record<string, unknown>): Promise<Unary> {
    const stream = await httpTestClient!.post_streaming<Chunk>(endpoint, { ...body, stream: true });
    let acc: Chunk | null = null;
    for await (const chunk of stream) {
      if (acc === null) {
        acc = chunk;
      } else {
        [acc] = opts.merge(acc, chunk);
      }
    }
    expect(acc).not.toBeNull();
    return opts.chunkToUnary(acc!);
  }

  describe.skipIf(!port)(opts.name, () => {
    for (const c of opts.cases) {
      const endpoint = c.endpoint ?? opts.endpoint;

      it(`${c.snapshot} (unary)`, async () => {
        const expected = opts.normalize(loadSnapshot<Unary>(opts.snapshotsDir, c.snapshot));
        expect(opts.normalize(await postUnary(endpoint, c.body))).toEqual(expected);
      });

      it(`${c.snapshot} (streaming)`, async () => {
        const expected = opts.normalize(loadSnapshot<Unary>(opts.snapshotsDir, c.snapshot));
        expect(opts.normalize(await postStreaming(endpoint, c.body))).toEqual(expected);
      });
    }
  });
}
