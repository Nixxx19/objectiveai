/// Discovered invention step from the prompt text, available tool names,
/// and optionally from calling the invention tools themselves.
///
/// The mock client inspects the invention tools to determine which step of
/// the invention pipeline it is being asked to execute, and which of the 4
/// routes (scalar leaf, scalar branch, vector leaf, vector branch) applies.
///
/// Discovery uses three layers:
/// 1. **Tool names** — which invention tools are present (e.g. `WriteEssay`,
///    `AppendTask`) determines the step.
/// 2. **Schema tool names** — `Read*JsonSchema` tools differ by route at the
///    InputSchema and Tasks steps.
/// 3. **Tool invocation** — calling read tools (e.g. `ReadInputSchema`,
///    `ReadTask`) and inspecting their output reveals the route at steps
///    where tool names alone are ambiguous (EssayTasks, Description).
///
/// See `DISCOVERY.md` for the full discovery logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventionStep {
    // Step 1: Essay
    // Prompt differs ("Scalar Function" vs "Vector Function") but tools are identical.
    EssayScalar,
    EssayVector,

    // Step 2: Input Schema
    // Schema tool names differ: ReadObjectInputSchemaJsonSchema (scalar)
    // vs ReadAlphaVectorFunctionInputSchemaJsonSchema (vector).
    InputSchemaScalar,
    InputSchemaVector,

    // Step 3: Essay Tasks
    // Tool names and prompt are identical across all 4 routes.
    // Scalar vs vector: call ReadInputSchema → ScalarFunctionInputSchema
    // has `"type": "object"` at root, VectorFunctionInputSchema has
    // `"input_split"` / `"input_merge"` fields.
    // Leaf vs branch: unknown at this step (tasks not yet written).
    EssayTasksScalar,
    EssayTasksVector,

    // Step 4: Tasks (Body)
    // Fully distinguishable via schema tool names.
    TasksScalarLeaf,
    TasksScalarBranch,
    TasksVectorLeaf,
    TasksVectorBranch,

    // Step 5: Description
    // Tool names and prompt are identical across all 4 routes.
    // Scalar vs vector: call ReadInputSchema (same as EssayTasks).
    // Leaf vs branch: call ReadTask(0) → leaf tasks serialize as
    // VectorCompletion expressions, branch tasks as Placeholder expressions.
    DescriptionScalarLeaf,
    DescriptionScalarBranch,
    DescriptionVectorLeaf,
    DescriptionVectorBranch,
}
