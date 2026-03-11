import { z } from "zod";
import { FunctionsExpressionExpressionSchema } from "./expression/expression";
import { FunctionsExpressionInputSchema } from "./expression/input";
import { FunctionsRemoteSchema } from "./remote";

export const FunctionsVectorFunctionTaskSchema = z.object({
  remote: FunctionsRemoteSchema.describe("The remote source where the function is hosted."),
  owner: z.string().describe("Repository owner."),
  repository: z.string().describe("Repository name."),
  commit: z.string().describe("Git commit SHA for the function version."),
  input: FunctionsExpressionInputSchema.describe("The resolved input to pass to the function."),
  output: FunctionsExpressionExpressionSchema.describe("Expression to transform the task result into a valid function output.\n\nReceives `output` as the nested function's result (Scalar or Vector).\nMust return a `TaskOutputOwned` valid for the parent function's type (scalar or vector).\nSee [`VectorFunctionTaskExpression::output`] for full documentation."),
}).describe("A compiled vector function task ready for execution.").meta({ title: "functions.VectorFunctionTask" });
export type FunctionsVectorFunctionTask = z.infer<typeof FunctionsVectorFunctionTaskSchema>;
