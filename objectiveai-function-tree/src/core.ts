// ---------------------------------------------------------------------------
// @objectiveai/function-tree/core — Framework-agnostic public API
//
// Types, data builders, and layout utilities without React dependency.
// Import from "@objectiveai/function-tree/core" when you need shared types
// or tree data structures without pulling in the React component.
// ---------------------------------------------------------------------------

// Types
export type {
  TreeNode,
  TreeNodeKind,
  TreeNodeState,
  TreeNodeData,
  TreeData,
  TreeMode,
  FunctionNodeData,
  VectorCompletionNodeData,
  EnsembleLlmNodeData,
  FunctionTreeConfig,
  InputFunctionExecution,
  InputFunctionDefinition,
  InputTaskDefinition,
  InputTask,
  InputVectorCompletionTask,
  InputFunctionExecutionTask,
  InputVote,
  InputCompletion,
  InputProfile,
  InputProfileTask,
  InputProfileEnsembleLlm,
} from "./types";

export { DEFAULT_CONFIG, NODE_SIZES, SCORE_COLORS, scoreColor } from "./types";

// Data transformation
export { buildTree, applyProfileWeights } from "./core/tree-data";
export { buildStructuralTree, extractPromptPreview } from "./core/structural-tree-data";

// Layout
export { layoutTree, treeBounds } from "./core/layout";
