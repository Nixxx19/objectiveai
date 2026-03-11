import { z } from "zod";
import { FunctionsExpressionExpressionSchema } from "./expression/expression";
import { FunctionsExpressionWithExpressionArrayOfFunctionsExpressionWithExpressionAgentCompletionsMessageMessageExpressionSchema, FunctionsExpressionWithExpressionArrayOfFunctionsExpressionWithExpressionAgentCompletionsMessageRichContentExpressionSchema } from "./expression/withExpression";

export const FunctionsVectorCompletionTaskExpressionSchema = z.object({
  skip: FunctionsExpressionExpressionSchema.nullable().describe("If this expression evaluates to true, skip the task. Receives: `input`.").optional(),
  map: FunctionsExpressionExpressionSchema.nullable().describe("Expression that evaluates to the number of mapped task instances.\nEach instance receives `map` as an integer index (0-based).").optional(),
  messages: FunctionsExpressionWithExpressionArrayOfFunctionsExpressionWithExpressionAgentCompletionsMessageMessageExpressionSchema.describe("Expression for the conversation messages (the prompt).\nReceives: `input`, `map` (if mapped)."),
  responses: FunctionsExpressionWithExpressionArrayOfFunctionsExpressionWithExpressionAgentCompletionsMessageRichContentExpressionSchema.describe("Expression for the possible responses the LLMs can vote for.\nReceives: `input`, `map` (if mapped)."),
  output: FunctionsExpressionExpressionSchema.describe("Expression to transform the task result into a valid function output.\n\nReceives `output` as the task's raw result (typically `Vector(scores)`).\n\nThe expression must return a `TaskOutputOwned` that is valid for the parent function's type:\n- For scalar functions: must return `Scalar(value)` where value is in [0, 1]\n- For vector functions: must return `Vector(values)` where values sum to ~1 and match the expected length\n\nThe function's final output is computed as a weighted average of all task outputs using\nprofile weights. If a function has only one task, that task's output becomes the function's\noutput directly."),
}).describe("Expression for a task that runs a vector completion (pre-compilation).").meta({ title: "functions.VectorCompletionTaskExpression" });
export type FunctionsVectorCompletionTaskExpression = z.infer<typeof FunctionsVectorCompletionTaskExpressionSchema>;
