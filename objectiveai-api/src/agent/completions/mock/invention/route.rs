/// Discovered invention step from the prompt text and available tool names.
///
/// The mock client inspects the invention tools to determine which step of
/// the invention pipeline it is being asked to execute, and which of the 4
/// routes (scalar leaf, scalar branch, vector leaf, vector branch) applies.
///
/// See `DISCOVERY.md` for the discovery logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventionStep {
    // Step 1: Essay
    // Prompt differs ("Scalar Function" vs "Vector Function") but tools are identical.
    EssayScalar,
    EssayVector,

    // Step 2: Input Schema
    // Scalar routes have schema tool "ReadObjectInputSchemaJsonSchema".
    // Vector routes have schema tool "ReadAlphaVectorFunctionInputSchemaJsonSchema".
    InputSchemaScalar,
    InputSchemaVector,

    // Step 3: Essay Tasks
    // Tools and prompt are identical across all 4 routes.
    // Scalar vs vector cannot be distinguished from tools alone at this step.
    // However, the prompt is also identical, so this is a single variant.
    EssayTasks,

    // Step 4: Tasks (Body)
    // This is the most differentiated step. Schema tools differ by all 4 routes.
    TasksScalarLeaf,
    TasksScalarBranch,
    TasksVectorLeaf,
    TasksVectorBranch,

    // Step 5: Description
    // Tools and prompt are identical across all 4 routes.
    Description,
}
