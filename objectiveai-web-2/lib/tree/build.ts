import type {
  FunctionDef,
  Task,
  TreeNode,
  TaskMeta,
  VectorCompletionTask,
  FunctionRefTask,
} from "./types";

/**
 * Build a tree from a root function and a registry of resolved functions.
 * IDs use path notation: "root.0.1" = root's first child's second child.
 */
export function buildTree(
  rootName: string,
  rootDef: FunctionDef,
  registry: Map<string, FunctionDef>
): TreeNode {
  return buildNode(rootName, rootDef, registry, "root");
}

function buildNode(
  name: string,
  def: FunctionDef,
  registry: Map<string, FunctionDef>,
  path: string
): TreeNode {
  const children: TreeNode[] = [];

  for (let i = 0; i < def.tasks.length; i++) {
    const task = def.tasks[i];
    const childPath = `${path}.${i}`;
    const child = taskToNode(task, registry, childPath);
    if (child) children.push(child);
  }

  return {
    id: path,
    kind: "function",
    label: name,
    functionType: parseFunctionType(def.type),
    description: def.description,
    children,
    taskMeta: {
      inputSchema: def.input_schema,
    },
  };
}

function taskToNode(
  task: Task,
  registry: Map<string, FunctionDef>,
  path: string
): TreeNode | null {
  if (task.type === "vector.completion") {
    return vectorCompletionNode(task as VectorCompletionTask, path);
  }

  if (task.type.startsWith("placeholder.")) {
    return null;
  }

  const ref = task as FunctionRefTask;
  const resolved = registry.get(`${ref.remote}/${ref.name}`) ?? registry.get(ref.name);

  const refMeta: TaskMeta = {
    inputExpr: ref.input,
    outputExpr: ref.output,
  };

  if (!resolved) {
    return {
      id: path,
      kind: "function",
      label: ref.name,
      functionType: parseFunctionType(ref.type),
      children: [],
      mapped: !!ref.map,
      taskMeta: refMeta,
    };
  }

  const node = buildNode(ref.name, resolved, registry, path);
  node.mapped = !!ref.map;
  // Merge ref expressions with the resolved function's own meta
  node.taskMeta = { ...node.taskMeta, ...refMeta };
  return node;
}

function vectorCompletionNode(
  task: VectorCompletionTask,
  path: string
): TreeNode {
  const responses = extractResponses(task.responses);

  return {
    id: path,
    kind: "vector-completion",
    label: "vector.completion",
    responses,
    children: [],
    mapped: !!task.map,
    taskMeta: {
      messages: task.messages,
      fullResponses: responses,
      outputExpr: task.output,
    },
  };
}

function extractResponses(
  responses: VectorCompletionTask["responses"]
): string[] {
  if (Array.isArray(responses)) {
    return responses.map((r) => {
      if (Array.isArray(r)) {
        return r
          .map((part) => {
            if (typeof part === "string") return part;
            if (typeof part === "object" && part.text) return part.text;
            return "…";
          })
          .join("");
      }
      return String(r);
    });
  }
  return ["(dynamic)"];
}

function parseFunctionType(type: string): string {
  return type.replace(/^alpha\./, "").replace(/\.function$/, "");
}
