import { z } from "zod";
import { FunctionsExpressionExpressionSchema } from "./expression/expression";
import { FunctionsExpressionInputSchema } from "./expression/input";
import { FunctionsExpressionInputSchemaSchema } from "./expression/inputSchema";

export const FunctionsPlaceholderVectorFunctionTaskSchema = z.object({
  input_schema: FunctionsExpressionInputSchemaSchema.describe("JSON Schema defining the expected input structure."),
  output_length: FunctionsExpressionExpressionSchema.describe("Expression computing the expected output vector length."),
  input_split: FunctionsExpressionExpressionSchema.describe("Expression transforming input into sub-inputs for swiss system."),
  input_merge: FunctionsExpressionExpressionSchema.describe("Expression merging sub-inputs back into one input."),
  input: FunctionsExpressionInputSchema.describe("The resolved input."),
  output: FunctionsExpressionExpressionSchema.describe("Expression to transform the equalized vector output."),
}).describe("A compiled placeholder vector function task.\n\nAlways produces `Vector(vec![1/N; output_length])` before\nthe output expression is applied.").meta({ title: "functions.PlaceholderVectorFunctionTask" });
export type FunctionsPlaceholderVectorFunctionTask = z.infer<typeof FunctionsPlaceholderVectorFunctionTaskSchema>;
