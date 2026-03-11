import { z } from "zod";
import { FunctionsExpressionExpressionSchema } from "./expression/expression";
import { FunctionsExpressionInputSchemaSchema } from "./expression/inputSchema";
import { FunctionsExpressionWithExpressionFunctionsExpressionInputExpressionSchema } from "./expression/withExpression";

export const FunctionsPlaceholderScalarFunctionTaskExpressionSchema = z.object({
  input_schema: FunctionsExpressionInputSchemaSchema.describe("JSON Schema defining the expected input structure."),
  skip: FunctionsExpressionExpressionSchema.nullable().describe("If this expression evaluates to true, skip the task. Receives: `input`.").optional(),
  map: FunctionsExpressionExpressionSchema.nullable().describe("Expression that evaluates to the number of mapped task instances.\nEach instance receives `map` as an integer index (0-based).").optional(),
  input: FunctionsExpressionWithExpressionFunctionsExpressionInputExpressionSchema.describe("Expression for the input to pass to the placeholder function.\nReceives: `input`, `map` (if mapped)."),
  output: FunctionsExpressionExpressionSchema.describe("Expression to transform the fixed 0.5 output.\nReceives: `input`, `output` as `Scalar(0.5)`."),
}).describe("Expression for a placeholder scalar function task (pre-compilation).\n\nLike [`ScalarFunctionTaskExpression`] but without owner/repository/commit.\nAlways produces a fixed output of 0.5.").meta({ title: "functions.PlaceholderScalarFunctionTaskExpression" });
export type FunctionsPlaceholderScalarFunctionTaskExpression = z.infer<typeof FunctionsPlaceholderScalarFunctionTaskExpressionSchema>;
