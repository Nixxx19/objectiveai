import { describe, it, expect } from "vitest";
import { Functions } from "../../../index.js";

// ── helpers ──────────────────────────────────────────────────────────

function arrayItemsInputSchema() {
  return {
    items: {
      type: "array",
      items: { type: "string" },
      minItems: 2,
      maxItems: 10,
    },
  };
}

function vectorFunctionTask(
  itemsExpr: string,
  opts?: { skip?: object; context?: string; repo?: string },
) {
  return {
    type: "alpha.vector.function",
    remote: "github",
    owner: "test",
    repository: opts?.repo ?? "test",
    commit: "abc123",
    ...(opts?.skip ? { skip: opts.skip } : {}),
    input: {
      ...(opts?.context
        ? { context: { $starlark: opts.context } }
        : {}),
      items: { $starlark: itemsExpr },
    },
  };
}

function scalarFunctionTask(inputExpr: string) {
  return {
    type: "alpha.scalar.function",
    remote: "github",
    owner: "test",
    repository: "test",
    commit: "abc123",
    input: { $starlark: inputExpr },
  };
}

function placeholderVectorFunctionTask(
  itemsExpr: string,
  childInputSchema?: object,
) {
  return {
    type: "placeholder.alpha.vector.function",
    depth: 1,
    min_branch_width: 1,
    max_branch_width: 3,
    min_leaf_width: 1,
    max_leaf_width: 3,
    name: "test",
    spec: "Rank items",
    input_schema: childInputSchema ?? arrayItemsInputSchema(),
    input: {
      items: { $starlark: itemsExpr },
    },
  };
}

function validBranch() {
  return {
    type: "alpha.vector.branch.function",
    description: "test",
    input_schema: arrayItemsInputSchema(),
    tasks: [vectorFunctionTask("input['items']")],
  };
}

// ── success tests ────────────────────────────────────────────────────

describe("alphaCheckBranchVectorFunction", () => {
  it("accepts a valid single vector function task", () => {
    expect(() =>
      Functions.AlphaVector.Check.alphaCheckBranchVectorFunction(
        validBranch() as any,
      ),
    ).not.toThrow();
  });

  it("accepts a single placeholder vector task", () => {
    const f = {
      ...validBranch(),
      tasks: [placeholderVectorFunctionTask("input['items']")],
    };
    expect(() =>
      Functions.AlphaVector.Check.alphaCheckBranchVectorFunction(f as any),
    ).not.toThrow();
  });

  it("accepts multiple vector tasks", () => {
    const f = {
      ...validBranch(),
      tasks: [
        vectorFunctionTask("input['items']"),
        vectorFunctionTask("input['items']", { repo: "test2" }),
      ],
    };
    expect(() =>
      Functions.AlphaVector.Check.alphaCheckBranchVectorFunction(f as any),
    ).not.toThrow();
  });

  it("accepts mixed vector and placeholder vector tasks", () => {
    const f = {
      ...validBranch(),
      tasks: [
        vectorFunctionTask("input['items']"),
        placeholderVectorFunctionTask(
          "[x + ' alt' for x in input['items']]",
        ),
      ],
    };
    expect(() =>
      Functions.AlphaVector.Check.alphaCheckBranchVectorFunction(f as any),
    ).not.toThrow();
  });

  it("accepts tasks with skip expression", () => {
    const f = {
      type: "alpha.vector.branch.function",
      description: "test",
      input_schema: {
        items: {
          type: "array",
          items: {
            type: "object",
            properties: {
              text: { type: "string" },
              skip_last: { type: "boolean" },
            },
            required: ["text", "skip_last"],
          },
          minItems: 2,
          maxItems: 10,
        },
      },
      tasks: [
        vectorFunctionTask("input['items']"),
        vectorFunctionTask("input['items']", {
          repo: "test2",
          skip: { $starlark: "input['items'][0]['skip_last']" },
        }),
      ],
    };
    expect(() =>
      Functions.AlphaVector.Check.alphaCheckBranchVectorFunction(f as any),
    ).not.toThrow();
  });

  // ── error tests ──────────────────────────────────────────────────

  it("rejects wrong type (leaf)", () => {
    const f = {
      type: "alpha.vector.leaf.function",
      description: "test",
      input_schema: arrayItemsInputSchema(),
      tasks: [],
    };
    expect(() =>
      Functions.AlphaVector.Check.alphaCheckBranchVectorFunction(f as any),
    ).toThrow(/AW01/);
  });

  it("rejects empty description", () => {
    const f = { ...validBranch(), description: "  " };
    expect(() =>
      Functions.AlphaVector.Check.alphaCheckBranchVectorFunction(f as any),
    ).toThrow(/QD01/);
  });

  it("rejects description too long", () => {
    const f = { ...validBranch(), description: "a".repeat(351) };
    expect(() =>
      Functions.AlphaVector.Check.alphaCheckBranchVectorFunction(f as any),
    ).toThrow(/QD02/);
  });

  it("rejects no tasks", () => {
    const f = { ...validBranch(), tasks: [] };
    expect(() =>
      Functions.AlphaVector.Check.alphaCheckBranchVectorFunction(f as any),
    ).toThrow(/AW02/);
  });

  it("rejects single scalar task (composition rule)", () => {
    const f = {
      ...validBranch(),
      tasks: [scalarFunctionTask("input")],
    };
    expect(() =>
      Functions.AlphaVector.Check.alphaCheckBranchVectorFunction(f as any),
    ).toThrow(/AW08/);
  });

  it("rejects over 50% scalar tasks", () => {
    const f = {
      ...validBranch(),
      tasks: [
        scalarFunctionTask("input"),
        scalarFunctionTask("input"),
        vectorFunctionTask("input['items']"),
      ],
    };
    expect(() =>
      Functions.AlphaVector.Check.alphaCheckBranchVectorFunction(f as any),
    ).toThrow(/AW09/);
  });

  it("rejects fixed task input (diversity fail)", () => {
    const f = {
      ...validBranch(),
      tasks: [
        vectorFunctionTask("input['items']"),
        vectorFunctionTask("['A', 'B']", { repo: "test2" }),
      ],
    };
    expect(() =>
      Functions.AlphaVector.Check.alphaCheckBranchVectorFunction(f as any),
    ).toThrow(/AW18/);
  });

  it("rejects all tasks skipped", () => {
    const f = {
      ...validBranch(),
      tasks: [
        vectorFunctionTask("input['items']", {
          skip: { $starlark: "True" },
        }),
        vectorFunctionTask("input['items']", {
          repo: "test2",
          skip: { $starlark: "True" },
        }),
      ],
    };
    expect(() =>
      Functions.AlphaVector.Check.alphaCheckBranchVectorFunction(f as any),
    ).toThrow(/CV42/);
  });

  it("rejects single-permutation input schema", () => {
    const f = {
      ...validBranch(),
      input_schema: {
        items: {
          type: "array",
          items: { type: "string", enum: ["only"] },
          minItems: 2,
          maxItems: 2,
        },
      },
    };
    expect(() =>
      Functions.AlphaVector.Check.alphaCheckBranchVectorFunction(f as any),
    ).toThrow(/QI01/);
  });
});
