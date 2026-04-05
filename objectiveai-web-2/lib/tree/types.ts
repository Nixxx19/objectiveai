// ── function.json types (mirrors the real schema) ──

export type Expression =
  | { $starlark: string }
  | { $jmespath: string }
  | { $special: string };

export type RichContent = string | { type: string; text?: string; [key: string]: unknown };

export interface VectorCompletionTask {
  type: "vector.completion";
  messages: Expression;
  responses: RichContent[][] | Expression;
  output?: Expression;
  skip?: Expression;
  map?: Expression;
}

export interface FunctionRefTask {
  type:
    | "alpha.scalar.function"
    | "alpha.vector.function"
    | "scalar.function"
    | "vector.function";
  remote: string;
  name: string;
  input: Expression | Record<string, Expression>;
  output?: Expression;
  skip?: Expression;
  map?: Expression;
}

export interface PlaceholderTask {
  type: `placeholder.${string}`;
  spec?: string;
}

export type Task = VectorCompletionTask | FunctionRefTask | PlaceholderTask;

export type FunctionType =
  | "alpha.scalar.leaf.function"
  | "alpha.vector.leaf.function"
  | "alpha.scalar.branch.function"
  | "alpha.vector.branch.function"
  | "scalar.function"
  | "vector.function";

export interface FunctionDef {
  type: FunctionType;
  description: string;
  input_schema: Record<string, unknown>;
  tasks: Task[];
  // Vector functions may have these
  output_length?: Expression;
  input_split?: Expression;
  input_merge?: Expression;
}

// ── Tree nodes (what we render) ──

export type NodeKind = "function" | "vector-completion";

export interface TreeNode {
  id: string;
  kind: NodeKind;
  label: string;
  /** e.g. "scalar.leaf", "scalar.branch", "vector.leaf" */
  functionType?: string;
  /** For vector-completion nodes: the response options */
  responses?: string[];
  /** Resolved children */
  children: TreeNode[];
  /** Description from function.json */
  description?: string;
  /** Whether this is a mapped task (fan-out) */
  mapped?: boolean;
  /** Task metadata for detail panel */
  taskMeta?: TaskMeta;
}

/** Metadata stored on tree nodes for the detail panel */
export interface TaskMeta {
  /** System/user messages (vector-completion) */
  messages?: unknown;
  /** Full response text (vector-completion, before truncation) */
  fullResponses?: string[];
  /** Output expression */
  outputExpr?: unknown;
  /** Input expression */
  inputExpr?: unknown;
  /** Input schema (function nodes) */
  inputSchema?: Record<string, unknown>;
}

// ── Layout (positions assigned by layout algorithm) ──

export interface LayoutNode {
  node: TreeNode;
  x: number;
  y: number;
  width: number;
  height: number;
  children: LayoutNode[];
}
