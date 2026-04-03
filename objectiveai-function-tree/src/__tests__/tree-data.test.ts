import { describe, it, expect } from "vitest";
import { buildTree, applyProfileWeights } from "../core/tree-data";
import type {
  InputFunctionExecution,
  InputVectorCompletionTask,
  InputFunctionExecutionTask,
  InputProfile,
  FunctionNodeData,
  VectorCompletionNodeData,
  EnsembleLlmNodeData,
} from "../types";

// ---------------------------------------------------------------------------
// Helpers: mock data builders
// ---------------------------------------------------------------------------

function makeVote(index: number, voteDistribution: number[] = [1, 0]) {
  return {
    model: `model-${index}-${"x".repeat(14)}`,
    ensemble_index: index,
    flat_ensemble_index: index,
    vote: voteDistribution,
    weight: 1,
    from_cache: false,
    from_rng: false,
  };
}

function makeCompletion(modelId: string, text: string) {
  return {
    model: modelId,
    choices: [{ delta: { content: text } }],
  };
}

function makeVCTask(
  index: number,
  taskPath: number[],
  votes: ReturnType<typeof makeVote>[] = [],
  scores: number[] = []
): InputVectorCompletionTask {
  return {
    index,
    task_index: index,
    task_path: taskPath,
    votes,
    scores,
    completions: votes.map((v) =>
      makeCompletion(v.model, `Reasoning for model ${v.flat_ensemble_index}`)
    ),
  };
}

function makeFuncTask(
  index: number,
  taskPath: number[],
  subTasks: (InputVectorCompletionTask | InputFunctionExecutionTask)[] = [],
  output?: number
): InputFunctionExecutionTask {
  return {
    index,
    task_index: index,
    task_path: taskPath,
    tasks: subTasks,
    output,
    function: `user/nested-func-${index}`,
  };
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe("buildTree", () => {
  it("returns null for null input", () => {
    expect(buildTree(null)).toBeNull();
  });

  it("builds a root-only tree for empty execution", () => {
    const exec: InputFunctionExecution = {
      id: "exec-1",
      function: "user/my-func",
      tasks: [],
    };

    const tree = buildTree(exec);
    expect(tree).not.toBeNull();
    expect(tree!.rootId).toBe("root");
    expect(tree!.nodes.size).toBe(1);

    const root = tree!.nodes.get("root")!;
    expect(root.kind).toBe("function");
    expect(root.label).toBe("my-func");
    expect(root.children).toEqual([]);
    expect((root.data as FunctionNodeData).taskCount).toBe(0);
  });

  it("builds tree with scalar execution (single VC task, votes stored on node)", () => {
    const exec: InputFunctionExecution = {
      id: "exec-2",
      function: "user/scorer",
      output: 0.75,
      tasks: [
        makeVCTask(0, [0], [makeVote(0, [0.8, 0.2]), makeVote(1, [0.7, 0.3])], [0.75, 0.25]),
      ],
    };

    const tree = buildTree(exec)!;
    expect(tree.nodes.size).toBe(4); // root + vc + 2 LLM nodes

    const root = tree.nodes.get("root")!;
    expect(root.state).toBe("complete");
    expect((root.data as FunctionNodeData).output).toBe(0.75);
    expect(root.children).toEqual(["vc-0"]);

    const vc = tree.nodes.get("vc-0")!;
    expect(vc.kind).toBe("vector-completion");
    expect(vc.state).toBe("complete");
    expect(vc.children.length).toBe(2); // 2 LLM child nodes from votes

    // Vote data stored on the VC node for DetailPanel access
    const vcData = vc.data as VectorCompletionNodeData;
    expect(vcData.voteCount).toBe(2);
    expect(vcData.votes).not.toBeNull();
    expect(vcData.votes!.length).toBe(2);
    expect(vcData.votes![0].vote).toEqual([0.8, 0.2]);
    expect(vcData.votes![1].vote).toEqual([0.7, 0.3]);
  });

  it("builds tree with vector execution", () => {
    const exec: InputFunctionExecution = {
      id: "exec-3",
      output: [0.4, 0.35, 0.25],
      tasks: [
        makeVCTask(0, [0], [makeVote(0), makeVote(1), makeVote(2)], [0.4, 0.35, 0.25]),
      ],
    };

    const tree = buildTree(exec)!;
    const root = tree.nodes.get("root")!;
    expect((root.data as FunctionNodeData).output).toEqual([0.4, 0.35, 0.25]);
  });

  it("builds tree with nested function tasks", () => {
    const exec: InputFunctionExecution = {
      id: "exec-4",
      function: "user/parent-func",
      output: 0.6,
      tasks: [
        makeFuncTask(0, [0], [
          makeVCTask(0, [0, 0], [makeVote(0)], [0.8, 0.2]),
          makeVCTask(1, [0, 1], [makeVote(0)], [0.5, 0.5]),
        ], 0.65),
        makeVCTask(1, [1], [makeVote(0), makeVote(1)], [0.55, 0.45]),
      ],
    };

    const tree = buildTree(exec)!;
    // root + func-task + 2 vc (1 vote each) + vc (2 votes) + 4 LLM nodes = 9
    expect(tree.nodes.size).toBe(9);

    const root = tree.nodes.get("root")!;
    expect(root.children.length).toBe(2); // func-0 and vc-1

    const funcTask = tree.nodes.get("func-0")!;
    expect(funcTask.kind).toBe("function");
    expect(funcTask.label).toBe("nested-func-0");
    expect(funcTask.children.length).toBe(2); // vc-0-0 and vc-0-1
    expect((funcTask.data as FunctionNodeData).output).toBe(0.65);
  });

  it("handles streaming partial data (no votes yet)", () => {
    const exec: InputFunctionExecution = {
      id: "exec-5",
      function: "user/func",
      tasks: [
        {
          index: 0,
          task_index: 0,
          task_path: [0],
          completions: [makeCompletion("model-0-xxxxxxxxxxxxxx", "Thinking...")],
        } as InputVectorCompletionTask,
      ],
    };

    const tree = buildTree(exec)!;
    expect(tree.nodes.size).toBe(2); // root + vc (no llm leaves without votes)
    const vc = tree.nodes.get("vc-0")!;
    expect(vc.state).toBe("streaming");
  });

  it("stores vote data on VC node for DetailPanel access", () => {
    const exec: InputFunctionExecution = {
      id: "exec-6",
      tasks: [
        makeVCTask(0, [0], [makeVote(0)], [1]),
      ],
    };

    const tree = buildTree(exec)!;
    const vc = tree.nodes.get("vc-0")!;
    const vcData = vc.data as VectorCompletionNodeData;
    expect(vcData.votes).not.toBeNull();
    expect(vcData.votes!.length).toBe(1);
    expect(vcData.votes![0].model).toBe("model-0-xxxxxxxxxxxxxx");
    expect(vcData.completions).not.toBeNull();
    expect(vcData.completions!.length).toBe(1);
  });

  it("prioritizes function task when task has both tasks array and scores", () => {
    // A task with both 'tasks' and 'scores' should be treated as a function task
    const exec: InputFunctionExecution = {
      id: "exec-6b",
      function: "user/parent",
      tasks: [
        {
          index: 0,
          task_index: 0,
          task_path: [0],
          tasks: [makeVCTask(0, [0, 0], [], [0.5, 0.5])],
          output: 0.5,
          scores: [0.5, 0.5], // This extra field should NOT cause misidentification
          function: "user/nested",
        } as unknown as InputFunctionExecutionTask,
      ],
    };

    const tree = buildTree(exec)!;
    const root = tree.nodes.get("root")!;
    const childId = root.children[0];
    const child = tree.nodes.get(childId)!;
    // Should be treated as a function node, not a vector completion
    expect(child.kind).toBe("function");
    expect(child.children.length).toBe(1); // The nested VC sub-task
  });

  it("marks error state correctly", () => {
    const exec: InputFunctionExecution = {
      id: "exec-7",
      error: { message: "Something failed" },
      tasks: [],
    };

    const tree = buildTree(exec)!;
    const root = tree.nodes.get("root")!;
    expect(root.state).toBe("error");
    expect((root.data as FunctionNodeData).error).toBe("Something failed");
  });

  it("populates Swiss system fields on function tasks", () => {
    const exec: InputFunctionExecution = {
      id: "exec-swiss",
      tasks: [
        {
          index: 0,
          task_path: [0],
          tasks: [makeVCTask(0, [0, 0], [makeVote(0)], [0.8, 0.2])],
          output: 0.8,
          swiss_round: 1,
          swiss_pool_index: 3,
        } as InputFunctionExecutionTask,
      ],
    };

    const tree = buildTree(exec)!;
    const funcNode = tree.nodes.get("func-0")!;
    const data = funcNode.data as FunctionNodeData;
    expect(data.swissRound).toBe(1);
    expect(data.swissPoolIndex).toBe(3);
    expect(funcNode.label).toBe("Round 1 · Pool 3");
  });

  it("Swiss fields are null when not present", () => {
    const exec: InputFunctionExecution = {
      id: "exec-no-swiss",
      tasks: [makeVCTask(0, [0], [makeVote(0)], [0.5, 0.5])],
    };

    const tree = buildTree(exec)!;
    const root = tree.nodes.get("root")!;
    const data = root.data as FunctionNodeData;
    expect(data.swissRound).toBeNull();
    expect(data.swissPoolIndex).toBeNull();
  });

  it("populates response labels on VC nodes when provided", () => {
    const exec: InputFunctionExecution = {
      id: "exec-labels",
      tasks: [makeVCTask(0, [0], [makeVote(0)], [0.7, 0.3])],
    };

    const labels = { "0": ["Excellent", "Poor"] };
    const tree = buildTree(exec, undefined, labels)!;
    const vcNode = tree.nodes.get("vc-0")!;
    const data = vcNode.data as VectorCompletionNodeData;
    expect(data.responses).toEqual(["Excellent", "Poor"]);
  });

  it("response labels are null when not provided", () => {
    const exec: InputFunctionExecution = {
      id: "exec-no-labels",
      tasks: [makeVCTask(0, [0], [makeVote(0)], [0.6, 0.4])],
    };

    const tree = buildTree(exec)!;
    const vcNode = tree.nodes.get("vc-0")!;
    const data = vcNode.data as VectorCompletionNodeData;
    expect(data.responses).toBeNull();
  });
});

// ---------------------------------------------------------------------------
// Edge weight tests
// ---------------------------------------------------------------------------

describe("edge weights", () => {
  it("root node has null edgeWeight", () => {
    const exec: InputFunctionExecution = {
      id: "ew-1",
      function: "user/func",
      tasks: [],
    };
    const tree = buildTree(exec)!;
    expect(tree.nodes.get("root")!.edgeWeight).toBeNull();
  });

  it("VC task nodes have null edgeWeight by default", () => {
    const exec: InputFunctionExecution = {
      id: "ew-2",
      tasks: [makeVCTask(0, [0], [], [])],
    };
    const tree = buildTree(exec)!;
    expect(tree.nodes.get("vc-0")!.edgeWeight).toBeNull();
  });

  it("LLM nodes get normalized edgeWeight from vote weights", () => {
    const exec: InputFunctionExecution = {
      id: "ew-3",
      tasks: [
        makeVCTask(0, [0], [
          { ...makeVote(0), weight: 2 },
          { ...makeVote(1), weight: 1 },
        ], [0.5, 0.5]),
      ],
    };
    const tree = buildTree(exec)!;
    const vc = tree.nodes.get("vc-0")!;
    const llm0 = tree.nodes.get(vc.children[0])!;
    const llm1 = tree.nodes.get(vc.children[1])!;
    // weight=2 / max(2) = 1.0, weight=1 / max(2) = 0.5
    expect(llm0.edgeWeight).toBe(1);
    expect(llm1.edgeWeight).toBe(0.5);
  });

  it("LLM nodes with equal weights all get edgeWeight of 1", () => {
    const exec: InputFunctionExecution = {
      id: "ew-4",
      tasks: [
        makeVCTask(0, [0], [makeVote(0), makeVote(1), makeVote(2)], [0.5, 0.3, 0.2]),
      ],
    };
    const tree = buildTree(exec)!;
    const vc = tree.nodes.get("vc-0")!;
    // All votes have weight=1, so all normalized to 1.0
    for (const childId of vc.children) {
      expect(tree.nodes.get(childId)!.edgeWeight).toBe(1);
    }
  });

  it("LLM nodes with zero max weight get null edgeWeight", () => {
    const exec: InputFunctionExecution = {
      id: "ew-5",
      tasks: [
        makeVCTask(0, [0], [
          { ...makeVote(0), weight: 0 },
          { ...makeVote(1), weight: 0 },
        ], [0.5, 0.5]),
      ],
    };
    const tree = buildTree(exec)!;
    const vc = tree.nodes.get("vc-0")!;
    for (const childId of vc.children) {
      expect(tree.nodes.get(childId)!.edgeWeight).toBeNull();
    }
  });
});

describe("applyProfileWeights", () => {
  it("applies per-task weights from profile", () => {
    const exec: InputFunctionExecution = {
      id: "pw-1",
      tasks: [
        makeVCTask(0, [0], [], []),
        makeVCTask(1, [1], [], []),
        makeVCTask(2, [2], [], []),
      ],
    };
    const tree = buildTree(exec)!;
    const profile: InputProfile = {
      profile: [3, 1, 2],
      tasks: [],
    };
    applyProfileWeights(tree, profile);

    // Normalized: 3/3=1, 1/3=0.333, 2/3=0.667
    const root = tree.nodes.get("root")!;
    const c0 = tree.nodes.get(root.children[0])!;
    const c1 = tree.nodes.get(root.children[1])!;
    const c2 = tree.nodes.get(root.children[2])!;
    expect(c0.edgeWeight).toBeCloseTo(1, 5);
    expect(c1.edgeWeight).toBeCloseTo(1 / 3, 5);
    expect(c2.edgeWeight).toBeCloseTo(2 / 3, 5);
  });

  it("applies per-LLM weights from profile tasks", () => {
    const exec: InputFunctionExecution = {
      id: "pw-2",
      tasks: [
        makeVCTask(0, [0], [makeVote(0), makeVote(1)], [0.6, 0.4]),
      ],
    };
    const tree = buildTree(exec)!;
    const profile: InputProfile = {
      profile: [1],
      tasks: [{ profile: [4, 2] }],
    };
    applyProfileWeights(tree, profile);

    const vc = tree.nodes.get("vc-0")!;
    const llm0 = tree.nodes.get(vc.children[0])!;
    const llm1 = tree.nodes.get(vc.children[1])!;
    // Profile overrides: 4/4=1, 2/4=0.5
    expect(llm0.edgeWeight).toBeCloseTo(1, 5);
    expect(llm1.edgeWeight).toBeCloseTo(0.5, 5);
  });

  it("does nothing with null profile", () => {
    const exec: InputFunctionExecution = {
      id: "pw-3",
      tasks: [makeVCTask(0, [0], [], [])],
    };
    const tree = buildTree(exec)!;
    applyProfileWeights(tree, null);
    expect(tree.nodes.get("vc-0")!.edgeWeight).toBeNull();
  });

  it("handles empty profile weights array", () => {
    const exec: InputFunctionExecution = {
      id: "pw-4",
      tasks: [makeVCTask(0, [0], [], [])],
    };
    const tree = buildTree(exec)!;
    const profile: InputProfile = {
      profile: [],
      tasks: [],
    };
    applyProfileWeights(tree, profile);
    expect(tree.nodes.get("vc-0")!.edgeWeight).toBeNull();
  });

  it("handles more profile weights than children", () => {
    const exec: InputFunctionExecution = {
      id: "pw-5",
      tasks: [makeVCTask(0, [0], [], [])],
    };
    const tree = buildTree(exec)!;
    const profile: InputProfile = {
      profile: [1, 2, 3],
      tasks: [],
    };
    applyProfileWeights(tree, profile);
    // Only the first child gets a weight
    expect(tree.nodes.get("vc-0")!.edgeWeight).toBeCloseTo(1 / 3, 5);
  });
});
