import { describe, it, expect } from "vitest";
import { Functions } from "../../index.js";

// ── helpers ──────────────────────────────────────────────────────────

const inputExpr = { $starlark: "input" };
const contentParts = [{ type: "text" as const, text: "Hello" }];

function qualityVcTask() {
  return {
    type: "vector.completion" as const,
    messages: [{ role: "user" as const, content: contentParts }],
    responses: [contentParts, contentParts],
    output: { $starlark: "output['scores'][0]" },
  };
}

function branchScalar(tasks: unknown[]) {
  return {
    type: "scalar.function",
    description: "test",
    input_schema: { type: "integer", minimum: 1, maximum: 10 },
    tasks,
  };
}

const mapExpr = { $starlark: "len(input)" };

function scalarFunctionTask(map?: unknown) {
  return {
    type: "scalar.function",
    remote: "github",
    owner: "test",
    repository: "test",
    commit: "abc123",
    input: inputExpr,
    output: { $starlark: "output" },
    ...(map !== undefined ? { map } : {}),
  };
}

function vectorFunctionTask(map?: unknown) {
  return {
    type: "vector.function",
    remote: "github",
    owner: "test",
    repository: "test",
    commit: "abc123",
    input: inputExpr,
    output: { $starlark: "output" },
    ...(map !== undefined ? { map } : {}),
  };
}

function placeholderScalarTask(map?: unknown) {
  return {
    type: "placeholder.scalar.function",
    input_schema: { type: "integer", minimum: 1, maximum: 10 },
    input: inputExpr,
    output: { $starlark: "output" },
    ...(map !== undefined ? { map } : {}),
  };
}

function placeholderVectorTask(map?: unknown) {
  return {
    type: "placeholder.vector.function",
    input_schema: {
      type: "array",
      items: { type: "string" },
      minItems: 2,
      maxItems: 10,
    },
    output_length: { $starlark: "len(input)" },
    input_split: { $starlark: "[[x] for x in input]" },
    input_merge: { $starlark: "[x[0] for x in input]" },
    input: inputExpr,
    output: { $starlark: "output" },
    ...(map !== undefined ? { map } : {}),
  };
}

// ── tests ────────────────────────────────────────────────────────────

describe("checkBranchScalarFunction", () => {
  // wrong type
  it("rejects vector function", () => {
    const f = {
      type: "vector.function",
      description: "test",
      input_schema: {
        type: "object",
        properties: {
          items: { type: "array", items: { type: "string" } },
        },
        required: ["items"],
      },
      output_length: { $starlark: "len(input['items'])" },
      input_split: {
        $starlark: "[{'items': [x]} for x in input['items']]",
      },
      input_merge: {
        $starlark: "{'items': [x['items'][0] for x in input]}",
      },
      tasks: [],
    };
    expect(() => Functions.Quality.checkBranchScalarFunction(f)).toThrow(
      /BS01/,
    );
  });

  // map on tasks
  it("rejects scalar.function task with map", () => {
    const f = branchScalar([scalarFunctionTask(mapExpr)]);
    expect(() => Functions.Quality.checkBranchScalarFunction(f)).toThrow(
      /BS04/,
    );
  });

  it("rejects placeholder.scalar.function task with map", () => {
    const f = branchScalar([placeholderScalarTask(mapExpr)]);
    expect(() => Functions.Quality.checkBranchScalarFunction(f)).toThrow(
      /BS05/,
    );
  });

  // wrong task types
  it("rejects vector.function task", () => {
    const f = branchScalar([vectorFunctionTask()]);
    expect(() => Functions.Quality.checkBranchScalarFunction(f)).toThrow(
      /BS06/,
    );
  });

  it("rejects placeholder.vector.function task", () => {
    const f = branchScalar([placeholderVectorTask()]);
    expect(() => Functions.Quality.checkBranchScalarFunction(f)).toThrow(
      /BS07/,
    );
  });

  it("rejects vector.completion task", () => {
    const f = branchScalar([qualityVcTask()]);
    expect(() => Functions.Quality.checkBranchScalarFunction(f)).toThrow(
      /BS08/,
    );
  });

  // success cases
  it("accepts valid single scalar.function", () => {
    const f = branchScalar([scalarFunctionTask()]);
    expect(() => Functions.Quality.checkBranchScalarFunction(f)).not.toThrow();
  });

  it("accepts valid single placeholder.scalar.function", () => {
    const f = branchScalar([placeholderScalarTask()]);
    expect(() => Functions.Quality.checkBranchScalarFunction(f)).not.toThrow();
  });

  it("accepts valid multiple tasks", () => {
    const f = branchScalar([scalarFunctionTask(), placeholderScalarTask()]);
    expect(() => Functions.Quality.checkBranchScalarFunction(f)).not.toThrow();
  });

  it("rejects empty tasks", () => {
    const f = branchScalar([]);
    expect(() => Functions.Quality.checkBranchScalarFunction(f)).toThrow(
      /BS03/,
    );
  });
});
