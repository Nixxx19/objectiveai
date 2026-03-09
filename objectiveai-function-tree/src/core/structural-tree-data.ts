import type {
  TreeNode,
  TreeData,
  InputFunctionDefinition,
  InputTaskDefinition,
} from "../types";
import { NODE_SIZES as SIZES } from "../types";
import { nodeId } from "./node-id";

// ---------------------------------------------------------------------------
// Build a structural tree from a function definition (no execution data).
// ---------------------------------------------------------------------------

/**
 * Transform a function definition into a TreeData for structural display.
 * Shows the task hierarchy before any execution occurs.
 *
 * @param definition  The function definition (from function.json)
 * @param label       Optional label for the root node (e.g., "owner/repo")
 * @param resolvedSubFunctions  Map of "owner/repo" → definition for recursive expansion
 * @param parentId    Internal: parent node ID for recursive calls
 * @param pathPrefix  Internal: task path prefix for recursive calls
 * @param nodes       Internal: accumulator for recursive calls
 */
export function buildStructuralTree(
  definition: InputFunctionDefinition | null,
  label?: string,
  resolvedSubFunctions?: Map<string, InputFunctionDefinition>,
): TreeData | null {
  if (!definition) return null;

  const nodes = new Map<string, TreeNode>();
  const rootId = "root";

  const rootNode: TreeNode = {
    id: rootId,
    kind: "function",
    label: label || "Function",
    parentId: null,
    children: [],
    x: 0,
    y: 0,
    width: SIZES.function.width,
    height: SIZES.function.height,
    state: "pending",
    data: {
      kind: "function",
      functionId: label ?? null,
      profileId: null,
      output: null,
      taskCount: definition.tasks.length,
      error: null,
      swissRound: null,
      swissPoolIndex: null,
      functionType: definition.type === "scalar.function" ? "scalar" : "vector",
    },
  };
  nodes.set(rootId, rootNode);

  processDefinitionTasks(
    definition.tasks,
    rootId,
    nodes,
    resolvedSubFunctions,
  );

  return { nodes, rootId, mode: "structural" };
}

function processDefinitionTasks(
  tasks: InputTaskDefinition[],
  parentId: string,
  nodes: Map<string, TreeNode>,
  resolvedSubFunctions?: Map<string, InputFunctionDefinition>,
): void {
  for (let i = 0; i < tasks.length; i++) {
    const task = tasks[i];
    const path = [i];

    switch (task.type) {
      case "vector.completion":
        processStructuralVCTask(task, i, path, parentId, nodes);
        break;
      case "scalar.function":
      case "vector.function":
        processStructuralFunctionTask(
          task,
          i,
          path,
          parentId,
          nodes,
          resolvedSubFunctions,
        );
        break;
      case "placeholder.scalar.function":
      case "placeholder.vector.function":
        processStructuralPlaceholderTask(task, i, path, parentId, nodes);
        break;
    }
  }
}

function processStructuralVCTask(
  task: InputTaskDefinition,
  index: number,
  path: number[],
  parentId: string,
  nodes: Map<string, TreeNode>,
): void {
  const id = nodeId("vc", path);

  // Response count: extract from array length, or null if it's an expression
  let responseCount: number | null = null;
  if (Array.isArray(task.responses)) {
    responseCount = task.responses.length;
  }

  const node: TreeNode = {
    id,
    kind: "vector-completion",
    label: `Task ${index}`,
    parentId,
    children: [],
    x: 0,
    y: 0,
    width: SIZES["vector-completion"].width,
    height: SIZES["vector-completion"].height,
    state: "pending",
    data: {
      kind: "vector-completion",
      taskIndex: index,
      taskPath: path,
      scores: null,
      responses: null,
      voteCount: 0,
      votes: null,
      completions: null,
      error: null,
      responseCount,
    },
  };

  nodes.set(id, node);
  const parent = nodes.get(parentId);
  if (parent) parent.children.push(id);
}

function processStructuralFunctionTask(
  task: InputTaskDefinition,
  index: number,
  path: number[],
  parentId: string,
  nodes: Map<string, TreeNode>,
  resolvedSubFunctions?: Map<string, InputFunctionDefinition>,
): void {
  const id = nodeId("func", path);
  const ownerRepo =
    task.owner && task.repository
      ? `${task.owner}/${task.repository}`
      : null;
  const displayLabel = ownerRepo
    ? task.repository!
    : `Task ${index}`;

  const funcType = task.type === "scalar.function" ? "scalar" : "vector";

  // Check if we have the sub-function definition for recursive expansion
  const subDef = ownerRepo
    ? resolvedSubFunctions?.get(ownerRepo) ?? null
    : null;

  const node: TreeNode = {
    id,
    kind: "function",
    label: displayLabel,
    parentId,
    children: [],
    x: 0,
    y: 0,
    width: SIZES.function.width,
    height: SIZES.function.height,
    state: "pending",
    data: {
      kind: "function",
      functionId: ownerRepo,
      profileId: null,
      output: null,
      taskCount: subDef?.tasks.length ?? 0,
      error: null,
      swissRound: null,
      swissPoolIndex: null,
      ownerRepo,
      functionType: funcType,
    },
  };

  nodes.set(id, node);
  const parent = nodes.get(parentId);
  if (parent) parent.children.push(id);

  // Recursively expand sub-function tasks
  if (subDef) {
    processSubFunctionTasks(
      subDef.tasks,
      id,
      path,
      nodes,
      resolvedSubFunctions,
    );
  }
}

function processSubFunctionTasks(
  tasks: InputTaskDefinition[],
  parentId: string,
  parentPath: number[],
  nodes: Map<string, TreeNode>,
  resolvedSubFunctions?: Map<string, InputFunctionDefinition>,
): void {
  for (let i = 0; i < tasks.length; i++) {
    const task = tasks[i];
    const path = [...parentPath, i];

    switch (task.type) {
      case "vector.completion":
        processStructuralVCTask(task, i, path, parentId, nodes);
        break;
      case "scalar.function":
      case "vector.function":
        processStructuralFunctionTask(
          task,
          i,
          path,
          parentId,
          nodes,
          resolvedSubFunctions,
        );
        break;
      case "placeholder.scalar.function":
      case "placeholder.vector.function":
        processStructuralPlaceholderTask(task, i, path, parentId, nodes);
        break;
    }
  }
}

function processStructuralPlaceholderTask(
  task: InputTaskDefinition,
  index: number,
  path: number[],
  parentId: string,
  nodes: Map<string, TreeNode>,
): void {
  const id = nodeId("vc", path);
  const isScalar = task.type === "placeholder.scalar.function";

  const node: TreeNode = {
    id,
    kind: "vector-completion",
    label: `Placeholder${isScalar ? " (scalar)" : " (vector)"}`,
    parentId,
    children: [],
    x: 0,
    y: 0,
    width: SIZES["vector-completion"].width,
    height: SIZES["vector-completion"].height,
    state: "pending",
    data: {
      kind: "vector-completion",
      taskIndex: index,
      taskPath: path,
      scores: null,
      responses: null,
      voteCount: 0,
      votes: null,
      completions: null,
      error: null,
      responseCount: null,
    },
  };

  nodes.set(id, node);
  const parent = nodes.get(parentId);
  if (parent) parent.children.push(id);
}
