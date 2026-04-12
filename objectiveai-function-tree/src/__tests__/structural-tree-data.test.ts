import { describe, it, expect } from "vitest";
import { buildStructuralTree } from "../core/structural-tree-data";
import type {
  InputFunctionDefinition,
  InputTaskDefinition,
  FunctionNodeData,
  VectorCompletionNodeData,
} from "../types";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function vcTask(responses: unknown[] = ["A", "B", "C"]): InputTaskDefinition {
  return {
    type: "vector.completion",
    responses,
    messages: [{ role: "user", content: "test" }],
  };
}

function scalarFuncTask(owner: string, repo: string): InputTaskDefinition {
  return {
    type: "scalar.function",
    owner,
    repository: repo,
    commit: "abc123",
  };
}

function vectorFuncTask(owner: string, repo: string): InputTaskDefinition {
  return {
    type: "vector.function",
    owner,
    repository: repo,
    commit: "def456",
  };
}

function placeholderScalar(): InputTaskDefinition {
  return { type: "placeholder.scalar.function" };
}

function placeholderVector(): InputTaskDefinition {
  return { type: "placeholder.vector.function" };
}

function makeDef(
  type: "scalar.function" | "vector.function",
  tasks: InputTaskDefinition[],
): InputFunctionDefinition {
  return { type, tasks };
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe("buildStructuralTree", () => {
  it("returns null for null input", () => {
    expect(buildStructuralTree(null)).toBeNull();
  });

  it("builds a tree with VC tasks only (leaf function)", () => {
    const def = makeDef("scalar.function", [
      vcTask(["Good", "Average", "Poor"]),
      vcTask(["Yes", "No"]),
    ]);

    const tree = buildStructuralTree(def, "my-scorer");
    expect(tree).not.toBeNull();
    expect(tree!.mode).toBe("structural");
    expect(tree!.rootId).toBe("root");

    // Root node
    const root = tree!.nodes.get("root")!;
    expect(root.kind).toBe("function");
    expect(root.label).toBe("my-scorer");
    expect(root.state).toBe("pending");
    expect(root.children).toEqual(["vc-0", "vc-1"]);
    const rootData = root.data as FunctionNodeData;
    expect(rootData.taskCount).toBe(2);
    expect(rootData.functionType).toBe("scalar");
    expect(rootData.output).toBeNull();

    // VC task 0
    const vc0 = tree!.nodes.get("vc-0")!;
    expect(vc0.kind).toBe("vector-completion");
    expect(vc0.label).toBe("Task 0");
    expect(vc0.state).toBe("pending");
    expect(vc0.parentId).toBe("root");
    const vc0Data = vc0.data as VectorCompletionNodeData;
    expect(vc0Data.responseCount).toBe(3);
    expect(vc0Data.scores).toBeNull();
    expect(vc0Data.voteCount).toBe(0);

    // VC task 1
    const vc1 = tree!.nodes.get("vc-1")!;
    const vc1Data = vc1.data as VectorCompletionNodeData;
    expect(vc1Data.responseCount).toBe(2);
  });

  it("builds a tree with nested function tasks", () => {
    const def = makeDef("scalar.function", [
      scalarFuncTask("objective-ai", "is-spam"),
      vcTask(["A", "B"]),
    ]);

    const tree = buildStructuralTree(def, "parent-fn");
    expect(tree).not.toBeNull();

    const root = tree!.nodes.get("root")!;
    expect(root.children).toEqual(["func-0", "vc-1"]);

    // Function task node
    const func0 = tree!.nodes.get("func-0")!;
    expect(func0.kind).toBe("function");
    expect(func0.label).toBe("is-spam");
    const func0Data = func0.data as FunctionNodeData;
    expect(func0Data.ownerRepo).toBe("objective-ai/is-spam");
    expect(func0Data.functionType).toBe("scalar");
    expect(func0Data.taskCount).toBe(0); // Not resolved
  });

  it("recursively expands resolved sub-functions", () => {
    const parentDef = makeDef("scalar.function", [
      scalarFuncTask("org", "child-fn"),
    ]);

    const childDef = makeDef("scalar.function", [
      vcTask(["X", "Y", "Z"]),
      vcTask(["P", "Q"]),
    ]);

    const resolved = new Map<string, InputFunctionDefinition>();
    resolved.set("org/child-fn", childDef);

    const tree = buildStructuralTree(parentDef, "parent", resolved);
    expect(tree).not.toBeNull();

    // Root → func-0 → vc-0-0, vc-0-1
    const func0 = tree!.nodes.get("func-0")!;
    expect(func0.children).toEqual(["vc-0-0", "vc-0-1"]);
    const func0Data = func0.data as FunctionNodeData;
    expect(func0Data.taskCount).toBe(2);

    // Child VC nodes
    const vc00 = tree!.nodes.get("vc-0-0")!;
    expect(vc00.parentId).toBe("func-0");
    const vc00Data = vc00.data as VectorCompletionNodeData;
    expect(vc00Data.responseCount).toBe(3);

    const vc01 = tree!.nodes.get("vc-0-1")!;
    const vc01Data = vc01.data as VectorCompletionNodeData;
    expect(vc01Data.responseCount).toBe(2);
  });

  it("handles placeholder tasks", () => {
    const def = makeDef("vector.function", [
      placeholderScalar(),
      placeholderVector(),
    ]);

    const tree = buildStructuralTree(def);
    expect(tree).not.toBeNull();

    const root = tree!.nodes.get("root")!;
    expect(root.children).toEqual(["vc-0", "vc-1"]);

    const ph0 = tree!.nodes.get("vc-0")!;
    expect(ph0.label).toBe("Placeholder (scalar)");

    const ph1 = tree!.nodes.get("vc-1")!;
    expect(ph1.label).toBe("Placeholder (vector)");
  });

  it("handles mixed task types", () => {
    const def = makeDef("scalar.function", [
      vcTask(["A", "B"]),
      scalarFuncTask("org", "scorer"),
      placeholderScalar(),
      vectorFuncTask("org", "ranker"),
    ]);

    const tree = buildStructuralTree(def, "mixed");
    expect(tree).not.toBeNull();

    const root = tree!.nodes.get("root")!;
    expect(root.children).toEqual(["vc-0", "func-1", "vc-2", "func-3"]);
    expect(tree!.nodes.size).toBe(5); // root + 4 children
  });

  it("uses 'Function' as default label", () => {
    const def = makeDef("vector.function", [vcTask()]);
    const tree = buildStructuralTree(def);
    expect(tree!.nodes.get("root")!.label).toBe("Function");
  });

  it("sets vector function type correctly", () => {
    const def = makeDef("vector.function", [vcTask()]);
    const tree = buildStructuralTree(def, "ranker");
    const rootData = tree!.nodes.get("root")!.data as FunctionNodeData;
    expect(rootData.functionType).toBe("vector");
  });

  it("node IDs match execution tree scheme", () => {
    // This is critical for animated transitions between structural and execution mode
    const def = makeDef("scalar.function", [
      vcTask(),
      vcTask(),
      scalarFuncTask("org", "sub"),
    ]);

    const tree = buildStructuralTree(def);
    expect(tree).not.toBeNull();

    // These IDs must match what buildTree() produces for the same structure
    expect(tree!.nodes.has("root")).toBe(true);
    expect(tree!.nodes.has("vc-0")).toBe(true);
    expect(tree!.nodes.has("vc-1")).toBe(true);
    expect(tree!.nodes.has("func-2")).toBe(true);
  });

  it("handles empty tasks array", () => {
    const def = makeDef("scalar.function", []);
    const tree = buildStructuralTree(def);
    expect(tree).not.toBeNull();
    expect(tree!.nodes.size).toBe(1); // Just root
    const rootData = tree!.nodes.get("root")!.data as FunctionNodeData;
    expect(rootData.taskCount).toBe(0);
  });
});
