import { z } from "zod";
import { FunctionsExpressionInputSchemaSchema } from "./inputSchema";

export const FunctionsExpressionAnyOfInputSchemaSchema = z.object({
  anyOf: z.array(z.lazy(() => FunctionsExpressionInputSchemaSchema)).describe("The possible schemas that the input can match."),
}).describe("Schema for a union of possible types - input must match at least one.").meta({ title: "functions.expression.AnyOfInputSchema" });
export type FunctionsExpressionAnyOfInputSchema = z.infer<typeof FunctionsExpressionAnyOfInputSchemaSchema>;
