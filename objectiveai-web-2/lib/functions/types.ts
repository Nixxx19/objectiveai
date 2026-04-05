/** A function as returned by the API list endpoint */
export interface FunctionListItem {
  remote: string;
  owner: string;
  repository: string;
  commit: string;
}

/** A resolved function with its definition fetched from GitHub */
export interface ResolvedFunction extends FunctionListItem {
  definition: FunctionDefinition;
}

/** The function.json schema — matches the ObjectiveAI specification */
export interface FunctionDefinition {
  type: string;
  description?: string;
  input_schema?: Record<string, unknown>;
  tasks: TaskDefinition[];
  output?: Record<string, unknown>;
  output_length?: Record<string, unknown>;
  input_split?: Record<string, unknown>;
  input_merge?: Record<string, unknown>;
}

export interface TaskDefinition {
  type: string;
  // Function ref fields (GitHub format)
  owner?: string;
  repository?: string;
  commit?: string;
  // Legacy/local format
  name?: string;
  remote?: string;
  // Common fields
  input?: Record<string, unknown>;
  output?: Record<string, unknown>;
  skip?: Record<string, unknown>;
  map?: Record<string, unknown>;
  messages?: Record<string, unknown>;
  responses?: unknown[];
}

/** Parsed function metadata for display */
export interface FunctionMeta {
  remote: string;
  owner: string;
  repository: string;
  commit: string;
  name: string;
  type: "scalar.leaf" | "scalar.branch" | "vector.leaf" | "vector.branch" | string;
  category: "scalar" | "vector";
  depth: "leaf" | "branch";
  description: string;
  taskCount: number;
  subFunctions: string[];
}
