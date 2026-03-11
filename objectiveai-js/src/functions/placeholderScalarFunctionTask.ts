import { z } from "zod";
import { FunctionsExpressionExpressionSchema } from "./expression/expression";
import { FunctionsExpressionInputSchema } from "./expression/input";
import { FunctionsExpressionInputSchemaSchema } from "./expression/inputSchema";

export const FunctionsPlaceholderScalarFunctionTaskSchema = z.object({
  input_schema: FunctionsExpressionInputSchemaSchema.describe("JSON Schema defining the expected input structure."),
  input: FunctionsExpressionInputSchema.describe("The resolved input."),
  output: FunctionsExpressionExpressionSchema.describe("Expression to transform the fixed 0.5 output."),
}).describe("A compiled placeholder scalar function task.\n\nAlways produces `Scalar(0.5)` before the output expression\nis applied.").meta({ title: "functions.PlaceholderScalarFunctionTask" });
export type FunctionsPlaceholderScalarFunctionTask = z.infer<typeof FunctionsPlaceholderScalarFunctionTaskSchema>;
