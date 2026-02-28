import { describe, it, expect } from "vitest";
import { Functions } from "../../../index.js";

// ── helpers ──────────────────────────────────────────────────────────

function integerInputSchema() {
  return {
    type: "object",
    properties: {
      value: { type: "integer", minimum: 1, maximum: 10 },
    },
    required: ["value"],
  };
}

function stringInputSchema() {
  return {
    type: "object",
    properties: {
      value: { type: "string" },
    },
    required: ["value"],
  };
}

function scalarFunctionTask(inputExpr: string, skip?: object) {
  return {
    type: "alpha.scalar.function",
    remote: "github",
    owner: "test",
    repository: "test",
    commit: "abc123",
    ...(skip ? { skip } : {}),
    input: { $starlark: inputExpr },
  };
}

function placeholderScalarFunctionTask(
  inputExpr: string,
  childInputSchema?: object,
) {
  return {
    type: "placeholder.alpha.scalar.function",
    depth: 1,
    min_branch_width: 1,
    max_branch_width: 3,
    min_leaf_width: 1,
    max_leaf_width: 5,
    name: "test",
    spec: "test spec",
    input_schema: childInputSchema ?? integerInputSchema(),
    input: { $starlark: inputExpr },
  };
}

function validBranch() {
  return {
    type: "alpha.scalar.branch.function",
    description: "test",
    input_schema: integerInputSchema(),
    tasks: [scalarFunctionTask("input")],
  };
}

// ── success tests ────────────────────────────────────────────────────

describe("alphaCheckBranchScalarFunction", () => {
  it("accepts a valid single scalar function task", () => {
    expect(() =>
      Functions.AlphaScalar.Check.alphaCheckBranchScalarFunction(
        validBranch() as any,
      ),
    ).not.toThrow();
  });

  it("accepts a single placeholder scalar task", () => {
    const f = {
      ...validBranch(),
      tasks: [placeholderScalarFunctionTask("input")],
    };
    expect(() =>
      Functions.AlphaScalar.Check.alphaCheckBranchScalarFunction(f as any),
    ).not.toThrow();
  });

  it("accepts multiple mixed tasks", () => {
    const f = {
      ...validBranch(),
      tasks: [
        scalarFunctionTask("input"),
        placeholderScalarFunctionTask("input"),
      ],
    };
    expect(() =>
      Functions.AlphaScalar.Check.alphaCheckBranchScalarFunction(f as any),
    ).not.toThrow();
  });

  it("accepts tasks with skip expressions", () => {
    const f = {
      type: "alpha.scalar.branch.function",
      description: "test",
      input_schema: {
        type: "object",
        properties: {
          text: { type: "string" },
          skip_last_task: { type: "boolean" },
        },
        required: ["text"],
      },
      tasks: [
        scalarFunctionTask("input"),
        scalarFunctionTask("input['text']", {
          $starlark: "input.get('skip_last_task', False)",
        }),
      ],
    };
    expect(() =>
      Functions.AlphaScalar.Check.alphaCheckBranchScalarFunction(f as any),
    ).not.toThrow();
  });

  it("accepts diversity with object field extraction", () => {
    const f = {
      type: "alpha.scalar.branch.function",
      description: "test",
      input_schema: {
        type: "object",
        properties: {
          title: { type: "string" },
          author: { type: "string" },
        },
        required: ["title", "author"],
      },
      tasks: [
        scalarFunctionTask("input['title']"),
        scalarFunctionTask("input['author']"),
      ],
    };
    expect(() =>
      Functions.AlphaScalar.Check.alphaCheckBranchScalarFunction(f as any),
    ).not.toThrow();
  });

  // ── error tests ──────────────────────────────────────────────────

  it("rejects wrong type (leaf)", () => {
    const f = {
      type: "alpha.scalar.leaf.function",
      description: "test",
      input_schema: integerInputSchema(),
      tasks: [],
    };
    expect(() =>
      Functions.AlphaScalar.Check.alphaCheckBranchScalarFunction(f as any),
    ).toThrow(/AB01/);
  });

  it("rejects empty description", () => {
    const f = { ...validBranch(), description: "  " };
    expect(() =>
      Functions.AlphaScalar.Check.alphaCheckBranchScalarFunction(f as any),
    ).toThrow(/QD01/);
  });

  it("rejects description too long", () => {
    const f = { ...validBranch(), description: "a".repeat(351) };
    expect(() =>
      Functions.AlphaScalar.Check.alphaCheckBranchScalarFunction(f as any),
    ).toThrow(/QD02/);
  });

  it("rejects no tasks", () => {
    const f = { ...validBranch(), tasks: [] };
    expect(() =>
      Functions.AlphaScalar.Check.alphaCheckBranchScalarFunction(f as any),
    ).toThrow(/AB03/);
  });

  it("rejects fixed task input (diversity fail)", () => {
    const f = {
      type: "alpha.scalar.branch.function",
      description: "test",
      input_schema: stringInputSchema(),
      tasks: [
        scalarFunctionTask("input"),
        scalarFunctionTask("'always_the_same'"),
      ],
    };
    expect(() =>
      Functions.AlphaScalar.Check.alphaCheckBranchScalarFunction(f as any),
    ).toThrow(/AB10/);
  });

  it("rejects all tasks skipped", () => {
    const f = {
      type: "alpha.scalar.branch.function",
      description: "test",
      input_schema: stringInputSchema(),
      tasks: [
        scalarFunctionTask("input", { $starlark: "True" }),
        {
          ...scalarFunctionTask("input"),
          repository: "test2",
          skip: { $starlark: "True" },
        },
      ],
    };
    expect(() =>
      Functions.AlphaScalar.Check.alphaCheckBranchScalarFunction(f as any),
    ).toThrow(/CV42/);
  });

  it("rejects single-permutation input schema", () => {
    const f = {
      ...validBranch(),
      input_schema: {
        type: "object",
        properties: {
          value: { type: "integer", minimum: 0, maximum: 0 },
        },
        required: ["value"],
      },
    };
    expect(() =>
      Functions.AlphaScalar.Check.alphaCheckBranchScalarFunction(f as any),
    ).toThrow(/QI01/);
  });
});
